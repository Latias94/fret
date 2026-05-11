use super::*;

#[test]
fn feature_payload_setters_update_snapshot_and_diagnostic_summaries() {
    let handle = CodeEditorHandle::new("aa\nbb\ncc");

    handle
        .set_diagnostic_spans(vec![
            DiagnosticSpan::new(
                6..8,
                fret_code_editor_view::DiagnosticSeverity::Hint,
                "hint",
            ),
            DiagnosticSpan::new(
                0..5,
                fret_code_editor_view::DiagnosticSeverity::Error,
                "error",
            ),
        ])
        .expect("diagnostics");
    handle
        .set_range_decorations(vec![RangeDecoration::new(0..2, "search.match")])
        .expect("decorations");
    handle
        .set_gutter_markers(vec![GutterMarker::logical_line(
            1,
            fret_code_editor_view::GutterMarkerKind::Diagnostic,
        )])
        .expect("gutter markers");
    handle
        .set_semantic_tokens(vec![SemanticToken::new(3..5, "variable")])
        .expect("semantic tokens");

    let snapshot = handle.feature_payload_snapshot();
    assert_eq!(snapshot.schema_version, 1);
    assert_eq!(snapshot.diagnostic_spans_count, 2);
    assert_eq!(snapshot.diagnostic_line_summaries_count, 3);
    assert_eq!(snapshot.range_decorations_count, 1);
    assert_eq!(snapshot.gutter_markers_count, 1);
    assert_eq!(snapshot.semantic_tokens_count, 1);

    let summaries = handle.diagnostic_line_summaries();
    assert_eq!(
        summaries.iter().map(|s| s.line).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
}

#[test]
fn feature_payload_setters_are_idempotent_for_same_normalized_value() {
    let handle = CodeEditorHandle::new("abcdef");
    handle
        .set_diagnostic_spans(vec![
            DiagnosticSpan::new(
                3..5,
                fret_code_editor_view::DiagnosticSeverity::Warning,
                "b",
            ),
            DiagnosticSpan::new(1..4, fret_code_editor_view::DiagnosticSeverity::Error, "a"),
        ])
        .expect("diagnostics");

    let (epoch_before, row_scene_resets_before) = {
        let st = handle.state.borrow();
        (st.feature_payloads.epoch(), st.cache_stats.row_scene_resets)
    };

    handle
        .set_diagnostic_spans(vec![
            DiagnosticSpan::new(1..4, fret_code_editor_view::DiagnosticSeverity::Error, "a"),
            DiagnosticSpan::new(
                3..5,
                fret_code_editor_view::DiagnosticSeverity::Warning,
                "b",
            ),
        ])
        .expect("diagnostics");

    let st = handle.state.borrow();
    assert_eq!(
        st.feature_payloads.epoch(),
        epoch_before,
        "idempotent feature payload setters must not bump feature epoch"
    );
    assert_eq!(
        st.cache_stats.row_scene_resets, row_scene_resets_before,
        "idempotent feature payload setters must not reset row scene caches"
    );
}

#[test]
fn feature_payloads_clear_on_buffer_edit_without_range_tracking() {
    let handle = CodeEditorHandle::new("abcdef");
    handle
        .set_diagnostic_spans(vec![DiagnosticSpan::new(
            0..2,
            fret_code_editor_view::DiagnosticSeverity::Error,
            "error",
        )])
        .expect("diagnostics");
    handle
        .set_range_decorations(vec![RangeDecoration::new(0..2, "search.match")])
        .expect("decorations");
    handle
        .set_gutter_markers(vec![GutterMarker::logical_line(
            0,
            fret_code_editor_view::GutterMarkerKind::Diagnostic,
        )])
        .expect("gutter markers");
    handle
        .set_semantic_tokens(vec![SemanticToken::new(0..2, "keyword")])
        .expect("semantic tokens");

    let epoch_before = handle.feature_payload_snapshot().epoch;
    {
        let mut st = handle.state.borrow_mut();
        st.selection = Selection {
            anchor: 1,
            focus: 1,
        };
        input::insert_text(&mut st, "x").expect("insert text");
    }

    let snapshot = handle.feature_payload_snapshot();
    assert!(
        snapshot.epoch > epoch_before,
        "buffer edits must bump the feature payload epoch when clearing stale payloads"
    );
    assert_eq!(snapshot.buffer_revision, handle.buffer_revision().0);
    assert_eq!(snapshot.diagnostic_spans_count, 0);
    assert_eq!(snapshot.diagnostic_line_summaries_count, 0);
    assert_eq!(snapshot.range_decorations_count, 0);
    assert_eq!(snapshot.gutter_markers_count, 0);
    assert_eq!(snapshot.semantic_tokens_count, 0);
}

#[test]
fn display_row_gutter_markers_are_pruned_when_display_map_shrinks() {
    let handle = CodeEditorHandle::new("abcdef");
    handle.set_soft_wrap_cols(Some(2));
    handle
        .set_gutter_markers(vec![GutterMarker::display_row(
            2,
            fret_code_editor_view::GutterMarkerKind::Diagnostic,
        )])
        .expect("display-row marker");

    let before = handle.feature_payload_snapshot();
    assert_eq!(before.gutter_markers_count, 1);

    handle.set_soft_wrap_cols(None);

    let after = handle.feature_payload_snapshot();
    assert!(
        after.epoch > before.epoch,
        "display-map pruning must bump the feature payload epoch"
    );
    assert_eq!(after.gutter_markers_count, 0);
}
