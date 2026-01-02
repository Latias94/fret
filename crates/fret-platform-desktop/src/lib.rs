//! Desktop platform implementations for `fret-platform` contracts.
//!
//! This crate is intentionally desktop-only:
//! - uses native clipboard/file-dialog/open-url backends (`arboard`, `rfd`, `webbrowser`)
//! - uses real filesystem paths for external drops and file dialog selections

pub mod clipboard;
pub mod external_drop;
pub mod file_dialog;
pub mod open_url;
