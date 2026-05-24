#[test]
fn button_link_render_keeps_runtime_action_state_anchors() {
    let link_render = include_str!("../src/ui/snippets/button/link_render.rs");

    for needle in [
        ".render(shadcn::ButtonRender::Link",
        ".action(\"ui_gallery.app.open\")",
        ".test_id(\"ui-gallery-button-render-link\")",
        ".test_id(\"ui-gallery-button-render-link-row\")",
    ] {
        assert!(
            link_render.contains(needle),
            "button link-render snippet should keep runtime-observable semantic-link anchors; missing `{needle}`",
        );
    }
}

#[test]
fn button_link_diag_script_gates_action_state() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json"
    );
    let stub = include_str!("../../../tools/diag-scripts/ui-gallery-button-link-render.json");
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-button-link-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-button-link-render\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"button\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"As Link / As Child (Semantic)\"",
        "\"ui-gallery-button-link-semantic-content\"",
        "\"ui-gallery-button-render-link-row\"",
        "\"ui-gallery-button-render-link\"",
        "\"role_is\"",
        "\"label_contains\"",
        "\"semantics_action_is\"",
        "\"focus_is\"",
        "\"wait_command_dispatch_trace\"",
        "\"app_snapshot_field_equals\"",
        "\"ui_gallery.app.open\"",
        "\"cmd.open\"",
        "\"started_from_focus\": true",
        "\"role\": \"link\"",
        "\"text\": \"Login\"",
    ] {
        assert!(
            script.contains(needle),
            "button link-render script should gate the expected runtime path; missing `{needle}`",
        );
    }

    assert!(
        stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json\""
        ),
        "button link-render redirect stub should point at the canonical button script",
    );
    assert!(
        suite.contains("tools/diag-scripts/ui-gallery/button/ui-gallery-button-link-render.json"),
        "button link action-state suite should reference the promoted script",
    );
}
