//! Portable platform contracts.
//!
//! This crate is intentionally backend-agnostic (no `winit`, no `wgpu`, no `accesskit`).
//!
//! Backend implementations live in crates like:
//! - `fret-platform-native` (native: Windows/macOS/Linux)
//! - `fret-platform-web` (wasm32: browser APIs)
//! - future: `fret-platform-ios` / `fret-platform-android`
//!
//! For module ownership and “where should this go?” guidance, see `crates/fret-platform/README.md`.

pub mod clipboard;
pub mod external_drop;
pub mod file_dialog;
pub mod open_url;

// -----------------------------------------------------------------------------
// Stable re-exports (portable platform surface)
// -----------------------------------------------------------------------------
pub use clipboard::{Clipboard, ClipboardError, ClipboardErrorKind};
pub use external_drop::{ExternalDropProvider, ExternalDropReadLimits};
pub use file_dialog::{
    FileDialogError, FileDialogErrorKind, FileDialogProvider, FileDialogReadLimits,
};
pub use open_url::{OpenUrl, OpenUrlError, OpenUrlErrorKind};
