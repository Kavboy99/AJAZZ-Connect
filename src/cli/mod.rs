//! Command-line entry points. All of them are read-only toward devices. The executable is a GUI
//! application on Windows, so results go to JSON files, not to the console.

mod inspect;
mod lighting_probe;
mod probe;
mod raster;
mod review;
mod review_backup_tool;
mod review_commit;
mod review_illumination;
mod review_lighting;
mod review_motion;
mod review_scenes;
mod review_state;
mod review_studio;

use anyhow::{Context, Result};
use std::ffi::OsString;
use std::path::PathBuf;

const USAGE: &str = "--probe REPORT.json | --inspect FILE REPORT.json | \
    --lighting-probe DRIVER REPORT.json | --review DIRECTORY";

fn path(args: &[OsString], at: usize, what: &str) -> Result<PathBuf> {
    args.get(at)
        .map(PathBuf::from)
        .with_context(|| format!("missing {what}. Usage: {USAGE}"))
}

/// Runs the command in `args` (without the program name), or returns `None` to start the window.
pub fn run(args: &[OsString]) -> Option<Result<()>> {
    let command = args.first()?.to_str().unwrap_or_default().to_owned();
    Some(match command.as_str() {
        "--probe" => path(args, 1, "report path").and_then(|out| probe::run(&out)),
        "--pack-backup" => path(args, 1, "source backup").and_then(|input| {
            let output = path(args, 2, "archive path")?;
            let snapshot = crate::storage::archive::load(&input)?;
            crate::storage::archive::save(&output, &snapshot)?;
            Ok(())
        }),
        "--inspect" => path(args, 1, "package path")
            .and_then(|input| Ok((input, path(args, 2, "report path")?)))
            .and_then(|(input, out)| inspect::run(&input, &out)),
        "--lighting-probe" => args
            .get(1)
            .and_then(|s| s.to_str())
            .map(str::to_owned)
            .with_context(|| format!("missing driver id. Usage: {USAGE}"))
            .and_then(|driver| Ok((driver, path(args, 2, "report path")?)))
            .and_then(|(driver, out)| lighting_probe::run(&driver, &out)),
        "--review" => path(args, 1, "review directory")
            .and_then(|dir| review::run(&dir, args.get(2).and_then(|s| s.to_str()))),
        "--review-window" => path(args, 1, "GPU metrics file").and_then(|out| {
            let theme = match args.get(2).and_then(|s| s.to_str()) {
                None | Some("dark") => crate::app::Theme::Dark,
                Some("light") => crate::app::Theme::Light,
                _ => anyhow::bail!("native review theme must be dark or light"),
            };
            crate::ui::run_preview(review_state::devices(), out, theme)
        }),
        other => Err(anyhow::anyhow!("unknown command {other}. Usage: {USAGE}")),
    })
}
