//! Portable share-sheet (export) contracts.
//!
//! Share/export is modeled as a best-effort platform capability. The runner maps share items to
//! platform-native share APIs (Android intents, iOS activity view controllers, etc.).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareItem {
    Text(String),
    Url(String),
    Bytes {
        name: String,
        mime: Option<String>,
        bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareSheetOutcome {
    Shared,
    Canceled,
    Unavailable,
    Failed { message: String },
}
