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

fn assert_sources_absent(relative_root: &str, forbidden_markers: &[&str]) {
    for path in rust_sources(relative_root) {
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();

        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "{} reintroduced forbidden source marker `{}`",
                path.display(),
                marker
            );
        }
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
fn selected_navigation_menu_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/navigation_menu/docs_demo.rs",
        &[
            "fn list_item(cx: &mut UiCx<'_>, muted_foreground: Color, model: Model<Option<Arc<str>>>, title: &'static str, description: &'static str, test_id: &'static str, command: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn icon_row(cx: &mut UiCx<'_>, model: Model<Option<Arc<str>>>, icon: &'static str, label: &'static str, test_id: &'static str, command: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn list_item(cx: &mut UiCx<'_>, muted_foreground: Color, model: Model<Option<Arc<str>>>, title: &'static str, description: &'static str, test_id: &'static str, command: &'static str,) -> AnyElement",
            "fn icon_row(cx: &mut UiCx<'_>, model: Model<Option<Arc<str>>>, icon: &'static str, label: &'static str, test_id: &'static str, command: &'static str,) -> AnyElement",
        ],
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
            "fn cell<T>(test_id: &'static str, card: T) -> impl IntoUiElement<fret_app::App> + use<T> where T: IntoUiElement<fret_app::App>",
        ],
        &[
            "fn cell(cx: &mut UiCx<'_>, test_id: &'static str, card: shadcn::Card,) -> AnyElement",
            "fn cell(cx: &mut UiCx<'_>, test_id: &'static str, card: shadcn::Card,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/card/demo.rs",
        &[
            "fn email_field(email: Model<String>) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn password_field(password: Model<String>) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn email_field(email: Model<String>) -> AnyElement",
            "fn password_field(password: Model<String>) -> AnyElement",
        ],
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
            "fn states_notes(_cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "fn props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "fn lifecycle_notes(_cx: &mut UiCx<'_>) -> impl UiChild + use<>",
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
fn selected_material3_page_helpers_prefer_uichild_over_anyelement() {
    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/pages/material3/shared.rs",
        &[
            "fn material3_variant_toggle_row(cx: &mut UiCx<'_>, material3_expressive: Model<bool>,) -> impl UiChild + use<>",
        ],
        &[
            "fn material3_variant_toggle_row(cx: &mut UiCx<'_>, material3_expressive: Model<bool>,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_material3_wrapper_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/material3/shared.rs",
        &[
            "fn render_material3_demo_page<D>(cx: &mut UiCx<'_>, intro: Option<&'static str>, demo: D, source: &'static str,) -> Vec<AnyElement> where D: IntoUiElement<fret_app::App>",
        ],
        &[
            "fn render_material3_demo_page(cx: &mut UiCx<'_>, intro: Option<&'static str>, demo: AnyElement, source: &'static str,) -> Vec<AnyElement>",
        ],
    );
}

#[test]
fn selected_doc_pages_prefer_docsection_build_for_typed_previews() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/aspect_ratio.rs",
        &[
            "let demo = snippets::demo::render_preview(cx, wide_image.clone());",
            "let square = snippets::square::render_preview(cx, square_image);",
            "let portrait = snippets::portrait::render_preview(cx, tall_image);",
            "let rtl = snippets::rtl::render_preview(cx, wide_image);",
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Square\", square)",
            "DocSection::build(cx, \"Portrait\", portrait)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "let demo = snippets::demo::render_preview(cx, wide_image.clone()).into_element(cx);",
            "let square = snippets::square::render_preview(cx, square_image).into_element(cx);",
            "let portrait = snippets::portrait::render_preview(cx, tall_image).into_element(cx);",
            "let rtl = snippets::rtl::render_preview(cx, wide_image).into_element(cx);",
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Square\", square)",
            "DocSection::new(\"Portrait\", portrait)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_artifact_demo.rs",
        &[
            "DocSection::build(cx, \"With Code Display\", snippets::artifact_code_display::render(cx))",
            "DocSection::build(cx, \"Close Toggle\", snippets::artifact_demo::render(cx))",
            "DocSection::build(cx, \"Notes\", render_notes(cx))",
        ],
        &[
            "DocSection::new(\"With Code Display\", snippets::artifact_code_display::render(cx))",
            "DocSection::new(\"Close Toggle\", snippets::artifact_demo::render(cx))",
            "DocSection::new(\"Notes\", render_notes(cx).into_element(cx))",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_model_selector_demo.rs",
        &[
            "let parts = parts_table(cx);",
            "DocSection::build(cx, \"Parts & Props\", parts)",
        ],
        &[
            "let parts = parts_table(cx);let parts = parts.into_element(cx);",
            "DocSection::new(\"Parts & Props\", parts)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_mic_selector_demo.rs",
        &[
            "let parts = parts_table(cx);",
            "DocSection::build(cx, \"Parts & Props\", parts)",
        ],
        &[
            "let parts = parts_table(cx);let parts = parts.into_element(cx);",
            "DocSection::new(\"Parts & Props\", parts)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_voice_selector_demo.rs",
        &[
            "let parts = parts_table(cx);",
            "DocSection::build(cx, \"Parts & Props\", parts)",
        ],
        &[
            "let parts = parts_table(cx);let parts = parts.into_element(cx);",
            "DocSection::new(\"Parts & Props\", parts)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_context_demo.rs",
        &[
            "let parts = parts_table(cx);",
            "DocSection::build(cx, \"Parts & Props\", parts)",
        ],
        &[
            "let parts = parts_table(cx);let parts = parts.into_element(cx);",
            "DocSection::new(\"Parts & Props\", parts)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_file_tree_demo.rs",
        &[
            "let basic = snippets::file_tree_basic::preview(cx);",
            "let expanded = snippets::file_tree_expanded::preview(cx);",
            "let large = snippets::file_tree_large::preview(cx);",
            "DocSection::build(cx, \"Basic Usage\", basic)",
            "DocSection::build(cx, \"Default Expanded\", expanded)",
            "DocSection::build(cx, \"Large (Virtualized)\", large)",
        ],
        &[
            "let basic = snippets::file_tree_basic::preview(cx).into_element(cx);",
            "let expanded = snippets::file_tree_expanded::preview(cx).into_element(cx);",
            "DocSection::new(\"Basic Usage\", basic)",
            "DocSection::new(\"Default Expanded\", expanded)",
            "DocSection::new(\"Large (Virtualized)\", large)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_commit_demo.rs",
        &[
            "let file_status = file_status_table(cx);",
            "let props = parts_props_table(cx);",
            "DocSection::build(cx, \"File Status\", file_status)",
            "DocSection::build(cx, \"Parts & Props\", props)",
        ],
        &[
            "let file_status = file_status_table(cx);let file_status = file_status.into_element(cx);",
            "let props = parts_props_table(cx);let props = props.into_element(cx);",
            "DocSection::new(\"File Status\", file_status)",
            "DocSection::new(\"Parts & Props\", props)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_test_results_demo.rs",
        &[
            "let status_colors = status_colors_table(cx);",
            "let props = parts_props_table(cx);",
            "DocSection::build(cx, \"Status Colors\", status_colors)",
            "DocSection::build(cx, \"Parts & Props\", props)",
        ],
        &[
            "let status_colors = status_colors_table(cx);let status_colors = status_colors.into_element(cx);",
            "let props = parts_props_table(cx);let props = props.into_element(cx);",
            "DocSection::new(\"Status Colors\", status_colors)",
            "DocSection::new(\"Parts & Props\", props)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_persona_demo.rs",
        &[
            "fn states_notes(_cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "fn lifecycle_notes(_cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "crate::ui::doc_layout::notes_block([",
            "let states = states_notes(cx);",
            "let props = props_table(cx);",
            "let lifecycle = lifecycle_notes(cx);",
            "DocSection::build(cx, \"States\", states)",
            "DocSection::build(cx, \"Props & Extensions\", props)",
            "DocSection::build(cx, \"Lifecycle & Ownership\", lifecycle)",
        ],
        &[
            "crate::ui::doc_layout::notes(",
            "let states = states_notes(cx);let states = states.into_element(cx);",
            "let props = props_table(cx);let props = props.into_element(cx);",
            "let lifecycle = lifecycle_notes(cx);let lifecycle = lifecycle.into_element(cx);",
            "DocSection::new(\"States\", states)",
            "DocSection::new(\"Props & Extensions\", props)",
            "DocSection::new(\"Lifecycle & Ownership\", lifecycle)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_checkpoint_demo.rs",
        &[
            "let props = checkpoint_props_table(cx).test_id(\"ui-gallery-ai-checkpoint-props\");",
            "DocSection::build(cx, \"Props\", props)",
        ],
        &[
            "let props = checkpoint_props_table(cx);let props = props.into_element(cx).test_id(\"ui-gallery-ai-checkpoint-props\");",
            "DocSection::new(\"Props\", props)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_chain_of_thought_demo.rs",
        &[
            "let props = chain_of_thought_props_table(cx).test_id(\"ui-gallery-ai-chain-of-thought-props\");",
            "DocSection::build(cx, \"Props\", props)",
        ],
        &[
            "let props = chain_of_thought_props_table(cx);let props = props.into_element(cx).test_id(\"ui-gallery-ai-chain-of-thought-props\");",
            "DocSection::new(\"Props\", props)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_shimmer_demo.rs",
        &[
            "let props = shimmer_props_table(cx).test_id(\"ui-gallery-ai-shimmer-props\");",
            "DocSection::build(cx, \"Props\", props)",
        ],
        &[
            "let props = shimmer_props_table(cx);let props = props.into_element(cx).test_id(\"ui-gallery-ai-shimmer-props\");",
            "DocSection::new(\"Props\", props)",
        ],
    );
}

#[test]
fn selected_doc_pages_prefer_docsection_build_for_typed_notes_blocks() {
    for relative_path in [
        "src/ui/pages/ai_agent_demo.rs",
        "src/ui/pages/ai_attachments_demo.rs",
        "src/ui/pages/ai_confirmation_demo.rs",
        "src/ui/pages/ai_inline_citation_demo.rs",
        "src/ui/pages/ai_message_demo.rs",
        "src/ui/pages/ai_speech_input_demo.rs",
        "src/ui/pages/ai_stack_trace_demo.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["notes_block([", "DocSection::build(cx, \"Notes\", notes)"],
            &["notes(cx,", "DocSection::new(\"Notes\", notes)"],
        );
    }

    for relative_path in [
        "src/ui/pages/ai_model_selector_demo.rs",
        "src/ui/pages/ai_mic_selector_demo.rs",
        "src/ui/pages/ai_voice_selector_demo.rs",
        "src/ui/pages/ai_context_demo.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "let features = doc_layout::notes_block([",
                "let notes = doc_layout::notes_block([",
                "DocSection::build(cx, \"Features\", features)",
                "DocSection::build(cx, \"Notes\", notes)",
            ],
            &[
                "let features = doc_layout::notes(",
                "let notes = doc_layout::notes(",
                "DocSection::new(\"Features\", features)",
                "DocSection::new(\"Notes\", notes)",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_commit_demo.rs",
        &[
            "let features = doc_layout::notes_block([",
            "let findings = doc_layout::notes_block([",
            "DocSection::build(cx, \"Features\", features)",
            "DocSection::build(cx, \"Notes\", findings)",
        ],
        &[
            "let features = doc_layout::notes(",
            "let findings = doc_layout::notes(",
            "DocSection::new(\"Features\", features)",
            "DocSection::new(\"Notes\", findings)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_test_results_demo.rs",
        &[
            "let features = crate::ui::doc_layout::notes_block([",
            "DocSection::build(cx, \"Features\", features)",
        ],
        &[
            "let features = crate::ui::doc_layout::notes(",
            "DocSection::new(\"Features\", features)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_chain_of_thought_demo.rs",
        &[
            "let features = doc_layout::notes_block([",
            "DocSection::build(cx, \"Features\", features)",
        ],
        &[
            "let features = doc_layout::notes(",
            "DocSection::new(\"Features\", features)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_shimmer_demo.rs",
        &[
            "let features = doc_layout::notes_block([",
            "DocSection::build(cx, \"Features\", features)",
        ],
        &[
            "let features = doc_layout::notes(",
            "DocSection::new(\"Features\", features)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/ai_checkpoint_demo.rs",
        &[
            "let features = doc_layout::notes_block([",
            "let customizable_icon = doc_layout::notes_block([",
            "let manual_checkpoints = doc_layout::notes_block([",
            "let automatic_checkpoints = doc_layout::notes_block([",
            "let branching = doc_layout::notes_block([",
            "DocSection::build(cx, \"Features\", features)",
            "DocSection::build(cx, \"Customizable Icon\", customizable_icon)",
            "DocSection::build(cx, \"Manual Checkpoints\", manual_checkpoints)",
            "DocSection::build(cx, \"Automatic Checkpoints\", automatic_checkpoints)",
            "DocSection::build(cx, \"Branching Conversations\", branching)",
        ],
        &[
            "let features = notes(",
            "let customizable_icon = notes(",
            "let manual_checkpoints = notes(",
            "let automatic_checkpoints = notes(",
            "let branching = notes(",
            "DocSection::new(\"Features\", features)",
            "DocSection::new(\"Customizable Icon\", customizable_icon)",
            "DocSection::new(\"Manual Checkpoints\", manual_checkpoints)",
            "DocSection::new(\"Automatic Checkpoints\", automatic_checkpoints)",
            "DocSection::new(\"Branching Conversations\", branching)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/avatar.rs",
        &[
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "DocSection::build(cx, \"API Reference\", api_reference)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/button.rs",
        &[
            "let cursor = doc_layout::notes_block([",
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "DocSection::build(cx, \"Cursor\", cursor)",
            "DocSection::build(cx, \"API Reference\", api_reference)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let cursor = doc_layout::notes(",
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"Cursor\", cursor)",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/button_group.rs",
        &[
            "let vs_toggle_group = doc_layout::notes_block([",
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "DocSection::build(cx, \"ButtonGroup vs ToggleGroup\", vs_toggle_group)",
            "DocSection::build(cx, \"API Reference\", api_reference)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let vs_toggle_group = doc_layout::notes(",
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"ButtonGroup vs ToggleGroup\", vs_toggle_group)",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/alert_dialog.rs",
        &[
            "let api_reference = doc_layout::notes_block([",
            "let extras = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            "let extras = DocSection::build(cx, \"Fret Extras\", extras)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let api_reference = doc_layout::notes(",
            "let extras = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Fret Extras\", extras)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/hover_card.rs",
        &[
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/dropdown_menu.rs",
        &[
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/calendar.rs",
        &[
            "let about = doc_layout::notes_block([",
            "let date_picker = doc_layout::notes_block([",
            "let selected_date_timezone = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let about = DocSection::build(cx, \"About\", about)",
            "let date_picker = DocSection::build(cx, \"Date Picker\", date_picker)",
            "let selected_date_timezone = DocSection::build(cx, \"Selected Date (With TimeZone)\", selected_date_timezone)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let about = doc_layout::notes(",
            "let date_picker = doc_layout::notes(",
            "let selected_date_timezone = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"About\", about)",
            "DocSection::new(\"Date Picker\", date_picker)",
            "DocSection::new(\"Selected Date (With TimeZone)\", selected_date_timezone)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    for relative_path in [
        "src/ui/pages/accordion.rs",
        "src/ui/pages/alert.rs",
        "src/ui/pages/dialog.rs",
        "src/ui/pages/navigation_menu.rs",
        "src/ui/pages/select.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "let notes = doc_layout::notes_block([",
                "let notes = DocSection::build(cx, \"Notes\", notes)",
            ],
            &[
                "let notes = doc_layout::notes(",
                "DocSection::new(\"Notes\", notes)",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/sheet.rs",
        &[
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/drawer.rs",
        &[
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/popover.rs",
        &[
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    for relative_path in [
        "src/ui/pages/context_menu.rs",
        "src/ui/pages/menubar.rs",
        "src/ui/pages/progress.rs",
        "src/ui/pages/pagination.rs",
        "src/ui/pages/tabs.rs",
        "src/ui/pages/scroll_area.rs",
        "src/ui/pages/slider.rs",
        "src/ui/pages/icons.rs",
        "src/ui/pages/typography.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "let notes = doc_layout::notes_block([",
                "let notes = DocSection::build(cx, \"Notes\", notes)",
            ],
            &[
                "let notes = doc_layout::notes(",
                "DocSection::new(\"Notes\", notes)",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/command.rs",
        &[
            "let notes_stack = doc_layout::notes_block([",
            "let notes_stack = DocSection::build(cx, \"Notes\", notes_stack)",
        ],
        &[
            "let notes_stack = doc_layout::notes(",
            "DocSection::new(\"Notes\", notes_stack)",
        ],
    );

    for relative_path in [
        "src/ui/pages/badge.rs",
        "src/ui/pages/checkbox.rs",
        "src/ui/pages/collapsible.rs",
        "src/ui/pages/empty.rs",
        "src/ui/pages/input.rs",
        "src/ui/pages/label.rs",
        "src/ui/pages/kbd.rs",
        "src/ui/pages/spinner.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "let api_reference = doc_layout::notes_block([",
                "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            ],
            &[
                "let api_reference = doc_layout::notes(",
                "DocSection::new(\"API Reference\", api_reference)",
            ],
        );
    }

    for relative_path in [
        "src/ui/pages/switch.rs",
        "src/ui/pages/toggle.rs",
        "src/ui/pages/toggle_group.rs",
        "src/ui/pages/separator.rs",
        "src/ui/pages/textarea.rs",
        "src/ui/pages/radio_group.rs",
        "src/ui/pages/skeleton.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "let api_reference = doc_layout::notes_block([",
                "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            ],
            &[
                "let api_reference = doc_layout::notes(",
                "DocSection::new(\"API Reference\", api_reference)",
            ],
        );
    }

    for relative_path in [
        "src/ui/pages/tooltip.rs",
        "src/ui/pages/table.rs",
        "src/ui/pages/image_object_fit.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "let notes = doc_layout::notes_block([",
                "let notes = DocSection::build(cx, \"Notes\", notes)",
            ],
            &[
                "let notes = doc_layout::notes(",
                "DocSection::new(\"Notes\", notes)",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/breadcrumb.rs",
        &[
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    for relative_path in [
        "src/ui/pages/card.rs",
        "src/ui/pages/input_otp.rs",
        "src/ui/pages/sidebar.rs",
        "src/ui/pages/aspect_ratio.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "let notes = doc_layout::notes_block([",
                "DocSection::build(cx, \"Notes\", notes)",
            ],
            &[
                "let notes = doc_layout::notes(",
                "DocSection::new(\"Notes\", notes)",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/card.rs",
        &["Default first-party teaching should prefer `card(...)` plus the slot helper family;"],
        &["use `Card::build(...)` or `card(...)`;"],
    );

    for relative_path in [
        "src/ui/pages/resizable.rs",
        "src/ui/pages/sonner.rs",
        "src/ui/pages/form.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["DocSection::build(cx, \"Notes\", notes)"],
            &["DocSection::new(\"Notes\", notes)"],
        );
    }
}

#[test]
fn selected_card_snippets_prefer_card_wrapper_family() {
    for relative_path in [
        "src/ui/snippets/card/usage.rs",
        "src/ui/snippets/card/size.rs",
        "src/ui/snippets/card/card_content.rs",
        "src/ui/snippets/card/compositions.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "shadcn::card(",
                "shadcn::card_header(",
                "shadcn::card_content(",
                "shadcn::card_footer(",
            ],
            &[
                "shadcn::Card::new(",
                "shadcn::CardHeader::new(",
                "shadcn::CardContent::new(",
                "shadcn::CardFooter::new(",
                "shadcn::CardFooter::build(",
            ],
        );
    }

    for relative_path in [
        "src/ui/snippets/card/demo.rs",
        "src/ui/snippets/card/rtl.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "shadcn::card(",
                "shadcn::card_header(",
                "shadcn::card_content(",
                "shadcn::card_footer(",
            ],
            &["shadcn::Card::new(", "shadcn::CardFooter::build("],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/card/image.rs",
        &[
            "shadcn::card(",
            "shadcn::card_header(",
            "shadcn::card_action(",
            "shadcn::card_footer(",
        ],
        &[
            "shadcn::Card::new(",
            "shadcn::CardHeader::new(",
            "shadcn::CardAction::new(",
            "shadcn::CardFooter::new(",
        ],
    );

    for relative_path in [
        "src/ui/snippets/tabs/demo.rs",
        "src/ui/snippets/input_otp/form.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "shadcn::card(",
                "shadcn::card_header(",
                "shadcn::card_content(",
                "shadcn::card_footer(",
            ],
            &[
                "shadcn::Card::new(",
                "shadcn::CardHeader::new(",
                "shadcn::CardContent::new(",
                "shadcn::CardFooter::new(",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/collapsible/basic.rs",
        &["shadcn::card(", "shadcn::card_content("],
        &["shadcn::Card::new(", "shadcn::CardContent::new("],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/motion_presets/fluid_tabs_demo.rs",
        &[
            "shadcn::card(",
            "shadcn::card_header(",
            "shadcn::card_content(",
        ],
        &[
            "shadcn::Card::new(",
            "shadcn::CardHeader::new(",
            "shadcn::CardContent::new(",
        ],
    );

    for relative_path in [
        "src/ui/snippets/motion_presets/overlay_demo.rs",
        "src/ui/snippets/motion_presets/stagger_demo.rs",
        "src/ui/snippets/motion_presets/stack_shift_list_demo.rs",
        "src/ui/snippets/accordion/card.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "shadcn::card(",
                "shadcn::card_header(",
                "shadcn::card_content(",
            ],
            &[
                "shadcn::Card::new(",
                "shadcn::CardHeader::new(",
                "shadcn::CardContent::new(",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/calendar/presets.rs",
        &[
            "shadcn::card(",
            "shadcn::card_content(",
            "shadcn::card_footer(",
        ],
        &[
            "shadcn::Card::new(",
            "shadcn::CardContent::new(",
            "shadcn::CardFooter::new(",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/form/upstream_demo.rs",
        &["shadcn::card(", "shadcn::card_content("],
        &["shadcn::Card::new(", "shadcn::CardContent::new("],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/speech_input_demo.rs",
        &["shadcn::card(", "shadcn::card_content("],
        &["shadcn::Card::new(", "shadcn::CardContent::new("],
    );

    for relative_path in [
        "src/ui/snippets/carousel/basic.rs",
        "src/ui/snippets/carousel/api.rs",
        "src/ui/snippets/carousel/demo.rs",
        "src/ui/snippets/carousel/duration_embla.rs",
        "src/ui/snippets/carousel/events.rs",
        "src/ui/snippets/carousel/expandable.rs",
        "src/ui/snippets/carousel/focus_watch.rs",
        "src/ui/snippets/carousel/loop_carousel.rs",
        "src/ui/snippets/carousel/loop_downgrade_cannot_loop.rs",
        "src/ui/snippets/carousel/options.rs",
        "src/ui/snippets/carousel/orientation_vertical.rs",
        "src/ui/snippets/carousel/parts.rs",
        "src/ui/snippets/carousel/plugin_autoplay.rs",
        "src/ui/snippets/carousel/plugin_autoplay_controlled.rs",
        "src/ui/snippets/carousel/plugin_autoplay_delays.rs",
        "src/ui/snippets/carousel/plugin_autoplay_stop_on_focus.rs",
        "src/ui/snippets/carousel/plugin_autoplay_stop_on_last_snap.rs",
        "src/ui/snippets/carousel/plugin_wheel_gestures.rs",
        "src/ui/snippets/carousel/rtl.rs",
        "src/ui/snippets/carousel/sizes.rs",
        "src/ui/snippets/carousel/sizes_thirds.rs",
        "src/ui/snippets/carousel/spacing.rs",
        "src/ui/snippets/carousel/spacing_responsive.rs",
        "src/ui/snippets/carousel/usage.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::card(", "shadcn::card_content("],
            &[
                "shadcn::Card::new(",
                "shadcn::CardHeader::new(",
                "shadcn::CardAction::new(",
                "shadcn::CardContent::new(",
                "shadcn::CardFooter::new(",
            ],
        );
    }

    for relative_path in [
        "src/ui/snippets/motion_presets/preset_selector.rs",
        "src/ui/snippets/motion_presets/token_snapshot.rs",
        "src/ui/snippets/skeleton/card.rs",
        "src/ui/snippets/accordion/extras.rs",
        "src/ui/snippets/collapsible/settings_panel.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "shadcn::card(",
                "shadcn::card_header(",
                "shadcn::card_content(",
            ],
            &[
                "shadcn::Card::new(",
                "shadcn::CardHeader::new(",
                "shadcn::CardContent::new(",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/card/meeting_notes.rs",
        &[
            "shadcn::card(",
            "shadcn::card_header(",
            "shadcn::card_action(",
            "shadcn::card_content(",
            "shadcn::card_footer(",
        ],
        &[
            "shadcn::Card::new(",
            "shadcn::CardHeader::new(",
            "shadcn::CardAction::new(",
            "shadcn::CardContent::new(",
            "shadcn::CardFooter::new(",
        ],
    );
}

#[test]
fn snippet_tree_does_not_reintroduce_legacy_shadcn_card_constructors() {
    assert_sources_absent(
        "src/ui/snippets",
        &[
            "shadcn::Card::new(",
            "shadcn::CardHeader::new(",
            "shadcn::CardAction::new(",
            "shadcn::CardContent::new(",
            "shadcn::CardFooter::new(",
        ],
    );
}

#[test]
fn page_tree_does_not_reintroduce_legacy_shadcn_card_constructors() {
    assert_sources_absent(
        "src/ui/pages",
        &[
            "shadcn::Card::new(",
            "shadcn::CardHeader::new(",
            "shadcn::CardAction::new(",
            "shadcn::CardContent::new(",
            "shadcn::CardFooter::new(",
        ],
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

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/carousel.rs",
        &[
            "let about = doc_layout::notes_block([",
            "let api_reference = doc_layout::notes_block([",
            "let about = DocSection::build(cx, \"About\", about)",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
        ],
        &[
            "let about = doc_layout::notes(",
            "let api_reference = doc_layout::notes(",
            "DocSection::new(\"About\", about)",
            "DocSection::new(\"API Reference\", api_reference)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/chart.rs",
        &[
            "let notes_stack = doc_layout::notes_block([",
            "let notes_stack = DocSection::build(cx, \"Notes\", notes_stack)",
        ],
        &[
            "let notes_stack = doc_layout::notes(",
            "DocSection::new(\"Notes\", notes_stack)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/combobox.rs",
        &[
            "let notes = doc_layout::notes_block([",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let notes = doc_layout::notes(",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    for relative_path in ["src/ui/pages/data_table.rs", "src/ui/pages/item.rs"] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "let api_reference = doc_layout::notes_block([",
                "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            ],
            &[
                "let api_reference = doc_layout::notes(",
                "DocSection::new(\"API Reference\", api_reference)",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/native_select.rs",
        &[
            "let native_select_vs_select = doc_layout::notes_block([",
            "let api_reference = doc_layout::notes_block([",
            "DocSection::build(cx, \"Native Select vs Select\", native_select_vs_select",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
        ],
        &[
            "let native_select_vs_select = doc_layout::notes(",
            "let api_reference = doc_layout::notes(",
            "DocSection::new(\"Native Select vs Select\", native_select_vs_select)",
            "DocSection::new(\"API Reference\", api_reference)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/date_picker.rs",
        &["DocSection::build(cx, \"Notes\", notes_stack)"],
        &["DocSection::new(\"Notes\", notes_stack)"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/field.rs",
        &[
            "let form = doc_layout::notes_block([",
            "let accessibility = doc_layout::notes_block([",
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let form = DocSection::build(cx, \"Form\", form)",
            "let accessibility = DocSection::build(cx, \"Accessibility\", accessibility)",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let form = doc_layout::notes(",
            "let accessibility = doc_layout::notes(",
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"Form\", form)",
            "DocSection::new(\"Accessibility\", accessibility)",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/input_group.rs",
        &[
            "let align = doc_layout::notes_block([",
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let align = DocSection::build(cx, \"Align\", align)",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let align = doc_layout::notes(",
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"Align\", align)",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );
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

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/context_menu/sides.rs",
        &[
            "fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str,) -> impl IntoUiElement<H> + use<H>",
            "fn side_menu<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, side: shadcn::DropdownMenuSide, trigger_test_id: &'static str, content_test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, test_id: &'static str,) -> AnyElement",
            "fn side_menu<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, side: shadcn::DropdownMenuSide, trigger_test_id: &'static str, content_test_id: &'static str,) -> AnyElement",
        ],
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
fn selected_pagination_snippets_prefer_pagination_wrapper_family() {
    for relative_path in [
        "src/ui/snippets/pagination/demo.rs",
        "src/ui/snippets/pagination/extras.rs",
        "src/ui/snippets/pagination/icons_only.rs",
        "src/ui/snippets/pagination/rtl.rs",
        "src/ui/snippets/pagination/simple.rs",
        "src/ui/snippets/pagination/usage.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "shadcn::pagination(|cx|",
                "shadcn::pagination_content(|cx|",
                "shadcn::pagination_item(",
            ],
            &[
                "shadcn::Pagination::new(",
                "shadcn::PaginationContent::new(",
                "shadcn::PaginationItem::new(",
            ],
        );
    }

    for relative_path in [
        "src/ui/snippets/pagination/demo.rs",
        "src/ui/snippets/pagination/extras.rs",
        "src/ui/snippets/pagination/rtl.rs",
        "src/ui/snippets/pagination/simple.rs",
        "src/ui/snippets/pagination/usage.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::pagination_link(|cx|"],
            &["shadcn::PaginationLink::new("],
        );
    }
}

#[test]
fn selected_table_snippets_prefer_table_wrapper_family() {
    for relative_path in [
        "src/ui/snippets/table/actions.rs",
        "src/ui/snippets/table/demo.rs",
        "src/ui/snippets/table/footer.rs",
        "src/ui/snippets/table/rtl.rs",
        "src/ui/snippets/checkbox/table.rs",
        "src/ui/snippets/typography/table.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "shadcn::table(",
                "shadcn::table_header(",
                "shadcn::table_body(",
                "shadcn::table_row(",
            ],
            &[
                "shadcn::Table::build(",
                "shadcn::TableHeader::build(",
                "shadcn::TableBody::build(",
                "shadcn::TableRow::build(",
                "shadcn::TableHead::new(",
                "shadcn::TableCell::new(",
                "shadcn::TableCell::build(",
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

    for relative_path in [
        "src/ui/snippets/resizable/rtl.rs",
        "src/ui/snippets/resizable/demo.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
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

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/scroll_area/demo.rs",
        &["fn tag_row<H: UiHost>(tag: Arc<str>) -> impl IntoUiElement<H> + use<H>"],
        &["fn tag_row<H: UiHost>(tag: Arc<str>) -> AnyElement"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/scroll_area/expand_at_bottom.rs",
        &[
            "fn toggle_button<H: UiHost>(expanded: Model<bool>, is_expanded: bool,) -> impl IntoUiElement<H> + use<H>",
            "fn empty_row<H: UiHost>(row_h: Px) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn toggle_button<H: UiHost>(expanded: Model<bool>, is_expanded: bool,) -> AnyElement",
            "fn empty_row<H: UiHost>(row_h: Px) -> AnyElement",
        ],
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
            "fn action_row(product: &'static str, price: &'static str, open_model: Model<bool>, key: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn align_end(cx: &mut UiCx<'_>, child: AnyElement) -> AnyElement",
            "fn action_row(product: &'static str, price: &'static str, open_model: Model<bool>, key: &'static str,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_field_and_form_snippets_prefer_field_wrapper_family() {
    for relative_path in [
        "src/ui/snippets/field/input.rs",
        "src/ui/snippets/field/fieldset.rs",
        "src/ui/snippets/field/field_group.rs",
        "src/ui/snippets/form/demo.rs",
        "src/ui/snippets/form/fieldset.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::field_set(|cx| {", "shadcn::field_group(|cx| {"],
            &[
                "shadcn::FieldSet::build(",
                "shadcn::FieldGroup::build(",
                "shadcn::FieldSet::new(",
                "shadcn::FieldGroup::new(",
            ],
        );
    }
}

#[test]
fn selected_control_snippets_prefer_field_group_wrapper_family() {
    for relative_path in [
        "src/ui/snippets/combobox/label.rs",
        "src/ui/snippets/input/form.rs",
        "src/ui/snippets/input/field_group.rs",
        "src/ui/snippets/toggle_group/label.rs",
        "src/ui/snippets/date_picker/time_picker.rs",
        "src/ui/snippets/date_picker/label.rs",
        "src/ui/snippets/native_select/label.rs",
        "src/ui/snippets/select/label.rs",
        "src/ui/snippets/select/align_item_with_trigger.rs",
        "src/ui/snippets/checkbox/with_title.rs",
        "src/ui/snippets/slider/label.rs",
        "src/ui/snippets/switch/choice_card.rs",
        "src/ui/snippets/switch/label.rs",
        "src/ui/snippets/radio_group/label.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::field_group(|cx| {"],
            &["shadcn::FieldGroup::new(", "shadcn::FieldGroup::build("],
        );
    }
}

#[test]
fn selected_radio_group_snippets_prefer_field_set_wrapper_family() {
    for relative_path in [
        "src/ui/snippets/radio_group/fieldset.rs",
        "src/ui/snippets/radio_group/extras.rs",
        "src/ui/snippets/radio_group/invalid.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::field_set(|cx| {"],
            &["shadcn::FieldSet::new(", "shadcn::FieldSet::build("],
        );
    }
}

#[test]
fn field_page_usage_prefers_field_wrapper_family() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/field.rs",
        &["shadcn::field_set(|cx| {", "shadcn::field_group(|cx| {"],
        &["shadcn::FieldSet::new(", "shadcn::FieldGroup::new("],
    );
}

#[test]
fn selected_ai_doc_pages_prefer_doc_layout_text_table_over_raw_table_builders() {
    for relative_path in [
        "src/ui/pages/ai_model_selector_demo.rs",
        "src/ui/pages/ai_context_demo.rs",
        "src/ui/pages/ai_mic_selector_demo.rs",
        "src/ui/pages/ai_voice_selector_demo.rs",
        "src/ui/pages/ai_shimmer_demo.rs",
        "src/ui/pages/ai_chain_of_thought_demo.rs",
        "src/ui/pages/ai_checkpoint_demo.rs",
        "src/ui/pages/ai_commit_demo.rs",
        "src/ui/pages/ai_test_results_demo.rs",
        "src/ui/pages/ai_persona_demo.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["doc_layout::text_table("],
            &[
                "shadcn::Table::build(",
                "shadcn::TableHeader::build(",
                "shadcn::TableBody::build(",
                "shadcn::TableRow::build(",
                "shadcn::TableHead::new(",
                "shadcn::TableCell::build(",
            ],
        );
    }
}

#[test]
fn selected_empty_snippets_prefer_empty_wrapper_family() {
    for relative_path in [
        "src/ui/snippets/empty/avatar.rs",
        "src/ui/snippets/empty/avatar_group.rs",
        "src/ui/snippets/empty/background.rs",
        "src/ui/snippets/empty/demo.rs",
        "src/ui/snippets/empty/input_group.rs",
        "src/ui/snippets/empty/outline.rs",
        "src/ui/snippets/empty/rtl.rs",
        "src/ui/snippets/empty/usage.rs",
        "src/ui/snippets/spinner/empty.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "shadcn::empty(|cx|",
                "shadcn::empty_header(|cx|",
                "shadcn::empty_content(|cx|",
                "shadcn::empty_title(",
                "shadcn::empty_description(",
            ],
            &[
                "shadcn::Empty::new(",
                "fret_ui_shadcn::empty::EmptyHeader::new(",
                "fret_ui_shadcn::empty::EmptyTitle::new(",
                "fret_ui_shadcn::empty::EmptyDescription::new(",
                "fret_ui_shadcn::empty::EmptyContent::new(",
            ],
        );
    }

    for relative_path in [
        "src/ui/snippets/empty/avatar.rs",
        "src/ui/snippets/empty/avatar_group.rs",
        "src/ui/snippets/empty/background.rs",
        "src/ui/snippets/empty/demo.rs",
        "src/ui/snippets/empty/outline.rs",
        "src/ui/snippets/empty/rtl.rs",
        "src/ui/snippets/empty/usage.rs",
        "src/ui/snippets/spinner/empty.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::empty_media(|cx|"],
            &["fret_ui_shadcn::empty::EmptyMedia::new("],
        );
    }
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
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/aspect_ratio/demo.rs",
        &[
            "pub fn render_preview<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "pub fn render_preview<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> AnyElement",
        ],
    );

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
                "pub fn render_preview<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                image_helper_old,
                "fn ratio_example<H: UiHost>(cx: &mut ElementContext<'_, H>, ratio: f32, max_w: Px, test_id: &'static str, content_test_id: &'static str, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> AnyElement",
                "pub fn render_preview<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> AnyElement",
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
            "shadcn::card(",
            "shadcn::card_header(",
            "shadcn::card_content(",
            "shadcn::card_footer(",
        ],
        &[
            "fn centered(cx: &mut UiCx<'_>, body: AnyElement) -> AnyElement",
            "shadcn::Card::new(",
            "shadcn::CardHeader::new(",
            "shadcn::CardContent::new(",
            "shadcn::CardFooter::new(",
        ],
    );

    for relative_path in [
        "src/ui/snippets/sonner/demo.rs",
        "src/ui/snippets/sonner/extras.rs",
        "src/ui/snippets/sonner/position.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "fn wrap_controls_row<H: UiHost>(gap: Space, children: Vec<AnyElement>,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn wrap_controls_row<H: UiHost>(cx: &mut ElementContext<'_, H>, gap: Space, children: Vec<AnyElement>,) -> AnyElement",
            ],
        );
    }
}

#[test]
fn selected_chart_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/chart/demo.rs",
        &[
            "let trending_footer = |cx: &mut UiCx<'_>, secondary: &'static str| -> impl IntoUiElement<fret_app::App> + use<>",
            "let chart_card = |cx: &mut UiCx<'_>, title: &'static str, description: &'static str, kind: DemoChartKind, footer_secondary: &'static str, test_id: &'static str| -> impl IntoUiElement<fret_app::App> + use<>",
            "shadcn::card(",
            "shadcn::card_header(",
            "shadcn::card_content(",
            "shadcn::card_footer(",
        ],
        &[
            "let trending_footer = |cx: &mut UiCx<'_>, secondary: &'static str| {",
            "let chart_card = |cx: &mut UiCx<'_>, title: &'static str, description: &'static str, kind: DemoChartKind, footer_secondary: &'static str, test_id: &'static str| {",
            "shadcn::Card::new(",
            "shadcn::CardHeader::new(",
            "shadcn::CardContent::new(",
            "shadcn::CardFooter::new(",
        ],
    );
}

#[test]
fn selected_alert_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/alert/interactive_links.rs",
        &[
            "fn interactive_link<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, last_link: Model<Option<Arc<str>>>, label: &'static str, tag: &'static str, href: &'static str, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn interactive_link<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, last_link: Model<Option<Arc<str>>>, label: &'static str, tag: &'static str, href: &'static str, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/alert/demo.rs",
        &[
            "fn interactive_link_text<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, text: &'static str, underlined_fragment: &'static str, href: &'static str, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn interactive_link_text<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, text: &'static str, underlined_fragment: &'static str, href: &'static str, test_id: &'static str,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_alert_snippets_prefer_alert_wrapper_family() {
    for relative_path in [
        "src/ui/snippets/alert/action.rs",
        "src/ui/snippets/alert/basic.rs",
        "src/ui/snippets/alert/custom_colors.rs",
        "src/ui/snippets/alert/demo.rs",
        "src/ui/snippets/alert/destructive.rs",
        "src/ui/snippets/alert/interactive_links.rs",
        "src/ui/snippets/alert/rich_title.rs",
        "src/ui/snippets/alert/rtl.rs",
        "src/ui/snippets/motion_presets/fluid_tabs_demo.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::alert("],
            &["shadcn::Alert::new("],
        );
    }
}

#[test]
fn selected_slider_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/slider/demo.rs",
        &[
            "fn controlled<H: UiHost>(cx: &mut ElementContext<'_, H>, controlled_values: Model<Vec<f32>>,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn controlled<H: UiHost>(cx: &mut ElementContext<'_, H>, controlled_values: Model<Vec<f32>>,) -> AnyElement",
        ],
    );
}

#[test]
fn selected_motion_presets_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/motion_presets/fluid_tabs_demo.rs",
        &[
            "fn panel(title: &'static str, description: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
        ],
        &[
            "fn panel(cx: &mut UiCx<'_>, title: &'static str, description: &'static str) -> AnyElement",
            "fn panel(cx: &mut UiCx<'_>, title: &'static str, description: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
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
            "fn build<P>(cx: &mut UiCx<'_>, title: &'static str, preview: P) -> Self where P: IntoUiElement<fret_app::App>",
            "fn notes_block<I, T>(lines: I) -> impl IntoUiElement<fret_app::App> + use<I, T> where I: IntoIterator<Item = T>, T: Into<Arc<str>>",
            "fn demo_shell<B>(cx: &mut UiCx<'_>, max_w: Px, body: B,) -> impl IntoUiElement<fret_app::App> + use<B> where B: IntoUiElement<fret_app::App>",
        ],
        &[
            "fn demo_shell(cx: &mut UiCx<'_>, max_w: Px, body: AnyElement) -> AnyElement",
            "fn notes_block<I, T>(cx: &mut UiCx<'_>, lines: I) -> AnyElement",
            "fn notes<I, T>(cx: &mut UiCx<'_>, lines: I) -> AnyElement",
        ],
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
