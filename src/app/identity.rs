//! The single source of the product's name and the names derived from it. Renaming the product
//! changes this file and the package name in `Cargo.toml`; the previous data directory goes to
//! `LEGACY_DATA_DIRS` so existing backups move over on first launch.

use std::path::{Path, PathBuf};

pub const PRODUCT_NAME: &str = "Lusorium Studio";

pub fn backup_title() -> String {
    format!("{PRODUCT_NAME} · Backup")
}

/// How the application introduces itself to local services such as the OpenRGB SDK server.
pub const CLIENT_NAME: &str = PRODUCT_NAME;

/// Prefix of the system-wide locks the application takes for devices it drives itself.
pub const LOCK_PREFIX: &str = "LusoriumStudio";

const DATA_DIR_NAME: &str = "LusoriumStudio";

/// Data directories of earlier names, newest first.
const LEGACY_DATA_DIRS: &[&str] = &["AjazzConnect"];

/// File-name stem for files the user saves (reports).
const FILE_STEM: &str = "LusoriumStudio";

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn local_app_data() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

/// `%LOCALAPPDATA%\<data dir>`: backups and settings.
pub fn data_dir() -> Option<PathBuf> {
    local_app_data().map(|base| base.join(DATA_DIR_NAME))
}

/// Moves the newest legacy data directory to the current name, unless the current one already
/// exists. Never merges or deletes: if anything is in the way, the old directory stays as it is.
fn migrate_in(base: &Path) {
    let current = base.join(DATA_DIR_NAME);
    if current.exists() {
        return;
    }
    if let Some(old) = LEGACY_DATA_DIRS
        .iter()
        .map(|name| base.join(name))
        .find(|dir| dir.is_dir())
    {
        let _ = std::fs::rename(old, current);
    }
}

/// Called once at startup, before anything reads or writes the data directory.
pub fn migrate_legacy_data() {
    if let Some(base) = local_app_data() {
        migrate_in(&base);
    }
}

pub fn report_file_name() -> String {
    format!(
        "{FILE_STEM}-report-{}.json",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("identity-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn legacy_directory_moves_with_its_backups() {
        let base = scratch("move");
        let old = base.join(LEGACY_DATA_DIRS[0]).join("backups");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("aa55-x.json"), b"{}").unwrap();
        migrate_in(&base);
        assert!(
            base.join(DATA_DIR_NAME)
                .join("backups/aa55-x.json")
                .exists()
        );
        assert!(!base.join(LEGACY_DATA_DIRS[0]).exists());
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn existing_current_directory_is_never_overwritten() {
        let base = scratch("keep");
        std::fs::create_dir_all(base.join(DATA_DIR_NAME)).unwrap();
        std::fs::create_dir_all(base.join(LEGACY_DATA_DIRS[0])).unwrap();
        migrate_in(&base);
        assert!(base.join(LEGACY_DATA_DIRS[0]).exists());
        std::fs::remove_dir_all(base).unwrap();
    }
}
