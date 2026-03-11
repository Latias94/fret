mod driver;
mod harness;
mod spec;

mod ui;
pub use driver::{build_app, build_driver, build_runner_config, run};

#[cfg(not(target_arch = "wasm32"))]
pub use driver::run_with_event_loop;

#[cfg(test)]
mod authoring_surface_policy_tests {
    const MENUBAR: &str = include_str!("driver/menubar.rs");
    const CHROME: &str = include_str!("driver/chrome.rs");
    const RUNTIME_DRIVER: &str = include_str!("driver/runtime_driver.rs");
    const SETTINGS_SHEET: &str = include_str!("driver/settings_sheet.rs");
    const THEME_RUNTIME: &str = include_str!("driver/theme_runtime.rs");
    const UI_MOD: &str = include_str!("ui/mod.rs");
    const PAGE_FIELD: &str = include_str!("ui/pages/field.rs");
    const PAGE_INPUT: &str = include_str!("ui/pages/input.rs");
    const PAGE_KBD: &str = include_str!("ui/pages/kbd.rs");
    const ACTION_FIRST_VIEW: &str = include_str!("ui/snippets/command/action_first_view.rs");

    #[test]
    fn gallery_sources_do_not_depend_on_the_legacy_fret_prelude() {
        assert!(!MENUBAR.contains("fret::prelude"));
        assert!(MENUBAR.contains("use fret::workspace_menu::{"));

        assert!(!ACTION_FIRST_VIEW.contains("use fret::prelude::*;"));
        assert!(ACTION_FIRST_VIEW.contains("use fret::advanced::prelude::*;"));
        assert!(ACTION_FIRST_VIEW.contains("KernelApp"));
        assert!(!ACTION_FIRST_VIEW.contains("ViewCx<'_, '_, App>"));
        assert!(!ACTION_FIRST_VIEW.contains("ElementContext<'_, App>"));
    }

    #[test]
    fn gallery_curated_shadcn_surfaces_stay_explicit() {
        for source in [CHROME, RUNTIME_DRIVER, UI_MOD] {
            assert!(!source.contains("use fret_ui_shadcn as shadcn;"));
            assert!(!source.contains("use fret_ui_shadcn::{self as shadcn"));
        }

        assert!(CHROME.contains("use fret_ui_shadcn::facade as shadcn;"));
        assert!(RUNTIME_DRIVER.contains("use fret_ui_shadcn::facade as shadcn;"));
        assert!(UI_MOD.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));
        assert!(SETTINGS_SHEET.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));

        assert!(!THEME_RUNTIME.contains("fret_ui_shadcn::shadcn_themes::"));
        assert!(THEME_RUNTIME.contains("shadcn::themes::ShadcnBaseColor::"));
        assert!(THEME_RUNTIME.contains("shadcn::themes::apply_shadcn_new_york"));

        for page in [PAGE_FIELD, PAGE_INPUT, PAGE_KBD] {
            assert!(page.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));
            assert!(!page.contains("use fret_ui_shadcn::{self as shadcn, prelude::*};"));
        }
    }
}
