//! Script runner engine extracted from `ui_diagnostics.rs`.
//!
//! This module exists to keep the main `ui_diagnostics.rs` file from growing without bound.
//! During the fearless refactor we move the large per-frame script driver here in small steps.
