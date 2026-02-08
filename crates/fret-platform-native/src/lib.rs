//! Native (non-wasm) platform implementations for `fret-platform` contracts.
//!
//! This crate is intentionally native-only:
//! - uses native clipboard/file-dialog/open-url backends (`arboard`, `rfd`, `webbrowser`)
//! - uses real filesystem paths for external drops and file dialog selections
//!
//! For module ownership and “where should this go?” guidance, see
//! `crates/fret-platform-native/README.md`.

pub mod clipboard;
pub mod external_drop;
pub mod file_dialog;
pub mod open_url;

// -----------------------------------------------------------------------------
// Stable re-exports (native platform surface)
// -----------------------------------------------------------------------------
pub use clipboard::{DesktopClipboard, NativeClipboard};
pub use external_drop::{DesktopExternalDrop, NativeExternalDrop};
pub use file_dialog::{DesktopFileDialog, NativeFileDialog};
pub use open_url::{DesktopOpenUrl, NativeOpenUrl};
