//! Opt-in structured diagnostics: typed metadata only, never paths, packet contents or input.
use crate::{app::identity, core::DriverError};
use serde::Serialize;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

const MAX_FILE: u64 = 2 * 1024 * 1024;
const KEEP_FILES: usize = 4;
static ENABLED: AtomicBool = AtomicBool::new(false);
static WRITER: Mutex<Option<Writer>> = Mutex::new(None);

#[derive(Serialize)]
struct Record {
    utc: String,
    event: &'static str,
    subject: &'static str,
    code: Option<u8>,
    bytes: Option<usize>,
    elapsed_us: Option<u128>,
    result: &'static str,
    source: Option<String>,
}

struct Writer {
    directory: PathBuf,
    file: File,
    bytes: u64,
    sequence: u32,
}
impl Writer {
    fn new(directory: PathBuf, sequence: u32) -> std::io::Result<Self> {
        std::fs::create_dir_all(&directory)?;
        let name = format!(
            "session-{}-{}-{sequence}.jsonl",
            chrono::Utc::now().format("%Y%m%d-%H%M%S-%f"),
            std::process::id()
        );
        let file = File::options()
            .write(true)
            .create_new(true)
            .open(directory.join(name))?;
        let mut paths: Vec<_> = std::fs::read_dir(&directory)?
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_name().to_string_lossy().starts_with("session-")
                    && e.path().extension().is_some_and(|e| e == "jsonl")
            })
            .map(|e| e.path())
            .collect();
        paths.sort();
        let remove = paths.len().saturating_sub(KEEP_FILES);
        for path in paths.into_iter().take(remove) {
            let _ = std::fs::remove_file(path);
        }
        Ok(Self {
            directory,
            file,
            bytes: 0,
            sequence,
        })
    }
    fn append(&mut self, record: &Record) -> std::io::Result<()> {
        let mut line = serde_json::to_vec(record).map_err(std::io::Error::other)?;
        line.push(b'\n');
        if self.bytes + line.len() as u64 > MAX_FILE {
            *self = Self::new(self.directory.clone(), self.sequence + 1)?;
        }
        self.file.write_all(&line)?;
        self.bytes += line.len() as u64;
        Ok(())
    }
}

pub fn directory() -> Option<PathBuf> {
    identity::data_dir().map(|p| p.join("logs"))
}
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn stream_io(
    kind: &'static str,
    operation: impl FnOnce() -> std::io::Result<usize>,
) -> std::io::Result<usize> {
    if !enabled() {
        return operation();
    }
    let started = Instant::now();
    let result = operation();
    emit(record(
        "transport.io",
        kind,
        None,
        result.as_ref().ok().copied(),
        Some(started.elapsed().as_micros()),
        if result.is_ok() { "ok" } else { "io" },
    ));
    result
}

pub fn configure(on: bool) -> std::io::Result<()> {
    if !on {
        event("logging", "disabled", "ok");
    }
    ENABLED.store(false, Ordering::Relaxed);
    let mut writer = WRITER
        .lock()
        .map_err(|_| std::io::Error::other("logger unavailable"))?;
    *writer = if on {
        Some(Writer::new(
            directory().ok_or_else(|| std::io::Error::other("no data directory"))?,
            0,
        )?)
    } else {
        None
    };
    ENABLED.store(on, Ordering::Relaxed);
    drop(writer);
    event("logging", "enabled", "ok");
    Ok(())
}

fn record(
    event: &'static str,
    subject: &'static str,
    code: Option<u8>,
    bytes: Option<usize>,
    elapsed_us: Option<u128>,
    result: &'static str,
) -> Record {
    Record {
        utc: chrono::Utc::now().to_rfc3339(),
        event,
        subject,
        code,
        bytes,
        elapsed_us,
        result,
        source: None,
    }
}
fn emit(record: Record) {
    if let Ok(mut writer) = WRITER.lock()
        && let Some(writer) = writer.as_mut()
        && writer.append(&record).is_err()
    {
        ENABLED.store(false, Ordering::Relaxed);
    }
}
pub fn event(event: &'static str, subject: &'static str, result: &'static str) {
    if enabled() {
        emit(record(event, subject, None, None, None, result));
    }
}
pub fn command(driver: &'static str, opcode: u8, write: bool) {
    if enabled() {
        emit(record(
            "protocol.command",
            driver,
            Some(opcode),
            None,
            None,
            if write { "write" } else { "read" },
        ));
    }
}
pub fn io<T>(
    kind: &'static str,
    length: usize,
    operation: impl FnOnce() -> Result<T, DriverError>,
) -> Result<T, DriverError> {
    if !enabled() {
        return operation();
    }
    let started = Instant::now();
    let result = operation();
    emit(record(
        "transport.io",
        kind,
        None,
        Some(length),
        Some(started.elapsed().as_micros()),
        result.as_ref().err().map_or("ok", error_code),
    ));
    result
}
pub fn error_code(error: &DriverError) -> &'static str {
    match error {
        DriverError::Unavailable(_) => "unavailable",
        DriverError::Busy => "busy",
        DriverError::Rejected(_) => "rejected",
        DriverError::Protocol(_) => "protocol",
        DriverError::Unconfirmed(_) => "unconfirmed",
        DriverError::ReadBackMismatch => "readback_mismatch",
        DriverError::Changed => "changed",
        DriverError::NoRollback => "no_rollback",
        DriverError::ForeignBackup => "foreign_backup",
        DriverError::Io(_) => "io",
    }
}

pub fn install_panic_hook() {
    static HOOK: OnceLock<()> = OnceLock::new();
    HOOK.get_or_init(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if enabled()
                && let Ok(mut guard) = WRITER.try_lock()
                && let Some(writer) = guard.as_mut()
            {
                // Panic payloads and backtraces may contain user data. Keep only a static marker.
                let mut item = record("panic", "unwinding", None, None, None, "fatal");
                item.source = info.location().and_then(|loc| {
                    Path::new(loc.file()).file_name().map(|name| {
                        format!("{}:{}:{}", name.to_string_lossy(), loc.line(), loc.column())
                    })
                });
                let _ = writer.append(&item);
                let _ = writer.file.sync_data();
            }
            previous(info);
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn files_rotate_and_records_never_include_error_payloads() -> std::io::Result<()> {
        let dir = std::env::temp_dir().join(format!(
            "log-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let mut writer = Writer::new(dir.clone(), 0)?;
        let private = DriverError::Io("SECRET_DEVICE_PATH_AND_SERIAL".into());
        let item = record("operation", "test", None, None, None, error_code(&private));
        for _ in 0..6 {
            writer.bytes = MAX_FILE;
            writer.append(&item)?;
        }
        drop(writer);
        let files: Vec<_> = std::fs::read_dir(&dir)?.collect::<Result<_, _>>()?;
        assert!(files.len() <= KEEP_FILES);
        for file in files {
            let text = std::fs::read_to_string(file.path())?;
            assert!(!text.contains("SECRET"));
            for line in text.lines() {
                let _: serde_json::Value = serde_json::from_str(line)?;
            }
            assert!(file.metadata()?.len() <= MAX_FILE);
        }
        std::fs::remove_dir_all(dir)
    }
}
