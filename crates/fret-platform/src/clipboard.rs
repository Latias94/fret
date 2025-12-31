//! Clipboard contracts.
//!
//! This module intentionally models only the portable data boundary.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardErrorKind {
    Unavailable,
    BackendError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardError {
    pub kind: ClipboardErrorKind,
}

pub trait Clipboard {
    fn set_text(&mut self, text: &str) -> Result<(), ClipboardError>;
    fn get_text(&mut self) -> Result<Option<String>, ClipboardError>;
}
