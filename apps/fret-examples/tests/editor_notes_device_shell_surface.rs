#[test]
fn editor_notes_device_shell_demo_keeps_shell_switch_explicit_and_reuses_inner_editor_content() {
    let source = include_str!("../src/editor_notes_device_shell_demo.rs");

    for needle in [
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
}
