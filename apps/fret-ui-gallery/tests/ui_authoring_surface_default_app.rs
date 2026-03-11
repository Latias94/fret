mod support;

use support::{assert_default_app_surface, manifest_path, read, read_path, rust_sources};

fn assert_curated_default_app_paths(
    relative_paths: &[&str],
    expected_patterns: &[&str],
    surface_label: &str,
) {
    for relative_path in relative_paths {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        assert_default_app_surface(&path, &source, expected_patterns, surface_label);
    }
}

#[test]
fn gallery_sources_do_not_depend_on_the_legacy_fret_prelude() {
    let menubar = read("src/driver/menubar.rs");
    let action_first_view = read("src/ui/snippets/command/action_first_view.rs");

    assert!(!menubar.contains("fret::prelude"));
    assert!(menubar.contains("use fret::workspace_menu::{"));

    assert!(!action_first_view.contains("use fret::prelude::*;"));
    assert!(action_first_view.contains("use fret::advanced::prelude::*;"));
    assert!(action_first_view.contains("use fret::app::App;"));
    assert!(action_first_view.contains("fn init(_app: &mut App, _window: AppWindowId) -> Self"));
    assert!(!action_first_view.contains("ViewCx<'_, '_, App>"));
    assert!(!action_first_view.contains("ViewCx<'_, '_, KernelApp>"));
    assert!(!action_first_view.contains(") -> Elements {"));
    assert!(action_first_view.contains("fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui"));
    assert!(action_first_view.contains("cx.state().local::<u32>()"));
    assert!(action_first_view.contains("cx.actions().models::<act::Ping>"));
    assert!(action_first_view.contains("cx.actions().availability::<act::Ping>"));
    assert!(
        action_first_view.contains(
            "pub fn render(cx: &mut UiCx<'_>, last_action: Model<Arc<str>>) -> AnyElement"
        )
    );
    assert!(!action_first_view.contains("KernelApp"));
    assert!(!action_first_view.contains("ElementContext<'_, App>"));
    assert!(!action_first_view.contains("cx.use_local"));
    assert!(!action_first_view.contains("cx.on_action_notify_"));
    assert!(!action_first_view.contains("cx.on_action_availability"));
}

#[test]
fn progress_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/progress/usage.rs",
            "src/ui/snippets/progress/label.rs",
            "src/ui/snippets/progress/rtl.rs",
            "src/ui/snippets/progress/controlled.rs",
            "src/ui/snippets/progress/demo.rs",
        ],
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
        "app-facing snippet surface",
    );
}

#[test]
fn combobox_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/combobox/auto_highlight.rs",
            "src/ui/snippets/combobox/basic.rs",
            "src/ui/snippets/combobox/clear_button.rs",
            "src/ui/snippets/combobox/conformance_demo.rs",
            "src/ui/snippets/combobox/custom_items.rs",
            "src/ui/snippets/combobox/disabled.rs",
            "src/ui/snippets/combobox/groups.rs",
            "src/ui/snippets/combobox/groups_with_separator.rs",
            "src/ui/snippets/combobox/input_group.rs",
            "src/ui/snippets/combobox/invalid.rs",
            "src/ui/snippets/combobox/label.rs",
            "src/ui/snippets/combobox/long_list.rs",
            "src/ui/snippets/combobox/multiple_selection.rs",
            "src/ui/snippets/combobox/rtl.rs",
            "src/ui/snippets/combobox/trigger_button.rs",
            "src/ui/snippets/combobox/usage.rs",
        ],
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> AnyElement",
            "pub fn render(cx: &mut UiCx<'_>,",
            "pub fn render(\n    cx: &mut UiCx<'_>,",
        ],
        "app-facing snippet surface",
    );
}

#[test]
fn slider_and_toast_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/slider/usage.rs",
            "src/ui/snippets/toast/deprecated.rs",
        ],
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
        "app-facing snippet surface",
    );
}

#[test]
fn navigation_menu_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/navigation_menu/demo.rs",
            "src/ui/snippets/navigation_menu/docs_demo.rs",
            "src/ui/snippets/navigation_menu/rtl.rs",
        ],
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
        "app-facing snippet surface",
    );
}

#[test]
fn chart_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/chart/contracts.rs",
            "src/ui/snippets/chart/demo.rs",
            "src/ui/snippets/chart/legend.rs",
            "src/ui/snippets/chart/rtl.rs",
            "src/ui/snippets/chart/tooltip.rs",
            "src/ui/snippets/chart/usage.rs",
        ],
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
        "app-facing snippet surface",
    );
}

#[test]
fn motion_preset_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/motion_presets/preset_selector.rs",
            "src/ui/snippets/motion_presets/fluid_tabs_demo.rs",
            "src/ui/snippets/motion_presets/overlay_demo.rs",
            "src/ui/snippets/motion_presets/stack_shift_list_demo.rs",
            "src/ui/snippets/motion_presets/stagger_demo.rs",
            "src/ui/snippets/motion_presets/token_snapshot.rs",
        ],
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> AnyElement",
            "pub fn render(cx: &mut UiCx<'_>,",
            "pub fn render(\n    cx: &mut UiCx<'_>,",
        ],
        "app-facing snippet surface",
    );
}

#[test]
fn carousel_snippets_prefer_ui_cx_on_the_default_app_surface() {
    for path in rust_sources("src/ui/snippets/carousel") {
        if path.file_name().is_some_and(|name| name == "mod.rs") {
            continue;
        }

        let source = read_path(&path);
        assert_default_app_surface(
            &path,
            &source,
            &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
            "app-facing snippet surface",
        );
    }
}

#[test]
fn item_snippets_prefer_ui_cx_on_the_default_app_surface() {
    for path in rust_sources("src/ui/snippets/item") {
        if path.file_name().is_some_and(|name| name == "mod.rs") {
            continue;
        }

        let source = read_path(&path);
        assert_default_app_surface(
            &path,
            &source,
            &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
            "app-facing snippet surface",
        );
    }
}

#[test]
fn tabs_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/tabs/demo.rs",
            "src/ui/snippets/tabs/disabled.rs",
            "src/ui/snippets/tabs/extras.rs",
            "src/ui/snippets/tabs/icons.rs",
            "src/ui/snippets/tabs/line.rs",
            "src/ui/snippets/tabs/list.rs",
            "src/ui/snippets/tabs/rtl.rs",
            "src/ui/snippets/tabs/vertical.rs",
            "src/ui/snippets/tabs/vertical_line.rs",
        ],
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
        "app-facing snippet surface",
    );
}

#[test]
fn card_snippets_prefer_ui_cx_on_the_default_app_surface() {
    for path in rust_sources("src/ui/snippets/card") {
        if path.file_name().is_some_and(|name| name == "mod.rs") {
            continue;
        }

        let source = read_path(&path);
        assert_default_app_surface(
            &path,
            &source,
            &[
                "pub fn render(cx: &mut UiCx<'_>) -> AnyElement",
                "pub fn render(cx: &mut UiCx<'_>,",
                "pub fn render(\n    cx: &mut UiCx<'_>,",
            ],
            "app-facing snippet surface",
        );
    }
}

#[test]
fn data_table_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/data_table/basic_demo.rs",
            "src/ui/snippets/data_table/default_demo.rs",
            "src/ui/snippets/data_table/guide_demo.rs",
            "src/ui/snippets/data_table/rtl_demo.rs",
        ],
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> AnyElement",
            "pub fn render(cx: &mut UiCx<'_>,",
            "pub fn render(\n    cx: &mut UiCx<'_>,",
        ],
        "app-facing snippet surface",
    );
}

#[test]
fn table_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/table/actions.rs",
            "src/ui/snippets/table/demo.rs",
            "src/ui/snippets/table/footer.rs",
            "src/ui/snippets/table/rtl.rs",
        ],
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
        "app-facing snippet surface",
    );
}

#[test]
fn remaining_app_facing_tail_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/breadcrumb/responsive.rs",
            "src/ui/snippets/date_picker/dropdowns.rs",
            "src/ui/snippets/form/notes.rs",
            "src/ui/snippets/sidebar/rtl.rs",
        ],
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
        "app-facing snippet surface",
    );
}

#[test]
fn curated_ai_doc_pages_prefer_ui_cx_on_the_default_app_surface() {
    for path in rust_sources("src/ui/pages") {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with("ai_") {
            continue;
        }

        let source = read_path(&path);
        assert_default_app_surface(
            &path,
            &source,
            &["cx: &mut UiCx<'_>"],
            "app-facing page surface",
        );
    }
}

#[test]
fn non_ai_leaf_doc_pages_prefer_ui_cx_on_the_default_app_surface() {
    let pages_root = manifest_path("src/ui/pages");

    for path in rust_sources("src/ui/pages") {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if path.parent() != Some(pages_root.as_path())
            || file_name == "mod.rs"
            || file_name.starts_with("ai_")
        {
            continue;
        }

        let source = read_path(&path);
        assert_default_app_surface(
            &path,
            &source,
            &["cx: &mut UiCx<'_>"],
            "app-facing page surface",
        );
    }
}

#[test]
fn pages_mod_router_prefers_ui_cx_on_the_default_app_surface() {
    let path = manifest_path("src/ui/pages/mod.rs");
    let source = read_path(&path);
    assert_default_app_surface(
        &path,
        &source,
        &["cx: &mut UiCx<'_>"],
        "app-facing page surface",
    );
}

#[test]
fn material3_doc_pages_prefer_ui_cx_on_the_default_app_surface() {
    for path in rust_sources("src/ui/pages/material3") {
        if path.file_name().is_some_and(|name| name == "mod.rs") {
            continue;
        }

        let source = read_path(&path);
        assert_default_app_surface(
            &path,
            &source,
            &["cx: &mut UiCx<'_>", "cx: &mut UiCx<'a>"],
            "app-facing Material 3 page surface",
        );
    }
}

#[test]
fn gallery_ui_shell_helpers_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &["src/ui/content.rs", "src/ui/nav.rs"],
        &["cx: &mut UiCx<'_>"],
        "app-facing gallery shell helper surface",
    );
}

#[test]
fn material3_legacy_preview_tree_is_retired() {
    let root_path = manifest_path("src/ui/previews/material3.rs");
    let previews_root = manifest_path("src/ui/previews/material3");

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
