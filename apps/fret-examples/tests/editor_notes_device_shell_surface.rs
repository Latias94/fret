#[test]
fn editor_notes_device_shell_demo_keeps_shell_switch_explicit_and_reuses_inner_editor_content() {
    let source = include_str!("../src/editor_notes_device_shell_demo.rs");

    for needle in [
        "let theme = cx.theme_snapshot();",
        "use fret::adaptive::{DeviceShellSwitchPolicy, device_shell_switch};",
        "device_shell_switch(",
        "WorkspaceFrame::new(center)",
        ".left(left_rail)",
        ".right(right_rail)",
        "shadcn::Drawer::new(drawer_open.clone())",
        "shadcn::DrawerPart::content_with(move |cx| {",
        "editor_notes_demo::render_selection_panel(cx, selected)",
        "editor_notes_demo::render_center_panel(",
        "editor_notes_demo::render_inspector_panel(",
        "\"editor-notes-device-shell-demo.drawer.trigger\"",
        "\"editor-notes-device-shell-demo.drawer.content\"",
    ] {
        assert!(
            source.contains(needle),
            "editor notes device shell demo should keep desktop/mobile shell ownership explicit while reusing the shared inner editor content; missing `{needle}`"
        );
    }

    assert!(
        !source.contains("Theme::global(&*cx.app).snapshot()"),
        "editor notes device shell demo should use the app-facing theme snapshot helper instead of reading theme through cx.app",
    );
}

#[test]
fn editor_notes_device_shell_demo_diag_script_keeps_desktop_rail_and_mobile_drawer_proof_steps() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-editor/editor-notes-demo/editor-notes-device-shell-demo-responsive-proof.json"
    );

    for needle in [
        "\"schema_version\": 2",
        "\"editor-notes-device-shell-demo.root\"",
        "\"editor-notes-device-shell-demo.left-rail\"",
        "\"editor-notes-device-shell-demo.right-rail\"",
        "\"editor-notes-device-shell-demo.mobile-header\"",
        "\"editor-notes-device-shell-demo.drawer.trigger\"",
        "\"editor-notes-device-shell-demo.drawer.content\"",
        "\"editor-notes-device-shell-demo.drawer.viewport\"",
        "\"editor-notes-device-shell-demo.drawer.close\"",
        "\"kind\": \"not_exists\"",
        "\"kind\": \"exists_under\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"type\": \"capture_layout_sidecar\"",
        "\"type\": \"capture_bundle\"",
        "\"type\": \"capture_screenshot\"",
    ] {
        assert!(
            script.contains(needle),
            "editor notes device shell diag script should keep the desktop rail / mobile drawer proof reviewable; missing `{needle}`"
        );
    }
}
