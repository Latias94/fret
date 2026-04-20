fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn combobox_stays_the_recipe_owned_device_shell_wrapper_exemplar() {
    let page = include_str!("../src/ui/pages/combobox.rs");
    let recipe = include_str!("../../../ecosystem/fret-ui-shadcn/src/combobox.rs");
    let normalized_recipe = normalize_ws(recipe);

    for needle in [
        "`Combobox::device_shell_responsive(true)` remains the explicit viewport/device-shell follow-up for the responsive example instead of widening the default docs path, and it stays recipe-owned even though the shell classification now delegates to `fret_ui_kit::adaptive::device_shell_mode(...)`.",
        "pub fn device_shell_responsive(mut self, responsive: bool) -> Self {",
        "pub fn device_shell_md_breakpoint(mut self, breakpoint: Px) -> Self {",
        "This remains a recipe-owned wrapper: the binary shell classification now delegates to",
    ] {
        let present = page.contains(needle) || recipe.contains(needle);
        assert!(
            present,
            "combobox should remain the explicit recipe-owned device-shell wrapper exemplar; missing `{needle}`",
        );
    }

    for needle in [
        "let device_shell_policy = fret_ui_kit::adaptive::DeviceShellSwitchPolicy::default()",
        "let is_desktop = fret_ui_kit::adaptive::device_shell_mode(",
    ] {
        assert!(
            normalized_recipe.contains(&normalize_ws(needle)),
            "combobox should delegate binary shell classification to the shared helper owner; missing `{needle}`",
        );
    }
}

#[test]
fn date_picker_and_breadcrumb_stay_on_the_shared_helper_lane() {
    let date_picker = include_str!("../src/ui/snippets/date_picker/dropdowns.rs");
    let breadcrumb = include_str!("../src/ui/snippets/breadcrumb/responsive.rs");

    assert!(
        date_picker.contains("use fret::adaptive::{DeviceShellSwitchPolicy, device_shell_switch};"),
        "date picker dropdowns should stay on the explicit shared helper lane",
    );
    assert!(
        breadcrumb.contains(
            "use fret::adaptive::{DeviceShellSwitchPolicy, device_shell_mode, device_shell_switch};"
        ),
        "breadcrumb responsive snippet should stay on the explicit shared helper lane",
    );

    for (name, source) in [("date picker", date_picker), ("breadcrumb", breadcrumb)] {
        assert!(
            source.contains("device_shell_switch("),
            "{name} should keep the shared helper visible at the app/gallery call site",
        );
        assert!(
            !source.contains("device_shell_responsive("),
            "{name} should not silently grow into the combobox recipe-owned wrapper lane",
        );
    }
}

#[test]
fn dialog_and_sidebar_boundaries_stay_outside_wrapper_growth() {
    let responsive_dialog = include_str!("../src/ui/snippets/drawer/responsive_dialog.rs");
    let sidebar_page = include_str!("../src/ui/pages/sidebar.rs");

    for needle in [
        "let desktop_dialog = shadcn::Dialog::new(",
        "let mobile_drawer = shadcn::Drawer::new(",
    ] {
        assert!(
            responsive_dialog.contains(needle),
            "responsive dialog should stay as an explicit dialog/drawer proof pairing; missing `{needle}`",
        );
    }
    assert!(
        !responsive_dialog.contains("device_shell_switch("),
        "responsive dialog should stay outside recipe-wrapper growth and keep both branches visible",
    );

    for needle in [
        "`SidebarProvider::device_shell_mode(...)` and `device_shell_switch_policy(...)` are app-shell/device-shell controls",
        "`Sidebar` should stay an app-shell surface; editor rails and inspector sidebars should use a separate container-aware surface",
    ] {
        assert!(
            sidebar_page.contains(needle),
            "sidebar page should keep the app-shell boundary explicit; missing `{needle}`",
        );
    }
}

#[test]
fn dialog_and_drawer_recipe_sources_do_not_ship_device_shell_wrapper_api_yet() {
    let dialog = include_str!("../../../ecosystem/fret-ui-shadcn/src/dialog.rs");
    let drawer = include_str!("../../../ecosystem/fret-ui-shadcn/src/drawer.rs");

    for (name, source) in [("dialog", dialog), ("drawer", drawer)] {
        for forbidden in [
            "device_shell_responsive(",
            "device_shell_md_breakpoint(",
            "device_shell_switch(",
            "DeviceShellSwitchPolicy",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} recipe source should not ship a family-specific device-shell wrapper surface yet; found `{forbidden}`",
            );
        }
    }
}

#[test]
fn shadcn_private_adaptive_shell_seam_stays_internal_and_narrow() {
    let lib = include_str!("../../../ecosystem/fret-ui-shadcn/src/lib.rs");
    let helper = include_str!("../../../ecosystem/fret-ui-shadcn/src/adaptive_shell.rs");
    let drawer = include_str!("../../../ecosystem/fret-ui-shadcn/src/drawer.rs");
    let sidebar = include_str!("../../../ecosystem/fret-ui-shadcn/src/sidebar.rs");
    let combobox = include_str!("../../../ecosystem/fret-ui-shadcn/src/combobox.rs");

    assert!(
        lib.contains("mod adaptive_shell;"),
        "shadcn should keep the adaptive shell helper crate-private; missing `mod adaptive_shell;`",
    );
    for forbidden in ["pub mod adaptive_shell;", "raw_module!(adaptive_shell);"] {
        assert!(
            !lib.contains(forbidden),
            "shadcn should not expose the adaptive shell helper through public or raw lanes; found `{forbidden}`",
        );
    }

    assert!(
        helper.contains("pub(crate) fn resolve_device_shell_mode<H: UiHost>("),
        "adaptive shell helper should keep a crate-private mode resolver seam",
    );
    assert!(
        helper.contains("pub(crate) fn is_desktop_shell<H: UiHost>("),
        "adaptive shell helper should stay on a crate-private callable seam",
    );
    for forbidden in [
        "pub fn resolve_device_shell_mode<",
        "pub fn is_desktop_shell<",
        "device_shell_switch(",
        "pub struct DeviceShellSwitchPolicy",
    ] {
        assert!(
            !helper.contains(forbidden),
            "adaptive shell helper should stay a thin private wrapper rather than a second public strategy owner; found `{forbidden}`",
        );
    }

    assert!(
        drawer.contains("crate::adaptive_shell::is_desktop_shell("),
        "drawer should keep using the private adaptive shell boolean seam for its internal parity breakpoint",
    );
    assert!(
        sidebar.contains("crate::adaptive_shell::resolve_device_shell_mode("),
        "sidebar should reuse the private adaptive shell mode seam instead of duplicating device-shell classification locally",
    );

    assert!(
        !combobox.contains("crate::adaptive_shell::is_desktop_shell("),
        "combobox should stay on the explicit shared helper lane rather than silently moving onto the private shadcn seam",
    );
}
