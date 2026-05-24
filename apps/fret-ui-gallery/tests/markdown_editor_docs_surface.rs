#[test]
fn markdown_span_link_diag_script_gates_action_state() {
    let source = include_str!("../src/ui/previews/pages/editors/markdown.rs");
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-markdown-span-link-gate-activate.json"
    );
    let stub = include_str!(
        "../../../tools/diag-scripts/ui-gallery-markdown-span-link-gate-activate.json"
    );
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-markdown-span-link-action-state/suite.json"
    );

    for needle in [
        "let link_gate_href: Arc<str> = Arc::<str>::from(\"https://example.com\")",
        "props.interactive_spans = Arc::from([fret_ui::element::SelectableTextInteractiveSpan {",
        "range: 0..href.len()",
        "tag: href.clone()",
        "cx.selectable_text_on_activate_span_for(",
        ".test_id(\"ui-gallery-markdown-span-link-gate\")",
        ".test_id(\"ui-gallery-markdown-span-link-activated\")",
    ] {
        assert!(
            source.contains(needle),
            "markdown editor source should keep runtime-observable span-link anchors; missing `{needle}`",
        );
    }

    for needle in [
        "\"ui-gallery-markdown-span-link-gate-activate\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"markdown_editor_source\"",
        "\"ui-gallery-page-markdown-editor-source\"",
        "\"ui-gallery-markdown-editor-root\"",
        "\"ui-gallery-markdown-span-link-gate\"",
        "\"ui-gallery-markdown-span-link-activated\"",
        "\"role_is\"",
        "\"value_contains\"",
        "\"semantics_action_is\"",
        "\"semantics_inline_span_includes\"",
        "\"click_selectable_text_span_stable\"",
        "\"role\": \"text\"",
        "\"role\": \"link\"",
        "\"action\": \"set_text_selection\"",
        "\"https://example.com\"",
        "\"Activated: https://example.com\"",
        "\"capture_layout_sidecar\"",
    ] {
        assert!(
            script.contains(needle),
            "markdown span-link script should gate the expected runtime path; missing `{needle}`",
        );
    }

    assert!(
        stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-markdown-span-link-gate-activate.json\""
        ),
        "markdown span-link redirect stub should point at the canonical text-wrap script",
    );
    assert!(
        suite.contains(
            "tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-markdown-span-link-gate-activate.json"
        ),
        "markdown span-link action-state suite should reference the promoted script",
    );
}
