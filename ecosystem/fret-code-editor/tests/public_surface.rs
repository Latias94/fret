use fret_code_editor::{
    CodeEditor, CodeEditorCacheSizeSnapshotV1, CodeEditorFeaturePayloadSnapshotV1,
    CodeEditorHandle, CodeEditorMemorySnapshotV1, CodeFontFeaturePolicy, CodeFontFeaturePreset,
    DiagnosticSeverity, DiagnosticSpan, FoldSpan, GutterMarker, GutterMarkerKind, InlaySpan,
    RangeDecoration, Selection, SemanticToken,
};
use fret_code_editor_buffer::Selection as BufferSelection;
use std::sync::Arc;

#[test]
fn crate_root_exports_public_signature_types() {
    let handle = CodeEditorHandle::new("fn main() {}\n");
    let policy = CodeFontFeaturePolicy {
        preset: CodeFontFeaturePreset::NoLigatures,
        overrides: Vec::new(),
    };

    handle.set_code_font_feature_policy(policy.clone());
    let _editor = CodeEditor::new(handle.clone()).code_font_features(policy);

    let _cache_sizes: CodeEditorCacheSizeSnapshotV1 = handle.cache_size_snapshot();
    let _memory: CodeEditorMemorySnapshotV1 = handle.memory_snapshot();
    let _feature_payloads: CodeEditorFeaturePayloadSnapshotV1 = handle.feature_payload_snapshot();
    let selection = Selection {
        anchor: 0,
        focus: 2,
    };
    let _buffer_selection: BufferSelection = selection;
    handle.set_selection(selection);

    handle.set_line_folds(
        0,
        vec![FoldSpan {
            range: 1..3,
            placeholder: Arc::<str>::from("..."),
        }],
    );
    handle.set_line_inlays(
        0,
        vec![InlaySpan {
            byte: 1,
            text: Arc::<str>::from(": usize"),
        }],
    );
    handle
        .set_diagnostic_spans(vec![DiagnosticSpan::new(
            0..2,
            DiagnosticSeverity::Error,
            "error",
        )])
        .expect("diagnostics are public");
    handle
        .set_range_decorations(vec![RangeDecoration::new(0..2, "search.match")])
        .expect("decorations are public");
    handle
        .set_gutter_markers(vec![GutterMarker::logical_line(
            0,
            GutterMarkerKind::Diagnostic,
        )])
        .expect("gutter markers are public");
    handle
        .set_semantic_tokens(vec![SemanticToken::new(0..2, "keyword")])
        .expect("semantic tokens are public");
}
