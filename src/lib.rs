//! Configuration and customization of PC peripherals. Layers, from the bottom: `core` (contracts),
//! `transport` (the only IO and `unsafe`), `profiles` (device data), `drivers` (one per protocol),
//! `app` (state without UI), `ui` and `cli`. See `docs/ARCHITECTURE.md`.

pub mod app;
pub mod classify;
pub mod cli;
pub mod core;
pub mod diagnostics;
pub mod drivers;
pub mod firmware;
pub mod profiles;
pub mod storage;
pub mod transport;
pub mod ui;
