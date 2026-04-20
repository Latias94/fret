fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn adaptive_module_exports_crate_local_device_shell_switch_surface() {
    let source = include_str!("../../../ecosystem/fret-ui-kit/src/adaptive.rs");

    for needle in [
        "pub enum DeviceShellMode {",
        "pub struct DeviceShellSwitchPolicy {",
        "pub fn device_shell_mode<'a, H: UiHost + 'a, Cx>(",
        "pub fn device_shell_switch<'a, H, Cx, DesktopBranch, MobileBranch, DesktopChild, MobileChild>(",
    ] {
        assert!(
            source.contains(needle),
            "adaptive module should expose the crate-local device-shell helper surface; missing `{needle}`"
        );
    }
}

#[test]
fn date_picker_dropdowns_now_use_shared_device_shell_switch_but_keep_explicit_shell_branches() {
    let source = include_str!("../src/ui/snippets/date_picker/dropdowns.rs");

    assert!(
        source.contains("use fret::adaptive::{")
            && source.contains("DeviceShellSwitchPolicy")
            && source.contains("device_shell_switch"),
        "date picker dropdowns should import the shared device-shell helper from `fret::adaptive`"
    );

    for needle in [
        "let overlay = device_shell_switch(",
        "DeviceShellSwitchPolicy::default()",
        "shadcn::Popover::from_open(desktop_open.clone())",
        "shadcn::Drawer::new(mobile_open.clone())",
    ] {
        assert!(
            source.contains(needle),
            "date picker dropdowns should use the shared device-shell helper while keeping explicit popover/drawer branches visible; missing `{needle}`"
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
        "`SidebarProvider::device_shell_mode(...)` and `device_shell_switch_policy(...)` are app-shell/device-shell controls",
        "`Sidebar` should stay an app-shell surface; editor rails and inspector sidebars should use a separate container-aware surface",
        "That forced-mobile example documents the app-shell sheet path only",
    ] {
        assert!(
            source.contains(needle),
            "sidebar page should keep the app-shell/device-shell boundary explicit; missing `{needle}`"
        );
    }
}

#[test]
fn breadcrumb_responsive_snippet_now_uses_shared_device_shell_helpers_but_keeps_explicit_shell_branches()
 {
    let source = include_str!("../src/ui/snippets/breadcrumb/responsive.rs");

    assert!(
        source.contains("use fret::adaptive::{")
            && source.contains("DeviceShellSwitchPolicy")
            && source.contains("device_shell_mode")
            && source.contains("device_shell_switch"),
        "breadcrumb responsive snippet should import the shared device-shell helpers from `fret::adaptive`"
    );

    for needle in [
        "let is_desktop = device_shell_mode(cx, Invalidation::Layout, shell_policy).is_desktop();",
        "vec![device_shell_switch(",
        "let dropdown = shadcn::DropdownMenu::from_open(open.clone())",
        "let drawer = shadcn::Drawer::new(open.clone());",
    ] {
        assert!(
            source.contains(needle),
            "breadcrumb responsive snippet should use shared device-shell helpers while keeping explicit dropdown/drawer branches visible; missing `{needle}`"
        );
    }
}

#[test]
fn combobox_surface_keeps_explicit_recipe_owned_device_shell_naming() {
    let page = include_str!("../src/ui/pages/combobox.rs");
    let recipe = include_str!("../../../ecosystem/fret-ui-shadcn/src/combobox.rs");

    for needle in [
        "`Combobox::device_shell_responsive(true)` remains the explicit viewport/device-shell follow-up",
        "pub fn device_shell_responsive(mut self, responsive: bool) -> Self {",
        "pub fn device_shell_md_breakpoint(mut self, breakpoint: Px) -> Self {",
        "This is intentionally **viewport-driven** (mobile vs desktop), not container-query-driven.",
    ] {
        let present = page.contains(needle) || recipe.contains(needle);
        assert!(
            present,
            "combobox surface should keep explicit recipe-owned device-shell naming visible; missing `{needle}`"
        );
    }
}

#[test]
fn fret_root_surface_now_exposes_explicit_device_shell_strategy_lane() {
    let root = include_str!("../../../ecosystem/fret/src/lib.rs");

    for needle in [
        "DeviceShellMode, DeviceShellSwitchPolicy,",
        "device_shell_mode, device_shell_switch,",
    ] {
        assert!(
            root.contains(needle),
            "fret root adaptive lane should re-export explicit device-shell strategy helpers; missing `{needle}`"
        );
    }
}
