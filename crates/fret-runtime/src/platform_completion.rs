//! Portable platform I/O completion payloads.
//!
//! This type is intentionally narrower than `Event` so platform backends can only inject
//! completion results (e.g. clipboard reads, external-drop reads) into the UI event stream.

use crate::ClipboardToken;
use fret_core::{
    ClipboardAccessError, ExternalDropDataEvent, FileDialogDataEvent, FileDialogSelection,
};

#[derive(Debug, Clone, PartialEq)]
pub enum PlatformCompletion {
    ClipboardReadText {
        token: ClipboardToken,
        text: String,
    },
    ClipboardReadFailed {
        token: ClipboardToken,
        error: ClipboardAccessError,
    },
    ExternalDropData(ExternalDropDataEvent),
    FileDialogSelection(FileDialogSelection),
    FileDialogData(FileDialogDataEvent),
    FileDialogCanceled,
}
