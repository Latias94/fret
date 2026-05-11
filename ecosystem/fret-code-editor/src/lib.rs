//! Code editor surface (UI integration) for Fret.
//!
//! This crate intentionally lives in `ecosystem/`: editor policies and feature iteration should not
//! leak into `crates/fret-ui` (mechanism-only) surfaces.
//!
//! Normative architecture and contract seams live in ADR 0185 (code editor ecosystem v1).

mod editor;

pub use fret_code_editor_buffer::Selection;
pub use fret_code_editor_view::code_wrap_policy::{CodeWrapPolicy, CodeWrapPreset};
pub use fret_code_editor_view::{
    DiagnosticLineSummary, DiagnosticSeverity, DiagnosticSourceKind, DiagnosticSpan,
    DiagnosticSpanError, FoldSpan, GutterMarker, GutterMarkerAnchor, GutterMarkerError,
    GutterMarkerHitTarget, GutterMarkerKind, GutterMarkerVisual, InlaySpan, RangeDecoration,
    RangeDecorationError, RangeDecorationHitTest, RangeDecorationLayer, SemanticToken,
    SemanticTokenError,
};

pub use editor::{
    CodeEditor, CodeEditorCacheSizeSnapshotV1, CodeEditorCacheStats,
    CodeEditorFeaturePayloadSnapshotV1, CodeEditorHandle, CodeEditorInteractionOptions,
    CodeEditorMemorySnapshotV1, CodeEditorPaintPerfFrame, CodeEditorTorture, CodeFontFeaturePolicy,
    CodeFontFeaturePreset, PreeditState,
};
