//! Code editor surface (UI integration) for Fret.
//!
//! This crate intentionally lives in `ecosystem/`: editor policies and feature iteration should not
//! leak into `crates/fret-ui` (mechanism-only) surfaces.
//!
//! Normative architecture and contract seams live in ADR 0185 (code editor ecosystem v1).

mod editor;

pub use fret_code_editor_buffer::{
    AppliedEdit, BufferDelta, DocId, DocUri, Edit, EditError, LineDelta, Revision, Selection,
    TextBuffer, TextBufferTransaction, TextBufferTx,
};
pub use fret_code_editor_view::code_wrap_policy::{CodeWrapPolicy, CodeWrapPreset};
pub use fret_code_editor_view::{
    CodeAction, CodeActionKind, CodeActionList, CodeActionListError, CompletionCandidate,
    CompletionCandidateKind, CompletionCommitKind, CompletionList, CompletionListError,
    DiagnosticLineSummary, DiagnosticSeverity, DiagnosticSourceKind, DiagnosticSpan,
    DiagnosticSpanError, DisplayMap, DisplayPoint, EditorAssistKind, EditorAssistRequest,
    EditorAssistRequestError, EditorAssistTrigger, FoldSpan, GutterMarker, GutterMarkerAnchor,
    GutterMarkerError, GutterMarkerHitTarget, GutterMarkerKind, GutterMarkerVisual, HoverPayload,
    HoverPayloadError, InlaySpan, RangeDecoration, RangeDecorationError, RangeDecorationHitTest,
    RangeDecorationLayer, SemanticToken, SemanticTokenError, validate_code_action_list,
    validate_completion_list, validate_editor_assist_request, validate_hover_payload,
};
pub use fret_runtime::TextBoundaryMode;

pub use editor::{
    CodeEditor, CodeEditorCacheSizeSnapshot, CodeEditorCacheStats,
    CodeEditorFeaturePayloadSnapshot, CodeEditorHandle, CodeEditorInteractionOptions,
    CodeEditorMemorySnapshot, CodeEditorPaintPerfFrame, CodeEditorTorture, CodeFontFeaturePolicy,
    CodeFontFeaturePreset, PreeditState,
};
