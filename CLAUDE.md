@AGENTS.md

## For Claude Code

- Before editing code read `docs/ARCHITECTURE.md`; before editing the interface read `docs/DESIGN.md`;
  before any work with a device read the "Hard hardware-safety rules" section above and `docs/DEVICES.md`.
- Do not run vendor EXEs and do not send commands to a device without a direct request from the user.
  The `--probe` diagnostics are allowed: they only read.
- If a task needs a decision that the documents do not cover (a new dependency crate, a change of GUI
  library, a write to device memory), ask the user first.
- Write code, comments, commit messages and documents in English. Reply to the user in the language they
  use.
