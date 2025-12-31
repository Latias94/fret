//! Portable platform I/O completion payloads.
//!
//! This type is intentionally narrower than `Event` so platform backends can only inject
//! completion results (e.g. clipboard reads, external-drop reads) into the UI event stream.

use crate::input::ExternalDropDataEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum PlatformCompletion {
    ClipboardText(String),
    ExternalDropData(ExternalDropDataEvent),
}
