use anyhow::{Context, Result};
use serde::Serialize;
use std::{io::Write, path::Path};

pub fn write_json_atomic<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .context("Не указано имя отчёта")?
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
    file.write_all(&serde_json::to_vec_pretty(data)?)?;
    file.sync_all()?;
    drop(file);
    if let Err(error) = std::fs::rename(&temp, path) {
        let _ = std::fs::remove_file(&temp);
        return Err(error.into());
    }
    Ok(())
}
