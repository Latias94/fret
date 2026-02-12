//! Portable “open-in” / incoming-open contracts.
//!
//! Incoming-open requests originate from the OS (file association open, share-target intents, etc.).
//! The portable surface is token-based: runners keep privileged platform handles behind tokens and
//! apps request bytes via effects in the runtime layer.

use crate::{
    ExternalDragFile, ExternalDropFileData, ExternalDropReadError, ExternalDropReadLimits,
    ids::IncomingOpenToken,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncomingOpenKind {
    FileLike,
    TextLike,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IncomingOpenItem {
    File(ExternalDragFile),
    Text {
        /// MIME type when known (e.g. `"text/plain"`). Platforms may leave this unset.
        media_type: Option<String>,
        /// Size estimate in bytes when known.
        estimated_size_bytes: Option<u64>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncomingOpenDataEvent {
    pub token: IncomingOpenToken,
    pub files: Vec<ExternalDropFileData>,
    pub texts: Vec<String>,
    pub errors: Vec<ExternalDropReadError>,
    pub limits: Option<ExternalDropReadLimits>,
}
