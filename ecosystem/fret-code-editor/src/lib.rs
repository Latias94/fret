//! Code editor surface (UI integration) for Fret.
//!
//! This crate intentionally lives in `ecosystem/`: editor policies and feature iteration should not
//! leak into `crates/fret-ui` (mechanism-only) surfaces.
//!
//! Normative architecture and contract seams live in ADR 0185 (code editor ecosystem v1).

mod editor;

pub use editor::{
    CodeEditor, CodeEditorCacheSizeSnapshotV1, CodeEditorCacheStats, CodeEditorHandle,
    CodeEditorInteractionOptions, CodeEditorMemorySnapshotV1, CodeEditorPaintPerfFrame,
    CodeEditorTorture, CodeFontFeaturePolicy, CodeFontFeaturePreset, PreeditState, Selection,
};
