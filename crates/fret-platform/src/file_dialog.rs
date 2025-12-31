//! File dialog platform contracts.
//!
//! The portable contract is token-based, similar to external drops (ADR 0053).

use crate::external_drop::ExternalDropReadLimits;
use fret_core::{FileDialogDataEvent, FileDialogOptions, FileDialogSelection, FileDialogToken};

pub type FileDialogReadLimits = ExternalDropReadLimits;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDialogErrorKind {
    Unsupported,
    BackendError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDialogError {
    pub kind: FileDialogErrorKind,
}

pub trait FileDialogProvider {
    fn open_files(
        &mut self,
        options: &FileDialogOptions,
    ) -> Result<Option<FileDialogSelection>, FileDialogError>;

    fn read_all(
        &mut self,
        token: FileDialogToken,
        limits: FileDialogReadLimits,
    ) -> Option<FileDialogDataEvent>;

    fn release(&mut self, token: FileDialogToken);
}

