fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn date_picker_dropdowns_still_show_raw_device_shell_branching_evidence() {
    let source = include_str!("../src/ui/snippets/date_picker/dropdowns.rs");

    for needle in [
        "let is_desktop = fret_ui_kit::declarative::viewport_queries::viewport_width_at_least(",
        "shadcn::Popover::from_open(open.clone())",
        "shadcn::Drawer::new(open.clone())",
    ] {
        assert!(
            source.contains(needle),
            "date picker dropdowns should keep the current device-shell branching evidence visible; missing `{needle}`"
        );
    }
}

#[test]
fn drawer_responsive_dialog_snippet_keeps_explicit_desktop_and_mobile_shells() {
    let source = include_str!("../src/ui/snippets/drawer/responsive_dialog.rs");
    let normalized = normalize_ws(source);

    for needle in [
        "let desktop_dialog = shadcn::Dialog::new(",
        "let mobile_drawer = shadcn::Drawer::new(",
        "ui::h_flex(move |_cx| [desktop_dialog, mobile_drawer])",
    ] {
        assert!(
            source.contains(needle),
            "responsive dialog snippet should keep explicit desktop/mobile shell branches; missing `{needle}`"
        );
    }

    assert!(
        normalized.contains(
            &normalize_ws(
                "ui::h_flex(move |_cx| [desktop_dialog, mobile_drawer]).gap(Space::N2).wrap().w_full().items_center()"
            )
        ),
        "responsive dialog snippet should keep the paired desktop/mobile shell proof surface reviewable"
    );
}

#[test]
fn sidebar_page_keeps_app_shell_device_shell_boundary_explicit() {
    let source = include_str!("../src/ui/pages/sidebar.rs");

    for needle in [
        "`SidebarProvider::is_mobile(...)` and `is_mobile_breakpoint(...)` are app-shell/device-shell controls",
        "`Sidebar` should stay an app-shell surface; editor rails and inspector sidebars should use a separate container-aware surface",
        "That forced-mobile example documents the app-shell sheet path only",
    ] {
        assert!(
            source.contains(needle),
            "sidebar page should keep the app-shell/device-shell boundary explicit; missing `{needle}`"
        );
    }
}
