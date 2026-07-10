use fret_code_editor::{
    CodeAction, CodeActionList, CodeEditor, CodeEditorCacheSizeSnapshot,
    CodeEditorFeaturePayloadSnapshot, CodeEditorHandle, CodeEditorMemorySnapshot,
    CodeFontFeaturePolicy, CodeFontFeaturePreset, CompletionCandidate, CompletionList,
    DiagnosticSeverity, DiagnosticSpan, DisplayMap, DisplayPoint, DocId, EditorAssistKind,
    EditorAssistRequest, FoldSpan, GutterMarker, GutterMarkerKind, HoverPayload, InlaySpan,
    RangeDecoration, Selection, SemanticToken, TextBoundaryMode, TextBuffer,
    validate_code_action_list, validate_completion_list, validate_editor_assist_request,
    validate_hover_payload,
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
    handle.set_text_boundary_mode(TextBoundaryMode::Identifier);
    let _editor = CodeEditor::new(handle.clone()).code_font_features(policy);

    let _cache_sizes: CodeEditorCacheSizeSnapshot = handle.cache_size_snapshot();
    let _memory: CodeEditorMemorySnapshot = handle.memory_snapshot();
    let _feature_payloads: CodeEditorFeaturePayloadSnapshot = handle.feature_payload_snapshot();
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

    let buffer = TextBuffer::new(DocId::new(), "fn main() {}\n".to_string()).unwrap();
    let display_map = DisplayMap::new(&buffer, None);
    let request = EditorAssistRequest::new(
        EditorAssistKind::Completion,
        buffer.revision(),
        selection,
        0..0,
        DisplayPoint::new(0, 0),
    );
    validate_editor_assist_request(&buffer, Some(&display_map), &request)
        .expect("assist requests are public");

    let mut completions = CompletionList::new(
        "request.completion.1",
        buffer.revision(),
        vec![CompletionCandidate::new("candidate.main", "main")],
    );
    completions.active_id = Some("candidate.main".into());
    validate_completion_list(&buffer, &completions).expect("completion lists are public");

    let hover = HoverPayload::new("hover.1", buffer.revision(), 0..2, "function item");
    validate_hover_payload(&buffer, &hover).expect("hover payloads are public");

    let actions = CodeActionList::new(
        "request.action.1",
        buffer.revision(),
        0..2,
        vec![CodeAction::new(
            "action.extract",
            "Extract function",
            "editor.extract_function",
        )],
    );
    validate_code_action_list(&buffer, &actions).expect("code action lists are public");
}

#[test]
fn app_author_docs_use_code_editor_owned_boundary_types_and_commands() {
    let docs = include_str!("../../../docs/code-editor.md");

    assert!(docs.contains("use fret_code_editor::TextBoundaryMode;"));
    assert!(!docs.contains("fret_runtime::TextBoundaryMode"));
    assert!(docs.contains("`edit.undo` / `edit.redo`"));
    assert!(!docs.contains("`text.undo` / `text.redo`"));
    assert!(!docs.contains("The v1 editor handle"));
}
