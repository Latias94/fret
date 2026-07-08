#[test]
fn breadcrumb_usage_keeps_runtime_action_state_anchors() {
    let usage = include_str!("../src/ui/snippets/breadcrumb/usage.rs");

    for needle in [
        "const CMD_APP_OPEN: &str = \"ui_gallery.app.open\";",
        "shadcn::BreadcrumbLink::new(\"Home\")",
        ".href(\"/\")",
        ".action(CMD_APP_OPEN)",
        ".test_id(\"ui-gallery-breadcrumb-usage-home-link\")",
        "shadcn::BreadcrumbLink::new(\"Components\")",
        ".href(\"/components\")",
        ".test_id(\"ui-gallery-breadcrumb-usage-components-link\")",
    ] {
        assert!(
            usage.contains(needle),
            "breadcrumb usage snippet should keep runtime-observable link anchors; missing `{needle}`",
        );
    }
}

#[test]
fn breadcrumb_usage_home_command_diag_script_gates_action_state() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-usage-home-command.json"
    );
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-breadcrumb-link-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-breadcrumb-usage-home-command\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"breadcrumb\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Usage\"",
        "\"ui-gallery-breadcrumb-usage\"",
        "\"ui-gallery-breadcrumb-usage-home-link\"",
        "\"ui-gallery-breadcrumb-usage-components-link\"",
        "\"role_is\"",
        "\"label_contains\"",
        "\"value_contains\"",
        "\"semantics_action_is\"",
        "\"focus_is\"",
        "\"wait_command_dispatch_trace\"",
        "\"app_snapshot_field_equals\"",
        "\"ui_gallery.app.open\"",
        "\"cmd.open\"",
        "\"started_from_focus\": true",
        "\"role\": \"link\"",
        "\"text\": \"/components\"",
    ] {
        assert!(
            script.contains(needle),
            "breadcrumb usage command script should gate the expected runtime path; missing `{needle}`",
        );
    }

    assert!(
        suite.contains(
            "tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-usage-home-command.json"
        ),
        "breadcrumb link action-state suite should reference the promoted script",
    );
}

#[test]
fn breadcrumb_dropdown_keeps_runtime_action_state_anchors() {
    let dropdown = include_str!("../src/ui/snippets/breadcrumb/dropdown.rs");

    for needle in [
        "const CMD_APP_OPEN: &str = \"ui_gallery.app.open\";",
        "shadcn::BreadcrumbLink::new(\"Home\")",
        ".href(\"/\")",
        ".action(CMD_APP_OPEN)",
        ".test_id(\"ui-gallery-breadcrumb-dropdown-home-link\")",
        "\"ui-gallery-breadcrumb-dropdown-trigger\"",
        ".test_id(\"ui-gallery-breadcrumb-dropdown-docs\")",
    ] {
        assert!(
            dropdown.contains(needle),
            "breadcrumb dropdown snippet should keep runtime-observable link/menu anchors; missing `{needle}`",
        );
    }
}

#[test]
fn breadcrumb_dropdown_link_diag_script_gates_action_state() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-links-semantic-link.json"
    );
    let stub =
        include_str!("../../../tools/diag-scripts/ui-gallery-breadcrumb-links-semantic-link.json");
    let misc_stub = include_str!(
        "../../../tools/diag-scripts/ui-gallery/misc/ui-gallery-breadcrumb-links-semantic-link.json"
    );
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-breadcrumb-link-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-breadcrumb-links-semantic-link\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"breadcrumb\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Dropdown\"",
        "\"ui-gallery-breadcrumb-dropdown\"",
        "\"ui-gallery-breadcrumb-dropdown-home-link\"",
        "\"ui-gallery-breadcrumb-dropdown-trigger\"",
        "\"role_is\"",
        "\"label_contains\"",
        "\"value_contains\"",
        "\"semantics_action_is\"",
        "\"focus_is\"",
        "\"wait_command_dispatch_trace\"",
        "\"app_snapshot_field_equals\"",
        "\"ui_gallery.app.open\"",
        "\"cmd.open\"",
        "\"started_from_focus\": true",
        "\"role\": \"link\"",
    ] {
        assert!(
            script.contains(needle),
            "breadcrumb dropdown link script should gate the expected runtime path; missing `{needle}`",
        );
    }

    for (name, source) in [("root redirect", stub), ("misc redirect", misc_stub)] {
        assert!(
            source.contains(
                "\"to\": \"tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-links-semantic-link.json\""
            ),
            "{name} should point at the canonical breadcrumb semantic-link script",
        );
    }
    assert!(
        suite.contains(
            "tools/diag-scripts/ui-gallery/breadcrumb/ui-gallery-breadcrumb-links-semantic-link.json"
        ),
        "breadcrumb link action-state suite should reference the dropdown semantic-link script",
    );
}
