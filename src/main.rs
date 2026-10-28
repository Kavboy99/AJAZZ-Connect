#![cfg_attr(windows, windows_subsystem = "windows")]
//! Entry point only: a command-line action, or the window.

use lusorium_studio::{app, cli, ui};

fn main() -> anyhow::Result<()> {
    app::identity::migrate_legacy_data();
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match cli::run(&args) {
        Some(result) => result,
        None => ui::run(),
    }
}
