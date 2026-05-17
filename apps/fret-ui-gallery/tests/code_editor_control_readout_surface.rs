mod support;

use support::read;

fn compact_rust(source: &str) -> String {
    source.split_whitespace().collect()
}

fn assert_contains_compact(source: &str, expected: &str) {
    let compact_source = compact_rust(source);
    let compact_expected = compact_rust(expected);
    assert!(
        compact_source.contains(&compact_expected),
        "expected source to contain `{expected}`"
    );
}

fn assert_not_contains_compact(source: &str, forbidden: &str) {
    let compact_source = compact_rust(source);
    let compact_forbidden = compact_rust(forbidden);
    assert!(
        !compact_source.contains(&compact_forbidden),
        "source reintroduced forbidden wrapping readout `{forbidden}`"
    );
}

#[test]
fn code_editor_header_state_readouts_use_single_line_control_readout() {
    let doc_layout = read("src/ui/doc_layout.rs");
    let status_bar = read("src/driver/status_bar.rs");
    let mvp_header = read("src/ui/previews/pages/editors/code_editor/mvp/header.rs");
    let torture = read("src/ui/previews/pages/editors/code_editor/torture.rs");

    assert_contains_compact(&doc_layout, "decl_text::text_control_readout(cx, text)");
    assert_contains_compact(&doc_layout, "decl_text::text_code_block(cx, code.clone())");
    assert_not_contains_compact(&doc_layout, "fn control_readout_text_props");
    assert_not_contains_compact(&doc_layout, "let monospace = fret_core::TextStyle");

    assert_contains_compact(
        &status_bar,
        "fn status_bar_readout_text(cx: &mut ElementContext<'_, App>, text: impl Into<Arc<str>>) -> AnyElement",
    );
    assert_contains_compact(&status_bar, "decl_text::text_control_readout(cx, text)");
    assert_contains_compact(
        &status_bar,
        "status_bar_readout_text(cx, format!(\"theme={} view_cache={} layout_us={} paint_us={}\"",
    );
    assert_contains_compact(&status_bar, "status_bar_readout_text(cx, \"inspector=on\")");
    assert_contains_compact(
        &status_bar,
        "vec![status_bar_readout_text(cx, status_last_action_text.clone())]",
    );

    for expected in [
        "doc_layout::control_readout_text(cx, if syntax_enabled {",
        "doc_layout::control_readout_text(cx, if boundary_identifier_enabled {",
        "doc_layout::control_readout_text(cx, if soft_wrap_enabled {",
    ] {
        assert_contains_compact(&mvp_header, expected);
    }

    for expected in [
        "doc_layout::control_readout_text(cx, \"Assist:\")",
        "doc_layout::control_readout_text(cx, if syntax_enabled {",
        "doc_layout::control_readout_text(cx, if boundary_identifier_enabled {",
        "doc_layout::control_readout_text(cx, if soft_wrap_enabled {",
        "doc_layout::control_readout_text(cx, if allow_decorations_under_preedit_enabled {",
        "doc_layout::control_readout_text(cx, if compose_inline_preedit_enabled {",
        "doc_layout::control_readout_text(cx, if folds_enabled {",
        "doc_layout::control_readout_text(cx, if inlays_enabled {",
        "doc_layout::control_readout_text(cx, format!(\"Interaction: {mode_label}\"))",
    ] {
        assert_contains_compact(&torture, expected);
    }

    for forbidden in [
        "cx.text(if syntax_enabled {",
        "cx.text(if boundary_identifier_enabled {",
        "cx.text(if soft_wrap_enabled {",
        "cx.text(if allow_decorations_under_preedit_enabled {",
        "cx.text(if compose_inline_preedit_enabled {",
        "cx.text(if folds_enabled {",
        "cx.text(if inlays_enabled {",
        "cx.text(format!(\"Interaction: {mode_label}\"))",
    ] {
        assert_not_contains_compact(&mvp_header, forbidden);
        assert_not_contains_compact(&torture, forbidden);
    }

    for forbidden in [
        "cx.text(format!(\"theme={} view_cache={} layout_us={} paint_us={}\"",
        "cx.text(\"inspector=on\")",
        "cx.text(status_last_action_text.as_ref())",
    ] {
        assert_not_contains_compact(&status_bar, forbidden);
    }
}
