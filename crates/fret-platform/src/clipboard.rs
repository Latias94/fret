//! Clipboard contracts.
//!
//! This module intentionally models only the portable data boundary.

pub type ClipboardErrorKind = fret_core::ClipboardAccessErrorKind;
pub type ClipboardError = fret_core::ClipboardAccessError;

pub trait Clipboard {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError>;
    fn get_text(&mut self) -> Result<Option<String>, ClipboardError>;
}
