#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() -> anyhow::Result<()> {
    lusorium_studio::app::identity::migrate_legacy_data();
    lusorium_studio::ui::run_backup()
}
