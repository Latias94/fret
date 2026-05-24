#[test]
fn item_demo_keeps_basic_and_link_action_state_anchors() {
    let demo = include_str!("../src/ui/snippets/item/demo.rs");

    for needle in [
        ".test_id(\"ui-gallery-item-docs-demo-basic\")",
        ".test_id(\"ui-gallery-item-docs-demo-sm-link\")",
        ".test_id(\"ui-gallery-item-docs-demo-sm-link-media\")",
        ".test_id(\"ui-gallery-item-docs-demo-sm-link-content\")",
        ".test_id(\"ui-gallery-item-docs-demo-sm-link-actions\")",
        "shadcn::ItemRender::Link",
        ".action(CMD_APP_OPEN)",
        ".a11y_label(\"Verified profile\")",
        ".test_id(\"ui-gallery-item-demo\")",
    ] {
        assert!(
            demo.contains(needle),
            "item demo should keep runtime-observable Basic and Link row anchors; missing `{needle}`",
        );
    }
}

#[test]
fn item_diag_script_gates_demo_link_action_state() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/item/ui-gallery-item-demo-link-action-state.json"
    );
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-item-demo-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-item-demo-link-action-state\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"item\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Demo\"",
        "\"ui-gallery-item-docs-demo-basic\"",
        "\"ui-gallery-item-docs-demo-sm-link\"",
        "\"role_is\"",
        "\"semantics_action_is\"",
        "\"focus_is\"",
        "\"wait_command_dispatch_trace\"",
        "\"app_snapshot_field_equals\"",
        "\"ui_gallery.app.open\"",
        "\"cmd.open\"",
    ] {
        assert!(
            script.contains(needle),
            "item Demo link/action-state script should gate the expected runtime path; missing `{needle}`",
        );
    }

    assert!(
        suite.contains(
            "tools/diag-scripts/ui-gallery/item/ui-gallery-item-demo-link-action-state.json"
        ),
        "item Demo action-state suite should reference the promoted script",
    );
}

#[test]
fn item_link_render_keeps_runtime_action_state_anchors() {
    let link_render = include_str!("../src/ui/snippets/item/link_render.rs");

    for needle in [
        ".render(shadcn::ItemRender::Link",
        ".action(CMD_APP_OPEN)",
        ".a11y_label(\"Dashboard\")",
        ".test_id(\"ui-gallery-item-link-render\")",
    ] {
        assert!(
            link_render.contains(needle),
            "item Link (render) snippet should keep runtime-observable link anchors; missing `{needle}`",
        );
    }
}

#[test]
fn item_link_render_diag_script_gates_action_state() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/item/ui-gallery-item-link-render.json"
    );
    let stub = include_str!("../../../tools/diag-scripts/ui-gallery-item-link-render.json");
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-item-link-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-item-link-render\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"item\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Link (render)\"",
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
        "\"text\": \"Dashboard\"",
    ] {
        assert!(
            script.contains(needle),
            "item Link (render) script should gate the expected runtime path; missing `{needle}`",
        );
    }

    assert!(
        stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/item/ui-gallery-item-link-render.json\""
        ),
        "item link-render redirect stub should point at the canonical item script",
    );
    assert!(
        suite.contains("tools/diag-scripts/ui-gallery/item/ui-gallery-item-link-render.json"),
        "item link action-state suite should reference the promoted script",
    );
}
