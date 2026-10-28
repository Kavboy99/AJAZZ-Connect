use serde::Serialize;
use std::{io, io::Write, path::Path};

/// Writes to a temporary sibling and renames it over the target, so a crash never leaves a
/// half-written report or backup behind.
pub fn write_json<T: Serialize>(path: &Path, data: &T) -> io::Result<()> {
    write_bytes(
        path,
        &serde_json::to_vec_pretty(data).map_err(io::Error::other)?,
    )
}

pub fn write_bytes(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no file name"))?
        .to_string_lossy();
    let temp = parent.join(format!(
        ".{file_name}.{}-{}.tmp",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let mut file = std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })
}
