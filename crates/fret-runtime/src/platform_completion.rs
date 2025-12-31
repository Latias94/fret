//! Portable platform I/O completion payloads.
//!
//! This type is intentionally narrower than `Event` so platform backends can only inject
//! completion results (e.g. clipboard reads, external-drop reads) into the UI event stream.

use fret_core::{ClipboardToken, ExternalDropDataEvent, FileDialogDataEvent, FileDialogSelection};

#[derive(Debug, Clone, PartialEq)]
pub enum PlatformCompletion {
    ClipboardText { token: ClipboardToken, text: String },
    ClipboardTextUnavailable { token: ClipboardToken },
    ExternalDropData(ExternalDropDataEvent),
    FileDialogSelection(FileDialogSelection),
    FileDialogData(FileDialogDataEvent),
    FileDialogCanceled,
}

