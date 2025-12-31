//! Portable file-dialog contracts.
//!
//! A file dialog selection is modeled as a token plus safe metadata. The UI/runtime can later
//! request bytes via effects, keeping the contract portable to sandboxed environments.

use crate::{ExternalDragFile, ExternalDropFileData, ExternalDropReadError, FileDialogToken};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDialogFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileDialogOptions {
    pub title: Option<String>,
    pub multiple: bool,
    pub filters: Vec<FileDialogFilter>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileDialogSelection {
    pub token: FileDialogToken,
    pub files: Vec<ExternalDragFile>,
}

pub type FileDialogFileData = ExternalDropFileData;
pub type FileDialogReadError = ExternalDropReadError;

#[derive(Debug, Clone, PartialEq)]
pub struct FileDialogDataEvent {
    pub token: FileDialogToken,
    pub files: Vec<FileDialogFileData>,
    pub errors: Vec<FileDialogReadError>,
}
