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
        "`SidebarProvider::is_mobile(...)` and `is_mobile_breakpoint(...)` are app-shell/device-shell controls",
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
