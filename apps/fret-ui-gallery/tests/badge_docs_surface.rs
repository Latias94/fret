#[test]
fn badge_link_render_keeps_runtime_action_state_anchors() {
    let link = include_str!("../src/ui/snippets/badge/link.rs");

    for needle in [
        ".render(shadcn::BadgeRender::Link",
        ".on_activate(Arc::new(|_host, _acx, _reason| {}))",
        ".test_id(\"ui-gallery-badge-link\")",
        ".test_id(\"ui-gallery-badge-link-row\")",
    ] {
        assert!(
            link.contains(needle),
            "badge link snippet should keep runtime-observable link anchors; missing `{needle}`",
        );
    }
}

#[test]
fn badge_link_diag_script_gates_action_state() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json"
    );
    let stub = include_str!("../../../tools/diag-scripts/ui-gallery-badge-link-render.json");
    let suite = include_str!(
        "../../../tools/diag-scripts/suites/ui-gallery-badge-link-action-state/suite.json"
    );

    for needle in [
        "\"ui-gallery-badge-link-render\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"badge\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"Link\"",
        "\"ui-gallery-badge-link-row\"",
        "\"ui-gallery-badge-link\"",
        "\"role_is\"",
        "\"semantics_action_is\"",
        "\"focus_is\"",
        "\"role\": \"link\"",
        "\"text\": \"Open Link\"",
    ] {
        assert!(
            script.contains(needle),
            "badge link-render script should gate the expected runtime path; missing `{needle}`",
        );
    }

    assert!(
        stub.contains(
            "\"to\": \"tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json\""
        ),
        "badge link-render redirect stub should point at the canonical badge script",
    );
    assert!(
        suite.contains("tools/diag-scripts/ui-gallery/badge/ui-gallery-badge-link-render.json"),
        "badge link action-state suite should reference the promoted script",
    );
}
