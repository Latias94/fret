//! Code editor surface (UI integration) for Fret.
//!
//! This crate intentionally lives in `ecosystem/`: editor policies and feature iteration should not
//! leak into `crates/fret-ui` (mechanism-only) surfaces.

mod editor;

pub use editor::{
    CodeEditor, CodeEditorCacheStats, CodeEditorHandle, CodeEditorTorture, PreeditState, Selection,
};
