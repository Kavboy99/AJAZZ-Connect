use crate::firmware;
use crate::storage::atomic;
use anyhow::Result;
use std::path::Path;

/// `--inspect FILE REPORT.json`: static analysis of a vendor package. Nothing is executed.
pub fn run(input: &Path, out: &Path) -> Result<()> {
    atomic::write_json(out, &firmware::inspect_file(input)?)?;
    Ok(())
}
