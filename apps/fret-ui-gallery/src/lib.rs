mod driver;
mod harness;
mod spec;

mod ui;
pub use driver::{build_app, build_driver, build_runner_config, run};

#[cfg(not(target_arch = "wasm32"))]
pub use driver::run_with_event_loop;

#[cfg(test)]
mod authoring_surface_policy_tests {
    use std::path::{Path, PathBuf};

    const MENUBAR: &str = include_str!("driver/menubar.rs");
    const CHROME: &str = include_str!("driver/chrome.rs");
    const RUNTIME_DRIVER: &str = include_str!("driver/runtime_driver.rs");
    const SETTINGS_SHEET: &str = include_str!("driver/settings_sheet.rs");
    const THEME_RUNTIME: &str = include_str!("driver/theme_runtime.rs");
    const UI_MOD: &str = include_str!("ui/mod.rs");
    const UI_CONTENT: &str = include_str!("ui/content.rs");
    const UI_NAV: &str = include_str!("ui/nav.rs");
    const UI_PREVIEWS_MAGIC: &str = include_str!("ui/previews/magic.rs");
    const PAGES_MOD: &str = include_str!("ui/pages/mod.rs");
    const PAGE_FIELD: &str = include_str!("ui/pages/field.rs");
    const PAGE_INPUT: &str = include_str!("ui/pages/input.rs");
    const PAGE_KBD: &str = include_str!("ui/pages/kbd.rs");
    const ACTION_FIRST_VIEW: &str = include_str!("ui/snippets/command/action_first_view.rs");
    const PROGRESS_USAGE: &str = include_str!("ui/snippets/progress/usage.rs");
    const PROGRESS_LABEL: &str = include_str!("ui/snippets/progress/label.rs");
    const PROGRESS_RTL: &str = include_str!("ui/snippets/progress/rtl.rs");
    const PROGRESS_CONTROLLED: &str = include_str!("ui/snippets/progress/controlled.rs");
    const PROGRESS_DEMO: &str = include_str!("ui/snippets/progress/demo.rs");
    const SLIDER_USAGE: &str = include_str!("ui/snippets/slider/usage.rs");
    const TOAST_DEPRECATED: &str = include_str!("ui/snippets/toast/deprecated.rs");
    const NAVIGATION_MENU_DEMO: &str = include_str!("ui/snippets/navigation_menu/demo.rs");
    const NAVIGATION_MENU_DOCS_DEMO: &str =
        include_str!("ui/snippets/navigation_menu/docs_demo.rs");
    const NAVIGATION_MENU_RTL: &str = include_str!("ui/snippets/navigation_menu/rtl.rs");
    const CHART_CONTRACTS: &str = include_str!("ui/snippets/chart/contracts.rs");
    const CHART_DEMO: &str = include_str!("ui/snippets/chart/demo.rs");
    const CHART_LEGEND: &str = include_str!("ui/snippets/chart/legend.rs");
    const CHART_RTL: &str = include_str!("ui/snippets/chart/rtl.rs");
    const CHART_TOOLTIP: &str = include_str!("ui/snippets/chart/tooltip.rs");
    const CHART_USAGE: &str = include_str!("ui/snippets/chart/usage.rs");
    const MOTION_PRESET_SELECTOR: &str =
        include_str!("ui/snippets/motion_presets/preset_selector.rs");
    const MOTION_FLUID_TABS_DEMO: &str =
        include_str!("ui/snippets/motion_presets/fluid_tabs_demo.rs");
    const MOTION_OVERLAY_DEMO: &str = include_str!("ui/snippets/motion_presets/overlay_demo.rs");
    const MOTION_STACK_SHIFT_LIST_DEMO: &str =
        include_str!("ui/snippets/motion_presets/stack_shift_list_demo.rs");
    const MOTION_STAGGER_DEMO: &str = include_str!("ui/snippets/motion_presets/stagger_demo.rs");
    const MOTION_TOKEN_SNAPSHOT: &str =
        include_str!("ui/snippets/motion_presets/token_snapshot.rs");

    fn collect_rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect_rust_sources(&path, out);
                continue;
            }

            if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }

    fn gallery_rust_sources() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        collect_rust_sources(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut paths,
        );
        paths.sort();
        paths
    }

    #[test]
    fn gallery_sources_do_not_depend_on_the_legacy_fret_prelude() {
        assert!(!MENUBAR.contains("fret::prelude"));
        assert!(MENUBAR.contains("use fret::workspace_menu::{"));

        assert!(!ACTION_FIRST_VIEW.contains("use fret::prelude::*;"));
        assert!(ACTION_FIRST_VIEW.contains("use fret::advanced::prelude::*;"));
        assert!(ACTION_FIRST_VIEW.contains("use fret::app::App;"));
        assert!(
            ACTION_FIRST_VIEW.contains("fn init(_app: &mut App, _window: AppWindowId) -> Self")
        );
        assert!(!ACTION_FIRST_VIEW.contains("ViewCx<'_, '_, App>"));
        assert!(!ACTION_FIRST_VIEW.contains("ViewCx<'_, '_, KernelApp>"));
        assert!(!ACTION_FIRST_VIEW.contains(") -> Elements {"));
        assert!(ACTION_FIRST_VIEW.contains("fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui"));
        assert!(ACTION_FIRST_VIEW.contains("cx.state().local::<u32>()"));
        assert!(ACTION_FIRST_VIEW.contains("cx.actions().models::<act::Ping>"));
        assert!(ACTION_FIRST_VIEW.contains("cx.actions().availability::<act::Ping>"));
        assert!(ACTION_FIRST_VIEW.contains(
            "pub fn render(cx: &mut UiCx<'_>, last_action: Model<Arc<str>>) -> AnyElement"
        ));
        assert!(!ACTION_FIRST_VIEW.contains("KernelApp"));
        assert!(!ACTION_FIRST_VIEW.contains("ElementContext<'_, App>"));
        assert!(!ACTION_FIRST_VIEW.contains("cx.use_local"));
        assert!(!ACTION_FIRST_VIEW.contains("cx.on_action_notify_"));
        assert!(!ACTION_FIRST_VIEW.contains("cx.on_action_availability"));
    }

    #[test]
    fn progress_snippets_prefer_ui_cx_on_the_default_app_surface() {
        for source in [
            PROGRESS_USAGE,
            PROGRESS_LABEL,
            PROGRESS_RTL,
            PROGRESS_CONTROLLED,
            PROGRESS_DEMO,
        ] {
            assert!(source.contains("use fret::UiCx;"));
            assert!(source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement"));
            assert!(!source.contains("use fret_app::App;"));
            assert!(!source.contains("ElementContext<'_, App>"));
        }
    }

    #[test]
    fn combobox_snippets_prefer_ui_cx_on_the_default_app_surface() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets/combobox");
        let curated_paths = [
            "auto_highlight.rs",
            "basic.rs",
            "clear_button.rs",
            "conformance_demo.rs",
            "custom_items.rs",
            "disabled.rs",
            "groups.rs",
            "groups_with_separator.rs",
            "input_group.rs",
            "invalid.rs",
            "label.rs",
            "long_list.rs",
            "multiple_selection.rs",
            "rtl.rs",
            "trigger_button.rs",
            "usage.rs",
        ];

        for relative_path in curated_paths {
            let path = src_root.join(relative_path);
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement")
                    || source.contains("pub fn render(cx: &mut UiCx<'_>,")
                    || source.contains("pub fn render(\n    cx: &mut UiCx<'_>,"),
                "{} should expose UiCx on the app-facing snippet surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
    }

    #[test]
    fn slider_and_toast_snippets_prefer_ui_cx_on_the_default_app_surface() {
        for source in [SLIDER_USAGE, TOAST_DEPRECATED] {
            assert!(source.contains("use fret::UiCx;"));
            assert!(source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement"));
            assert!(!source.contains("use fret_app::App;"));
            assert!(!source.contains("ElementContext<'_, App>"));
        }
    }

    #[test]
    fn navigation_menu_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
        for source in [
            NAVIGATION_MENU_DEMO,
            NAVIGATION_MENU_DOCS_DEMO,
            NAVIGATION_MENU_RTL,
        ] {
            assert!(source.contains("use fret::UiCx;"));
            assert!(source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement"));
            assert!(!source.contains("use fret_app::App;"));
            assert!(!source.contains("ElementContext<'_, App>"));
        }
    }

    #[test]
    fn chart_snippets_prefer_ui_cx_on_the_default_app_surface() {
        for source in [
            CHART_CONTRACTS,
            CHART_DEMO,
            CHART_LEGEND,
            CHART_RTL,
            CHART_TOOLTIP,
            CHART_USAGE,
        ] {
            assert!(source.contains("use fret::UiCx;"));
            assert!(source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement"));
            assert!(!source.contains("use fret_app::App;"));
            assert!(!source.contains("ElementContext<'_, App>"));
        }
    }

    #[test]
    fn motion_preset_snippets_prefer_ui_cx_on_the_default_app_surface() {
        for source in [
            MOTION_PRESET_SELECTOR,
            MOTION_FLUID_TABS_DEMO,
            MOTION_OVERLAY_DEMO,
            MOTION_STACK_SHIFT_LIST_DEMO,
            MOTION_STAGGER_DEMO,
            MOTION_TOKEN_SNAPSHOT,
        ] {
            assert!(source.contains("use fret::UiCx;"));
            assert!(
                source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement")
                    || source.contains("pub fn render(cx: &mut UiCx<'_>,")
                    || source.contains("pub fn render(\n    cx: &mut UiCx<'_>,"),
            );
            assert!(!source.contains("use fret_app::App;"));
            assert!(!source.contains("ElementContext<'_, App>"));
        }
    }

    #[test]
    fn carousel_snippets_prefer_ui_cx_on_the_default_app_surface() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets/carousel");
        let mut paths = Vec::new();
        collect_rust_sources(&src_root, &mut paths);
        paths.sort();

        for path in paths {
            if path.file_name().is_some_and(|name| name == "mod.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement"),
                "{} should expose UiCx on the app-facing snippet surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
    }

    #[test]
    fn item_snippets_prefer_ui_cx_on_the_default_app_surface() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets/item");
        let mut paths = Vec::new();
        collect_rust_sources(&src_root, &mut paths);
        paths.sort();

        for path in paths {
            if path.file_name().is_some_and(|name| name == "mod.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement"),
                "{} should expose UiCx on the app-facing snippet surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
    }

    #[test]
    fn tabs_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets/tabs");
        let curated_paths = [
            "demo.rs",
            "disabled.rs",
            "extras.rs",
            "icons.rs",
            "line.rs",
            "list.rs",
            "rtl.rs",
            "vertical.rs",
            "vertical_line.rs",
        ];

        for relative_path in curated_paths {
            let path = src_root.join(relative_path);
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement"),
                "{} should expose UiCx on the app-facing snippet surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
    }

    #[test]
    fn card_snippets_prefer_ui_cx_on_the_default_app_surface() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets/card");
        let mut paths = Vec::new();
        collect_rust_sources(&src_root, &mut paths);
        paths.sort();

        for path in paths {
            if path.file_name().is_some_and(|name| name == "mod.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement")
                    || source.contains("pub fn render(cx: &mut UiCx<'_>,")
                    || source.contains("pub fn render(\n    cx: &mut UiCx<'_>,"),
                "{} should expose UiCx on the app-facing snippet surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
    }

    #[test]
    fn data_table_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets/data_table");
        let curated_paths = [
            "basic_demo.rs",
            "default_demo.rs",
            "guide_demo.rs",
            "rtl_demo.rs",
        ];

        for relative_path in curated_paths {
            let path = src_root.join(relative_path);
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement")
                    || source.contains("pub fn render(cx: &mut UiCx<'_>,")
                    || source.contains("pub fn render(\n    cx: &mut UiCx<'_>,"),
                "{} should expose UiCx on the app-facing snippet surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
    }

    #[test]
    fn table_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets/table");
        let curated_paths = ["actions.rs", "demo.rs", "footer.rs", "rtl.rs"];

        for relative_path in curated_paths {
            let path = src_root.join(relative_path);
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement"),
                "{} should expose UiCx on the app-facing snippet surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
    }

    #[test]
    fn remaining_app_facing_tail_snippets_prefer_ui_cx_on_the_default_app_surface() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets");
        let curated_paths = [
            "breadcrumb/responsive.rs",
            "date_picker/dropdowns.rs",
            "form/notes.rs",
            "sidebar/rtl.rs",
        ];

        for relative_path in curated_paths {
            let path = src_root.join(relative_path);
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("pub fn render(cx: &mut UiCx<'_>) -> AnyElement"),
                "{} should expose UiCx on the app-facing snippet surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
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

    #[test]
    fn curated_ai_doc_pages_prefer_ui_cx_on_the_default_app_surface() {
        let pages_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/pages");
        let mut paths = Vec::new();
        collect_rust_sources(&pages_root, &mut paths);
        paths.sort();

        for path in paths {
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !file_name.starts_with("ai_") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("cx: &mut UiCx<'_>"),
                "{} should expose UiCx on the app-facing page surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
    }

    #[test]
    fn non_ai_leaf_doc_pages_prefer_ui_cx_on_the_default_app_surface() {
        let pages_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/pages");
        let mut paths = Vec::new();
        collect_rust_sources(&pages_root, &mut paths);
        paths.sort();

        for path in paths {
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if path.parent() != Some(pages_root.as_path())
                || file_name == "mod.rs"
                || file_name.starts_with("ai_")
            {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("cx: &mut UiCx<'_>"),
                "{} should expose UiCx on the app-facing page surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
    }

    #[test]
    fn pages_mod_router_prefers_ui_cx_on_the_default_app_surface() {
        assert!(PAGES_MOD.contains("use fret::UiCx;"));
        assert!(PAGES_MOD.contains("cx: &mut UiCx<'_>"));
        assert!(!PAGES_MOD.contains("ElementContext<'_, App>"));
        assert!(!PAGES_MOD.contains("use fret_app::App;"));
    }

    #[test]
    fn material3_doc_pages_prefer_ui_cx_on_the_default_app_surface() {
        let pages_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/pages/material3");
        let mut paths = Vec::new();
        collect_rust_sources(&pages_root, &mut paths);
        paths.sort();

        for path in paths {
            if path.file_name().is_some_and(|name| name == "mod.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the default app helper context alias",
                path.display()
            );
            assert!(
                source.contains("cx: &mut UiCx<'_>") || source.contains("cx: &mut UiCx<'a>"),
                "{} should expose UiCx on the app-facing Material 3 page surface",
                path.display()
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>")
                    && !source.contains("ElementContext<'a, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
        }
    }

    #[test]
    fn gallery_ui_shell_helpers_prefer_ui_cx_on_the_default_app_surface() {
        for (label, source) in [("ui/content.rs", UI_CONTENT), ("ui/nav.rs", UI_NAV)] {
            assert!(
                source.contains("use fret::UiCx;"),
                "{label} should use the default app helper context alias"
            );
            assert!(
                source.contains("cx: &mut UiCx<'_>"),
                "{label} should expose UiCx on the app-facing gallery shell helper surface"
            );
            assert!(
                !source.contains("use fret_app::App;"),
                "{label} should not teach the raw app runtime name"
            );
            assert!(
                !source.contains("ElementContext<'_, App>"),
                "{label} reintroduced the raw ElementContext app surface"
            );
        }
    }

    #[test]
    fn material3_legacy_preview_tree_is_retired() {
        let root_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/material3.rs");
        let previews_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/material3");

        assert!(
            !root_path.exists(),
            "{} should stay deleted after the Material 3 page migration",
            root_path.display()
        );
        assert!(
            !previews_root.exists(),
            "{} should stay deleted after the Material 3 page migration",
            previews_root.display()
        );
    }

    #[test]
    fn magic_preview_prefers_ui_cx_on_the_internal_gallery_surface() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/magic.rs");
        assert!(UI_PREVIEWS_MAGIC.contains("use fret::UiCx;"));
        assert!(UI_PREVIEWS_MAGIC.contains("cx: &mut UiCx<'_>"));
        assert!(!UI_PREVIEWS_MAGIC.contains("use fret_app::App;"));
        assert!(!UI_PREVIEWS_MAGIC.contains("ElementContext<'_, App>"));
        assert!(
            !UI_PREVIEWS_MAGIC.contains("ElementContext<'a, App>"),
            "{} reintroduced the raw ElementContext app surface",
            path.display()
        );
    }

    #[test]
    fn component_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
        let components_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/pages/components");
        let mut paths = Vec::new();
        collect_rust_sources(&components_root, &mut paths);
        paths.sort();

        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>")
                    && !source.contains("ElementContext<'a, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );

            if source.contains("cx: &mut") || source.contains("FnOnce(&mut") {
                assert!(
                    source.contains("use fret::UiCx;"),
                    "{} should use the shared helper context alias",
                    path.display()
                );
                assert!(
                    source.contains("cx: &mut UiCx<'_>")
                        || source.contains("cx: &mut UiCx<'a>")
                        || source.contains("FnOnce(&mut UiCx<'_>)")
                        || source.contains("FnOnce(&mut UiCx<'a>)"),
                    "{} should expose UiCx on the internal component preview surface",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn harness_preview_shells_prefer_ui_cx_on_the_internal_gallery_surface() {
        let harness_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/pages/harness");
        let curated_paths = [
            "intro.rs",
            "layout.rs",
            "view_cache.rs",
            "hit_test_only_paint_cache_probe.rs",
            "ui_kit_list_torture.rs",
            "virtual_list_torture.rs",
        ];

        for relative_path in curated_paths {
            let path = harness_root.join(relative_path);
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>")
                    && !source.contains("ElementContext<'a, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );
            assert!(
                source.contains("use fret::UiCx;"),
                "{} should use the shared helper context alias",
                path.display()
            );
            assert!(
                source.contains("cx: &mut UiCx<'_>")
                    || source.contains("cx: &mut UiCx<'a>")
                    || source.contains("FnOnce(&mut UiCx<'_>)")
                    || source.contains("FnOnce(&mut UiCx<'a>)"),
                "{} should expose UiCx on the internal harness preview surface",
                path.display()
            );
        }
    }

    #[test]
    fn gallery_atom_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
        let atoms_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/gallery/atoms");
        let mut paths = Vec::new();
        collect_rust_sources(&atoms_root, &mut paths);
        paths.sort();

        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>")
                    && !source.contains("ElementContext<'a, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );

            if source.contains("cx: &mut") || source.contains("FnOnce(&mut") {
                assert!(
                    source.contains("use fret::UiCx;"),
                    "{} should use the shared helper context alias",
                    path.display()
                );
                assert!(
                    source.contains("cx: &mut UiCx<'_>")
                        || source.contains("cx: &mut UiCx<'a>")
                        || source.contains("FnOnce(&mut UiCx<'_>)")
                        || source.contains("FnOnce(&mut UiCx<'a>)"),
                    "{} should expose UiCx on the internal atom preview surface",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn gallery_form_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
        let forms_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/gallery/forms");
        let mut paths = Vec::new();
        collect_rust_sources(&forms_root, &mut paths);
        paths.sort();

        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>")
                    && !source.contains("ElementContext<'a, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );

            if source.contains("cx: &mut") || source.contains("FnOnce(&mut") {
                assert!(
                    source.contains("use fret::UiCx;"),
                    "{} should use the shared helper context alias",
                    path.display()
                );
                assert!(
                    source.contains("cx: &mut UiCx<'_>")
                        || source.contains("cx: &mut UiCx<'a>")
                        || source.contains("FnOnce(&mut UiCx<'_>)")
                        || source.contains("FnOnce(&mut UiCx<'a>)"),
                    "{} should expose UiCx on the internal form preview surface",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn gallery_data_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
        let data_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/gallery/data");
        let mut paths = Vec::new();
        collect_rust_sources(&data_root, &mut paths);
        paths.sort();

        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>")
                    && !source.contains("ElementContext<'a, App>")
                    && !source.contains("ElementContext<'_, H>")
                    && !source.contains("ElementContext<'a, H>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );

            if source.contains("cx: &mut") || source.contains("FnOnce(&mut") {
                assert!(
                    source.contains("use fret::UiCx;"),
                    "{} should use the shared helper context alias",
                    path.display()
                );
                assert!(
                    source.contains("cx: &mut UiCx<'_>")
                        || source.contains("cx: &mut UiCx<'a>")
                        || source.contains("FnOnce(&mut UiCx<'_>)")
                        || source.contains("FnOnce(&mut UiCx<'a>)"),
                    "{} should expose UiCx on the internal data preview surface",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn gallery_overlay_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
        let overlays_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/gallery/overlays");
        let mut paths = Vec::new();
        collect_rust_sources(&overlays_root, &mut paths);
        paths.sort();

        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>")
                    && !source.contains("ElementContext<'a, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );

            if source.contains("cx: &mut") || source.contains("FnOnce(&mut") {
                assert!(
                    source.contains("use fret::UiCx;"),
                    "{} should use the shared helper context alias",
                    path.display()
                );
                assert!(
                    source.contains("cx: &mut UiCx<'_>")
                        || source.contains("cx: &mut UiCx<'a>")
                        || source.contains("FnOnce(&mut UiCx<'_>)")
                        || source.contains("FnOnce(&mut UiCx<'a>)"),
                    "{} should expose UiCx on the internal overlay preview surface",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn editor_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
        let editors_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/pages/editors");
        let mut paths = Vec::new();
        collect_rust_sources(&editors_root, &mut paths);
        paths.sort();

        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>")
                    && !source.contains("ElementContext<'a, App>")
                    && !source.contains("ElementContext<'_, H>")
                    && !source.contains("ElementContext<'a, H>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );

            if source.contains("cx: &mut")
                || source.contains("FnOnce(&mut")
                || source.contains("Fn(&mut")
            {
                assert!(
                    source.contains("use fret::UiCx;"),
                    "{} should use the shared helper context alias",
                    path.display()
                );
                assert!(
                    source.contains("cx: &mut UiCx<'_>")
                        || source.contains("cx: &mut UiCx<'a>")
                        || source.contains("FnOnce(&mut UiCx<'_>)")
                        || source.contains("FnOnce(&mut UiCx<'a>)")
                        || source.contains("Fn(&mut UiCx<'_>)")
                        || source.contains("Fn(&mut UiCx<'a>)"),
                    "{} should expose UiCx on the internal editor preview surface",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn page_torture_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
        let torture_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/pages/torture");
        let mut paths = Vec::new();
        collect_rust_sources(&torture_root, &mut paths);
        paths.sort();

        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>")
                    && !source.contains("ElementContext<'a, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );

            if source.contains("cx: &mut")
                || source.contains("FnOnce(&mut")
                || source.contains("Fn(&mut")
            {
                assert!(
                    source.contains("use fret::UiCx;"),
                    "{} should use the shared helper context alias",
                    path.display()
                );
                assert!(
                    source.contains("cx: &mut UiCx<'_>")
                        || source.contains("cx: &mut UiCx<'a>")
                        || source.contains("FnOnce(&mut UiCx<'_>)")
                        || source.contains("FnOnce(&mut UiCx<'a>)")
                        || source.contains("Fn(&mut UiCx<'_>)")
                        || source.contains("Fn(&mut UiCx<'a>)"),
                    "{} should expose UiCx on the internal torture preview surface",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn gallery_torture_preview_modules_prefer_ui_cx_on_the_internal_gallery_surface() {
        let torture_root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/previews/gallery/torture");
        let mut paths = Vec::new();
        collect_rust_sources(&torture_root, &mut paths);
        paths.sort();

        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("use fret_app::App;"),
                "{} should not teach the raw app runtime name",
                path.display()
            );
            assert!(
                !source.contains("ElementContext<'_, App>")
                    && !source.contains("ElementContext<'a, App>"),
                "{} reintroduced the raw ElementContext app surface",
                path.display()
            );

            if source.contains("cx: &mut")
                || source.contains("FnOnce(&mut")
                || source.contains("Fn(&mut")
            {
                assert!(
                    source.contains("use fret::UiCx;"),
                    "{} should use the shared helper context alias",
                    path.display()
                );
                assert!(
                    source.contains("cx: &mut UiCx<'_>")
                        || source.contains("cx: &mut UiCx<'a>")
                        || source.contains("FnOnce(&mut UiCx<'_>)")
                        || source.contains("FnOnce(&mut UiCx<'a>)")
                        || source.contains("Fn(&mut UiCx<'_>)")
                        || source.contains("Fn(&mut UiCx<'a>)"),
                    "{} should expose UiCx on the internal gallery torture preview surface",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn gallery_source_tree_rejects_legacy_shadcn_alias_patterns() {
        for path in gallery_rust_sources() {
            if path.ends_with("src/lib.rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("fret_ui_fret_ui_shadcn::"),
                "{} duplicated an explicit fret_ui_shadcn path prefix",
                path.display()
            );
            assert!(
                !source.contains("use fret_ui_shadcn as shadcn;"),
                "{} reintroduced the legacy root alias import",
                path.display()
            );
            assert!(
                !source.contains("use fret_ui_shadcn::{self as shadcn"),
                "{} reintroduced the legacy self-as-shadcn import",
                path.display()
            );

            for (line_idx, line) in source.lines().enumerate() {
                for (offset, _) in line.match_indices("shadcn::") {
                    let previous = line[..offset].chars().next_back();
                    if previous.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
                        continue;
                    }

                    let after = &line[offset + "shadcn::".len()..];
                    let segment_len = after
                        .chars()
                        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                        .count();
                    if segment_len == 0 {
                        continue;
                    }

                    let segment = &after[..segment_len];
                    let is_module_path = after[segment_len..].starts_with("::");
                    let starts_with_lowercase = segment
                        .chars()
                        .next()
                        .is_some_and(|ch| ch.is_ascii_lowercase());
                    if is_module_path
                        && starts_with_lowercase
                        && !matches!(segment, "raw" | "themes" | "app")
                    {
                        panic!(
                            "{}:{} reintroduced non-curated shadcn module path `shadcn::{}::`",
                            path.display(),
                            line_idx + 1,
                            segment
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn gallery_ai_snippet_batch_prefers_curated_shadcn_facade_imports() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let curated_paths = [
            "ui/snippets/ai/audio_player_demo.rs",
            "ui/snippets/ai/canvas_world_layer_spike.rs",
            "ui/snippets/ai/checkpoint_demo.rs",
            "ui/snippets/ai/code_block_demo.rs",
            "ui/snippets/ai/commit_custom_children.rs",
            "ui/snippets/ai/confirmation_demo.rs",
            "ui/snippets/ai/confirmation_request.rs",
            "ui/snippets/ai/model_selector_demo.rs",
            "ui/snippets/ai/persona_demo.rs",
            "ui/snippets/ai/persona_state_management.rs",
            "ui/snippets/ai/plan_demo.rs",
            "ui/snippets/ai/prompt_input_docs_demo.rs",
            "ui/snippets/ai/prompt_input_referenced_sources_demo.rs",
            "ui/snippets/ai/reasoning_demo.rs",
            "ui/snippets/ai/speech_input_demo.rs",
            "ui/snippets/ai/task_demo.rs",
            "ui/snippets/ai/transcript_torture.rs",
            "ui/snippets/ai/workflow_canvas_demo.rs",
            "ui/snippets/ai/workflow_toolbar_demo.rs",
        ];

        for relative_path in curated_paths {
            let path = src_root.join(relative_path);
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"),
                "{} should import the curated shadcn facade",
                path.display()
            );

            for line in source.lines() {
                if !line.contains("fret_ui_shadcn::") {
                    continue;
                }

                assert_eq!(
                    line.trim(),
                    "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
                    "{} reintroduced a direct fret_ui_shadcn root path: {}",
                    path.display(),
                    line.trim()
                );
            }
        }
    }

    #[test]
    fn gallery_ai_snippet_tree_avoids_direct_shadcn_root_paths() {
        let ai_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/snippets/ai");
        let mut paths = Vec::new();
        collect_rust_sources(&ai_root, &mut paths);
        paths.sort();

        for path in paths {
            let source = std::fs::read_to_string(&path).unwrap();
            for (line_idx, line) in source.lines().enumerate() {
                if !line.contains("fret_ui_shadcn::") {
                    continue;
                }

                let trimmed = line.trim();
                let allowed_import = trimmed == "use fret_ui_shadcn::prelude::*;"
                    || trimmed == "use fret_ui_shadcn::{facade as shadcn, prelude::*};"
                    || (trimmed.starts_with("use fret_ui_shadcn::prelude::{")
                        && trimmed.ends_with("};"));
                assert!(
                    allowed_import,
                    "{}:{} reintroduced a direct fret_ui_shadcn root path outside allowed imports: {}",
                    path.display(),
                    line_idx + 1,
                    trimmed
                );
            }
        }
    }

    #[test]
    fn gallery_shadcn_extras_batch_uses_explicit_raw_escape_hatch() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let curated_paths = [
            "ui/snippets/shadcn_extras/announcement.rs",
            "ui/snippets/shadcn_extras/avatar_stack.rs",
            "ui/snippets/shadcn_extras/banner.rs",
            "ui/snippets/shadcn_extras/kanban.rs",
            "ui/snippets/shadcn_extras/marquee.rs",
            "ui/snippets/shadcn_extras/rating.rs",
            "ui/snippets/shadcn_extras/relative_time.rs",
            "ui/snippets/shadcn_extras/tags.rs",
            "ui/snippets/shadcn_extras/ticker.rs",
        ];

        for relative_path in curated_paths {
            let path = src_root.join(relative_path);
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"),
                "{} should import the curated shadcn facade",
                path.display()
            );
            assert!(
                source.contains("shadcn::raw::extras::"),
                "{} should use the explicit raw extras escape hatch",
                path.display()
            );

            for line in source.lines() {
                if !line.contains("fret_ui_shadcn::") {
                    continue;
                }

                assert_eq!(
                    line.trim(),
                    "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
                    "{} reintroduced a direct fret_ui_shadcn root path: {}",
                    path.display(),
                    line.trim()
                );
            }
        }
    }

    #[test]
    fn gallery_breadcrumb_primitive_batch_uses_explicit_raw_escape_hatch() {
        let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let curated_paths = [
            "ui/snippets/breadcrumb/demo.rs",
            "ui/snippets/breadcrumb/dropdown.rs",
            "ui/snippets/breadcrumb/link_component.rs",
            "ui/snippets/breadcrumb/responsive.rs",
            "ui/snippets/breadcrumb/rtl.rs",
            "ui/snippets/breadcrumb/usage.rs",
        ];

        for relative_path in curated_paths {
            let path = src_root.join(relative_path);
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                source.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"),
                "{} should import the curated shadcn facade",
                path.display()
            );
            assert!(
                source.contains("use shadcn::raw::breadcrumb::primitives as bc;"),
                "{} should use the explicit raw breadcrumb primitive escape hatch",
                path.display()
            );

            for line in source.lines() {
                if !line.contains("fret_ui_shadcn::") {
                    continue;
                }

                assert_eq!(
                    line.trim(),
                    "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
                    "{} reintroduced a direct fret_ui_shadcn root path: {}",
                    path.display(),
                    line.trim()
                );
            }
        }
    }
}
