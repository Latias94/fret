#[test]
fn empty_demo_keeps_action_state_anchors() {
    let demo = include_str!("../src/ui/snippets/empty/demo.rs");

    for needle in [
        ".test_id(\"ui-gallery-empty-demo-create-project\")",
        ".test_id(\"ui-gallery-empty-demo-import-project\")",
        ".test_id(\"ui-gallery-empty-demo-actions\")",
        ".test_id(\"ui-gallery-empty-demo-title\")",
        ".test_id(\"ui-gallery-empty-demo-header\")",
        ".render(shadcn::ButtonRender::Link",
        ".on_activate(Arc::new(|_host, _acx, _reason| {}))",
        ".test_id(\"ui-gallery-empty-demo-learn-more\")",
        ".test_id(\"ui-gallery-empty-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "empty demo should keep runtime-observable action-state anchors; missing `{needle}`",
        );
    }
}

#[test]
fn empty_diag_script_gates_demo_action_state() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/empty/ui-gallery-empty-demo-action-state.json"
    );
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-empty-demo-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-empty-demo-action-state\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"empty\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Demo\"",
        "\"ui-gallery-empty-demo-title\"",
        "\"ui-gallery-empty-demo-create-project\"",
        "\"ui-gallery-empty-demo-import-project\"",
        "\"ui-gallery-empty-demo-learn-more\"",
        "\"role_is\"",
        "\"semantics_action_is\"",
        "\"focus_is\"",
        "\"role\": \"link\"",
    ] {
        assert!(
            script.contains(needle),
            "empty Demo action-state script should gate the expected runtime path; missing `{needle}`",
        );
    }

    assert!(
        suite.contains(
            "tools/diag-scripts/ui-gallery/empty/ui-gallery-empty-demo-action-state.json"
        ),
        "empty Demo action-state suite should reference the promoted script",
    );
}

#[test]
fn empty_page_keeps_upstream_docs_order_before_rtl_follow_up() {
    let page = include_str!("../src/ui/pages/empty.rs");

    assert!(
        page.contains(
            "vec![demo, usage, outline, background, avatar, avatar_group, input_group, api_reference, rtl]"
        ),
        "Empty page should keep the upstream docs path through API Reference before the Fret-only RTL follow-up",
    );
    assert!(
        page.contains("`RTL` remains an explicit Fret follow-up."),
        "Empty page should label RTL as a Fret follow-up because current upstream Empty docs do not include an RTL example",
    );
}
