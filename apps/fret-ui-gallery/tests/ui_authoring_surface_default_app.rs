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

fn assert_selected_page_helpers_prefer_ui_child(
    relative_path: &str,
    required_markers: &[&str],
    forbidden_markers: &[&str],
) {
    let path = manifest_path(relative_path);
    let source = read_path(&path);
    let normalized = source.split_whitespace().collect::<String>();

    assert!(
        normalized.contains("usefret::{UiChild,UiCx};"),
        "{} should import UiChild alongside UiCx on the page surface",
        path.display()
    );

    for marker in required_markers {
        let marker = marker.split_whitespace().collect::<String>();
        assert!(
            normalized.contains(&marker),
            "{} is missing marker `{}`",
            path.display(),
            marker
        );
    }

    for marker in forbidden_markers {
        let marker = marker.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains(&marker),
            "{} reintroduced legacy page helper marker `{}`",
            path.display(),
            marker
        );
    }
}

fn assert_selected_generic_helpers_prefer_into_ui_element(
    relative_path: &str,
    required_markers: &[&str],
    forbidden_markers: &[&str],
) {
    let path = manifest_path(relative_path);
    let source = read_path(&path);
    let normalized = source.split_whitespace().collect::<String>();

    for marker in required_markers {
        let marker = marker.split_whitespace().collect::<String>();
        assert!(
            normalized.contains(&marker),
            "{} is missing marker `{}`",
            path.display(),
            marker
        );
    }

    for marker in forbidden_markers {
        let marker = marker.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains(&marker),
            "{} reintroduced legacy helper marker `{}`",
            path.display(),
            marker
        );
    }
}

#[test]
fn gallery_sources_do_not_depend_on_the_legacy_fret_prelude() {
    let menubar = read("src/driver/menubar.rs");
    let action_first_view = read("src/ui/snippets/command/action_first_view.rs");
    let action_first_view_normalized = action_first_view.split_whitespace().collect::<String>();

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
    assert!(action_first_view_normalized.contains(
        "cx.actions().availability::<act::Ping>(|_host,_acx|CommandAvailability::Available);"
    ));
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
fn selected_tabs_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/tabs/demo.rs",
        &[
            "fn field(label: &'static str, model: Model<String>, a11y: &'static str, password: bool,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn field(label: &'static str, model: Model<String>, a11y: &'static str, password: bool,) -> AnyElement",
        ],
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
fn selected_card_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/card/meeting_notes.rs",
        &[
            "fn marker(cx: &mut UiCx<'_>, text: &'static str) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn item(cx: &mut UiCx<'_>, n: &'static str, content: &'static str, test_id: Option<&'static str>,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn marker(cx: &mut UiCx<'_>, text: &'static str) -> AnyElement",
            "fn item(cx: &mut UiCx<'_>, n: &'static str, content: &'static str, test_id: Option<&'static str>,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/card/compositions.rs",
        &[
            "fn cell(cx: &mut UiCx<'_>, test_id: &'static str, card: shadcn::Card,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &["fn cell(cx: &mut UiCx<'_>, test_id: &'static str, card: shadcn::Card,) -> AnyElement"],
    );
}

#[test]
fn selected_collapsible_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/collapsible/basic.rs",
        &[
            "fn rotated_lucide<H: UiHost>(cx: &mut ElementContext<'_, H>, id: &'static str, rotation_deg: f32,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn rotated_lucide<H: UiHost>(cx: &mut ElementContext<'_, H>, id: &'static str, rotation_deg: f32,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/collapsible/settings_panel.rs",
        &[
            "fn radius_input<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str, a11y: &'static str, value: Model<String>,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn radius_input<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str, a11y: &'static str, value: Model<String>,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/collapsible/rtl.rs",
        &[
            "fn details_collapsible<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id_prefix: &'static str, open: Option<Model<bool>>, label: &'static str, status: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn details_collapsible<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id_prefix: &'static str, open: Option<Model<bool>>, label: &'static str, status: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/collapsible/file_tree.rs",
        &[
            "fn rotated_lucide<H: UiHost>(cx: &mut ElementContext<'_, H>, id: &'static str, rotation_deg: f32,) -> impl IntoUiElement<H> + use<H>",
            "fn file_leaf<H: UiHost>(cx: &mut ElementContext<'_, H>, key: &'static str, label: &'static str,) -> impl IntoUiElement<H> + use<H>",
            "fn folder<H: UiHost>(cx: &mut ElementContext<'_, H>, key: &'static str, label: &'static str, open_model: Model<bool>, children: Vec<AnyElement>,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn rotated_lucide<H: UiHost>(cx: &mut ElementContext<'_, H>, id: &'static str, rotation_deg: f32,) -> AnyElement",
            "fn file_leaf<H: UiHost>(cx: &mut ElementContext<'_, H>, key: &'static str, label: &'static str,) -> AnyElement",
            "fn folder<H: UiHost>(cx: &mut ElementContext<'_, H>, key: &'static str, label: &'static str, open_model: Model<bool>, children: Vec<AnyElement>,) -> AnyElement",
        ],
    );
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
fn selected_ai_doc_page_helpers_prefer_uichild_over_anyelement() {
    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_persona_demo.rs",
        &[
            "fn states_notes(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "fn props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "fn lifecycle_notes(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        &[
            "fn states_notes(cx: &mut UiCx<'_>) -> AnyElement",
            "fn props_table(cx: &mut UiCx<'_>) -> AnyElement",
            "fn lifecycle_notes(cx: &mut UiCx<'_>) -> AnyElement",
        ],
    );

    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_commit_demo.rs",
        &[
            "fn file_status_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "fn parts_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        &[
            "fn file_status_table(cx: &mut UiCx<'_>) -> AnyElement",
            "fn parts_props_table(cx: &mut UiCx<'_>) -> AnyElement",
        ],
    );

    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_context_demo.rs",
        &["fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &["fn parts_table(cx: &mut UiCx<'_>) -> AnyElement"],
    );

    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_model_selector_demo.rs",
        &["fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &["fn parts_table(cx: &mut UiCx<'_>) -> AnyElement"],
    );

    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_voice_selector_demo.rs",
        &["fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &["fn parts_table(cx: &mut UiCx<'_>) -> AnyElement"],
    );

    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_mic_selector_demo.rs",
        &["fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &["fn parts_table(cx: &mut UiCx<'_>) -> AnyElement"],
    );

    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_checkpoint_demo.rs",
        &["fn checkpoint_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &["fn checkpoint_props_table(cx: &mut UiCx<'_>) -> AnyElement"],
    );

    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_shimmer_demo.rs",
        &["fn shimmer_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &["fn shimmer_props_table(cx: &mut UiCx<'_>) -> AnyElement"],
    );

    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_test_results_demo.rs",
        &[
            "fn status_colors_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "fn parts_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        &[
            "fn status_colors_table(cx: &mut UiCx<'_>) -> AnyElement",
            "fn parts_props_table(cx: &mut UiCx<'_>) -> AnyElement",
        ],
    );

    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_artifact_demo.rs",
        &["fn render_notes(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &["fn render_notes(cx: &mut UiCx<'_>) -> AnyElement"],
    );

    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/ai_chain_of_thought_demo.rs",
        &["fn chain_of_thought_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &["fn chain_of_thought_props_table(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn selected_badge_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/badge/demo.rs",
        "src/ui/snippets/badge/spinner.rs",
        "src/ui/snippets/badge/rtl.rs",
        "src/ui/snippets/badge/counts.rs",
        "src/ui/snippets/badge/colors.rs",
        "src/ui/snippets/badge/icon.rs",
        "src/ui/snippets/badge/variants.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["fn row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>"],
            &[
                "fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>) -> AnyElement",
            ],
        );
    }
}

#[test]
fn selected_avatar_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/avatar/with_badge.rs",
        "src/ui/snippets/avatar/fallback_only.rs",
        "src/ui/snippets/avatar/sizes.rs",
        "src/ui/snippets/avatar/badge_icon.rs",
        "src/ui/snippets/avatar/dropdown.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn wrap_row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F> where F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>",
            ],
            &[
                "fn wrap_row<H: UiHost>(cx: &mut ElementContext<'_, H>, children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>) -> AnyElement",
            ],
        );
    }

    for relative_path in [
        "src/ui/snippets/avatar/demo.rs",
        "src/ui/snippets/avatar/group.rs",
        "src/ui/snippets/avatar/group_count.rs",
        "src/ui/snippets/avatar/group_count_icon.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn avatar_with_image<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<ImageId>>, size: shadcn::AvatarSize, fallback_text: &'static str,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn avatar_with_image<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<ImageId>>, size: shadcn::AvatarSize, fallback_text: &'static str,) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/avatar/sizes.rs",
        &[
            "fn avatar_with_image<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<ImageId>>, size: shadcn::AvatarSize, fallback_text: &'static str, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn avatar_with_image<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<ImageId>>, size: shadcn::AvatarSize, fallback_text: &'static str, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/avatar/demo.rs",
        &[
            "fn avatar_with_badge<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<ImageId>>, fallback_text: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn avatar_with_badge<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<ImageId>>, fallback_text: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/avatar/with_badge.rs",
        &[
            "fn avatar_with_badge<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<fret_core::ImageId>>, size: shadcn::AvatarSize, badge: shadcn::AvatarBadge, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn avatar_with_badge<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<fret_core::ImageId>>, size: shadcn::AvatarSize, badge: shadcn::AvatarBadge, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/avatar/fallback_only.rs",
        &[
            "fn avatar_fallback_only<H: UiHost>(cx: &mut ElementContext<'_, H>, size: shadcn::AvatarSize, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn avatar_fallback_only<H: UiHost>(cx: &mut ElementContext<'_, H>, size: shadcn::AvatarSize, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/avatar/group.rs",
        &[
            "fn group<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<ImageId>>, size: shadcn::AvatarSize, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn group<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<ImageId>>, size: shadcn::AvatarSize, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/avatar/group_count.rs",
        &[
            "fn group_with_count<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<ImageId>>, size: shadcn::AvatarSize, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn group_with_count<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Model<Option<ImageId>>, size: shadcn::AvatarSize, test_id: &'static str,) -> AnyElement",
        ],
    );

    for relative_path in [
        "src/ui/snippets/avatar/group_count_icon.rs",
        "src/ui/snippets/avatar/badge_icon.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn icon<H: UiHost>(cx: &mut ElementContext<'_, H>, name: &'static str, size: Px, fg: ColorRef,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn icon<H: UiHost>(cx: &mut ElementContext<'_, H>, name: &'static str, size: Px, fg: ColorRef,) -> AnyElement",
            ],
        );
    }
}

#[test]
fn selected_button_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/button/demo.rs",
        "src/ui/snippets/button/link_render.rs",
        "src/ui/snippets/button/rtl.rs",
        "src/ui/snippets/button/loading.rs",
        "src/ui/snippets/button/with_icon.rs",
        "src/ui/snippets/button/variants.rs",
        "src/ui/snippets/button/button_group.rs",
        "src/ui/snippets/button/rounded.rs",
        "src/ui/snippets/button/size.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn wrap_row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F> where F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>",
            ],
            &[
                "fn wrap_row<H: UiHost>(cx: &mut ElementContext<'_, H>, children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/button/size.rs",
        &[
            "fn row<H: UiHost>(_cx: &mut ElementContext<'_, H>, label: &'static str, text_size: shadcn::ButtonSize, icon_size: shadcn::ButtonSize,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, text_size: shadcn::ButtonSize, icon_size: shadcn::ButtonSize,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_hover_card_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/hover_card/sides.rs",
        &[
            "fn card<H: UiHost>(cx: &mut ElementContext<'_, H>, side: shadcn::HoverCardSide, label: &'static str, trigger_test_id: &'static str, content_test_id: &'static str, root_test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn card<H: UiHost>(cx: &mut ElementContext<'_, H>, side: shadcn::HoverCardSide, label: &'static str, trigger_test_id: &'static str, content_test_id: &'static str, root_test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/hover_card/trigger_delays.rs",
        &[
            "fn demo_content<H: UiHost>(cx: &mut ElementContext<'_, H>, title: &'static str, desc: &'static str, joined: &'static str, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn demo_content<H: UiHost>(cx: &mut ElementContext<'_, H>, title: &'static str, desc: &'static str, joined: &'static str, test_id: &'static str,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_tooltip_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/tooltip/rtl.rs",
        "src/ui/snippets/tooltip/sides.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn make_tooltip<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, side: shadcn::TooltipSide, content: &'static str,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn make_tooltip<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, side: shadcn::TooltipSide, content: &'static str,) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/tooltip/rtl.rs",
        &[
            "fn make_tooltip_with_test_ids<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, trigger_test_id: &'static str, side: shadcn::TooltipSide, content: &'static str, panel_test_id: &'static str, text_test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn make_tooltip_with_test_ids<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, trigger_test_id: &'static str, side: shadcn::TooltipSide, content: &'static str, panel_test_id: &'static str, text_test_id: &'static str,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_context_menu_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/context_menu/basic.rs",
        "src/ui/snippets/context_menu/radio.rs",
        "src/ui/snippets/context_menu/checkboxes.rs",
        "src/ui/snippets/context_menu/groups.rs",
        "src/ui/snippets/context_menu/icons.rs",
        "src/ui/snippets/context_menu/shortcuts.rs",
        "src/ui/snippets/context_menu/destructive.rs",
        "src/ui/snippets/context_menu/rtl.rs",
        "src/ui/snippets/context_menu/submenu.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn trigger_surface<H: UiHost>(label: &'static str) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/context_menu/demo.rs",
        &[
            "fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>) -> impl IntoUiElement<H> + use<H>",
        ],
        &["fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn selected_combobox_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/combobox/long_list.rs",
        "src/ui/snippets/combobox/input_group.rs",
        "src/ui/snippets/combobox/trigger_button.rs",
        "src/ui/snippets/combobox/groups_with_separator.rs",
        "src/ui/snippets/combobox/groups.rs",
        "src/ui/snippets/combobox/disabled.rs",
        "src/ui/snippets/combobox/custom_items.rs",
        "src/ui/snippets/combobox/clear_button.rs",
        "src/ui/snippets/combobox/invalid.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn state_row(cx: &mut UiCx<'_>, text: Arc<str>, test_id: Arc<str>,) -> impl IntoUiElement<fret_app::App> + use<>",
            ],
            &["fn state_row(cx: &mut UiCx<'_>, text: Arc<str>, test_id: Arc<str>) -> AnyElement"],
        );
    }
}

#[test]
fn selected_combobox_state_rows_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/combobox/long_list.rs",
        "src/ui/snippets/combobox/input_group.rs",
        "src/ui/snippets/combobox/trigger_button.rs",
        "src/ui/snippets/combobox/groups_with_separator.rs",
        "src/ui/snippets/combobox/groups.rs",
        "src/ui/snippets/combobox/disabled.rs",
        "src/ui/snippets/combobox/custom_items.rs",
        "src/ui/snippets/combobox/clear_button.rs",
        "src/ui/snippets/combobox/invalid.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn state_rows(cx: &mut UiCx<'_>, value: &Model<Option<Arc<str>>>, query: &Model<String>, test_id_prefix: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            ],
            &[
                "fn state_rows(cx: &mut UiCx<'_>, value: &Model<Option<Arc<str>>>, query: &Model<String>, test_id_prefix: &'static str,) -> AnyElement",
            ],
        );
    }
}

#[test]
fn selected_pagination_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/pagination/simple.rs",
        "src/ui/snippets/pagination/usage.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["fn page_number<H: UiHost>(label: &'static str) -> impl IntoUiElement<H> + use<H>"],
            &[
                "fn page_number<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement",
            ],
        );
    }
}

#[test]
fn selected_carousel_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/carousel/basic.rs",
        "src/ui/snippets/carousel/sizes.rs",
        "src/ui/snippets/carousel/plugin_wheel_gestures.rs",
        "src/ui/snippets/carousel/spacing_responsive.rs",
        "src/ui/snippets/carousel/loop_carousel.rs",
        "src/ui/snippets/carousel/loop_downgrade_cannot_loop.rs",
        "src/ui/snippets/carousel/spacing.rs",
        "src/ui/snippets/carousel/sizes_thirds.rs",
        "src/ui/snippets/carousel/parts.rs",
        "src/ui/snippets/carousel/duration_embla.rs",
        "src/ui/snippets/carousel/rtl.rs",
        "src/ui/snippets/carousel/plugin_autoplay.rs",
        "src/ui/snippets/carousel/plugin_autoplay_controlled.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn slide_card(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual,) -> impl IntoUiElement<fret_app::App> + use<>",
                "fn slide(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual,) -> impl IntoUiElement<fret_app::App> + use<>",
            ],
            &[
                "fn slide_card(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual) -> AnyElement",
                "fn slide(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/carousel/options.rs",
        &[
            "fn slide_card(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &["fn slide_card(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual) -> AnyElement"],
    );

    for relative_path in [
        "src/ui/snippets/carousel/api.rs",
        "src/ui/snippets/carousel/plugin_autoplay_delays.rs",
        "src/ui/snippets/carousel/plugin_autoplay_stop_on_last_snap.rs",
        "src/ui/snippets/carousel/events.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn slide_card(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual,) -> impl IntoUiElement<fret_app::App> + use<>",
            ],
            &["fn slide_card(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual) -> AnyElement"],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/carousel/plugin_autoplay_stop_on_focus.rs",
        &[
            "fn slide(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &["fn slide(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual) -> AnyElement"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/carousel/usage.rs",
        &[
            "fn slide_card(cx: &mut UiCx<'_>, idx: usize) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn slide(cx: &mut UiCx<'_>, idx: usize) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn slide_card(cx: &mut UiCx<'_>, idx: usize) -> AnyElement",
            "fn slide(cx: &mut UiCx<'_>, idx: usize) -> AnyElement",
        ],
    );
}

#[test]
fn selected_skeleton_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/skeleton/avatar.rs",
        "src/ui/snippets/skeleton/rtl.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["fn round<H: UiHost>(size: f32) -> impl IntoUiElement<H> + use<H>"],
            &["fn round<H: UiHost>(cx: &mut ElementContext<'_, H>, size: f32) -> AnyElement"],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/skeleton/form.rs",
        &["fn row<H: UiHost>(label_w: Px) -> impl IntoUiElement<H> + use<H>"],
        &["fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, label_w: Px) -> AnyElement"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/skeleton/table.rs",
        &["fn row<H: UiHost>() -> impl IntoUiElement<H> + use<H>"],
        &["fn row<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn selected_popover_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/popover/basic.rs",
        "src/ui/snippets/popover/demo.rs",
        "src/ui/snippets/popover/with_form.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn centered<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B> where B: IntoUiElement<H>",
            ],
            &[
                "fn centered<H: UiHost>(cx: &mut ElementContext<'_, H>, body: AnyElement) -> AnyElement",
            ],
        );
    }
}

#[test]
fn selected_resizable_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/resizable/usage.rs",
        &[
            "fn panel<H: UiHost>(_cx: &mut ElementContext<'_, H>, label: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &["fn panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement"],
    );

    for relative_path in [
        "src/ui/snippets/resizable/vertical.rs",
        "src/ui/snippets/resizable/handle.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn box_group<H: UiHost, B>(cx: &mut ElementContext<'_, H>, layout: LayoutRefinement, body: B,) -> impl IntoUiElement<H> + use<H, B> where B: IntoUiElement<H>",
                "fn panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn box_group<H: UiHost>(cx: &mut ElementContext<'_, H>, layout: LayoutRefinement, body: AnyElement,) -> AnyElement",
                "fn panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/resizable/rtl.rs",
        &[
            "fn box_group<H: UiHost, B>(cx: &mut ElementContext<'_, H>, layout: LayoutRefinement, body: B,) -> impl IntoUiElement<H> + use<H, B> where B: IntoUiElement<H>",
            "fn panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, height: Option<Px>,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn box_group<H: UiHost>(cx: &mut ElementContext<'_, H>, layout: LayoutRefinement, body: AnyElement,) -> AnyElement",
            "fn panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, height: Option<Px>,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_scroll_area_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/scroll_area/nested_scroll_routing.rs",
        &[
            "fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, i: usize) -> impl IntoUiElement<H> + use<H>",
        ],
        &["fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, i: usize) -> AnyElement"],
    );
}

#[test]
fn selected_data_table_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/data_table/basic_demo.rs",
        "src/ui/snippets/data_table/default_demo.rs",
        "src/ui/snippets/data_table/guide_demo.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B> where B: IntoUiElement<fret_app::App>",
            ],
            &["fn align_end(cx: &mut UiCx<'_>, child: AnyElement) -> AnyElement"],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/data_table/default_demo.rs",
        &[
            "fn footer(cx: &mut UiCx<'_>, state: Model<TableState>, output: Model<TableViewOutput>,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn footer(cx: &mut UiCx<'_>, state: Model<TableState>, output: Model<TableViewOutput>,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/data_table/basic_demo.rs",
        &[
            "fn bottom_controls(cx: &mut UiCx<'_>, state: Model<TableState>, output: Model<TableViewOutput>,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn bottom_controls(cx: &mut UiCx<'_>, state: Model<TableState>, output: Model<TableViewOutput>,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/data_table/rtl_demo.rs",
        &[
            "fn bottom_controls(cx: &mut UiCx<'_>, state: Model<TableState>, output: Model<TableViewOutput>, lang: Lang,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn bottom_controls(cx: &mut UiCx<'_>, state: Model<TableState>, output: Model<TableViewOutput>, lang: Lang,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/data_table/rtl_demo.rs",
        &[
            "fn align_inline_start<B>(cx: &mut UiCx<'_>, child: B) -> impl IntoUiElement<fret_app::App> + use<B> where B: IntoUiElement<fret_app::App>",
        ],
        &["fn align_inline_start(cx: &mut UiCx<'_>, child: AnyElement) -> AnyElement"],
    );
}

#[test]
fn selected_table_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/table/demo.rs",
        "src/ui/snippets/table/footer.rs",
        "src/ui/snippets/table/rtl.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn make_invoice_table(rows: &[(&'static str, &'static str, &'static str, &'static str)], include_footer: bool, test_id: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            ],
            &[
                "fn make_invoice_table(rows: &[(&'static str, &'static str, &'static str, &'static str)], include_footer: bool, test_id: &'static str,) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/table/actions.rs",
        &[
            "fn align_end<B>(child: B) -> impl IntoUiElement<fret_app::App> + use<B> where B: IntoUiElement<fret_app::App>",
            "fn action_row(cx: &mut UiCx<'_>, product: &'static str, price: &'static str, open_model: Model<bool>, key: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn align_end(cx: &mut UiCx<'_>, child: AnyElement) -> AnyElement",
            "fn action_row(cx: &mut UiCx<'_>, product: &'static str, price: &'static str, open_model: Model<bool>, key: &'static str,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_dropdown_menu_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/dropdown_menu/mod.rs",
        &[
            "fn preview_frame<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B> where B: IntoUiElement<H>",
            "fn preview_frame_with<H: UiHost, F, B>(cx: &mut ElementContext<'_, H>, build: F,) -> impl IntoUiElement<H> + use<H, F, B> where F: FnOnce(&mut ElementContext<'_, H>) -> B, B: IntoUiElement<H>",
        ],
        &[
            "fn preview_frame<H: UiHost>(cx: &mut ElementContext<'_, H>, body: AnyElement) -> AnyElement",
            "fn preview_frame_with<H: UiHost>(cx: &mut ElementContext<'_, H>, build: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_ai_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/ai/context_default.rs",
        "src/ui/snippets/ai/context_demo.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn centered<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B> where B: IntoUiElement<H>",
            ],
            &[
                "fn centered<H: UiHost>(cx: &mut ElementContext<'_, H>, body: AnyElement) -> AnyElement",
            ],
        );
    }

    for relative_path in [
        "src/ui/snippets/ai/file_tree_basic.rs",
        "src/ui/snippets/ai/file_tree_expanded.rs",
        "src/ui/snippets/ai/file_tree_large.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "pub fn preview<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>,) -> impl IntoUiElement<H> + use<H>",
            ],
            &["pub fn preview<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/test_results_demo.rs",
        &[
            "fn progress_section<H: UiHost>(cx: &mut ElementContext<'_, H>) -> impl IntoUiElement<H> + use<H>",
        ],
        &["fn progress_section<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/attachments_usage.rs",
        &[
            "fn render_grid_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn render_grid_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/file_tree_demo.rs",
        &[
            "fn invisible_marker<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn invisible_marker<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/file_tree_large.rs",
        &[
            "fn invisible_marker<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn invisible_marker<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/speech_input_demo.rs",
        &[
            "fn body_text<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>, style: TextStyle, color: Color, align: TextAlign,) -> impl IntoUiElement<H> + use<H>",
            "fn clear_action<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, transcript: Model<String>,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn body_text<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>, style: TextStyle, color: Color, align: TextAlign,) -> AnyElement",
            "fn clear_action<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, transcript: Model<String>,) -> AnyElement",
        ],
    );

    for relative_path in [
        "src/ui/snippets/ai/attachments_grid.rs",
        "src/ui/snippets/ai/attachments_list.rs",
    ] {
        let helper = if relative_path.ends_with("attachments_grid.rs") {
            "fn render_grid_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData, on_remove: ui_ai::OnAttachmentRemove, test_id: Option<&'static str>, remove_test_id: Option<&'static str>,) -> impl IntoUiElement<H> + use<H>"
        } else {
            "fn render_list_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData, on_remove: ui_ai::OnAttachmentRemove, test_id: Option<&'static str>,) -> impl IntoUiElement<H> + use<H>"
        };

        let old_helper = if relative_path.ends_with("attachments_grid.rs") {
            "fn render_grid_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData, on_remove: ui_ai::OnAttachmentRemove, test_id: Option<&'static str>, remove_test_id: Option<&'static str>,) -> AnyElement"
        } else {
            "fn render_list_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData, on_remove: ui_ai::OnAttachmentRemove, test_id: Option<&'static str>,) -> AnyElement"
        };

        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[helper],
            &[old_helper],
        );
    }
}

#[test]
fn selected_breadcrumb_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/breadcrumb/dropdown.rs",
        &[
            "fn dot_separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> impl IntoUiElement<H> + use<H>",
        ],
        &["fn dot_separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn selected_button_group_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/button_group/api_reference.rs",
        &[
            "pub fn basic_button_group<H: UiHost>(cx: &mut ElementContext<'_, H>,) -> impl IntoUiElement<H> + use<H>",
            "pub fn button_group_with_separator<H: UiHost>(cx: &mut ElementContext<'_, H>,) -> impl IntoUiElement<H> + use<H>",
            "pub fn button_group_with_text<H: UiHost>(cx: &mut ElementContext<'_, H>,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "pub fn basic_button_group<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement",
            "pub fn button_group_with_separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement",
            "pub fn button_group_with_text<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement",
        ],
    );
}

#[test]
fn selected_toggle_group_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/toggle_group/size.rs",
        &[
            "fn group<H: UiHost>(cx: &mut ElementContext<'_, H>, size: shadcn::ToggleSize,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn group<H: UiHost>(cx: &mut ElementContext<'_, H>, size: shadcn::ToggleSize) -> AnyElement",
        ],
    );
}

#[test]
fn selected_drawer_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/drawer/demo.rs",
        &[
            "fn goal_adjust_button<H: UiHost>(goal: Model<i32>, adjustment: i32, icon: &'static str, a11y_label: &'static str, disabled: bool, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
            "fn goal_chart<H: UiHost>(cx: &mut ElementContext<'_, H>, goal: i32,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn goal_adjust_button<H: UiHost>(cx: &mut ElementContext<'_, H>, goal: Model<i32>, adjustment: i32, icon: &'static str, a11y_label: &'static str, disabled: bool, test_id: &'static str,) -> AnyElement",
            "fn goal_chart<H: UiHost>(cx: &mut ElementContext<'_, H>, goal: i32) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/drawer/responsive_dialog.rs",
        &[
            "fn profile_field<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, model: Model<String>, input_test_id: Option<&'static str>,) -> impl IntoUiElement<H> + use<H>",
            "fn profile_form<H: UiHost>(cx: &mut ElementContext<'_, H>, email: Model<String>, username: Model<String>, test_ids: Option<ProfileFormTestIds>,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn profile_field<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, model: Model<String>, input_test_id: Option<&'static str>,) -> AnyElement",
            "fn profile_form<H: UiHost>(cx: &mut ElementContext<'_, H>, email: Model<String>, username: Model<String>, test_ids: Option<ProfileFormTestIds>,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/drawer/sides.rs",
        &[
            "fn side_button<H: UiHost>(cx: &mut ElementContext<'_, H>, title: &'static str, direction: shadcn::DrawerDirection, open: Model<bool>, test_id_prefix: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn side_button<H: UiHost>(cx: &mut ElementContext<'_, H>, title: &'static str, direction: shadcn::DrawerDirection, open: Model<bool>, test_id_prefix: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/drawer/scrollable_content.rs",
        &[
            "fn paragraph_block<H: UiHost>(cx: &mut ElementContext<'_, H>, prefix: &'static str, rows: usize,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn paragraph_block<H: UiHost>(cx: &mut ElementContext<'_, H>, prefix: &'static str, rows: usize,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_sheet_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/sheet/demo.rs",
        "src/ui/snippets/sheet/rtl.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn profile_fields<H: UiHost>(cx: &mut ElementContext<'_, H>, name: Model<String>, username: Model<String>,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn profile_fields<H: UiHost>(cx: &mut ElementContext<'_, H>, name: Model<String>, username: Model<String>,) -> AnyElement",
            ],
        );
    }
}

#[test]
fn selected_separator_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/separator/menu.rs",
        &[
            "fn section<H: UiHost>(cx: &mut ElementContext<'_, H>, title: &'static str, description: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn section<H: UiHost>(cx: &mut ElementContext<'_, H>, title: &'static str, description: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/separator/list.rs",
        &[
            "fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, value: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, value: &'static str,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_sidebar_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/sidebar/demo.rs",
        "src/ui/snippets/sidebar/controlled.rs",
        "src/ui/snippets/sidebar/mobile.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn menu_button<H: UiHost>(cx: &mut ElementContext<'_, H>, selected_model: Model<Arc<str>>, active_value: &Arc<str>, value: &'static str, label: &'static str, icon: &'static str, test_id: Arc<str>,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn menu_button<H: UiHost>(cx: &mut ElementContext<'_, H>, selected_model: Model<Arc<str>>, active_value: &Arc<str>, value: &'static str, label: &'static str, icon: &'static str, test_id: Arc<str>,) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/sidebar/rtl.rs",
        &[
            "fn menu_button(cx: &mut UiCx<'_>, selected_model: Model<Arc<str>>, active_value: &Arc<str>, value: &'static str, label: &'static str, icon: &'static str, test_id: Arc<str>,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn menu_button(cx: &mut UiCx<'_>, selected_model: Model<Arc<str>>, active_value: &Arc<str>, value: &'static str, label: &'static str, icon: &'static str, test_id: Arc<str>,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_aspect_ratio_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/aspect_ratio/portrait.rs",
        "src/ui/snippets/aspect_ratio/square.rs",
        "src/ui/snippets/aspect_ratio/rtl.rs",
    ] {
        let image_helper = if relative_path.ends_with("portrait.rs") {
            "fn portrait_image<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>, content_test_id: &'static str,) -> impl IntoUiElement<H> + use<H>"
        } else if relative_path.ends_with("square.rs") {
            "fn square_image<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>, content_test_id: &'static str,) -> impl IntoUiElement<H> + use<H>"
        } else {
            "fn rtl_image<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>, content_test_id: &'static str,) -> impl IntoUiElement<H> + use<H>"
        };

        let image_helper_old = if relative_path.ends_with("portrait.rs") {
            "fn portrait_image<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>, content_test_id: &'static str,) -> AnyElement"
        } else if relative_path.ends_with("square.rs") {
            "fn square_image<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>, content_test_id: &'static str,) -> AnyElement"
        } else {
            "fn rtl_image<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>, content_test_id: &'static str,) -> AnyElement"
        };

        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                image_helper,
                "fn ratio_example<H: UiHost>(cx: &mut ElementContext<'_, H>, ratio: f32, max_w: Px, test_id: &'static str, content_test_id: &'static str, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                image_helper_old,
                "fn ratio_example<H: UiHost>(cx: &mut ElementContext<'_, H>, ratio: f32, max_w: Px, test_id: &'static str, content_test_id: &'static str, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> AnyElement",
            ],
        );
    }
}

#[test]
fn selected_dialog_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/dialog/demo.rs",
        "src/ui/snippets/dialog/rtl.rs",
        "src/ui/snippets/dialog/scrollable_content.rs",
        "src/ui/snippets/dialog/sticky_footer.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                if relative_path.ends_with("demo.rs") || relative_path.ends_with("rtl.rs") {
                    "fn profile_fields<H: UiHost>(cx: &mut ElementContext<'_, H>, name: Model<String>, username: Model<String>,) -> impl IntoUiElement<H> + use<H>"
                } else {
                    "fn lorem_block<H: UiHost>(cx: &mut ElementContext<'_, H>, prefix: &'static str, lines: usize,) -> impl IntoUiElement<H> + use<H>"
                },
            ],
            &[
                if relative_path.ends_with("demo.rs") || relative_path.ends_with("rtl.rs") {
                    "fn profile_fields<H: UiHost>(cx: &mut ElementContext<'_, H>, name: Model<String>, username: Model<String>,) -> AnyElement"
                } else {
                    "fn lorem_block<H: UiHost>(cx: &mut ElementContext<'_, H>, prefix: &'static str, lines: usize,) -> AnyElement"
                },
            ],
        );
    }
}

#[test]
fn selected_item_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/item/avatar.rs",
        &[
            "fn icon_button(cx: &mut UiCx<'_>, icon_id: &'static str, variant: shadcn::ButtonVariant, test_id: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn item_team(cx: &mut UiCx<'_>, test_id: &'static str, action_test_id: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn icon_button(cx: &mut UiCx<'_>, icon_id: &'static str, variant: shadcn::ButtonVariant, test_id: &'static str,) -> AnyElement",
            "fn item_team(cx: &mut UiCx<'_>, test_id: &'static str, action_test_id: &'static str) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/item/icon.rs",
        &[
            "fn icon(cx: &mut UiCx<'_>, id: &'static str) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn item_icon(cx: &mut UiCx<'_>, icon_id: &'static str, title: &'static str, description: &'static str, test_id: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn icon(cx: &mut UiCx<'_>, id: &'static str) -> AnyElement",
            "fn item_icon(cx: &mut UiCx<'_>, icon_id: &'static str, title: &'static str, description: &'static str, test_id: &'static str,) -> AnyElement",
        ],
    );

    for relative_path in [
        "src/ui/snippets/item/link.rs",
        "src/ui/snippets/item/link_render.rs",
        "src/ui/snippets/item/dropdown.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn icon(cx: &mut UiCx<'_>, id: &'static str) -> impl IntoUiElement<fret_app::App> + use<>",
            ],
            &["fn icon(cx: &mut UiCx<'_>, id: &'static str) -> AnyElement"],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/item/extras_rtl.rs",
        &[
            "fn outline_button_sm(cx: &mut UiCx<'_>, label: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn item_basic(cx: &mut UiCx<'_>, title: &'static str, description: &'static str, actions: Vec<AnyElement>, test_id: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn outline_button_sm(cx: &mut UiCx<'_>, label: &'static str) -> AnyElement",
            "fn item_basic(cx: &mut UiCx<'_>, title: &'static str, description: &'static str, actions: Vec<AnyElement>, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/item/gallery.rs",
        &[
            "fn icon(cx: &mut UiCx<'_>, id: &'static str) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn icon_button(cx: &mut UiCx<'_>, icon_id: &'static str, variant: shadcn::ButtonVariant, test_id: Arc<str>,) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn outline_button(cx: &mut UiCx<'_>, label: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn outline_button_sm(cx: &mut UiCx<'_>, label: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn item_basic(cx: &mut UiCx<'_>, variant: shadcn::ItemVariant, title: &'static str, description: Option<&'static str>, actions: Vec<AnyElement>, test_id: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn item_icon(cx: &mut UiCx<'_>, variant: shadcn::ItemVariant, icon_id: &'static str, title: &'static str, description: Option<&'static str>, actions: Vec<AnyElement>, test_id: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn item_avatar(cx: &mut UiCx<'_>, username: &'static str, message: &'static str, initials: &'static str, test_id: Arc<str>, add_action_test_id: Arc<str>,) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn item_team(cx: &mut UiCx<'_>, test_id: &'static str, action_test_id: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn icon(cx: &mut UiCx<'_>, id: &'static str) -> AnyElement",
            "fn icon_button(cx: &mut UiCx<'_>, icon_id: &'static str, variant: shadcn::ButtonVariant, test_id: Arc<str>,) -> AnyElement",
            "fn outline_button(cx: &mut UiCx<'_>, label: &'static str) -> AnyElement",
            "fn outline_button_sm(cx: &mut UiCx<'_>, label: &'static str) -> AnyElement",
            "fn item_basic(cx: &mut UiCx<'_>, variant: shadcn::ItemVariant, title: &'static str, description: Option<&'static str>, actions: Vec<AnyElement>, test_id: &'static str,) -> AnyElement",
            "fn item_icon(cx: &mut UiCx<'_>, variant: shadcn::ItemVariant, icon_id: &'static str, title: &'static str, description: Option<&'static str>, actions: Vec<AnyElement>, test_id: &'static str,) -> AnyElement",
            "fn item_avatar(cx: &mut UiCx<'_>, username: &'static str, message: &'static str, initials: &'static str, test_id: Arc<str>, add_action_test_id: Arc<str>,) -> AnyElement",
            "fn item_team(cx: &mut UiCx<'_>, test_id: &'static str, action_test_id: &'static str) -> AnyElement",
        ],
    );
}

#[test]
fn selected_toast_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/toast/deprecated.rs",
        &[
            "fn centered<B>(body: B) -> impl IntoUiElement<fret_app::App> + use<B> where B: IntoUiElement<fret_app::App>",
        ],
        &["fn centered(cx: &mut UiCx<'_>, body: AnyElement) -> AnyElement"],
    );
}

#[test]
fn selected_motion_presets_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/motion_presets/fluid_tabs_demo.rs",
        &[
            "fn panel(cx: &mut UiCx<'_>, title: &'static str, description: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn panel(cx: &mut UiCx<'_>, title: &'static str, description: &'static str) -> AnyElement",
        ],
    );
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
fn gallery_internal_wrapper_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/doc_layout.rs",
        &[
            "fn demo_shell<B>(cx: &mut UiCx<'_>, max_w: Px, body: B,) -> impl IntoUiElement<fret_app::App> + use<B> where B: IntoUiElement<fret_app::App>",
        ],
        &["fn demo_shell(cx: &mut UiCx<'_>, max_w: Px, body: AnyElement) -> AnyElement"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/previews/pages/editors/code_editor/mvp/gates.rs",
        &[
            "fn gate_panel<B>(cx: &mut UiCx<'_>, theme: &Theme, child: B,) -> impl IntoUiElement<fret_app::App> + use<B> where B: IntoUiElement<fret_app::App>",
        ],
        &["fn gate_panel(cx: &mut UiCx<'_>, theme: &Theme, child: AnyElement) -> AnyElement"],
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
