mod support;

use std::collections::BTreeSet;

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

fn assert_normalized_markers_present(relative_path: &str, required_markers: &[&str]) -> String {
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

    normalized
}

fn assert_material3_snippet_prefers_copyable_root(
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
            "{} is missing Material 3 overlay authoring marker `{}`",
            path.display(),
            marker
        );
    }

    for marker in forbidden_markers {
        let marker = marker.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains(&marker),
            "{} reintroduced legacy Material 3 overlay teaching marker `{}`",
            path.display(),
            marker
        );
    }
}

fn assert_selected_snippets_prefer_grouped_uicx_listeners(
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
            "{} is missing grouped UiCx listener marker `{}`",
            path.display(),
            marker
        );
    }

    for marker in forbidden_markers {
        let marker = marker.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains(&marker),
            "{} reintroduced forbidden activation import marker `{}`",
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

fn assert_sources_absent_normalized(relative_root: &str, forbidden_markers: &[&str]) {
    for path in rust_sources(relative_root) {
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();

        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "{} reintroduced forbidden normalized source marker `{}`",
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
    assert!(action_first_view.contains("use fret::component::prelude::*;"));
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
    assert!(action_first_view.contains("pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>"));
    assert!(action_first_view.contains("let last_action = super::last_action_model(cx);"));
    assert!(!action_first_view.contains("KernelApp"));
    assert!(!action_first_view.contains("ElementContext<'_, App>"));
    assert!(!action_first_view.contains("cx.use_local"));
    assert!(!action_first_view.contains("cx.on_action_notify_"));
    assert!(!action_first_view.contains("cx.on_action_availability"));
}

#[test]
fn copyable_ui_gallery_snippet_lane_has_no_top_level_raw_render_roots() {
    assert_sources_absent(
        "src/ui/snippets",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
    assert_sources_absent_normalized(
        "src/ui/snippets",
        &["pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn ui_gallery_diagnostics_raw_render_roots_are_explicitly_documented() {
    let mut raw_root_count = 0usize;

    for path in rust_sources("src/ui/diagnostics") {
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();
        let is_raw_render_root = normalized
            .contains("pubfnrender<H:UiHost+'static>(cx:&mutElementContext<'_,H>)->AnyElement");
        if !is_raw_render_root {
            continue;
        }

        raw_root_count += 1;
        assert!(
            source.contains("Intentional diagnostics raw boundary:"),
            "{} should explain why the diagnostics harness stays raw",
            path.display()
        );
    }

    assert!(
        raw_root_count >= 1,
        "src/ui/diagnostics should contain at least one audited diagnostics raw root"
    );
}

#[test]
fn selected_button_and_sidebar_snippets_prefer_grouped_uicx_listeners() {
    for relative_path in [
        "src/ui/snippets/ai/chat_demo.rs",
        "src/ui/snippets/ai/prompt_input_referenced_sources_demo.rs",
        "src/ui/snippets/ai/reasoning_demo.rs",
        "src/ui/snippets/ai/transcript_torture.rs",
        "src/ui/snippets/ai/persona_demo.rs",
        "src/ui/snippets/ai/task_demo.rs",
        "src/ui/snippets/drawer/demo.rs",
        "src/ui/snippets/data_table/basic_demo.rs",
        "src/ui/snippets/data_table/default_demo.rs",
        "src/ui/snippets/data_table/rtl_demo.rs",
        "src/ui/snippets/scroll_area/nested_scroll_routing.rs",
        "src/ui/snippets/sidebar/demo.rs",
        "src/ui/snippets/sidebar/controlled.rs",
        "src/ui/snippets/sidebar/mobile.rs",
        "src/ui/snippets/sidebar/rtl.rs",
        "src/ui/snippets/sonner/demo.rs",
        "src/ui/snippets/sonner/usage.rs",
        "src/ui/snippets/sonner/extras.rs",
        "src/ui/snippets/sonner/position.rs",
    ] {
        assert_selected_snippets_prefer_grouped_uicx_listeners(
            relative_path,
            &[
                "use fret::app::UiCxActionsExt as _;",
                ".on_activate(cx.actions().listen(",
            ],
            &["use fret::app::AppActivateExt as _;"],
        );
    }
}

#[test]
fn selected_ai_snippets_prefer_grouped_uicx_listeners_when_widgets_have_native_hook_slots() {
    for (relative_path, required_markers) in [
        (
            "src/ui/snippets/ai/workflow_controls_demo.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "ui_ai::WorkflowControlsButton::new(\"Zoom in\", fret_icons::ids::ui::PLUS)",
                ".on_activate(cx.actions().listen(",
            ][..],
        ),
        (
            "src/ui/snippets/ai/workflow_node_graph_demo.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "ui_ai::WorkflowControlsButton::new(\"Zoom in\", IconId::new_static(\"lucide.plus\"))",
                ".on_activate(cx.actions().listen(zoom_in))",
            ][..],
        ),
        (
            "src/ui/snippets/ai/message_demo.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "ui_ai::MessageAction::new(\"Copy\")",
                ".on_activate(cx.actions().listen(set_action(\"assistant.copy\")))",
            ][..],
        ),
        (
            "src/ui/snippets/ai/message_usage.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "ui_ai::MessageAction::new(\"Retry\")",
                ".on_activate(cx.actions().listen(set_action(\"assistant.retry\")))",
            ][..],
        ),
        (
            "src/ui/snippets/ai/artifact_demo.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "ui_ai::ArtifactClose::new()",
                ".on_activate(cx.actions().listen(",
            ][..],
        ),
        (
            "src/ui/snippets/ai/artifact_code_display.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "ui_ai::ArtifactAction::new()",
                ".on_activate(cx.actions().listen(status_action(",
            ][..],
        ),
        (
            "src/ui/snippets/ai/checkpoint_demo.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "ui_ai::CheckpointTrigger::new([cx.text(checkpoint.trigger_label)])",
                ".on_activate(cx.actions().listen(restore_to_checkpoint.clone()))",
            ][..],
        ),
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();

        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                normalized.contains(&marker),
                "{} is missing grouped UiCx listener marker `{}`",
                path.display(),
                marker
            );
        }

        assert!(
            !normalized.contains("usefret::app::AppActivateExtas_;"),
            "{} should no longer import AppActivateExt once the widget exposes a native hook surface",
            path.display()
        );
    }
}

#[test]
fn selected_ai_snippets_prefer_grouped_uicx_actions_when_widgets_have_native_action_slots() {
    for (relative_path, required_markers) in [
        (
            "src/ui/snippets/ai/confirmation_demo.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "cx.actions().models::<act::RequestApproval>(",
                "shadcn::Button::new(\"Request approval\")",
                ".action(act::RequestApproval)",
                "ui_ai::ConfirmationAction::new(\"Reject\")",
                ".action(act::RejectApproval)",
                "ui_ai::ConfirmationAction::new(\"Approve\")",
                ".action(act::ApproveApproval)",
            ][..],
        ),
        (
            "src/ui/snippets/ai/conversation_demo.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "cx.actions().models::<act::DownloadConversation>(",
                "ui_ai::ConversationDownload::new(\"Download conversation\")",
                ".action(act::DownloadConversation)",
            ][..],
        ),
        (
            "src/ui/snippets/ai/prompt_input_docs_demo.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "cx.actions().models::<act::ToggleSearch>(",
                "ui_ai::PromptInputButton::new(\"Search\")",
                ".action(act::ToggleSearch)",
            ][..],
        ),
        (
            "src/ui/snippets/ai/web_preview_demo.rs",
            &[
                "use fret::app::UiCxActionsExt as _;",
                "cx.actions().models::<act::NavigateBack>(",
                "cx.actions().models::<act::NavigateForward>(",
                "ui_ai::WebPreviewNavigationButton::new([cx.text(\"←\")])",
                ".action(act::NavigateBack)",
                "ui_ai::WebPreviewNavigationButton::new([cx.text(\"→\")])",
                ".action(act::NavigateForward)",
            ][..],
        ),
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();

        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                normalized.contains(&marker),
                "{} is missing grouped UiCx action marker `{}`",
                path.display(),
                marker
            );
        }

        assert!(
            !normalized.contains("usefret::app::AppActivateExtas_;"),
            "{} should no longer import AppActivateExt once the widget has a native `.action(...)` slot",
            path.display()
        );
        assert!(
            !normalized.contains(".listen("),
            "{} should stay on grouped `UiCx` actions + widget `.action(...)` instead of AppActivate `.listen(...)`",
            path.display()
        );
    }
}

#[test]
fn action_first_view_snippet_prefers_action_alias_for_activation_only_widgets() {
    let normalized = assert_normalized_markers_present(
        "src/ui/snippets/command/action_first_view.rs",
        &["shadcn::Badge::new(\"Ping via activate sugar\").action(act::Ping)"],
    );

    assert!(
        !normalized.contains("usefret::app::AppActivateExtas_;"),
        "action-first view snippet should stay on native widget action slots instead of importing activation bridge sugar"
    );
    assert!(
        !normalized.contains(".dispatch::<act::Ping>()"),
        "action-first view snippet should prefer the value-based activation alias over turbofish dispatch"
    );
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
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/progress",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn badge_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/badge/demo.rs",
            "src/ui/snippets/badge/usage.rs",
            "src/ui/snippets/badge/spinner.rs",
            "src/ui/snippets/badge/rtl.rs",
            "src/ui/snippets/badge/counts.rs",
            "src/ui/snippets/badge/colors.rs",
            "src/ui/snippets/badge/link.rs",
            "src/ui/snippets/badge/icon.rs",
            "src/ui/snippets/badge/variants.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/badge",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn badge_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/badge.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Variants\", variants)",
            "DocSection::build(cx, \"With Icon\", with_icon)",
            "DocSection::build(cx, \"With Spinner\", with_spinner)",
            "DocSection::build(cx, \"Link\", link)",
            "DocSection::build(cx, \"Custom Colors\", colors)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Counts (Fret)\", counts)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Variants\", variants)",
            "DocSection::new(\"With Icon\", with_icon)",
            "DocSection::new(\"With Spinner\", with_spinner)",
            "DocSection::new(\"Link\", link)",
            "DocSection::new(\"Custom Colors\", colors)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Counts (Fret)\", counts)",
        ],
    );
}

#[test]
fn badge_docs_snippets_keep_centered_rows_and_explicit_inline_child_slots() {
    for relative_path in [
        "src/ui/snippets/badge/demo.rs",
        "src/ui/snippets/badge/variants.rs",
        "src/ui/snippets/badge/icon.rs",
        "src/ui/snippets/badge/spinner.rs",
        "src/ui/snippets/badge/colors.rs",
        "src/ui/snippets/badge/rtl.rs",
        "src/ui/snippets/badge/counts.rs",
        "src/ui/snippets/badge/link.rs",
    ] {
        assert_normalized_markers_present(relative_path, &[".justify_center()"]);
    }

    assert_normalized_markers_present(
        "src/ui/snippets/badge/spinner.rs",
        &[
            ".leading_children([shadcn::Spinner::new().into_element(cx)])",
            ".trailing_children([shadcn::Spinner::new().into_element(cx)])",
        ],
    );
}

#[test]
fn aspect_ratio_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/aspect_ratio/demo.rs",
            "src/ui/snippets/aspect_ratio/usage.rs",
            "src/ui/snippets/aspect_ratio/portrait.rs",
            "src/ui/snippets/aspect_ratio/square.rs",
            "src/ui/snippets/aspect_ratio/rtl.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/aspect_ratio",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn aspect_ratio_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/aspect_ratio.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Square\", square)",
            "DocSection::build(cx, \"Portrait\", portrait)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "let api_reference = doc_layout::notes_block([",
            "DocSection::build(cx, \"API Reference\", api_reference)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Square\", square)",
            "DocSection::new(\"Portrait\", portrait)",
            "DocSection::new(\"RTL\", rtl)",
            "let api_reference = doc_layout::notes(",
            "DocSection::new(\"API Reference\", api_reference)",
        ],
    );
}

#[test]
fn context_menu_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/context_menu/demo.rs",
            "src/ui/snippets/context_menu/basic.rs",
            "src/ui/snippets/context_menu/usage.rs",
            "src/ui/snippets/context_menu/submenu.rs",
            "src/ui/snippets/context_menu/shortcuts.rs",
            "src/ui/snippets/context_menu/groups.rs",
            "src/ui/snippets/context_menu/icons.rs",
            "src/ui/snippets/context_menu/checkboxes.rs",
            "src/ui/snippets/context_menu/radio.rs",
            "src/ui/snippets/context_menu/destructive.rs",
            "src/ui/snippets/context_menu/sides.rs",
            "src/ui/snippets/context_menu/rtl.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/context_menu",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn context_menu_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/context_menu.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Submenu\", submenu)",
            "DocSection::build(cx, \"Shortcuts\", shortcuts)",
            "DocSection::build(cx, \"Groups\", groups)",
            "DocSection::build(cx, \"Icons\", icons)",
            "DocSection::build(cx, \"Checkboxes\", checkboxes)",
            "DocSection::build(cx, \"Radio\", radio)",
            "DocSection::build(cx, \"Destructive\", destructive)",
            "DocSection::build(cx, \"Sides\", sides)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"API Reference\", api_reference)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Submenu\", submenu)",
            "DocSection::new(\"Shortcuts\", shortcuts)",
            "DocSection::new(\"Groups\", groups)",
            "DocSection::new(\"Icons\", icons)",
            "DocSection::new(\"Checkboxes\", checkboxes)",
            "DocSection::new(\"Radio\", radio)",
            "DocSection::new(\"Destructive\", destructive)",
            "DocSection::new(\"Sides\", sides)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"API Reference\", api_reference)",
        ],
    );
}

#[test]
fn context_menu_default_snippets_prefer_the_typed_compose_root_lane() {
    for relative_path in [
        "src/ui/snippets/context_menu/demo.rs",
        "src/ui/snippets/context_menu/basic.rs",
        "src/ui/snippets/context_menu/usage.rs",
        "src/ui/snippets/context_menu/submenu.rs",
        "src/ui/snippets/context_menu/shortcuts.rs",
        "src/ui/snippets/context_menu/groups.rs",
        "src/ui/snippets/context_menu/icons.rs",
        "src/ui/snippets/context_menu/checkboxes.rs",
        "src/ui/snippets/context_menu/radio.rs",
        "src/ui/snippets/context_menu/destructive.rs",
        "src/ui/snippets/context_menu/sides.rs",
        "src/ui/snippets/context_menu/rtl.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::ContextMenu::uncontrolled(cx)", ".compose()"],
            &[],
        );
    }

    let page = read("src/ui/pages/context_menu.rs");
    assert!(
        page.contains(
            "No extra generic heterogeneous children API is currently warranted: the explicit `ContextMenuEntry` tree is the Fret-equivalent structured surface for upstream nested menu children, and a generic children lane would add hidden scope/collection contracts without unlocking new behavior."
        ),
        "src/ui/pages/context_menu.rs should record why ContextMenu stays on the explicit entry-tree surface instead of widening to a generic children API"
    );
}

#[test]
fn context_menu_docs_examples_keep_dashed_context_region_triggers() {
    for relative_path in [
        "src/ui/snippets/context_menu/demo.rs",
        "src/ui/snippets/context_menu/basic.rs",
        "src/ui/snippets/context_menu/submenu.rs",
        "src/ui/snippets/context_menu/shortcuts.rs",
        "src/ui/snippets/context_menu/groups.rs",
        "src/ui/snippets/context_menu/icons.rs",
        "src/ui/snippets/context_menu/checkboxes.rs",
        "src/ui/snippets/context_menu/radio.rs",
        "src/ui/snippets/context_menu/destructive.rs",
        "src/ui/snippets/context_menu/sides.rs",
        "src/ui/snippets/context_menu/rtl.rs",
    ] {
        let normalized = assert_normalized_markers_present(
            relative_path,
            &[
                "shadcn::AspectRatio::with_child(content)",
                "DashPatternV1::new(Px(4.0), Px(4.0), Px(0.0))",
            ],
        );
        let path = manifest_path(relative_path);
        assert!(
            !normalized.contains("shadcn::Button::new("),
            "{} should keep the docs-like dashed context region instead of regressing to an outline button trigger",
            path.display()
        );
    }

    assert_normalized_markers_present(
        "src/ui/snippets/context_menu/rtl.rs",
        &[".side(shadcn::DropdownMenuSide::InlineEnd)"],
    );

    let page = read("src/ui/pages/context_menu.rs");
    assert!(
        page.contains(
            "The RTL example now exercises logical-side placement directly: `ContextMenuContent::side(shadcn::DropdownMenuSide::InlineEnd)` matches the upstream Base UI docs while submenu chevrons still follow direction-provider parity."
        ),
        "src/ui/pages/context_menu.rs should record that the RTL example now uses logical `inline-end` placement directly"
    );
}

#[test]
fn context_menu_docs_examples_keep_pointer_aware_trigger_copy() {
    for relative_path in [
        "src/ui/snippets/context_menu/demo.rs",
        "src/ui/snippets/context_menu/basic.rs",
        "src/ui/snippets/context_menu/submenu.rs",
        "src/ui/snippets/context_menu/shortcuts.rs",
        "src/ui/snippets/context_menu/groups.rs",
        "src/ui/snippets/context_menu/icons.rs",
        "src/ui/snippets/context_menu/checkboxes.rs",
        "src/ui/snippets/context_menu/radio.rs",
        "src/ui/snippets/context_menu/destructive.rs",
        "src/ui/snippets/context_menu/sides.rs",
        "src/ui/snippets/context_menu/rtl.rs",
    ] {
        assert_normalized_markers_present(
            relative_path,
            &[
                "primary_pointer_is_coarse(cx, Invalidation::Layout, false)",
                "Long press",
            ],
        );
    }

    let page = read("src/ui/pages/context_menu.rs");
    assert!(
        page.contains(
            "Docs-backed trigger copy now adapts to the committed primary pointer capability, so touch-first windows read `Long press here` / `Long press (...)` without needing any new context-menu mechanism work."
        ),
        "src/ui/pages/context_menu.rs should record that docs-backed trigger copy now adapts to fine/coarse pointer capability"
    );
}

#[test]
fn context_menu_usage_and_basic_examples_stay_docs_aligned() {
    let usage = read("src/ui/snippets/context_menu/usage.rs");
    assert!(
        usage.contains("shadcn::ContextMenuTrigger::build("),
        "src/ui/snippets/context_menu/usage.rs should keep teaching the named `ContextMenuTrigger` surface from the upstream docs usage block"
    );

    let basic = read("src/ui/snippets/context_menu/basic.rs");
    assert!(
        basic.contains("shadcn::ContextMenuItem::new(\"Back\")")
            && basic.contains("shadcn::ContextMenuItem::new(\"Forward\")")
            && basic.contains(".disabled(true)")
            && basic.contains("shadcn::ContextMenuItem::new(\"Reload\")"),
        "src/ui/snippets/context_menu/basic.rs should stay aligned with the upstream Basic example (`Back`, disabled `Forward`, `Reload`)"
    );
}

#[test]
fn context_menu_rtl_example_keeps_the_richer_upstream_preview_shape() {
    let rtl = read("src/ui/snippets/context_menu/rtl.rs");
    assert!(
        rtl.contains("ContextMenuContent::new()")
            && rtl.contains(".side(shadcn::DropdownMenuSide::InlineEnd)")
            && rtl.contains("ContextMenuItem::new(\"Navigation\")")
            && rtl.contains("ContextMenuItem::new(\"More Tools\")")
            && rtl.contains("ContextMenuCheckboxItem::from_checked(")
            && rtl.contains("ContextMenuRadioGroup::from_value("),
        "src/ui/snippets/context_menu/rtl.rs should keep the richer upstream RTL preview shape while preserving explicit `inline-end` placement"
    );

    let page = read("src/ui/pages/context_menu.rs");
    assert!(
        page.contains(
            "The RTL preview now stays closer to the upstream Base UI example shape too: dual submenus, checkbox toggles, and a radio group all render under `LayoutDirection::Rtl` while keeping the explicit `inline-end` teaching point."
        ),
        "src/ui/pages/context_menu.rs should record that the RTL section now keeps the richer upstream preview structure"
    );
}

#[test]
fn context_menu_checkboxes_and_radio_examples_stay_docs_aligned() {
    let checkboxes = read("src/ui/snippets/context_menu/checkboxes.rs");
    assert!(
        checkboxes.contains("ContextMenuGroup::new(vec![")
            && checkboxes.contains("\"Show Bookmarks Bar\"")
            && checkboxes.contains("\"Show Full URLs\"")
            && checkboxes.contains("\"Show Developer Tools\""),
        "src/ui/snippets/context_menu/checkboxes.rs should stay aligned with the upstream checkbox labels and grouped structure"
    );

    let radio = read("src/ui/snippets/context_menu/radio.rs");
    assert!(
        radio.contains("ContextMenuLabel::new(\"People\")")
            && radio.contains("ContextMenuLabel::new(\"Theme\")")
            && radio.contains("\"Pedro Duarte\"")
            && radio.contains("\"Colm Tuite\"")
            && radio.contains("\"Light\"")
            && radio.contains("\"Dark\"")
            && radio.contains("\"System\""),
        "src/ui/snippets/context_menu/radio.rs should keep both upstream radio groups (`People` and `Theme`)"
    );
}

#[test]
fn context_menu_groups_and_icons_examples_stay_docs_aligned() {
    let groups = read("src/ui/snippets/context_menu/groups.rs");
    assert!(
        groups.contains("ContextMenuLabel::new(\"File\")")
            && groups.contains("\"New File\"")
            && groups.contains("\"Open File\"")
            && groups.contains("\"Save\"")
            && groups.contains("ContextMenuLabel::new(\"Edit\")")
            && groups.contains("\"Undo\"")
            && groups.contains("\"Redo\"")
            && groups.contains("\"Cut\"")
            && groups.contains("\"Copy\"")
            && groups.contains("\"Paste\"")
            && groups.contains("\"Delete\"")
            && groups.contains("ContextMenuShortcut::new(\"⌘N\")")
            && groups.contains("ContextMenuShortcut::new(\"⌘O\")")
            && groups.contains("ContextMenuShortcut::new(\"⌘S\")")
            && groups.contains("ContextMenuShortcut::new(\"⌘Z\")")
            && groups.contains("ContextMenuShortcut::new(\"⇧⌘Z\")")
            && groups.contains("ContextMenuShortcut::new(\"⌘X\")")
            && groups.contains("ContextMenuShortcut::new(\"⌘C\")")
            && groups.contains("ContextMenuShortcut::new(\"⌘V\")")
            && groups.contains("ContextMenuShortcut::new(\"⌫\")"),
        "src/ui/snippets/context_menu/groups.rs should keep the upstream groups example shape (`File`, `Edit`, clipboard actions, destructive `Delete`) with matching shortcuts"
    );

    let icons = read("src/ui/snippets/context_menu/icons.rs");
    assert!(
        icons.contains("\"Copy\"")
            && icons.contains("\"Cut\"")
            && icons.contains("\"Paste\"")
            && icons.contains("\"Delete\"")
            && icons.contains("IconId::new_static(\"lucide.copy\")")
            && icons.contains("IconId::new_static(\"lucide.scissors\")")
            && icons.contains("IconId::new_static(\"lucide.clipboard-paste\")")
            && icons.contains("IconId::new_static(\"lucide.trash\")")
            && icons.contains("ContextMenuGroup::new(vec!["),
        "src/ui/snippets/context_menu/icons.rs should keep the upstream icons example shape (`Copy`, `Cut`, `Paste`, destructive `Delete`) with the matching lucide icons"
    );
}

#[test]
fn context_menu_submenu_shortcuts_destructive_and_demo_examples_stay_docs_aligned() {
    let submenu = read("src/ui/snippets/context_menu/submenu.rs");
    assert!(
        submenu.contains("ContextMenuShortcut::new(\"⌘C\")")
            && submenu.contains("ContextMenuShortcut::new(\"⌘X\")")
            && submenu.contains("\"Save Page...\"")
            && submenu.contains("\"Create Shortcut...\"")
            && submenu.contains("\"Name Window...\"")
            && submenu.contains("\"Developer Tools\"")
            && submenu.contains("\"Delete\"")
            && submenu.contains("ContextMenuGroup::new(vec!["),
        "src/ui/snippets/context_menu/submenu.rs should stay aligned with the upstream submenu example (copy/cut shortcuts plus the grouped More Tools submenu)"
    );

    let shortcuts = read("src/ui/snippets/context_menu/shortcuts.rs");
    assert!(
        shortcuts.contains("\"Back\"")
            && shortcuts.contains("\"Forward\"")
            && shortcuts.contains(".disabled(true)")
            && shortcuts.contains("\"Reload\"")
            && shortcuts.contains("\"Save\"")
            && shortcuts.contains("\"Save As...\"")
            && shortcuts.contains("ContextMenuShortcut::new(\"⌘[\")")
            && shortcuts.contains("ContextMenuShortcut::new(\"⌘]\")")
            && shortcuts.contains("ContextMenuShortcut::new(\"⌘R\")")
            && shortcuts.contains("ContextMenuShortcut::new(\"⌘S\")")
            && shortcuts.contains("ContextMenuShortcut::new(\"⇧⌘S\")"),
        "src/ui/snippets/context_menu/shortcuts.rs should keep the upstream shortcuts example (`Back`, disabled `Forward`, `Reload`, `Save`, `Save As...`) with matching accelerators"
    );

    let destructive = read("src/ui/snippets/context_menu/destructive.rs");
    assert!(
        destructive.contains("\"Edit\"")
            && destructive.contains("\"Share\"")
            && destructive.contains("\"Delete\"")
            && destructive.contains("IconId::new_static(\"lucide.pencil\")")
            && destructive.contains("IconId::new_static(\"lucide.share\")")
            && destructive.contains("IconId::new_static(\"lucide.trash\")")
            && destructive.contains("ContextMenuItemVariant::Destructive"),
        "src/ui/snippets/context_menu/destructive.rs should stay aligned with the upstream destructive example (`Edit`, `Share`, destructive `Delete`) and matching lucide icons"
    );

    let demo = read("src/ui/snippets/context_menu/demo.rs");
    assert!(
        demo.contains(".min_width(Px(192.0))")
            && demo.contains(".submenu_min_width(Px(176.0))")
            && demo.contains("ContextMenuSub::new(")
            && demo.contains("ContextMenuShortcut::new(\"⌘[\")")
            && demo.contains("\"Show Bookmarks\"")
            && demo.contains("\"Show Full URLs\"")
            && demo.contains("\"Pedro Duarte\"")
            && demo.contains("\"Colm Tuite\""),
        "src/ui/snippets/context_menu/demo.rs should keep the upstream combined docs example shape, including the `w-48` content width and `w-44` submenu width"
    );

    let sides = read("src/ui/snippets/context_menu/sides.rs");
    assert!(
        sides.contains("\"Right click (top)\"")
            && sides.contains("\"Right click (right)\"")
            && sides.contains("\"Right click (bottom)\"")
            && sides.contains("\"Right click (left)\"")
            && sides.contains("DropdownMenuSide::Top")
            && sides.contains("DropdownMenuSide::Right")
            && sides.contains("DropdownMenuSide::Bottom")
            && sides.contains("DropdownMenuSide::Left"),
        "src/ui/snippets/context_menu/sides.rs should keep the upstream sides example labels and explicit side assignments for top/right/bottom/left"
    );
}

#[test]
fn dropdown_menu_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/dropdown_menu/avatar.rs",
            "src/ui/snippets/dropdown_menu/basic.rs",
            "src/ui/snippets/dropdown_menu/checkboxes.rs",
            "src/ui/snippets/dropdown_menu/checkboxes_icons.rs",
            "src/ui/snippets/dropdown_menu/complex.rs",
            "src/ui/snippets/dropdown_menu/demo.rs",
            "src/ui/snippets/dropdown_menu/destructive.rs",
            "src/ui/snippets/dropdown_menu/icons.rs",
            "src/ui/snippets/dropdown_menu/parts.rs",
            "src/ui/snippets/dropdown_menu/radio_group.rs",
            "src/ui/snippets/dropdown_menu/radio_icons.rs",
            "src/ui/snippets/dropdown_menu/rtl.rs",
            "src/ui/snippets/dropdown_menu/shortcuts.rs",
            "src/ui/snippets/dropdown_menu/submenu.rs",
            "src/ui/snippets/dropdown_menu/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/dropdown_menu",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn menu_snippets_keep_build_parts_only_for_the_intentional_parts_example() {
    let dropdown_parts = manifest_path("src/ui/snippets/dropdown_menu/parts.rs");
    let dropdown_parts_source = read_path(&dropdown_parts);
    assert!(
        dropdown_parts_source.contains(".build_parts("),
        "{} should remain the explicit lower-level adapter example",
        dropdown_parts.display()
    );

    for path in rust_sources("src/ui/snippets/dropdown_menu") {
        if path == dropdown_parts {
            continue;
        }

        let source = read_path(&path);
        assert!(
            !source.contains(".build_parts("),
            "{} should prefer the typed `compose()` root on the default snippet surface",
            path.display()
        );
    }

    for path in rust_sources("src/ui/snippets/context_menu") {
        let source = read_path(&path);
        assert!(
            !source.contains(".build_parts("),
            "{} should prefer the typed `compose()` root on the default snippet surface",
            path.display()
        );
    }
}

#[test]
fn dropdown_menu_page_records_why_it_stays_on_the_explicit_entry_tree_surface() {
    let page = read("src/ui/pages/dropdown_menu.rs");
    assert!(
        page.contains(
            "No extra generic heterogeneous children API is currently warranted: the explicit `DropdownMenuEntry` tree is the Fret-equivalent structured surface for upstream nested menu children, and a generic children lane would add hidden scope/collection contracts without unlocking new behavior."
        ),
        "src/ui/pages/dropdown_menu.rs should record why DropdownMenu stays on the explicit entry-tree surface instead of widening to a generic children API"
    );
    assert!(
        page.contains(
            "The lead `Demo` preview now keeps the official `dropdown-menu-demo.tsx` row order more closely, including the `Keyboard shortcuts` action and the ungrouped `GitHub` / `Support` / `API` rows after the second separator."
        ),
        "src/ui/pages/dropdown_menu.rs should record the docs-surface alignment choices for the lead demo preview"
    );
}

#[test]
fn menu_pages_mark_adapter_surfaces_as_advanced_not_default() {
    let dropdown_page = read("src/ui/pages/dropdown_menu.rs");
    assert!(
        dropdown_page
            .contains("advanced adapter surface for already-landed or closure-driven seams"),
        "src/ui/pages/dropdown_menu.rs should explain why `Parts` stays outside the default compose() teaching path"
    );
    assert!(
        dropdown_page.contains(
            "Advanced Trigger/Content adapter surface kept outside the default copyable docs path."
        ),
        "src/ui/pages/dropdown_menu.rs should label the Parts section as an advanced adapter surface"
    );

    let context_page = read("src/ui/pages/context_menu.rs");
    assert!(
        context_page.contains(
            "Those lower-level adapter seams are still advanced API, not the default copyable teaching lane."
        ),
        "src/ui/pages/context_menu.rs should keep the lower-level adapter seams marked as advanced-only guidance"
    );
}

#[test]
fn selected_parts_pages_mark_adapter_surfaces_as_advanced_not_default() {
    let dialog_page = read("src/ui/pages/dialog.rs");
    assert!(
        dialog_page.contains(
            "`Usage` is the default copyable path; `Parts` stays as the advanced adapter section for explicit `DialogTrigger` / `DialogPortal` / `DialogOverlay` ownership."
        ),
        "src/ui/pages/dialog.rs should distinguish the default compose() lane from the advanced Parts lane"
    );
    assert!(
        dialog_page.contains(
            "Advanced part surface adapters for explicit Trigger/Portal/Overlay ownership."
        ),
        "src/ui/pages/dialog.rs should label the Parts section as an advanced adapter surface"
    );

    let sheet_page = read("src/ui/pages/sheet.rs");
    assert!(
        sheet_page.contains(
            "`Usage` is the default copyable path; `Parts` stays after `API Reference` as a focused advanced follow-up for explicit part adapters (`SheetTrigger` / `SheetPortal` / `SheetOverlay`)."
        ),
        "src/ui/pages/sheet.rs should distinguish the default compose() lane from the advanced Parts lane"
    );
    assert!(
        sheet_page.contains(
            "Advanced part surface adapters for explicit Trigger/Portal/Overlay ownership."
        ),
        "src/ui/pages/sheet.rs should label the Parts section as an advanced adapter surface"
    );

    let alert_dialog_page = read("src/ui/pages/alert_dialog.rs");
    assert!(
        alert_dialog_page.contains(
            "`Usage` is the default copyable path; `Parts` remains an advanced adapter lane for explicit root-part ownership."
        ),
        "src/ui/pages/alert_dialog.rs should distinguish the default compose() lane from the advanced Parts lane"
    );
    assert!(
        alert_dialog_page.contains(
            "Advanced part surface adapters for explicit shadcn-style root-part ownership."
        ),
        "src/ui/pages/alert_dialog.rs should label the Parts section as an advanced adapter surface"
    );
}

#[test]
fn menubar_page_distinguishes_compact_and_copyable_parts_lanes() {
    let menubar_page = read("src/ui/pages/menubar.rs");
    assert!(
        menubar_page.contains(
            "Compact Fret-first root authoring uses `Menubar::new([MenubarMenu::new(...).entries([...])])`."
        ),
        "src/ui/pages/menubar.rs should document the compact typed root lane"
    );
    assert!(
        menubar_page.contains(
            "`MenubarTrigger::new(...).into_menu().entries_parts(...)` remains the upstream-shaped copyable lane; the `Parts` section is a focused adapter example on that same lane rather than an advanced escape hatch."
        ),
        "src/ui/pages/menubar.rs should distinguish the compact typed lane from the upstream-shaped parts lane"
    );
    assert!(
        menubar_page
            .contains("Focused Trigger/Content adapter example on the same copyable parts lane."),
        "src/ui/pages/menubar.rs should keep the Parts section on the copyable parts lane rather than marking it as advanced"
    );
}

#[test]
fn carousel_page_distinguishes_compact_builder_and_upstream_parts_lanes() {
    let carousel_page = read("src/ui/pages/carousel.rs");
    assert!(
        carousel_page.contains(
            "`Usage` now mirrors the upstream docs-shaped parts lane, `Compact Builder` keeps the ergonomic Fret shorthand visible, and `Parts` remains the explicit adapter/diagnostics seam on that same copyable lane rather than an advanced escape hatch."
        ),
        "src/ui/pages/carousel.rs should distinguish upstream usage, the compact shorthand lane, and the explicit parts seam"
    );
    assert!(
        !carousel_page
            .contains("Default compact builder path for common Fret carousel call sites."),
        "src/ui/pages/carousel.rs should keep the old compact-builder wording out of the Usage section"
    );
    assert!(
        carousel_page
            .contains("Compact Fret shorthand for common app call sites: `Carousel::new(items)`."),
        "src/ui/pages/carousel.rs should expose Compact Builder as the Fret shorthand lane"
    );
    assert!(
        carousel_page
            .contains("Upstream shadcn docs shape using `CarouselContent`, `CarouselItem`, `CarouselPrevious`, and `CarouselNext`."),
        "src/ui/pages/carousel.rs should label Usage as the docs-aligned upstream-shaped lane"
    );
    assert!(
        carousel_page.contains(
            "Focused adapter example on the same upstream-shaped lane when callers want explicit part values or diagnostics-specific control IDs."
        ),
        "src/ui/pages/carousel.rs should keep Parts as an explicit adapter seam on the upstream-shaped lane"
    );
    assert!(
        carousel_page.contains(
            "`Compact Builder` keeps `Carousel::new(items)` visible for app code, `Parts` keeps the explicit adapter/diagnostics seam visible, and `Loop` is a dedicated `loop=true` preview that the upstream docs only imply through `Options`."
        ),
        "src/ui/pages/carousel.rs should explain why the page switches into Fret follow-ups after the upstream docs path"
    );
    assert!(
        carousel_page
            .contains(".code_rust_from_file_region(snippets::api::DOCS_SOURCE, \"example\")"),
        "src/ui/pages/carousel.rs should show the docs-aligned compact API example source"
    );
    assert!(
        carousel_page
            .contains(".code_rust_from_file_region(snippets::events::DOCS_SOURCE, \"example\")"),
        "src/ui/pages/carousel.rs should show the docs-aligned Events example source"
    );
    assert!(
        carousel_page.contains(
            ".code_rust_from_file_region(snippets::plugin_autoplay::DOCS_SOURCE, \"example\")"
        ),
        "src/ui/pages/carousel.rs should show the docs-aligned Plugin example source"
    );
    assert!(
        carousel_page
            .contains(".code_rust_from_file_region(snippets::rtl::DOCS_SOURCE, \"example\")"),
        "src/ui/pages/carousel.rs should show the docs-aligned RTL example source"
    );
}

#[test]
fn direct_recipe_root_pages_mark_their_default_lane_without_inventing_compose() {
    let select_page = read("src/ui/pages/select.rs");
    assert!(
        select_page.contains(
            "`Select::new(...)` / `new_controllable(...)` plus the direct builder chain (`.trigger(...).value(...).content(...).entries(...)`) are now the default copyable root story; `into_element_parts(...)` remains the focused upstream-shaped adapter on the same lane rather than a separate `compose()` story."
        ),
        "src/ui/pages/select.rs should keep Select on the direct recipe root lane"
    );

    let combobox_page = read("src/ui/pages/combobox.rs");
    assert!(
        combobox_page.contains(
            "`Combobox::new(value, open)` plus the direct builder chain (`.trigger(...).input(...).clear(...).content(...)`) is the default recipe root lane, while `into_element_parts(...)` stays the focused upstream-shaped patch seam on that same lane rather than a separate `compose()` story."
        ),
        "src/ui/pages/combobox.rs should keep Combobox on the direct recipe root lane"
    );

    let command_page = read("src/ui/pages/command.rs");
    assert!(
        command_page.contains(
            "`command(...)` / `CommandPalette` remain the default recipe root story; split `CommandInput` / `CommandList` / `CommandItem` authoring stays out of the default surface until a shared context contract is explicitly introduced."
        ),
        "src/ui/pages/command.rs should keep Command on the direct recipe root lane"
    );
}

#[test]
fn selected_select_snippets_prefer_direct_builder_chain_for_default_recipe_root() {
    assert_sources_absent("src/ui/snippets/select", &[".into_element_parts("]);
}

#[test]
fn selected_combobox_snippets_prefer_direct_builder_chain_for_default_recipe_root() {
    for relative_path in [
        "src/ui/snippets/combobox/basic.rs",
        "src/ui/snippets/combobox/usage.rs",
        "src/ui/snippets/combobox/trigger_button.rs",
        "src/ui/snippets/combobox/label.rs",
        "src/ui/snippets/combobox/clear_button.rs",
        "src/ui/snippets/combobox/groups.rs",
        "src/ui/snippets/combobox/groups_with_separator.rs",
        "src/ui/snippets/combobox/auto_highlight.rs",
        "src/ui/snippets/combobox/disabled.rs",
        "src/ui/snippets/combobox/invalid.rs",
        "src/ui/snippets/combobox/custom_items.rs",
        "src/ui/snippets/combobox/long_list.rs",
        "src/ui/snippets/combobox/input_group.rs",
        "src/ui/snippets/combobox/rtl.rs",
        "src/ui/snippets/combobox/conformance_demo.rs",
        "src/ui/snippets/combobox/multiple_selection.rs",
    ] {
        let normalized = assert_normalized_markers_present(
            relative_path,
            &[".trigger(", ".input(", ".into_element(cx)"],
        );
        assert!(
            normalized.contains("shadcn::Combobox::new(")
                || normalized.contains("shadcn::ComboboxChips::new("),
            "{} should stay on the compact combobox-family root lane",
            manifest_path(relative_path).display()
        );
        assert!(
            !normalized.contains(".into_element_parts("),
            "{} should prefer the direct builder chain over `into_element_parts(...)` on the default root lane",
            manifest_path(relative_path).display()
        );
    }
    assert_sources_absent("src/ui/snippets/combobox", &[".into_element_parts("]);
}

#[test]
fn selected_combobox_input_group_snippet_prefers_typed_input_addons() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/combobox/input_group.rs",
        &[
            ".input(",
            ".children([shadcn::InputGroupAddon::new([icon::icon(",
            ".align(shadcn::InputGroupAddonAlign::InlineStart)",
            "state_rows(cx, &value, &query, \"ui-gallery-combobox-input-group\").into_element(cx)",
        ],
        &["ui::h_row(|cx|"],
    );

    let page = read("src/ui/pages/combobox.rs");
    assert!(
        page.contains(
            "`Extras: Input Group` demonstrates typed `ComboboxInput::children([InputGroupAddon...])` composition for inline addons; keep that surface narrow instead of widening to generic arbitrary children."
        ),
        "src/ui/pages/combobox.rs should document the typed ComboboxInput addon surface instead of teaching a generic children escape hatch"
    );
}

#[test]
fn navigation_menu_and_pagination_pages_keep_their_dual_lane_story() {
    let navigation_menu_page = read("src/ui/pages/navigation_menu.rs");
    assert!(
        navigation_menu_page.contains(
            "`navigation_menu(cx, model, |cx| ..)` is now the default first-party root helper, while `NavigationMenu::new(model)` remains available when callers want the explicit root builder seam."
        ),
        "src/ui/pages/navigation_menu.rs should keep the compact default lane explicit"
    );
    assert!(
        navigation_menu_page.contains(
            "`NavigationMenuRoot/List/Item/Trigger/Content/Link/Viewport/Indicator` remain the upstream-shaped lane on the same family rather than an advanced escape hatch."
        ),
        "src/ui/pages/navigation_menu.rs should keep the upstream-shaped lane explicit"
    );

    let pagination_page = read("src/ui/pages/pagination.rs");
    assert!(
        pagination_page.contains(
            "`Usage` now teaches the upstream-shaped parts lane directly: `Pagination`, `PaginationContent`, `PaginationItem`, and `PaginationLink` already support explicit composable children without needing an extra generic `compose()` API."
        ),
        "src/ui/pages/pagination.rs should keep the upstream-shaped parts lane explicit"
    );
    assert!(
        pagination_page.contains(
            "`Compact Builder` keeps the Fret shorthand lane explicit for common app call sites: `pagination(...)`, `pagination_content(...)`, `pagination_item(...)`, and `pagination_link(...)` reduce child landing noise without replacing the upstream-shaped parts surface."
        ),
        "src/ui/pages/pagination.rs should keep the compact wrapper lane explicit"
    );
}

#[test]
fn dropdown_menu_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/dropdown_menu.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Submenu\", submenu)",
            "DocSection::build(cx, \"Shortcuts\", shortcuts)",
            "DocSection::build(cx, \"Icons\", icons)",
            "DocSection::build(cx, \"Checkboxes\", checkboxes)",
            "DocSection::build(cx, \"Checkboxes Icons\", checkboxes_icons)",
            "DocSection::build(cx, \"Radio Group\", radio_group)",
            "DocSection::build(cx, \"Radio Icons\", radio_icons)",
            "DocSection::build(cx, \"Destructive\", destructive)",
            "DocSection::build(cx, \"Avatar\", avatar)",
            "DocSection::build(cx, \"Complex\", complex)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Parts\", parts)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Submenu\", submenu)",
            "DocSection::new(\"Shortcuts\", shortcuts)",
            "DocSection::new(\"Icons\", icons)",
            "DocSection::new(\"Checkboxes\", checkboxes)",
            "DocSection::new(\"Checkboxes Icons\", checkboxes_icons)",
            "DocSection::new(\"Radio Group\", radio_group)",
            "DocSection::new(\"Radio Icons\", radio_icons)",
            "DocSection::new(\"Destructive\", destructive)",
            "DocSection::new(\"Avatar\", avatar)",
            "DocSection::new(\"Complex\", complex)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Parts\", parts)",
        ],
    );
}

#[test]
fn dropdown_menu_remaining_examples_stay_base_docs_aligned() {
    let checkboxes = read("src/ui/snippets/dropdown_menu/checkboxes.rs");
    assert!(
        checkboxes.contains(".min_width(Px(160.0))"),
        "src/ui/snippets/dropdown_menu/checkboxes.rs should keep the base docs `w-40` content width"
    );
    assert!(
        !checkboxes.contains(".min_width(Px(224.0))"),
        "src/ui/snippets/dropdown_menu/checkboxes.rs should not drift back to the old `w-56` width"
    );

    let checkboxes_icons = read("src/ui/snippets/dropdown_menu/checkboxes_icons.rs");
    assert!(
        checkboxes_icons.contains("Button::new(\"Notifications\")")
            && checkboxes_icons.contains("DropdownMenuLabel::new(\"Notification Preferences\")")
            && checkboxes_icons.contains("\"Email notifications\"")
            && checkboxes_icons.contains("\"SMS notifications\"")
            && checkboxes_icons.contains("\"Push notifications\"")
            && checkboxes_icons.contains("IconId::new_static(\"lucide.mail\")")
            && checkboxes_icons.contains("IconId::new_static(\"lucide.message-square\")")
            && checkboxes_icons.contains("IconId::new_static(\"lucide.bell\")")
            && checkboxes_icons.contains(".min_width(Px(192.0))"),
        "src/ui/snippets/dropdown_menu/checkboxes_icons.rs should stay aligned with the base docs notifications example, including labels, icons, and `w-48` width"
    );

    let radio_group = read("src/ui/snippets/dropdown_menu/radio_group.rs");
    assert!(
        radio_group.contains(".min_width(Px(128.0))"),
        "src/ui/snippets/dropdown_menu/radio_group.rs should keep the base docs `w-32` content width"
    );
    assert!(
        !radio_group.contains(".min_width(Px(224.0))"),
        "src/ui/snippets/dropdown_menu/radio_group.rs should not drift back to the old `w-56` width"
    );

    let avatar = read("src/ui/snippets/dropdown_menu/avatar.rs");
    assert!(
        avatar.contains("demo_image(cx)")
            && avatar.contains("AvatarImage::maybe(avatar_image)")
            && avatar.contains(".when_image_missing(avatar_image)")
            && avatar.contains(".delay_ms(120)"),
        "src/ui/snippets/dropdown_menu/avatar.rs should keep the shared demo image + fallback pipeline from the upstream avatar example"
    );
    assert!(
        !avatar.contains(".min_width(Px("),
        "src/ui/snippets/dropdown_menu/avatar.rs should not reintroduce an explicit content width absent from the base docs example"
    );

    let rtl = read("src/ui/snippets/dropdown_menu/rtl.rs");
    assert!(
        rtl.contains("with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {")
            && rtl.contains(".align(shadcn::DropdownMenuAlign::End)")
            && rtl.contains(".min_width(Px(144.0))")
            && rtl.contains("DropdownMenuSubTrigger::new(\"الحساب\")")
            && rtl.contains("DropdownMenuSubTrigger::new(\"دعوة المستخدمين\")")
            && rtl.contains("DropdownMenuSubTrigger::new(\"المزيد\")")
            && rtl.contains("DropdownMenuCheckboxItem::from_checked(")
            && rtl.contains("DropdownMenuRadioGroup::from_value(")
            && rtl.contains("DropdownMenuItem::new(\"تسجيل الخروج\")"),
        "src/ui/snippets/dropdown_menu/rtl.rs should keep the richer base docs RTL preview shape: submenu stack, checkbox section, radio group, end alignment, and destructive logout"
    );
}

#[test]
fn menubar_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/menubar/checkbox.rs",
            "src/ui/snippets/menubar/demo.rs",
            "src/ui/snippets/menubar/parts.rs",
            "src/ui/snippets/menubar/radio.rs",
            "src/ui/snippets/menubar/rtl.rs",
            "src/ui/snippets/menubar/submenu.rs",
            "src/ui/snippets/menubar/usage.rs",
            "src/ui/snippets/menubar/with_icons.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/menubar",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn menubar_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/menubar.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Checkbox\", checkbox)",
            "DocSection::build(cx, \"Radio\", radio)",
            "DocSection::build(cx, \"Submenu\", submenu)",
            "DocSection::build(cx, \"With Icons\", with_icons)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Parts\", parts)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Checkbox\", checkbox)",
            "DocSection::new(\"Radio\", radio)",
            "DocSection::new(\"Submenu\", submenu)",
            "DocSection::new(\"With Icons\", with_icons)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Parts\", parts)",
        ],
    );
}

#[test]
fn button_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/button/demo.rs",
            "src/ui/snippets/button/usage.rs",
            "src/ui/snippets/button/size.rs",
            "src/ui/snippets/button/default.rs",
            "src/ui/snippets/button/outline.rs",
            "src/ui/snippets/button/secondary.rs",
            "src/ui/snippets/button/ghost.rs",
            "src/ui/snippets/button/destructive.rs",
            "src/ui/snippets/button/link.rs",
            "src/ui/snippets/button/icon.rs",
            "src/ui/snippets/button/with_icon.rs",
            "src/ui/snippets/button/rounded.rs",
            "src/ui/snippets/button/loading.rs",
            "src/ui/snippets/button/button_group.rs",
            "src/ui/snippets/button/link_render.rs",
            "src/ui/snippets/button/rtl.rs",
            "src/ui/snippets/button/variants.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/button",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn button_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/button.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Size\", size)",
            "DocSection::build(cx, \"Default\", default)",
            "DocSection::build(cx, \"Outline\", outline)",
            "DocSection::build(cx, \"Secondary\", secondary)",
            "DocSection::build(cx, \"Ghost\", ghost)",
            "DocSection::build(cx, \"Destructive\", destructive)",
            "DocSection::build(cx, \"Link\", link)",
            "DocSection::build(cx, \"Icon\", icon_only)",
            "DocSection::build(cx, \"With Icon\", with_icon)",
            "DocSection::build(cx, \"Rounded\", rounded)",
            "DocSection::build(cx, \"Spinner\", spinner)",
            "DocSection::build(cx, \"Button Group\", button_group)",
            "DocSection::build(cx, \"Link (Semantic)\", link_render)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Variants Overview (Fret)\", variants)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Size\", size)",
            "DocSection::new(\"Default\", default)",
            "DocSection::new(\"Outline\", outline)",
            "DocSection::new(\"Secondary\", secondary)",
            "DocSection::new(\"Ghost\", ghost)",
            "DocSection::new(\"Destructive\", destructive)",
            "DocSection::new(\"Link\", link)",
            "DocSection::new(\"Icon\", icon_only)",
            "DocSection::new(\"With Icon\", with_icon)",
            "DocSection::new(\"Rounded\", rounded)",
            "DocSection::new(\"Spinner\", spinner)",
            "DocSection::new(\"Button Group\", button_group)",
            "DocSection::new(\"Link (Semantic)\", link_render)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Variants Overview (Fret)\", variants)",
        ],
    );
}

#[test]
fn button_group_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/button_group/accessibility.rs",
            "src/ui/snippets/button_group/button_group_select.rs",
            "src/ui/snippets/button_group/demo.rs",
            "src/ui/snippets/button_group/dropdown_menu.rs",
            "src/ui/snippets/button_group/flex_1_items.rs",
            "src/ui/snippets/button_group/input.rs",
            "src/ui/snippets/button_group/input_group.rs",
            "src/ui/snippets/button_group/nested.rs",
            "src/ui/snippets/button_group/orientation.rs",
            "src/ui/snippets/button_group/popover.rs",
            "src/ui/snippets/button_group/rtl.rs",
            "src/ui/snippets/button_group/separator.rs",
            "src/ui/snippets/button_group/size.rs",
            "src/ui/snippets/button_group/split.rs",
            "src/ui/snippets/button_group/text.rs",
            "src/ui/snippets/button_group/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/button_group",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn button_group_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/button_group.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Accessibility\", accessibility)",
            "DocSection::build(cx, \"Orientation\", orientation)",
            "DocSection::build(cx, \"Size\", size)",
            "DocSection::build(cx, \"Nested\", nested)",
            "DocSection::build(cx, \"Separator\", separator)",
            "DocSection::build(cx, \"Split\", split)",
            "DocSection::build(cx, \"Input\", input)",
            "DocSection::build(cx, \"Input Group\", input_group)",
            "DocSection::build(cx, \"Dropdown Menu\", dropdown)",
            "DocSection::build(cx, \"Select\", select)",
            "DocSection::build(cx, \"Popover\", popover)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"ButtonGroupText\", text)",
            "DocSection::build(cx, \"Flex-1 items (Fret)\", flex_1)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Accessibility\", accessibility)",
            "DocSection::new(\"Orientation\", orientation)",
            "DocSection::new(\"Size\", size)",
            "DocSection::new(\"Nested\", nested)",
            "DocSection::new(\"Separator\", separator)",
            "DocSection::new(\"Split\", split)",
            "DocSection::new(\"Input\", input)",
            "DocSection::new(\"Input Group\", input_group)",
            "DocSection::new(\"Dropdown Menu\", dropdown)",
            "DocSection::new(\"Select\", select)",
            "DocSection::new(\"Popover\", popover)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"ButtonGroupText\", text)",
            "DocSection::new(\"Flex-1 items (Fret)\", flex_1)",
        ],
    );
}

#[test]
fn input_group_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/input_group/align_block_end.rs",
            "src/ui/snippets/input_group/align_block_start.rs",
            "src/ui/snippets/input_group/align_inline_end.rs",
            "src/ui/snippets/input_group/align_inline_start.rs",
            "src/ui/snippets/input_group/button.rs",
            "src/ui/snippets/input_group/button_group.rs",
            "src/ui/snippets/input_group/custom_input.rs",
            "src/ui/snippets/input_group/demo.rs",
            "src/ui/snippets/input_group/dropdown.rs",
            "src/ui/snippets/input_group/icon.rs",
            "src/ui/snippets/input_group/kbd.rs",
            "src/ui/snippets/input_group/label.rs",
            "src/ui/snippets/input_group/rtl.rs",
            "src/ui/snippets/input_group/spinner.rs",
            "src/ui/snippets/input_group/text.rs",
            "src/ui/snippets/input_group/textarea.rs",
            "src/ui/snippets/input_group/tooltip.rs",
            "src/ui/snippets/input_group/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/input_group",
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement",
            ".into_element_parts(",
        ],
    );
}

#[test]
fn input_group_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/input_group.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Align / inline-start\", align_inline_start)",
            "DocSection::build(cx, \"Align / inline-end\", align_inline_end)",
            "DocSection::build(cx, \"Align / block-start\", align_block_start)",
            "DocSection::build(cx, \"Align / block-end\", align_block_end)",
            "DocSection::build(cx, \"Icon\", icon)",
            "DocSection::build(cx, \"Text\", text)",
            "DocSection::build(cx, \"Button\", button)",
            "DocSection::build(cx, \"Kbd\", kbd)",
            "DocSection::build(cx, \"Dropdown\", dropdown)",
            "DocSection::build(cx, \"Spinner\", spinner)",
            "DocSection::build(cx, \"Textarea\", textarea)",
            "DocSection::build(cx, \"Custom Input\", custom_input)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Tooltip\", tooltip)",
            "DocSection::build(cx, \"Label Association\", label)",
            "DocSection::build(cx, \"Button Group\", button_group)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Align / inline-start\", align_inline_start)",
            "DocSection::new(\"Align / inline-end\", align_inline_end)",
            "DocSection::new(\"Align / block-start\", align_block_start)",
            "DocSection::new(\"Align / block-end\", align_block_end)",
            "DocSection::new(\"Icon\", icon)",
            "DocSection::new(\"Text\", text)",
            "DocSection::new(\"Button\", button)",
            "DocSection::new(\"Kbd\", kbd)",
            "DocSection::new(\"Dropdown\", dropdown)",
            "DocSection::new(\"Spinner\", spinner)",
            "DocSection::new(\"Textarea\", textarea)",
            "DocSection::new(\"Custom Input\", custom_input)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Tooltip\", tooltip)",
            "DocSection::new(\"Label Association\", label)",
            "DocSection::new(\"Button Group\", button_group)",
        ],
    );
}

#[test]
fn selected_input_group_snippets_prefer_compact_slot_shorthand() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/input_group/dropdown.rs",
        &[
            "shadcn::DropdownMenu::uncontrolled(cx)",
            ".compose()",
            ".trigger(trigger)",
            "shadcn::InputGroup::new(value)",
            ".placeholder(\"Enter file name\")",
            ".control_test_id(\"ui-gallery-input-group-dropdown-control\")",
            ".trailing([dropdown])",
            ".trailing_has_button(true)",
            ".into_element(cx)",
        ],
        &[".into_element_parts(", ".build_parts("],
    );

    let page = read("src/ui/pages/input_group.rs");
    assert!(
        page.contains(
            "Prefer the high-level `InputGroup::new(model)` shorthand for first-party app code; keep the part-based surface for direct shadcn docs parity when you explicitly want addon/control parts."
        ),
        "src/ui/pages/input_group.rs should keep the compact shorthand as the first-party usage lane"
    );
    assert!(
        page.contains(".code_rust_from_file_region(snippets::usage::SOURCE, \"example\")"),
        "src/ui/pages/input_group.rs should show the compact shorthand Usage section from a real snippet file"
    );
    assert!(
        !page.contains(".code_rust("),
        "src/ui/pages/input_group.rs should avoid page-local hand-written Rust strings for Usage"
    );
    assert!(
        page.contains(
            "Both public surfaces stay intentional: the compact `InputGroup::new(model)` slot shorthand is the first-party ergonomic lane, while the part-based primitives remain the direct docs-parity lane."
        ),
        "src/ui/pages/input_group.rs should keep the dual-lane narrative explicit"
    );
    assert!(
        page.contains(
            "The `Dropdown` example intentionally stays on `DropdownMenu::compose()`; swapping the trigger to `InputGroupButton` does not by itself require falling back to `build_parts(...)`."
        ),
        "src/ui/pages/input_group.rs should keep nested dropdown triggers on the default DropdownMenu compose lane when no lower-level adapter seam is needed"
    );
    assert!(
        page.contains(
            "`Custom Input` now uses the narrow `custom_input(...)` / `custom_textarea(...)` seam for caller-owned controls; a generic root `children(...)` API is still intentionally absent."
        ),
        "src/ui/pages/input_group.rs should document the narrow custom-control seam instead of widening InputGroup to generic root children"
    );
}

#[test]
fn input_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/input/badge.rs",
            "src/ui/snippets/input/basic.rs",
            "src/ui/snippets/input/button_group.rs",
            "src/ui/snippets/input/demo.rs",
            "src/ui/snippets/input/disabled.rs",
            "src/ui/snippets/input/field.rs",
            "src/ui/snippets/input/field_group.rs",
            "src/ui/snippets/input/file.rs",
            "src/ui/snippets/input/form.rs",
            "src/ui/snippets/input/grid.rs",
            "src/ui/snippets/input/inline.rs",
            "src/ui/snippets/input/input_group.rs",
            "src/ui/snippets/input/invalid.rs",
            "src/ui/snippets/input/label.rs",
            "src/ui/snippets/input/required.rs",
            "src/ui/snippets/input/rtl.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/input",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn input_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/input.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Field\", field)",
            "DocSection::build(cx, \"Field Group\", field_group)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"Invalid\", invalid)",
            "DocSection::build(cx, \"File\", file)",
            "DocSection::build(cx, \"Inline\", inline)",
            "DocSection::build(cx, \"Grid\", grid)",
            "DocSection::build(cx, \"Required\", required)",
            "DocSection::build(cx, \"Badge\", badge)",
            "DocSection::build(cx, \"Input Group\", input_group)",
            "DocSection::build(cx, \"Button Group\", button_group)",
            "DocSection::build(cx, \"Form\", form)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Label Association\", label)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Field\", field)",
            "DocSection::new(\"Field Group\", field_group)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"Invalid\", invalid)",
            "DocSection::new(\"File\", file)",
            "DocSection::new(\"Inline\", inline)",
            "DocSection::new(\"Grid\", grid)",
            "DocSection::new(\"Required\", required)",
            "DocSection::new(\"Badge\", badge)",
            "DocSection::new(\"Input Group\", input_group)",
            "DocSection::new(\"Button Group\", button_group)",
            "DocSection::new(\"Form\", form)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Label Association\", label)",
        ],
    );
}

#[test]
fn field_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/field/anatomy.rs",
            "src/ui/snippets/field/checkbox.rs",
            "src/ui/snippets/field/choice_card.rs",
            "src/ui/snippets/field/field_group.rs",
            "src/ui/snippets/field/fieldset.rs",
            "src/ui/snippets/field/input.rs",
            "src/ui/snippets/field/radio.rs",
            "src/ui/snippets/field/responsive.rs",
            "src/ui/snippets/field/rtl.rs",
            "src/ui/snippets/field/select.rs",
            "src/ui/snippets/field/slider.rs",
            "src/ui/snippets/field/switch.rs",
            "src/ui/snippets/field/textarea.rs",
            "src/ui/snippets/field/usage.rs",
            "src/ui/snippets/field/validation_and_errors.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/field",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn field_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/field.rs",
        &[
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Anatomy\", anatomy)",
            "DocSection::build(cx, \"Input\", input)",
            "DocSection::build(cx, \"Textarea\", textarea)",
            "DocSection::build(cx, \"Select\", select)",
            "DocSection::build(cx, \"Slider\", slider)",
            "DocSection::build(cx, \"Fieldset\", fieldset)",
            "DocSection::build(cx, \"Checkbox\", checkbox)",
            "DocSection::build(cx, \"Radio\", radio)",
            "DocSection::build(cx, \"Switch\", switch)",
            "DocSection::build(cx, \"Choice Card\", choice_card)",
            "DocSection::build(cx, \"Field Group\", field_group)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Responsive Layout\", responsive)",
            "DocSection::build(cx, \"Validation and Errors\", validation_and_errors)",
        ],
        &[
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Anatomy\", anatomy)",
            "DocSection::new(\"Input\", input)",
            "DocSection::new(\"Textarea\", textarea)",
            "DocSection::new(\"Select\", select)",
            "DocSection::new(\"Slider\", slider)",
            "DocSection::new(\"Fieldset\", fieldset)",
            "DocSection::new(\"Checkbox\", checkbox)",
            "DocSection::new(\"Radio\", radio)",
            "DocSection::new(\"Switch\", switch)",
            "DocSection::new(\"Choice Card\", choice_card)",
            "DocSection::new(\"Field Group\", field_group)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Responsive Layout\", responsive)",
            "DocSection::new(\"Validation and Errors\", validation_and_errors)",
        ],
    );
}

#[test]
fn textarea_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/textarea/button.rs",
            "src/ui/snippets/textarea/demo.rs",
            "src/ui/snippets/textarea/disabled.rs",
            "src/ui/snippets/textarea/field.rs",
            "src/ui/snippets/textarea/invalid.rs",
            "src/ui/snippets/textarea/label.rs",
            "src/ui/snippets/textarea/rtl.rs",
            "src/ui/snippets/textarea/usage.rs",
            "src/ui/snippets/textarea/with_text.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/textarea",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn textarea_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/textarea.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Field\", field)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"Invalid\", invalid)",
            "DocSection::build(cx, \"Button\", button)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"With Text\", with_text)",
            "DocSection::build(cx, \"Label Association\", label)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Field\", field)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"Invalid\", invalid)",
            "DocSection::new(\"Button\", button)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"With Text\", with_text)",
            "DocSection::new(\"Label Association\", label)",
        ],
    );
}

#[test]
fn input_otp_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/input_otp/alphanumeric.rs",
            "src/ui/snippets/input_otp/compact_builder.rs",
            "src/ui/snippets/input_otp/controlled.rs",
            "src/ui/snippets/input_otp/demo.rs",
            "src/ui/snippets/input_otp/disabled.rs",
            "src/ui/snippets/input_otp/form.rs",
            "src/ui/snippets/input_otp/four_digits.rs",
            "src/ui/snippets/input_otp/invalid.rs",
            "src/ui/snippets/input_otp/pattern.rs",
            "src/ui/snippets/input_otp/rtl.rs",
            "src/ui/snippets/input_otp/separator.rs",
            "src/ui/snippets/input_otp/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/input_otp",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn input_otp_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/input_otp.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"About\", about)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Pattern\", pattern)",
            "DocSection::build(cx, \"Separator\", separator)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"Controlled\", controlled)",
            "DocSection::build(cx, \"Invalid\", invalid)",
            "DocSection::build(cx, \"Four Digits\", four_digits)",
            "DocSection::build(cx, \"Alphanumeric\", alphanumeric)",
            "DocSection::build(cx, \"Form\", form)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"API Reference\", api_reference)",
            "DocSection::build(cx, \"Compact Builder\", compact_builder)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"About\", about)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Pattern\", pattern)",
            "DocSection::new(\"Separator\", separator)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"Controlled\", controlled)",
            "DocSection::new(\"Invalid\", invalid)",
            "DocSection::new(\"Four Digits\", four_digits)",
            "DocSection::new(\"Alphanumeric\", alphanumeric)",
            "DocSection::new(\"Form\", form)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Compact Builder\", compact_builder)",
        ],
    );
}

#[test]
fn input_otp_gallery_keeps_docs_bridge_and_compact_builder_lanes_distinct() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/input_otp/usage.rs",
        &[
            "shadcn::InputOTP::new(",
            ".into_element_parts(",
            "shadcn::InputOTPGroup::new([",
            "shadcn::InputOTPSeparator::default().into()",
        ],
        &[],
    );
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/input_otp/compact_builder.rs",
        &[
            "shadcn::InputOTP::new(",
            ".group_size(Some(3))",
            ".into_element(cx)",
        ],
        &[".into_element_parts("],
    );

    for relative_path in [
        "src/ui/snippets/input_otp/alphanumeric.rs",
        "src/ui/snippets/input_otp/compact_builder.rs",
        "src/ui/snippets/input_otp/controlled.rs",
        "src/ui/snippets/input_otp/demo.rs",
        "src/ui/snippets/input_otp/disabled.rs",
        "src/ui/snippets/input_otp/form.rs",
        "src/ui/snippets/input_otp/four_digits.rs",
        "src/ui/snippets/input_otp/invalid.rs",
        "src/ui/snippets/input_otp/pattern.rs",
        "src/ui/snippets/input_otp/rtl.rs",
        "src/ui/snippets/input_otp/separator.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::InputOTP::new(", ".into_element(cx)"],
            &[".into_element_parts("],
        );
    }

    for relative_path in [
        "src/ui/snippets/input_otp/alphanumeric.rs",
        "src/ui/snippets/input_otp/compact_builder.rs",
        "src/ui/snippets/input_otp/demo.rs",
        "src/ui/snippets/input_otp/disabled.rs",
        "src/ui/snippets/input_otp/form.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[".group_size(Some(3))"],
            &[],
        );
    }

    for relative_path in [
        "src/ui/snippets/input_otp/invalid.rs",
        "src/ui/snippets/input_otp/separator.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[".group_size(Some(2))"],
            &[],
        );
    }

    let page = read("src/ui/pages/input_otp.rs");
    assert!(
        page.contains(
            "`Usage` now mirrors the upstream parts-shaped docs lane, while `Compact Builder` keeps `InputOTP::new(model)` plus `group_size(...)` visible as the Fret shorthand after the docs path."
        ),
        "src/ui/pages/input_otp.rs should explain the split between the docs bridge and the compact shorthand"
    );
    assert!(
        page.contains(
            "`InputOTPGroup` / `InputOTPSlot` / `InputOTPSeparator` plus `into_element_parts(...)` already cover the docs-shaped composition bridge, so a separate generic children API is not needed here."
        ),
        "src/ui/pages/input_otp.rs should explain why the existing bridge is sufficient"
    );
    assert!(
        page.contains(
            "Preview mirrors the shadcn Input OTP docs path first: Demo, About, Usage, Pattern, Separator, Disabled, Controlled, Invalid, Four Digits, Alphanumeric, Form, RTL, API Reference. `Compact Builder` stays as the explicit Fret shorthand follow-up."
        ),
        "src/ui/pages/input_otp.rs should mirror the shadcn docs path before the compact follow-up"
    );
}

#[test]
fn select_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/select/align_item_with_trigger.rs",
            "src/ui/snippets/select/demo.rs",
            "src/ui/snippets/select/diag_surface.rs",
            "src/ui/snippets/select/disabled.rs",
            "src/ui/snippets/select/groups.rs",
            "src/ui/snippets/select/invalid.rs",
            "src/ui/snippets/select/label.rs",
            "src/ui/snippets/select/rtl.rs",
            "src/ui/snippets/select/scrollable.rs",
            "src/ui/snippets/select/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/select",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn select_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/select.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"API Reference\", api_reference)",
            "DocSection::build(cx, \"Label Association\", label)",
            "DocSection::build(cx, \"Diag Surface\", diag_surface)",
            "DocSection::build(cx, \"Align Item With Trigger\", align_item)",
            "DocSection::build(cx, \"Groups\", groups)",
            "DocSection::build(cx, \"Scrollable\", scrollable)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"Invalid\", invalid)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Label Association\", label)",
            "DocSection::new(\"Diag Surface\", diag_surface)",
            "DocSection::new(\"Align Item With Trigger\", align_item)",
            "DocSection::new(\"Groups\", groups)",
            "DocSection::new(\"Scrollable\", scrollable)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"Invalid\", invalid)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn calendar_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/calendar/basic.rs",
            "src/ui/snippets/calendar/booked_dates.rs",
            "src/ui/snippets/calendar/custom_cell_size.rs",
            "src/ui/snippets/calendar/date_and_time_picker.rs",
            "src/ui/snippets/calendar/date_of_birth_picker.rs",
            "src/ui/snippets/calendar/demo.rs",
            "src/ui/snippets/calendar/hijri.rs",
            "src/ui/snippets/calendar/locale.rs",
            "src/ui/snippets/calendar/month_year_selector.rs",
            "src/ui/snippets/calendar/natural_language_picker.rs",
            "src/ui/snippets/calendar/presets.rs",
            "src/ui/snippets/calendar/range.rs",
            "src/ui/snippets/calendar/responsive_mixed_semantics.rs",
            "src/ui/snippets/calendar/rtl.rs",
            "src/ui/snippets/calendar/usage.rs",
            "src/ui/snippets/calendar/week_numbers.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/calendar",
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement",
            "-> AnyElement",
        ],
    );
}

#[test]
fn calendar_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/calendar.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Persian / Hijri / Jalali Calendar\", hijri)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Range Calendar\", range)",
            "DocSection::build(cx, \"Month and Year Selector\", month_year_selector)",
            "DocSection::build(cx, \"Presets\", presets)",
            "DocSection::build(cx, \"Date and Time Picker\", date_and_time_picker)",
            "DocSection::build(cx, \"Booked dates\", booked_dates)",
            "DocSection::build(cx, \"Custom Cell Size\", custom_cell_size)",
            "DocSection::build(cx, \"Week Numbers\", week_numbers)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Date of Birth Picker\", date_of_birth_picker)",
            "DocSection::build(cx, \"Natural Language Picker\", natural_language_picker)",
            "DocSection::build(cx, \"Locale (WIP)\", locale)",
            "DocSection::build(cx, \"Responsive semantics (Fret)\", responsive_mixed_semantics,)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Persian / Hijri / Jalali Calendar\", hijri)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Range Calendar\", range)",
            "DocSection::new(\"Month and Year Selector\", month_year_selector)",
            "DocSection::new(\"Presets\", presets)",
            "DocSection::new(\"Date and Time Picker\", date_and_time_picker)",
            "DocSection::new(\"Booked dates\", booked_dates)",
            "DocSection::new(\"Custom Cell Size\", custom_cell_size)",
            "DocSection::new(\"Week Numbers\", week_numbers)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Date of Birth Picker\", date_of_birth_picker)",
            "DocSection::new(\"Natural Language Picker\", natural_language_picker)",
            "DocSection::new(\"Locale (WIP)\", locale)",
            "DocSection::new(\"Responsive semantics (Fret)\", responsive_mixed_semantics)",
        ],
    );
}

#[test]
fn alert_dialog_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/alert_dialog/basic.rs",
            "src/ui/snippets/alert_dialog/demo.rs",
            "src/ui/snippets/alert_dialog/destructive.rs",
            "src/ui/snippets/alert_dialog/detached_trigger.rs",
            "src/ui/snippets/alert_dialog/media.rs",
            "src/ui/snippets/alert_dialog/parts.rs",
            "src/ui/snippets/alert_dialog/rich_content.rs",
            "src/ui/snippets/alert_dialog/rtl.rs",
            "src/ui/snippets/alert_dialog/small.rs",
            "src/ui/snippets/alert_dialog/small_with_media.rs",
            "src/ui/snippets/alert_dialog/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/alert_dialog",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn alert_dialog_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/alert_dialog.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Small\", small)",
            "DocSection::build(cx, \"Media\", media)",
            "DocSection::build(cx, \"Small with Media\", small_with_media)",
            "DocSection::build(cx, \"Destructive\", destructive)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Parts\", parts)",
            "DocSection::build(cx, \"Detached Trigger\", detached_trigger)",
            "DocSection::build(cx, \"Rich Content\", rich_content)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Small\", small)",
            "DocSection::new(\"Media\", media)",
            "DocSection::new(\"Small with Media\", small_with_media)",
            "DocSection::new(\"Destructive\", destructive)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Parts\", parts)",
            "DocSection::new(\"Detached Trigger\", detached_trigger)",
            "DocSection::new(\"Rich Content\", rich_content)",
        ],
    );
}

#[test]
fn dialog_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/dialog/custom_close_button.rs",
            "src/ui/snippets/dialog/demo.rs",
            "src/ui/snippets/dialog/no_close_button.rs",
            "src/ui/snippets/dialog/parts.rs",
            "src/ui/snippets/dialog/rtl.rs",
            "src/ui/snippets/dialog/scrollable_content.rs",
            "src/ui/snippets/dialog/sticky_footer.rs",
            "src/ui/snippets/dialog/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/dialog",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn dialog_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/dialog.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Custom Close Button\", custom_close)",
            "DocSection::build(cx, \"No Close Button\", no_close)",
            "DocSection::build(cx, \"Sticky Footer\", sticky_footer)",
            "DocSection::build(cx, \"Scrollable Content\", scrollable_content)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Parts\", parts)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Custom Close Button\", custom_close)",
            "DocSection::new(\"No Close Button\", no_close)",
            "DocSection::new(\"Sticky Footer\", sticky_footer)",
            "DocSection::new(\"Scrollable Content\", scrollable_content)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Parts\", parts)",
        ],
    );
}

#[test]
fn drawer_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/drawer/demo.rs",
            "src/ui/snippets/drawer/responsive_dialog.rs",
            "src/ui/snippets/drawer/rtl.rs",
            "src/ui/snippets/drawer/scrollable_content.rs",
            "src/ui/snippets/drawer/sides.rs",
            "src/ui/snippets/drawer/snap_points.rs",
            "src/ui/snippets/drawer/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/drawer",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn drawer_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/drawer.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Scrollable Content\", scrollable_content)",
            "DocSection::build(cx, \"Sides\", sides)",
            "DocSection::build(cx, \"Responsive Dialog\", responsive_dialog)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Snap Points\", snap_points)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Scrollable Content\", scrollable_content)",
            "DocSection::new(\"Sides\", sides)",
            "DocSection::new(\"Responsive Dialog\", responsive_dialog)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Snap Points\", snap_points)",
        ],
    );
}

#[test]
fn drawer_page_marks_usage_as_default_and_snap_points_as_follow_up() {
    let drawer_page = read("src/ui/pages/drawer.rs");
    assert!(
        drawer_page.contains(
            "`Usage` is the default copyable `children([...])` path, while `Snap Points` stays a Vaul/Fret policy follow-up rather than a separate root-authoring lane."
        ),
        "src/ui/pages/drawer.rs should distinguish the default children() lane from the Vaul/Fret follow-up lane"
    );
    assert!(
        drawer_page
            .contains("Default copyable `children([...])` path for common Drawer call sites."),
        "src/ui/pages/drawer.rs should label Usage as the default copyable children() path"
    );
    assert!(
        drawer_page.contains(
            "Vaul/Fret policy follow-up built on the same Drawer root while drag settles to the nearest snap point."
        ),
        "src/ui/pages/drawer.rs should keep Snap Points documented as a follow-up on the same root lane"
    );
}

#[test]
fn drawer_snippets_prefer_children_root_path() {
    for relative_path in [
        "src/ui/snippets/drawer/demo.rs",
        "src/ui/snippets/drawer/usage.rs",
        "src/ui/snippets/drawer/scrollable_content.rs",
        "src/ui/snippets/drawer/sides.rs",
        "src/ui/snippets/drawer/responsive_dialog.rs",
        "src/ui/snippets/drawer/rtl.rs",
        "src/ui/snippets/drawer/snap_points.rs",
    ] {
        let normalized = assert_normalized_markers_present(
            relative_path,
            &[".children([", "shadcn::DrawerPart::trigger("],
        );
        assert!(
            normalized.contains("shadcn::DrawerPart::content_with("),
            "{} should keep Drawer content on the default children() lane",
            manifest_path(relative_path).display()
        );
    }
}

#[test]
fn drawer_snap_points_snippet_prefers_children_root_path() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/drawer/snap_points.rs",
        &[
            "shadcn::Drawer::new_controllable(cx, None, false)",
            ".snap_points(vec![",
            ".children([",
            "shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(",
            "shadcn::DrawerPart::content_with(",
            "shadcn::DrawerClose::from_scope().build(",
        ],
        &[
            "let open = cx.local_model(|| false);",
            "shadcn::Drawer::new(open)",
            ".toggle_model(",
            ".compose()",
        ],
    );
}

#[test]
fn curated_drawer_snippets_prefer_drawer_close_scope_for_footer_close_actions() {
    for relative_path in [
        "src/ui/snippets/drawer/demo.rs",
        "src/ui/snippets/drawer/usage.rs",
        "src/ui/snippets/drawer/scrollable_content.rs",
        "src/ui/snippets/drawer/sides.rs",
        "src/ui/snippets/drawer/responsive_dialog.rs",
        "src/ui/snippets/drawer/rtl.rs",
        "src/ui/snippets/drawer/snap_points.rs",
    ] {
        assert_normalized_markers_present(
            relative_path,
            &["shadcn::DrawerClose::from_scope().build("],
        );
    }
}

#[test]
fn sheet_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/sheet/demo.rs",
            "src/ui/snippets/sheet/no_close_button.rs",
            "src/ui/snippets/sheet/parts.rs",
            "src/ui/snippets/sheet/rtl.rs",
            "src/ui/snippets/sheet/side.rs",
            "src/ui/snippets/sheet/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/sheet",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn sheet_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/sheet.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Side\", side)",
            "DocSection::build(cx, \"No Close Button\", no_close_button)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Parts\", parts)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Side\", side)",
            "DocSection::new(\"No Close Button\", no_close_button)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Parts\", parts)",
        ],
    );
}

#[test]
fn sheet_page_marks_usage_as_default_and_parts_as_follow_up() {
    let sheet_page = read("src/ui/pages/sheet.rs");
    assert!(
        sheet_page.contains(
            "`Usage` is the default copyable `children([...])` path, while `Parts` stays after `API Reference` as a focused advanced follow-up for explicit part adapters (`SheetTrigger` / `SheetPortal` / `SheetOverlay`)."
        ),
        "src/ui/pages/sheet.rs should distinguish the default children() lane from the explicit part-adapter follow-up lane"
    );
    assert!(
        sheet_page.contains("Default copyable `children([...])` path for common Sheet call sites."),
        "src/ui/pages/sheet.rs should label Usage as the default copyable children() path"
    );
    assert!(
        sheet_page.contains(
            "`Usage` now teaches the root `children([...])` path because it is closer to upstream nested children composition; `compose()` stays as the focused builder-style follow-up and `Parts` keeps explicit adapter ownership visible."
        ),
        "src/ui/pages/sheet.rs should keep compose() documented as a follow-up after the default children() lane"
    );
}

#[test]
fn sheet_usage_snippet_prefers_children_root_path() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/sheet/usage.rs",
        &[
            "shadcn::Sheet::new_controllable(cx, None, false)",
            ".children([",
            "shadcn::SheetPart::trigger(shadcn::SheetTrigger::build(",
            "shadcn::SheetPart::content(shadcn::SheetContent::build(",
        ],
        &[".compose()", ".content_with("],
    );
}

#[test]
fn sheet_curated_snippets_prefer_children_root_path() {
    for relative_path in [
        "src/ui/snippets/sheet/demo.rs",
        "src/ui/snippets/sheet/usage.rs",
        "src/ui/snippets/sheet/no_close_button.rs",
        "src/ui/snippets/sheet/rtl.rs",
        "src/ui/snippets/sheet/side.rs",
    ] {
        let normalized = assert_normalized_markers_present(
            relative_path,
            &[".children([", "shadcn::SheetPart::trigger("],
        );
        assert!(
            normalized.contains("shadcn::SheetPart::content("),
            "{} should keep Sheet content on the default children() lane",
            manifest_path(relative_path).display()
        );
    }
}

#[test]
fn curated_sheet_snippets_prefer_sheet_close_scope_for_custom_close_actions() {
    for relative_path in [
        "src/ui/snippets/sheet/demo.rs",
        "src/ui/snippets/sheet/no_close_button.rs",
        "src/ui/snippets/sheet/rtl.rs",
        "src/ui/snippets/sheet/side.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::SheetClose::from_scope().build("],
            &[".toggle_model("],
        );
    }
}

#[test]
fn spinner_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/spinner/badges.rs",
            "src/ui/snippets/spinner/buttons.rs",
            "src/ui/snippets/spinner/customization.rs",
            "src/ui/snippets/spinner/demo.rs",
            "src/ui/snippets/spinner/empty.rs",
            "src/ui/snippets/spinner/extras.rs",
            "src/ui/snippets/spinner/input_group.rs",
            "src/ui/snippets/spinner/rtl.rs",
            "src/ui/snippets/spinner/sizes.rs",
            "src/ui/snippets/spinner/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/spinner",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn spinner_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/spinner.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Customization\", customization)",
            "DocSection::build(cx, \"Size\", sizes)",
            "DocSection::build(cx, \"Button\", buttons)",
            "DocSection::build(cx, \"Badge\", badges)",
            "DocSection::build(cx, \"Input Group\", input_group)",
            "DocSection::build(cx, \"Empty\", empty)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Extras\", extras)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Customization\", customization)",
            "DocSection::new(\"Size\", sizes)",
            "DocSection::new(\"Button\", buttons)",
            "DocSection::new(\"Badge\", badges)",
            "DocSection::new(\"Input Group\", input_group)",
            "DocSection::new(\"Empty\", empty)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Extras\", extras)",
        ],
    );
}

#[test]
fn form_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/form/controls.rs",
            "src/ui/snippets/form/demo.rs",
            "src/ui/snippets/form/fieldset.rs",
            "src/ui/snippets/form/input.rs",
            "src/ui/snippets/form/rtl.rs",
            "src/ui/snippets/form/textarea.rs",
            "src/ui/snippets/form/upstream_demo.rs",
            "src/ui/snippets/form/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/form",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn form_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/form.rs",
        &[
            "DocSection::build(cx, \"Form Demo\", upstream_demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Input\", input)",
            "DocSection::build(cx, \"Textarea\", textarea)",
            "DocSection::build(cx, \"Checkbox + Switch\", controls)",
            "DocSection::build(cx, \"Fieldset\", fieldset)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "DocSection::new(\"Form Demo\", upstream_demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Input\", input)",
            "DocSection::new(\"Textarea\", textarea)",
            "DocSection::new(\"Checkbox + Switch\", controls)",
            "DocSection::new(\"Fieldset\", fieldset)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Notes\", notes)",
        ],
    );
}

#[test]
fn empty_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/empty/avatar.rs",
            "src/ui/snippets/empty/avatar_group.rs",
            "src/ui/snippets/empty/background.rs",
            "src/ui/snippets/empty/demo.rs",
            "src/ui/snippets/empty/input_group.rs",
            "src/ui/snippets/empty/outline.rs",
            "src/ui/snippets/empty/rtl.rs",
            "src/ui/snippets/empty/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/empty",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn empty_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/empty.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Outline\", outline)",
            "DocSection::build(cx, \"Background\", background)",
            "DocSection::build(cx, \"Avatar\", avatar)",
            "DocSection::build(cx, \"Avatar Group\", avatar_group)",
            "DocSection::build(cx, \"InputGroup\", input_group)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Outline\", outline)",
            "DocSection::new(\"Background\", background)",
            "DocSection::new(\"Avatar\", avatar)",
            "DocSection::new(\"Avatar Group\", avatar_group)",
            "DocSection::new(\"InputGroup\", input_group)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn breadcrumb_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/breadcrumb/basic.rs",
            "src/ui/snippets/breadcrumb/collapsed.rs",
            "src/ui/snippets/breadcrumb/custom_separator.rs",
            "src/ui/snippets/breadcrumb/demo.rs",
            "src/ui/snippets/breadcrumb/dropdown.rs",
            "src/ui/snippets/breadcrumb/link_component.rs",
            "src/ui/snippets/breadcrumb/rtl.rs",
            "src/ui/snippets/breadcrumb/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/breadcrumb",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn breadcrumb_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/breadcrumb.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Custom Separator\", custom_separator)",
            "DocSection::build(cx, \"Dropdown\", dropdown)",
            "DocSection::build(cx, \"Collapsed\", collapsed)",
            "DocSection::build(cx, \"Link Component\", link_component)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Responsive\", responsive)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Custom Separator\", custom_separator)",
            "DocSection::new(\"Dropdown\", dropdown)",
            "DocSection::new(\"Collapsed\", collapsed)",
            "DocSection::new(\"Link Component\", link_component)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Responsive\", responsive)",
        ],
    );
}

#[test]
fn collapsible_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/collapsible/basic.rs",
            "src/ui/snippets/collapsible/controlled_state.rs",
            "src/ui/snippets/collapsible/demo.rs",
            "src/ui/snippets/collapsible/file_tree.rs",
            "src/ui/snippets/collapsible/rtl.rs",
            "src/ui/snippets/collapsible/settings_panel.rs",
            "src/ui/snippets/collapsible/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/collapsible",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn collapsible_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/collapsible.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Controlled State\", controlled_state)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Settings Panel\", settings)",
            "DocSection::build(cx, \"File Tree\", file_tree)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Controlled State\", controlled_state)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Settings Panel\", settings)",
            "DocSection::new(\"File Tree\", file_tree)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn skeleton_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/skeleton/avatar.rs",
            "src/ui/snippets/skeleton/card.rs",
            "src/ui/snippets/skeleton/demo.rs",
            "src/ui/snippets/skeleton/form.rs",
            "src/ui/snippets/skeleton/rtl.rs",
            "src/ui/snippets/skeleton/table.rs",
            "src/ui/snippets/skeleton/text.rs",
            "src/ui/snippets/skeleton/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/skeleton",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn skeleton_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/skeleton.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Avatar\", avatar)",
            "DocSection::build(cx, \"Card\", card)",
            "DocSection::build(cx, \"Text\", text_section)",
            "DocSection::build(cx, \"Form\", form)",
            "DocSection::build(cx, \"Table\", table)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Avatar\", avatar)",
            "DocSection::new(\"Card\", card)",
            "DocSection::new(\"Text\", text_section)",
            "DocSection::new(\"Form\", form)",
            "DocSection::new(\"Table\", table)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn pagination_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/pagination/compact_builder.rs",
            "src/ui/snippets/pagination/custom_text.rs",
            "src/ui/snippets/pagination/demo.rs",
            "src/ui/snippets/pagination/extras.rs",
            "src/ui/snippets/pagination/icons_only.rs",
            "src/ui/snippets/pagination/routing.rs",
            "src/ui/snippets/pagination/rtl.rs",
            "src/ui/snippets/pagination/simple.rs",
            "src/ui/snippets/pagination/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/pagination",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn pagination_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/pagination.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Simple\", simple)",
            "DocSection::build(cx, \"Icons Only\", icons_only)",
            "DocSection::build(cx, \"Routing\", routing)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Custom Text\", custom_text)",
            "DocSection::build(cx, \"Compact Builder\", compact_builder)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Simple\", simple)",
            "DocSection::new(\"Icons Only\", icons_only)",
            "DocSection::new(\"Routing\", routing)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Custom Text\", custom_text)",
            "DocSection::new(\"Compact Builder\", compact_builder)",
        ],
    );
}

#[test]
fn alert_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/alert/action.rs",
            "src/ui/snippets/alert/basic.rs",
            "src/ui/snippets/alert/custom_colors.rs",
            "src/ui/snippets/alert/demo.rs",
            "src/ui/snippets/alert/destructive.rs",
            "src/ui/snippets/alert/interactive_links.rs",
            "src/ui/snippets/alert/rich_title.rs",
            "src/ui/snippets/alert/rtl.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/alert",
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement",
            "pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement",
        ],
    );
}

#[test]
fn alert_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/alert.rs",
        &[
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"With Icons\", demo)",
            "DocSection::build(cx, \"Destructive\", destructive)",
            "DocSection::build(cx, \"With Actions\", action)",
            "DocSection::build(cx, \"Rich Title\", rich_title)",
            "DocSection::build(cx, \"Interactive Links\", interactive_links)",
            "DocSection::build(cx, \"Custom Colors\", custom_colors)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"With Icons\", demo)",
            "DocSection::new(\"Destructive\", destructive)",
            "DocSection::new(\"With Actions\", action)",
            "DocSection::new(\"Rich Title\", rich_title)",
            "DocSection::new(\"Interactive Links\", interactive_links)",
            "DocSection::new(\"Custom Colors\", custom_colors)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn sidebar_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/sidebar/controlled.rs",
            "src/ui/snippets/sidebar/demo.rs",
            "src/ui/snippets/sidebar/mobile.rs",
            "src/ui/snippets/sidebar/rtl.rs",
            "src/ui/snippets/sidebar/usage.rs",
            "src/ui/snippets/sidebar/use_sidebar.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/sidebar",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn sidebar_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/sidebar.rs",
        &[
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"SidebarProvider\", controlled)",
            "DocSection::build(cx, \"Sidebar\", demo)",
            "DocSection::build(cx, \"useSidebar\", use_sidebar)",
            "DocSection::build(cx, \"Extras: Mobile\", mobile)",
            "DocSection::build(cx, \"Extras: RTL\", rtl)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"SidebarProvider\", controlled)",
            "DocSection::new(\"Sidebar\", demo)",
            "DocSection::new(\"useSidebar\", use_sidebar)",
            "DocSection::new(\"Extras: Mobile\", mobile)",
        ],
    );
}

#[test]
fn label_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/label/demo.rs",
            "src/ui/snippets/label/label_in_field.rs",
            "src/ui/snippets/label/rtl.rs",
            "src/ui/snippets/label/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/label",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn label_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/label.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Label in Field\", label_in_field)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"API Reference\", api_reference)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Label in Field\", label_in_field)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn kbd_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/kbd/button.rs",
            "src/ui/snippets/kbd/demo.rs",
            "src/ui/snippets/kbd/group.rs",
            "src/ui/snippets/kbd/input_group.rs",
            "src/ui/snippets/kbd/rtl.rs",
            "src/ui/snippets/kbd/tooltip.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/kbd",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn kbd_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/kbd.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Group\", group)",
            "DocSection::build(cx, \"Button\", button)",
            "DocSection::build(cx, \"Tooltip\", tooltip)",
            "DocSection::build(cx, \"Input Group\", input_group)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"API Reference\", api_reference)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Group\", group)",
            "DocSection::new(\"Button\", button)",
            "DocSection::new(\"Tooltip\", tooltip)",
            "DocSection::new(\"Input Group\", input_group)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn icons_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/icons/grid.rs",
            "src/ui/snippets/icons/spinner.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/icons",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn icons_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/icons.rs",
        &[
            "DocSection::build(cx, \"Icons\", grid)",
            "DocSection::build(cx, \"Spinner\", spinner_row)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "DocSection::new(\"Icons\", grid)",
            "DocSection::new(\"Spinner\", spinner_row)",
        ],
    );
}

#[test]
fn sonner_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/sonner/demo.rs",
            "src/ui/snippets/sonner/extras.rs",
            "src/ui/snippets/sonner/notes.rs",
            "src/ui/snippets/sonner/position.rs",
            "src/ui/snippets/sonner/setup.rs",
            "src/ui/snippets/sonner/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent("src/ui/snippets/sonner", &["pub fn render<H: UiHost>("]);
}

#[test]
fn sonner_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/sonner.rs",
        &[
            "DocSection::build(cx, \"Setup\", setup)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Position\", position)",
            "DocSection::build(cx, \"Extras\", extras)",
            "DocSection::build(cx, \"Notes\", notes)",
            "let toaster = snippets::local_toaster(cx).into_element(cx);",
        ],
        &[
            "DocSection::new(\"Setup\", setup)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Position\", position)",
            "DocSection::new(\"Extras\", extras)",
            "preview_sonner(cx, last_action, sonner_position)",
        ],
    );
}

#[test]
fn sonner_local_toaster_prefers_ui_child_over_anyelement() {
    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/snippets/sonner/mod.rs",
        &["pub(crate) fn local_toaster(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &["pub(crate) fn local_toaster(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn date_picker_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/date_picker/basic.rs",
            "src/ui/snippets/date_picker/demo.rs",
            "src/ui/snippets/date_picker/dob.rs",
            "src/ui/snippets/date_picker/dropdowns.rs",
            "src/ui/snippets/date_picker/input.rs",
            "src/ui/snippets/date_picker/label.rs",
            "src/ui/snippets/date_picker/natural_language.rs",
            "src/ui/snippets/date_picker/notes.rs",
            "src/ui/snippets/date_picker/presets.rs",
            "src/ui/snippets/date_picker/range.rs",
            "src/ui/snippets/date_picker/rtl.rs",
            "src/ui/snippets/date_picker/time_picker.rs",
            "src/ui/snippets/date_picker/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/date_picker",
        &["pub fn render<H: UiHost>("],
    );
}

#[test]
fn date_picker_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/date_picker.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Range Picker\", range)",
            "DocSection::build(cx, \"Date of Birth\", dob)",
            "DocSection::build(cx, \"Input\", input)",
            "DocSection::build(cx, \"Time Picker\", time_picker)",
            "DocSection::build(cx, \"Natural Language Picker\", natural_language)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Label Association\", label)",
            "DocSection::build(cx, \"Extras: With Presets\", presets)",
            "DocSection::build(cx, \"Extras: With Dropdowns\", dropdowns)",
            "DocSection::build(cx, \"Notes\", notes_stack)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Range Picker\", range)",
            "DocSection::new(\"Date of Birth\", dob)",
            "DocSection::new(\"Input\", input)",
            "DocSection::new(\"Time Picker\", time_picker)",
            "DocSection::new(\"Natural Language Picker\", natural_language)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Label Association\", label)",
            "DocSection::new(\"Extras: With Presets\", presets)",
            "preview_date_picker(cx, open, month, selected)",
        ],
    );
}

#[test]
fn avatar_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/avatar/badge_icon.rs",
            "src/ui/snippets/avatar/basic.rs",
            "src/ui/snippets/avatar/demo.rs",
            "src/ui/snippets/avatar/dropdown.rs",
            "src/ui/snippets/avatar/fallback_only.rs",
            "src/ui/snippets/avatar/group.rs",
            "src/ui/snippets/avatar/group_count.rs",
            "src/ui/snippets/avatar/group_count_icon.rs",
            "src/ui/snippets/avatar/rtl.rs",
            "src/ui/snippets/avatar/sizes.rs",
            "src/ui/snippets/avatar/usage.rs",
            "src/ui/snippets/avatar/with_badge.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent("src/ui/snippets/avatar", &["pub fn render<H: UiHost>("]);
}

#[test]
fn avatar_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/avatar.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Badge\", with_badge)",
            "DocSection::build(cx, \"Badge with Icon\", badge_icon)",
            "DocSection::build(cx, \"Avatar Group\", avatar_group)",
            "DocSection::build(cx, \"Avatar Group Count\", group_count)",
            "DocSection::build(cx, \"Avatar Group with Icon\", group_count_icon)",
            "DocSection::build(cx, \"Sizes\", sizes)",
            "DocSection::build(cx, \"Dropdown\", dropdown)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Fallback only (Fret)\", fallback)",
            "DocSection::build(cx, \"API Reference\", api_reference)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Badge\", with_badge)",
            "DocSection::new(\"Badge with Icon\", badge_icon)",
            "DocSection::new(\"Avatar Group\", avatar_group)",
            "DocSection::new(\"Avatar Group Count\", group_count)",
            "DocSection::new(\"Avatar Group with Icon\", group_count_icon)",
            "DocSection::new(\"Sizes\", sizes)",
            "DocSection::new(\"Dropdown\", dropdown)",
            "DocSection::new(\"RTL\", rtl)",
            "preview_avatar(cx, avatar_image)",
        ],
    );
}

#[test]
fn command_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/command/action_first_view.rs",
            "src/ui/snippets/command/basic.rs",
            "src/ui/snippets/command/docs_demo.rs",
            "src/ui/snippets/command/groups.rs",
            "src/ui/snippets/command/loading.rs",
            "src/ui/snippets/command/rtl.rs",
            "src/ui/snippets/command/scrollable.rs",
            "src/ui/snippets/command/shortcuts.rs",
            "src/ui/snippets/command/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent("src/ui/snippets/command", &["pub fn render<H: UiHost>("]);
    assert_sources_absent(
        "src/ui/snippets/command",
        &["CommandInput::new(", "CommandList::new("],
    );
}

#[test]
fn command_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/command.rs",
        &[
            "DocSection::build(cx, \"Demo\", docs_demo_palette)",
            "DocSection::build(cx, \"Usage\", usage_palette)",
            "DocSection::build(cx, \"Basic\", basic_dialog)",
            "DocSection::build(cx, \"Shortcuts\", shortcuts_section)",
            "DocSection::build(cx, \"Groups\", groups_palette)",
            "DocSection::build(cx, \"Scrollable\", scrollable_palette)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Loading\", loading_palette)",
            "DocSection::build(cx, \"Action-first (View runtime)\", action_first_view_runtime)",
            "DocSection::build(cx, \"Notes\", notes_stack)",
        ],
        &[
            "DocSection::new(\"Demo\", docs_demo_palette)",
            "DocSection::new(\"Usage\", usage_palette)",
            "DocSection::new(\"Basic\", basic_dialog)",
            "DocSection::new(\"Shortcuts\", shortcuts_section)",
            "DocSection::new(\"Groups\", groups_palette)",
            "DocSection::new(\"Scrollable\", scrollable_palette)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Loading\", loading_palette)",
            "preview_command_palette(cx, last_action)",
        ],
    );
}

#[test]
fn popover_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/popover/align.rs",
            "src/ui/snippets/popover/basic.rs",
            "src/ui/snippets/popover/demo.rs",
            "src/ui/snippets/popover/rtl.rs",
            "src/ui/snippets/popover/usage.rs",
            "src/ui/snippets/popover/with_form.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/popover",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn popover_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/popover.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Align\", align)",
            "DocSection::build(cx, \"With Form\", with_form)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Align\", align)",
            "DocSection::new(\"With Form\", with_form)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn hover_card_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/hover_card/basic.rs",
            "src/ui/snippets/hover_card/demo.rs",
            "src/ui/snippets/hover_card/positioning.rs",
            "src/ui/snippets/hover_card/rtl.rs",
            "src/ui/snippets/hover_card/sides.rs",
            "src/ui/snippets/hover_card/trigger_delays.rs",
            "src/ui/snippets/hover_card/usage.rs",
        ],
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "pub fn render(cx: &mut UiCx<'_>,",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/hover_card",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn hover_card_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/hover_card.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Trigger Delays\", trigger_delays)",
            "DocSection::build(cx, \"Positioning\", positioning)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Sides\", sides)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Trigger Delays\", trigger_delays)",
            "DocSection::new(\"Positioning\", positioning)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Sides\", sides)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn tooltip_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/tooltip/demo.rs",
            "src/ui/snippets/tooltip/disabled_button.rs",
            "src/ui/snippets/tooltip/keyboard_focus.rs",
            "src/ui/snippets/tooltip/keyboard_shortcut.rs",
            "src/ui/snippets/tooltip/long_content.rs",
            "src/ui/snippets/tooltip/rtl.rs",
            "src/ui/snippets/tooltip/sides.rs",
            "src/ui/snippets/tooltip/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/tooltip",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn tooltip_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/tooltip.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo_tooltip)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Side\", side_row)",
            "DocSection::build(cx, \"With Keyboard Shortcut\", keyboard_tooltip)",
            "DocSection::build(cx, \"Disabled Button\", disabled_tooltip)",
            "DocSection::build(cx, \"RTL\", rtl_row)",
            "DocSection::build(cx, \"Long Content\", long_content_tooltip)",
            "DocSection::build(cx, \"Keyboard Focus\", focus_row)",
        ],
        &[
            "DocSection::new(\"Demo\", demo_tooltip)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Side\", side_row)",
            "DocSection::new(\"With Keyboard Shortcut\", keyboard_tooltip)",
            "DocSection::new(\"Disabled Button\", disabled_tooltip)",
            "DocSection::new(\"RTL\", rtl_row)",
            "DocSection::new(\"Long Content\", long_content_tooltip)",
            "DocSection::new(\"Keyboard Focus\", focus_row)",
        ],
    );
}

#[test]
fn progress_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/progress.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Label\", label)",
            "DocSection::build(cx, \"Controlled\", controlled)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Label\", label)",
            "DocSection::new(\"Controlled\", controlled)",
            "DocSection::new(\"RTL\", rtl)",
        ],
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
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            "pub fn render(cx: &mut UiCx<'_>, value: Model<Option<Arc<str>>>, open: Model<bool>, query: Model<String>,) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent("src/ui/snippets/combobox", &["-> AnyElement"]);
}

#[test]
fn combobox_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/combobox.rs",
        &[
            "DocSection::build(cx, \"Conformance Demo\", conformance_demo)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Label Association\", label)",
            "DocSection::build(cx, \"Auto Highlight\", auto_highlight)",
            "DocSection::build(cx, \"Clear Button\", clear)",
            "DocSection::build(cx, \"Groups\", groups)",
            "DocSection::build(cx, \"Groups + Separator\", groups_with_separator)",
            "DocSection::build(cx, \"Trigger Button\", trigger_button)",
            "DocSection::build(cx, \"Multiple Selection\", multiple)",
            "DocSection::build(cx, \"Extras: Custom Items\", custom_items)",
            "DocSection::build(cx, \"Extras: Long List\", long_list)",
            "DocSection::build(cx, \"Extras: Invalid\", invalid)",
            "DocSection::build(cx, \"Extras: Disabled\", disabled)",
            "DocSection::build(cx, \"Extras: Input Group\", input_group)",
            "DocSection::build(cx, \"Extras: RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Conformance Demo\", conformance_demo)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Label Association\", label)",
            "DocSection::new(\"Auto Highlight\", auto_highlight)",
            "DocSection::new(\"Clear Button\", clear)",
            "DocSection::new(\"Groups\", groups)",
            "DocSection::new(\"Groups + Separator\", groups_with_separator)",
            "DocSection::new(\"Trigger Button\", trigger_button)",
            "DocSection::new(\"Multiple Selection\", multiple)",
            "DocSection::new(\"Extras: Custom Items\", custom_items)",
            "DocSection::new(\"Extras: Long List\", long_list)",
            "DocSection::new(\"Extras: Invalid\", invalid)",
            "DocSection::new(\"Extras: Disabled\", disabled)",
            "DocSection::new(\"Extras: Input Group\", input_group)",
            "DocSection::new(\"Extras: RTL\", rtl)",
        ],
    );
}

#[test]
fn toast_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &["src/ui/snippets/toast/deprecated.rs"],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/toast",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn toast_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/toast.rs",
        &["DocSection::build(cx, \"Deprecated\", deprecated)"],
        &["DocSection::new(\"Deprecated\", deprecated)"],
    );
}

#[test]
fn slider_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/slider/demo.rs",
            "src/ui/snippets/slider/extras.rs",
            "src/ui/snippets/slider/label.rs",
            "src/ui/snippets/slider/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/slider",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn native_select_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/native_select/demo.rs",
            "src/ui/snippets/native_select/disabled.rs",
            "src/ui/snippets/native_select/invalid.rs",
            "src/ui/snippets/native_select/label.rs",
            "src/ui/snippets/native_select/rtl.rs",
            "src/ui/snippets/native_select/usage.rs",
            "src/ui/snippets/native_select/with_groups.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/native_select",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn navigation_menu_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/navigation_menu/demo.rs",
            "src/ui/snippets/navigation_menu/docs_demo.rs",
            "src/ui/snippets/navigation_menu/link_component.rs",
            "src/ui/snippets/navigation_menu/rtl.rs",
            "src/ui/snippets/navigation_menu/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/navigation_menu",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn selected_navigation_menu_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    for relative_path in [
        "src/ui/snippets/navigation_menu/usage.rs",
        "src/ui/snippets/navigation_menu/link_component.rs",
        "src/ui/snippets/navigation_menu/demo.rs",
        "src/ui/snippets/navigation_menu/docs_demo.rs",
        "src/ui/snippets/navigation_menu/rtl.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::navigation_menu("],
            &["shadcn::NavigationMenu::new("],
        );
    }

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
fn navigation_menu_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/navigation_menu.rs",
        &[
            "DocSection::build(cx, \"Demo\", docs_demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Link Component\", link_component)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Container Query Toggle\", demo_with_toggle)",
        ],
        &[
            "DocSection::new(\"Demo\", docs_demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Link Component\", link_component)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Container Query Toggle\", demo_with_toggle)",
        ],
    );
}

#[test]
fn scroll_area_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/scroll_area/demo.rs",
            "src/ui/snippets/scroll_area/usage.rs",
            "src/ui/snippets/scroll_area/horizontal.rs",
            "src/ui/snippets/scroll_area/nested_scroll_routing.rs",
            "src/ui/snippets/scroll_area/rtl.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );
}

#[test]
fn scroll_area_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/scroll_area.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Horizontal\", horizontal)",
            "DocSection::build(cx, \"Nested scroll routing\", nested_scroll_routing)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build_diagnostics(cx, \"Scrollbar drag baseline\", drag_baseline)",
            "DocSection::build_diagnostics(cx, \"Expand at bottom\", expand_at_bottom)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Horizontal\", horizontal)",
            "DocSection::new(\"Nested scroll routing\", nested_scroll_routing)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::build(cx, \"Scrollbar drag baseline\", drag_baseline)",
            "DocSection::build(cx, \"Expand at bottom\", expand_at_bottom)",
        ],
    );
}

#[test]
fn scroll_area_app_facing_snippet_lane_has_no_raw_boundaries() {
    for path in rust_sources("src/ui/snippets/scroll_area") {
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();
        let is_raw_render_root = normalized
            .contains("pubfnrender<H:UiHost+'static>(cx:&mutElementContext<'_,H>)->AnyElement");
        assert!(
            !is_raw_render_root,
            "{} should stay on the copyable default app-facing lane",
            path.display()
        );
    }
}

#[test]
fn scroll_area_diagnostics_lane_keeps_intentional_raw_boundaries() {
    let expected_raw_roots = BTreeSet::from([
        manifest_path("src/ui/diagnostics/scroll_area/drag_baseline.rs")
            .display()
            .to_string(),
        manifest_path("src/ui/diagnostics/scroll_area/expand_at_bottom.rs")
            .display()
            .to_string(),
    ]);
    let mut actual_raw_roots = BTreeSet::new();

    for path in rust_sources("src/ui/diagnostics/scroll_area") {
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();
        let is_raw_render_root = normalized
            .contains("pubfnrender<H:UiHost+'static>(cx:&mutElementContext<'_,H>)->AnyElement");
        if !is_raw_render_root {
            continue;
        }

        actual_raw_roots.insert(path.display().to_string());
        assert!(
            source.contains("Intentional diagnostics raw boundary:"),
            "{} should explain why the diagnostics harness stays raw",
            path.display()
        );
    }

    assert_eq!(
        actual_raw_roots, expected_raw_roots,
        "src/ui/diagnostics/scroll_area should keep exactly the two audited diagnostics harness raw roots",
    );
}

#[test]
fn chart_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/chart/contracts.rs",
            "src/ui/snippets/chart/demo.rs",
            "src/ui/snippets/chart/grid_axis.rs",
            "src/ui/snippets/chart/legend.rs",
            "src/ui/snippets/chart/rtl.rs",
            "src/ui/snippets/chart/tooltip.rs",
            "src/ui/snippets/chart/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/chart",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn chart_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/chart.rs",
        &[
            "DocSection::build(cx, \"Component\", demo_cards)",
            "DocSection::build(cx, \"First Chart\", first_chart)",
            "DocSection::build(cx, \"Chart Config\", config)",
            "DocSection::build(cx, \"Theming\", theming)",
            "DocSection::build(cx, \"Grid / Axis (Fret)\", grid_axis)",
            "DocSection::build(cx, \"Contracts\", contracts_overview)",
            "DocSection::build(cx, \"Tooltip\", tooltip_content)",
            "DocSection::build(cx, \"Legend\", legend_content)",
            "DocSection::build(cx, \"Accessibility\", accessibility)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Notes\", notes_stack)",
        ],
        &[
            "DocSection::new(\"Component\", demo_cards)",
            "DocSection::new(\"First Chart\", first_chart)",
            "DocSection::new(\"Chart Config\", config)",
            "DocSection::new(\"Theming\", theming)",
            "DocSection::new(\"Grid / Axis (Fret)\", grid_axis)",
            "DocSection::new(\"Contracts\", contracts_overview)",
            "DocSection::new(\"Tooltip\", tooltip_content)",
            "DocSection::new(\"Legend\", legend_content)",
            "DocSection::new(\"Accessibility\", accessibility)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Notes\", notes_stack)",
        ],
    );
}

#[test]
fn chart_tooltip_docs_keep_custom_children_seam_explicit() {
    let tooltip = read("src/ui/snippets/chart/tooltip.rs");
    assert!(
        tooltip.contains(".formatter(|context|"),
        "src/ui/snippets/chart/tooltip.rs should keep the structured formatter example visible"
    );
    assert!(
        tooltip.contains(".into_element_parts(cx, |cx, context|"),
        "src/ui/snippets/chart/tooltip.rs should keep the custom children adapter seam visible"
    );
    assert!(
        tooltip.contains(".into_element_parts_with_label("),
        "src/ui/snippets/chart/tooltip.rs should keep the combined custom label + row adapter seam visible"
    );

    let contracts = read("src/ui/snippets/chart/contracts.rs");
    assert!(
        contracts.contains(
            "custom header/row composition via into_element_label_parts(...), into_element_parts(...), and into_element_parts_with_label(...)"
        ),
        "src/ui/snippets/chart/contracts.rs should document the header-only, row-only, and combined tooltip composition seams"
    );

    let page = read("src/ui/pages/chart.rs");
    assert!(
        page.contains(
            "For fully custom tooltip header/rows, `ChartTooltipContent::into_element_label_parts(cx, ...)`, `ChartTooltipContent::into_element_parts(cx, ...)`, and `ChartTooltipContent::into_element_parts_with_label(cx, ...)` cover header-only, row-only, or fully combined children composition."
        ),
        "src/ui/pages/chart.rs should explain the advanced custom header/row tooltip seams"
    );
    assert!(
        page.contains(
            "Tooltip examples now read in a shadcn-like order: props first, config-driven colors and key remapping second, then formatter plus header-only, row-only, and combined custom children seams."
        ),
        "src/ui/pages/chart.rs should keep the Tooltip section description aligned with the custom header/row seams"
    );
}

#[test]
fn chart_page_keeps_shadcn_docs_path_before_fret_follow_ups() {
    let page = read("src/ui/pages/chart.rs");
    let normalized = page.split_whitespace().collect::<String>();

    assert!(
        page.contains(
            "Composition-first chart recipe surface: build the chart body inside `chart_container(config, |cx| ...)`, then opt into `ChartTooltip` and `ChartLegend` only where needed."
        ),
        "src/ui/pages/chart.rs should keep the Component section on the composition-first child-authoring lane"
    );
    assert!(
        page.contains(
            "Preview mirrors the shadcn Chart docs path first: `Component`, `First Chart`, `Chart Config`, `Theming`, `Tooltip`, `Legend`, `Accessibility`, and `RTL`. After that, Gallery keeps Fret-specific follow-ups explicit: `Grid / Axis (Fret)`, `Contracts`, and `Notes`."
        ),
        "src/ui/pages/chart.rs should explain the shadcn docs path before the Fret-specific follow-ups"
    );
    assert!(
        page.contains(
            "Focused Fret follow-up: grid and axis remain spec-owned on `delinea::ChartSpec` today, so the copyable setup lives beside the retained chart engine instead of the `ChartContainer` child lane."
        ),
        "src/ui/pages/chart.rs should keep the spec-owned grid/axis follow-up explicit on the page surface"
    );
    assert!(
        page.contains(
            "Grid and axis stay in the retained chart spec instead of separate child widgets."
        ),
        "src/ui/pages/chart.rs should explain why the shadcn `Add Grid` and `Add Axis` steps stay inside the retained chart spec on Fret"
    );
    assert!(
        normalized.contains(
            "vec![component,first_chart,config,theming,tooltip_content,legend_content,accessibility,rtl,grid_axis,contracts_overview,notes_stack,]"
        ),
        "src/ui/pages/chart.rs should place the grid/axis follow-up ahead of `Contracts` and `Notes`"
    );
    assert!(
        !normalized.contains(
            "vec![component,first_chart,config,theming,contracts_overview,tooltip_content,legend_content,accessibility,rtl,notes_stack,]"
        ),
        "src/ui/pages/chart.rs should not reinsert `Contracts` into the middle of the shadcn docs path"
    );
    assert!(
        !normalized.contains(
            "vec![component,first_chart,config,theming,tooltip_content,legend_content,accessibility,grid_axis,rtl,contracts_overview,notes_stack,]"
        ),
        "src/ui/pages/chart.rs should keep `Grid / Axis (Fret)` after `RTL`, not inside the shadcn docs path"
    );
    assert!(
        !page.contains("DocSection::build(cx, \"Demo\", demo_cards)"),
        "src/ui/pages/chart.rs should keep the top section aligned to shadcn's `Component` naming instead of `Demo`"
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
            "use fret::{UiChild, UiCx};",
            "pub fn render",
            "-> impl UiChild + use<",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent("src/ui/snippets/motion_presets", &["-> AnyElement"]);
}

#[test]
fn motion_presets_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/motion_presets.rs",
        &[
            "DocSection::build(cx, \"Preset selector\", preset_selector)",
            "DocSection::build(cx, \"Token snapshot\", token_snapshot)",
            "DocSection::build(cx, \"Overlay demo\", overlay_demo)",
            "DocSection::build(cx, \"Fluid tabs demo\", fluid_tabs_demo)",
            "DocSection::build(cx, \"Stagger / sequence demo\", stagger_demo)",
            "DocSection::build(cx, \"Stack shift list demo\", stack_shift_list_demo)",
        ],
        &[
            "DocSection::new(\"Preset selector\", preset_selector)",
            "DocSection::new(\"Token snapshot\", token_snapshot)",
            "DocSection::new(\"Overlay demo\", overlay_demo)",
            "DocSection::new(\"Fluid tabs demo\", fluid_tabs_demo)",
            "DocSection::new(\"Stagger / sequence demo\", stagger_demo)",
            "DocSection::new(\"Stack shift list demo\", stack_shift_list_demo)",
        ],
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
            &[
                "use fret::{UiChild, UiCx};",
                "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            ],
            "app-facing snippet surface",
        );
    }

    assert_sources_absent(
        "src/ui/snippets/carousel",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn carousel_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/carousel.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Parts\", parts)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Sizes (1/3)\", sizes_thirds)",
            "DocSection::build(cx, \"Sizes\", sizes)",
            "DocSection::build(cx, \"Spacing\", spacing)",
            "DocSection::build(cx, \"Spacing (Responsive)\", spacing_responsive)",
            "DocSection::build(cx, \"Orientation (Vertical)\", orientation_vertical)",
            "DocSection::build(cx, \"Options\", options)",
            "DocSection::build(cx, \"API\", api)",
            "DocSection::build(cx, \"Events\", events)",
            "DocSection::build(cx, \"Plugin (Autoplay)\", plugin)",
            "DocSection::build(cx, \"Plugin (Autoplay, Controlled)\", plugin_controlled)",
            "DocSection::build(cx, \"Plugin (Autoplay, stopOnInteraction via focus)\", plugin_stop_on_focus)",
            "DocSection::build(cx, \"Plugin (Autoplay, stopOnLastSnap)\", plugin_stop_on_last_snap)",
            "DocSection::build(cx, \"Plugin (Autoplay, per-snap delays)\", plugin_delays)",
            "DocSection::build(cx, \"Plugin (Wheel gestures)\", plugin_wheel)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Loop\", loop_carousel)",
            "DocSection::build_diagnostics(cx, \"Loop downgrade (cannotLoop)\",",
            "DocSection::build_diagnostics(cx, \"Focus\", focus)",
            "DocSection::build_diagnostics(cx, \"Duration (Embla)\", duration)",
            "DocSection::build_diagnostics(cx, \"Expandable\", expandable)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Parts\", parts)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Sizes (1/3)\", sizes_thirds)",
            "DocSection::new(\"Sizes\", sizes)",
            "DocSection::new(\"Spacing\", spacing)",
            "DocSection::new(\"Spacing (Responsive)\", spacing_responsive)",
            "DocSection::new(\"Orientation (Vertical)\", orientation_vertical)",
            "DocSection::new(\"Options\", options)",
            "DocSection::new(\"API\", api)",
            "DocSection::new(\"Events\", events)",
            "DocSection::new(\"Plugin (Autoplay)\", plugin)",
            "DocSection::new(\"Plugin (Autoplay, Controlled)\", plugin_controlled)",
            "DocSection::new(\"Plugin (Autoplay, stopOnInteraction via focus)\", plugin_stop_on_focus)",
            "DocSection::new(\"Plugin (Autoplay, stopOnLastSnap)\", plugin_stop_on_last_snap)",
            "DocSection::new(\"Plugin (Autoplay, per-snap delays)\", plugin_delays)",
            "DocSection::new(\"Plugin (Wheel gestures)\", plugin_wheel)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Loop\", loop_carousel)",
            "DocSection::new(\"Loop downgrade (cannotLoop)\", loop_downgrade_cannot_loop)",
            "DocSection::new(\"Focus\", focus)",
            "DocSection::new(\"Duration (Embla)\", duration)",
            "DocSection::new(\"Expandable\", expandable)",
        ],
    );
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
            &[
                "use fret::{UiChild, UiCx};",
                "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            ],
            "app-facing snippet surface",
        );
    }

    assert_sources_absent(
        "src/ui/snippets/item",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn item_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/item.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Item vs Field\", item_vs_field)",
            "DocSection::build(cx, \"Variant\", variants)",
            "DocSection::build(cx, \"Size\", size)",
            "DocSection::build(cx, \"Icon\", icon)",
            "DocSection::build(cx, \"Avatar\", avatar)",
            "DocSection::build(cx, \"Image\", image)",
            "DocSection::build(cx, \"Group\", group)",
            "DocSection::build(cx, \"Header\", header)",
            "DocSection::build(cx, \"Link\", link)",
            "DocSection::build(cx, \"Dropdown\", dropdown)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Gallery\", gallery)",
            "DocSection::build(cx, \"Link (render)\", link_render)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Item vs Field\", item_vs_field)",
            "DocSection::new(\"Variant\", variants)",
            "DocSection::new(\"Size\", size)",
            "DocSection::new(\"Icon\", icon)",
            "DocSection::new(\"Avatar\", avatar)",
            "DocSection::new(\"Image\", image)",
            "DocSection::new(\"Group\", group)",
            "DocSection::new(\"Header\", header)",
            "DocSection::new(\"Link\", link)",
            "DocSection::new(\"Dropdown\", dropdown)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Gallery\", gallery)",
            "DocSection::new(\"Link (render)\", link_render)",
        ],
    );
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
            "src/ui/snippets/tabs/usage.rs",
            "src/ui/snippets/tabs/vertical.rs",
            "src/ui/snippets/tabs/vertical_line.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/tabs",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
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
fn selected_tabs_snippets_prefer_builder_preserving_helpers() {
    for relative_path in [
        "src/ui/snippets/tabs/demo.rs",
        "src/ui/snippets/tabs/disabled.rs",
        "src/ui/snippets/tabs/extras.rs",
        "src/ui/snippets/tabs/icons.rs",
        "src/ui/snippets/tabs/line.rs",
        "src/ui/snippets/tabs/list.rs",
        "src/ui/snippets/tabs/rtl.rs",
        "src/ui/snippets/tabs/usage.rs",
        "src/ui/snippets/tabs/vertical.rs",
        "src/ui/snippets/tabs/vertical_line.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::tabs_uncontrolled("],
            &["shadcn::Tabs::uncontrolled(", "shadcn::TabsRoot::new("],
        );
    }
}

#[test]
fn tabs_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/tabs.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Line\", line)",
            "DocSection::build(cx, \"Vertical\", vertical)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"Icons\", icons)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"List\", list)",
            "DocSection::build(cx, \"Vertical (Line)\", vertical_line)",
            "DocSection::build(cx, \"Extras\", extras)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Line\", line)",
            "DocSection::new(\"Vertical\", vertical)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"Icons\", icons)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"List\", list)",
            "DocSection::new(\"Vertical (Line)\", vertical_line)",
            "DocSection::new(\"Extras\", extras)",
        ],
    );
}

#[test]
fn card_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/card/card_content.rs",
            "src/ui/snippets/card/compositions.rs",
            "src/ui/snippets/card/demo.rs",
            "src/ui/snippets/card/image.rs",
            "src/ui/snippets/card/meeting_notes.rs",
            "src/ui/snippets/card/rtl.rs",
            "src/ui/snippets/card/size.rs",
            "src/ui/snippets/card/title_children.rs",
            "src/ui/snippets/card/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/card",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> AnyElement",
            "pub fn render(cx: &mut UiCx<'_>,",
        ],
    );
}

#[test]
fn card_rich_title_snippet_prefers_copyable_card_title_children_helper() {
    let normalized = assert_normalized_markers_present(
        "src/ui/snippets/card/title_children.rs",
        &[
            "shadcn::card_title_children(|cx|",
            "cx.styled_text(rich_title_text())",
            "icon::icon(cx, IconId::new_static(\"lucide.sparkles\"))",
        ],
    );

    assert!(
        !normalized.contains("CardTitle::build("),
        "src/ui/snippets/card/title_children.rs reintroduced the lower-level `CardTitle::build(...)` teaching surface",
    );
    assert!(
        !normalized.contains("CardTitle::new_children("),
        "src/ui/snippets/card/title_children.rs should prefer the app-facing `shadcn::card_title_children(...)` helper",
    );
}

#[test]
fn card_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/card.rs",
        &[
            "DocSection::build(cx, \"Demo\", login)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Size\", size)",
            "DocSection::build(cx, \"Image\", image)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"API Reference\", api_reference)",
            "DocSection::build(cx, \"Rich Title (Fret)\", rich_title)",
            "DocSection::build(cx, \"Compositions\", compositions)",
            "DocSection::build(cx, \"CardContent\", card_content_inline_button)",
            "DocSection::build(cx, \"Meeting Notes\", meeting_notes)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "DocSection::new(\"Demo\", login)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Size\", size)",
            "DocSection::new(\"Image\", image)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"API Reference\", api_reference)",
            "DocSection::new(\"Rich Title (Fret)\", rich_title)",
            "DocSection::new(\"Compositions\", compositions)",
            "DocSection::new(\"CardContent\", card_content_inline_button)",
            "DocSection::new(\"Meeting Notes\", meeting_notes)",
            "preview_card(cx, event_cover_image)",
        ],
    );
}

#[test]
fn image_object_fit_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/image_object_fit/mapping.rs",
            "src/ui/snippets/image_object_fit/sampling.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/image_object_fit",
        &[
            "pub fn render<H: UiHost>(",
            "pub fn render(cx: &mut ElementContext<'_, H>",
            "-> AnyElement",
        ],
    );
}

#[test]
fn image_object_fit_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/image_object_fit.rs",
        &[
            "DocSection::build(cx, \"Fit mapping\", mapping)",
            "DocSection::build(cx, \"Sampling\", sampling)",
            "DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "DocSection::new(\"Fit mapping\", mapping)",
            "DocSection::new(\"Sampling\", sampling)",
            "preview_image_object_fit(cx, theme, square_image, wide_image, tall_image, streaming_image)",
        ],
    );
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
            "fn detail_card<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str, title: &'static str, detail: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn detail_card<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str, title: &'static str, detail: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/collapsible/file_tree.rs",
        &[
            "fn rotated_lucide<H: UiHost>(cx: &mut ElementContext<'_, H>, id: &'static str, rotation_deg: f32,) -> impl IntoUiElement<H> + use<H>",
            "fn file_leaf<H: UiHost>(cx: &mut ElementContext<'_, H>, key: &'static str, label: &'static str,) -> impl IntoUiElement<H> + use<H>",
            "fn folder<H: UiHost>(cx: &mut ElementContext<'_, H>, key: &'static str, label: &'static str, children: &'static [TreeItem],) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn rotated_lucide<H: UiHost>(cx: &mut ElementContext<'_, H>, id: &'static str, rotation_deg: f32,) -> AnyElement",
            "fn file_leaf<H: UiHost>(cx: &mut ElementContext<'_, H>, key: &'static str, label: &'static str,) -> AnyElement",
            "fn folder<H: UiHost>(cx: &mut ElementContext<'_, H>, key: &'static str, label: &'static str, children: &'static [TreeItem],) -> AnyElement",
        ],
    );
}

#[test]
fn data_table_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/data_table/basic_demo.rs",
            "src/ui/snippets/data_table/code_outline.rs",
            "src/ui/snippets/data_table/default_demo.rs",
            "src/ui/snippets/data_table/guide_demo.rs",
            "src/ui/snippets/data_table/rtl_demo.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );
}

#[test]
fn data_table_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/data_table.rs",
        &[
            "DocSection::build(cx, \"Default Recipe (Fret)\", default_demo)",
            "DocSection::build(cx, \"Basic Table\", basic_demo)",
            "DocSection::build(cx, \"Guide Demo\", guide_demo)",
            "DocSection::build(cx, \"RTL\", rtl_demo)",
            "DocSection::build(cx, \"Guide Outline\", code_preview)",
        ],
        &[
            "DocSection::new(\"Default Recipe (Fret)\", default_demo)",
            "DocSection::new(\"Basic Table\", basic_demo)",
            "DocSection::new(\"Guide Demo\", guide_demo)",
            "DocSection::new(\"RTL\", rtl_demo)",
            "DocSection::new(\"Guide Outline\", code_preview)",
        ],
    );
}

#[test]
fn table_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    for path in rust_sources("src/ui/snippets/table") {
        if path.file_name().is_some_and(|name| name == "mod.rs") {
            continue;
        }

        let source = read_path(&path);
        assert_default_app_surface(
            &path,
            &source,
            &[
                "use fret::{UiChild, UiCx};",
                "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            ],
            "app-facing snippet surface",
        );
    }

    assert_sources_absent(
        "src/ui/snippets/table",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> AnyElement",
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement",
        ],
    );
}

#[test]
fn table_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/table.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Footer\", footer)",
            "DocSection::build(cx, \"Actions\", actions)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Footer\", footer)",
            "DocSection::new(\"Actions\", actions)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn remaining_app_facing_tail_snippets_prefer_ui_cx_on_the_default_app_surface() {
    for relative_path in [
        "src/ui/snippets/breadcrumb/responsive.rs",
        "src/ui/snippets/date_picker/dropdowns.rs",
        "src/ui/snippets/form/notes.rs",
        "src/ui/snippets/sidebar/rtl.rs",
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        assert_default_app_surface(
            &path,
            &source,
            &[
                "use fret::{UiChild, UiCx};",
                "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
            ],
            "app-facing snippet surface",
        );

        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains("pubfnrender(cx:&mutUiCx<'_>)->AnyElement"),
            "{} reintroduced `UiCx -> AnyElement` on the default app surface",
            path.display()
        );
    }
}

#[test]
fn remaining_app_facing_tail_pages_use_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/breadcrumb.rs",
        &["DocSection::build(cx, \"Responsive\", responsive)"],
        &["DocSection::new(\"Responsive\", responsive)"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/date_picker.rs",
        &["DocSection::build(cx, \"Extras: With Dropdowns\", dropdowns)"],
        &["DocSection::new(\"Extras: With Dropdowns\", dropdowns)"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/form.rs",
        &["DocSection::build(cx, \"Notes\", notes)"],
        &["DocSection::new(\"Notes\", notes)"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/sidebar.rs",
        &["DocSection::build(cx, \"Extras: RTL\", rtl)"],
        &["DocSection::new(\"Extras: RTL\", rtl)"],
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
fn curated_ai_doc_pages_use_typed_doc_sections() {
    for path in rust_sources("src/ui/pages") {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.starts_with("ai_") {
            continue;
        }

        let source = read_path(&path);
        assert!(
            !source.contains("DocSection::new("),
            "{} should keep using DocSection::build(cx, ...) on the first-party AI docs surface",
            path.display()
        );
        assert!(
            source.contains("DocSection::build(cx, "),
            "{} should keep an explicit typed DocSection builder on the first-party AI docs surface",
            path.display()
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
            "let api_reference = doc_layout::notes_block([",
            "let notes = doc_layout::notes_block([",
            "let about = DocSection::build(cx, \"About\", about)",
            "let date_picker = DocSection::build(cx, \"Date Picker\", date_picker)",
            "let selected_date_timezone = DocSection::build(cx, \"Selected Date (With TimeZone)\", selected_date_timezone)",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
            "let notes = DocSection::build(cx, \"Notes\", notes)",
        ],
        &[
            "let about = doc_layout::notes(",
            "let date_picker = doc_layout::notes(",
            "let selected_date_timezone = doc_layout::notes(",
            "let api_reference = doc_layout::notes(",
            "let notes = doc_layout::notes(",
            "DocSection::new(\"About\", about)",
            "DocSection::new(\"Date Picker\", date_picker)",
            "DocSection::new(\"Selected Date (With TimeZone)\", selected_date_timezone)",
            "DocSection::new(\"API Reference\", api_reference)",
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
        "src/ui/pages/card.rs",
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

    for relative_path in ["src/ui/pages/input_otp.rs", "src/ui/pages/sidebar.rs"] {
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
        "src/ui/pages/aspect_ratio.rs",
        &[
            "let api_reference = doc_layout::notes_block([",
            "let api_reference = DocSection::build(cx, \"API Reference\", api_reference)",
        ],
        &[
            "let api_reference = doc_layout::notes(",
            "DocSection::new(\"API Reference\", api_reference)",
        ],
    );

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
        "src/ui/snippets/carousel/compact_builder.rs",
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
                "fn avatar_with_image<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<ImageId>, size: shadcn::AvatarSize, fallback_text: &'static str,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn avatar_with_image<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<ImageId>, size: shadcn::AvatarSize, fallback_text: &'static str,) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/avatar/sizes.rs",
        &[
            "fn avatar_with_image<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<ImageId>, size: shadcn::AvatarSize, fallback_text: &'static str, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
            "shadcn::avatar_sized(",
        ],
        &[
            "fn avatar_with_image<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<ImageId>, size: shadcn::AvatarSize, fallback_text: &'static str, test_id: &'static str,) -> AnyElement",
            "shadcn::Avatar::new([image, fallback]).size(size)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/avatar/demo.rs",
        &[
            "fn avatar_with_badge<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<ImageId>, fallback_text: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn avatar_with_badge<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<ImageId>, fallback_text: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/avatar/with_badge.rs",
        &[
            "fn avatar_with_badge<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<fret_core::ImageId>, size: shadcn::AvatarSize, badge: shadcn::AvatarBadge, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn avatar_with_badge<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<fret_core::ImageId>, size: shadcn::AvatarSize, badge: shadcn::AvatarBadge, test_id: &'static str,) -> AnyElement",
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
            "fn group<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<ImageId>, size: shadcn::AvatarSize, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn group<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<ImageId>, size: shadcn::AvatarSize, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/avatar/group_count.rs",
        &[
            "fn group_with_count<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<ImageId>, size: shadcn::AvatarSize, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn group_with_count<H: UiHost>(cx: &mut ElementContext<'_, H>, avatar_image: Option<ImageId>, size: shadcn::AvatarSize, test_id: &'static str,) -> AnyElement",
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
                "fn trigger_surface<H: UiHost>(fine_label: &'static str, coarse_label: &'static str, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                "fn trigger_surface<H: UiHost>(fine_label: &'static str, coarse_label: &'static str, test_id: &'static str,) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/context_menu/demo.rs",
        &[
            "fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/context_menu/sides.rs",
        &[
            "fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, fine_label: &'static str, coarse_label: &'static str, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
            "fn side_menu<H: UiHost>(cx: &mut ElementContext<'_, H>, fine_label: &'static str, coarse_label: &'static str, side: shadcn::DropdownMenuSide, trigger_test_id: &'static str, content_test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, fine_label: &'static str, coarse_label: &'static str, test_id: &'static str,) -> AnyElement",
            "fn side_menu<H: UiHost>(cx: &mut ElementContext<'_, H>, fine_label: &'static str, coarse_label: &'static str, side: shadcn::DropdownMenuSide, trigger_test_id: &'static str, content_test_id: &'static str,) -> AnyElement",
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
        "src/ui/snippets/pagination/compact_builder.rs",
        "src/ui/snippets/pagination/custom_text.rs",
        "src/ui/snippets/pagination/routing.rs",
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
fn selected_pagination_snippets_keep_wrapper_and_parts_lanes_explicit() {
    for relative_path in [
        "src/ui/snippets/pagination/compact_builder.rs",
        "src/ui/snippets/pagination/custom_text.rs",
        "src/ui/snippets/pagination/demo.rs",
        "src/ui/snippets/pagination/extras.rs",
        "src/ui/snippets/pagination/icons_only.rs",
        "src/ui/snippets/pagination/rtl.rs",
        "src/ui/snippets/pagination/simple.rs",
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
        "src/ui/snippets/pagination/compact_builder.rs",
        "src/ui/snippets/pagination/custom_text.rs",
        "src/ui/snippets/pagination/demo.rs",
        "src/ui/snippets/pagination/extras.rs",
        "src/ui/snippets/pagination/rtl.rs",
        "src/ui/snippets/pagination/simple.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::pagination_link(|cx|"],
            &["shadcn::PaginationLink::new("],
        );
    }

    for relative_path in [
        "src/ui/snippets/pagination/routing.rs",
        "src/ui/snippets/pagination/usage.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "shadcn::Pagination::new(",
                "shadcn::PaginationContent::new(",
                "shadcn::PaginationItem::new(",
                "shadcn::PaginationLink::new(",
            ],
            &[
                "shadcn::pagination(|cx|",
                "shadcn::pagination_content(|cx|",
                "shadcn::pagination_item(",
                "shadcn::pagination_link(|cx|",
            ],
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

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/carousel/compact_builder.rs",
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
fn selected_carousel_usage_compact_builder_and_parts_snippets_keep_their_lane_story() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/carousel/usage.rs",
        &[
            "shadcn::Carousel::default()",
            ".test_id(\"ui-gallery-carousel-usage\")",
            ".into_element_parts_content(",
        ],
        &["shadcn::Carousel::new(items)"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/carousel/compact_builder.rs",
        &[
            "shadcn::Carousel::new(items)",
            ".test_id(\"ui-gallery-carousel-compact-builder\")",
            ".into_element(cx)",
        ],
        &[".into_element_parts(", ".into_element_parts_content("],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/carousel/parts.rs",
        &[
            "shadcn::Carousel::default()",
            ".test_id(\"ui-gallery-carousel-parts\")",
            ".test_id(\"ui-gallery-carousel-parts-previous\")",
            ".test_id(\"ui-gallery-carousel-parts-next\")",
            ".into_element_parts(",
        ],
        &[".into_element_parts_content("],
    );
}

#[test]
fn selected_carousel_docs_examples_follow_the_compact_builder_lane() {
    for relative_path in [
        "src/ui/snippets/carousel/sizes_thirds.rs",
        "src/ui/snippets/carousel/sizes.rs",
        "src/ui/snippets/carousel/spacing.rs",
        "src/ui/snippets/carousel/spacing_responsive.rs",
        "src/ui/snippets/carousel/orientation_vertical.rs",
        "src/ui/snippets/carousel/options.rs",
        "src/ui/snippets/carousel/loop_carousel.rs",
    ] {
        let normalized = assert_normalized_markers_present(
            relative_path,
            &["shadcn::Carousel::new(items)", ".into_element(cx)"],
        );
        assert!(
            !normalized.contains(".into_element_parts("),
            "{} should keep docs-first carousel examples on the compact builder lane",
            manifest_path(relative_path).display()
        );
    }

    let carousel_page = read("src/ui/pages/carousel.rs");
    assert!(
        carousel_page.contains(
            "The docs-path examples below (`Sizes`, `Spacing`, `Orientation`, `Options`) and the docs-aligned previews (`Demo`, `API`, base autoplay plugin, `RTL`) still stay on the compact builder lane unless a snippet explicitly needs control-level parts or diagnostics-specific control IDs."
        ),
        "src/ui/pages/carousel.rs should explain which examples remain on the upstream docs path compact-builder lane"
    );
}

#[test]
fn carousel_page_keeps_docs_width_lane_distinct_from_fixed_width_diagnostics_harnesses() {
    for relative_path in [
        "src/ui/snippets/carousel/options.rs",
        "src/ui/snippets/carousel/loop_carousel.rs",
        "src/ui/snippets/carousel/api.rs",
        "src/ui/snippets/carousel/events.rs",
        "src/ui/snippets/carousel/plugin_autoplay.rs",
        "src/ui/snippets/carousel/plugin_autoplay_delays.rs",
        "src/ui/snippets/carousel/plugin_autoplay_stop_on_last_snap.rs",
    ] {
        let normalized = assert_normalized_markers_present(relative_path, &[".max_w(max_w_xs)"]);
        assert!(
            !normalized.contains(".w_px(max_w_xs)"),
            "{} should keep the upstream docs width lane (`w_full + max_w`) instead of a fixed-width harness",
            manifest_path(relative_path).display()
        );
    }

    let duration_normalized = assert_normalized_markers_present(
        "src/ui/snippets/carousel/duration_embla.rs",
        &["LayoutRefinement::default().w_px(max_w_xs)"],
    );
    assert!(
        !duration_normalized.contains("LayoutRefinement::default().w_full().max_w(max_w_xs)"),
        "{} should keep the fixed-width harness lane for deterministic engine evidence",
        manifest_path("src/ui/snippets/carousel/duration_embla.rs").display()
    );

    let carousel_page = read("src/ui/pages/carousel.rs");
    assert!(
        carousel_page.contains(
            "Docs-path snippets keep the upstream `w_full().max_w(...)` width lane on the carousel root itself; diagnostics follow-ups may switch to fixed-width shells (`w_px(...)`) when deterministic control geometry matters more than copyable docs parity."
        ),
        "src/ui/pages/carousel.rs should explain why docs snippets keep the upstream root width lane while diagnostics harnesses may use fixed-width shells"
    );
}

#[test]
fn carousel_page_keeps_basic_preview_out_of_the_upstream_docs_path() {
    let carousel_page = read("src/ui/pages/carousel.rs");
    let normalized = carousel_page.split_whitespace().collect::<String>();

    assert!(
        carousel_page.contains(
            "`Basic` remains a gallery follow-up baseline preview because the upstream docs jump straight from `Usage` into the `Sizes` examples instead of showing a separate single-slide baseline section."
        ),
        "src/ui/pages/carousel.rs should explain why `Basic` stays after the upstream docs path"
    );
    assert!(
        carousel_page.contains(
            "Preview mirrors the shadcn Carousel docs path first: Demo, About, Usage, Examples (Sizes/Spacing/Orientation), Options, API, Events, Plugin, RTL. After that, Gallery keeps Fret-only follow-ups explicit: `Fret Follow-ups`, `Basic`, extra plugin variants, `Compact Builder`, `Parts`, a dedicated `Loop` preview, engine/motion diagnostics, then `API Reference`."
        ),
        "src/ui/pages/carousel.rs should describe `Basic` as a follow-up baseline preview rather than part of the upstream docs path"
    );
    assert!(
        normalized.contains(
            "vec![demo,about,usage,sizes_thirds,sizes,spacing,spacing_responsive,orientation_vertical,options,api,events,plugin,rtl,fret_follow_ups,basic,plugin_controlled,plugin_stop_on_focus,plugin_stop_on_last_snap,plugin_delays,plugin_wheel,compact_builder,parts,loop_carousel,loop_downgrade_cannot_loop,focus,duration,expandable,api_reference,]"
        ),
        "src/ui/pages/carousel.rs should place `Basic` after `Fret Follow-ups` instead of inside the upstream docs path"
    );
    assert!(
        !normalized.contains(
            "vec![demo,about,usage,basic,sizes_thirds,sizes,spacing,spacing_responsive,orientation_vertical,options,api,events,plugin,rtl,"
        ),
        "src/ui/pages/carousel.rs should not keep `Basic` before the docs-path size examples"
    );
}

#[test]
fn carousel_page_keeps_extra_plugin_variants_out_of_the_upstream_docs_path() {
    let carousel_page = read("src/ui/pages/carousel.rs");
    let normalized = carousel_page.split_whitespace().collect::<String>();

    assert!(
        carousel_page.contains(
            "`Plugin (Autoplay, Controlled)`, `Plugin (Autoplay, stopOnInteraction via focus)`, `Plugin (Autoplay, stopOnLastSnap)`, `Plugin (Autoplay, per-snap delays)`, and `Plugin (Wheel gestures)` remain follow-ups because the upstream docs only show the base autoplay plugin example."
        ),
        "src/ui/pages/carousel.rs should explain why the extra plugin variants stay after the upstream docs path"
    );
    assert!(
        carousel_page.contains(
            "Preview mirrors the shadcn Carousel docs path first: Demo, About, Usage, Examples (Sizes/Spacing/Orientation), Options, API, Events, Plugin, RTL. After that, Gallery keeps Fret-only follow-ups explicit: `Fret Follow-ups`, `Basic`, extra plugin variants, `Compact Builder`, `Parts`, a dedicated `Loop` preview, engine/motion diagnostics, then `API Reference`."
        ),
        "src/ui/pages/carousel.rs should describe the narrowed docs path before the follow-up plugin variants"
    );
    assert!(
        normalized.contains(
            "vec![demo,about,usage,sizes_thirds,sizes,spacing,spacing_responsive,orientation_vertical,options,api,events,plugin,rtl,fret_follow_ups,basic,plugin_controlled,plugin_stop_on_focus,plugin_stop_on_last_snap,plugin_delays,plugin_wheel,compact_builder,parts,loop_carousel,loop_downgrade_cannot_loop,focus,duration,expandable,api_reference,]"
        ),
        "src/ui/pages/carousel.rs should place the extra plugin variants after `Fret Follow-ups` instead of inside the upstream docs path"
    );
    assert!(
        !normalized.contains(
            "vec![demo,about,usage,basic,sizes_thirds,sizes,spacing,spacing_responsive,orientation_vertical,options,api,events,plugin,plugin_controlled,plugin_stop_on_focus,plugin_stop_on_last_snap,plugin_delays,plugin_wheel,rtl,"
        ),
        "src/ui/pages/carousel.rs should not place the extra plugin variants before `RTL`"
    );
}

#[test]
fn carousel_parts_lane_is_limited_to_explicit_parts_and_diagnostics_snippets() {
    let expected = BTreeSet::from([
        "src/ui/snippets/carousel/events.rs".to_string(),
        "src/ui/snippets/carousel/parts.rs".to_string(),
        "src/ui/snippets/carousel/rtl.rs".to_string(),
        "src/ui/snippets/carousel/usage.rs".to_string(),
    ]);

    let actual = rust_sources("src/ui/snippets/carousel")
        .into_iter()
        .filter_map(|path| {
            let source = read_path(&path);
            (source.contains(".into_element_parts(")
                || source.contains(".into_element_parts_content("))
            .then(|| {
                path.strip_prefix(manifest_path(""))
                    .unwrap()
                    .display()
                    .to_string()
            })
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual, expected,
        "carousel parts-lane usage drifted; only the docs-aligned Usage snippet plus explicit parts/diagnostics seams should exercise the parts adapters"
    );
}

#[test]
fn selected_carousel_diagnostics_snippets_prefer_compact_builder_when_parts_are_unnecessary() {
    for relative_path in [
        "src/ui/snippets/carousel/api.rs",
        "src/ui/snippets/carousel/demo.rs",
        "src/ui/snippets/carousel/expandable.rs",
        "src/ui/snippets/carousel/focus_watch.rs",
        "src/ui/snippets/carousel/loop_downgrade_cannot_loop.rs",
        "src/ui/snippets/carousel/plugin_autoplay.rs",
        "src/ui/snippets/carousel/plugin_autoplay_controlled.rs",
        "src/ui/snippets/carousel/plugin_autoplay_delays.rs",
        "src/ui/snippets/carousel/plugin_autoplay_stop_on_focus.rs",
        "src/ui/snippets/carousel/plugin_autoplay_stop_on_last_snap.rs",
        "src/ui/snippets/carousel/plugin_wheel_gestures.rs",
    ] {
        let normalized = assert_normalized_markers_present(relative_path, &[".into_element(cx)"]);
        assert!(
            normalized.contains("shadcn::Carousel::new("),
            "{} should build the compact builder lane from `Carousel::new(...)`",
            manifest_path(relative_path).display()
        );
        assert!(
            !normalized.contains(".into_element_parts("),
            "{} should stay on the compact builder lane because it does not need explicit control parts",
            manifest_path(relative_path).display()
        );
    }

    let duration_normalized = assert_normalized_markers_present(
        "src/ui/snippets/carousel/duration_embla.rs",
        &[
            "shadcn::Carousel::new(duration_items_fast)",
            "shadcn::Carousel::new(duration_items_slow)",
            ".into_element(cx)",
        ],
    );
    assert!(
        !duration_normalized.contains(".into_element_parts("),
        "{} should stay on the compact builder lane because it does not need explicit control parts",
        manifest_path("src/ui/snippets/carousel/duration_embla.rs").display()
    );
}

#[test]
fn selected_button_group_composition_snippets_follow_child_family_default_lanes() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/button_group/input_group.rs",
        &[
            "shadcn::InputGroup::new(message_value)",
            ".control_test_id(\"ui-gallery-button-group-input-group-control\")",
            ".trailing([voice_tooltip])",
            ".trailing_has_button(true)",
            ".into_element(cx)",
        ],
        &[".into_element_parts("],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/button_group/button_group_select.rs",
        &[
            "shadcn::Select::new(currency_value.clone(), currency_open.clone())",
            ".trigger(",
            ".value(shadcn::SelectValue::new())",
            ".content(",
            ".entries(entries)",
            ".into_element(cx)",
        ],
        &[".into_element_parts("],
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
            "shadcn::resizable_panel_group(",
        ],
        &[
            "fn panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement",
            "shadcn::ResizablePanelGroup::new(",
        ],
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
                "shadcn::resizable_panel_group(",
            ],
            &[
                "fn box_group<H: UiHost>(cx: &mut ElementContext<'_, H>, layout: LayoutRefinement, body: AnyElement,) -> AnyElement",
                "fn panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement",
                "shadcn::ResizablePanelGroup::new(",
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
                "shadcn::resizable_panel_group(",
            ],
            &[
                "fn box_group<H: UiHost>(cx: &mut ElementContext<'_, H>, layout: LayoutRefinement, body: AnyElement,) -> AnyElement",
                "fn panel<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str, height: Option<Px>,) -> AnyElement",
                "shadcn::ResizablePanelGroup::new(",
            ],
        );
    }
}

#[test]
fn selected_scroll_area_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/scroll_area/usage.rs",
        &["shadcn::ScrollArea::new([content])"],
        &["shadcn::scroll_area("],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/scroll_area/demo.rs",
        &[
            "fn tag_row(tag: Arc<str>) -> impl IntoUiElement<fret_app::App> + use<>",
            "shadcn::ScrollArea::new([content])",
        ],
        &[
            "fn tag_row(tag: Arc<str>) -> AnyElement",
            "shadcn::scroll_area(",
            "shadcn::ScrollArea::build(",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/scroll_area/horizontal.rs",
        &[
            "shadcn::ScrollAreaViewport::new([rail])",
            "shadcn::ScrollAreaRoot::new(viewport)",
            "shadcn::ScrollBar::new()",
        ],
        &["shadcn::scroll_area(", "shadcn::ScrollArea::new("],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/scroll_area/rtl.rs",
        &["shadcn::ScrollArea::new([content])"],
        &["shadcn::scroll_area("],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/scroll_area/compact_helper.rs",
        &["shadcn::scroll_area(cx, |_cx| [content])"],
        &["shadcn::ScrollArea::new("],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/scroll_area/nested_scroll_routing.rs",
        &[
            "fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, i: usize) -> impl IntoUiElement<H> + use<H>",
            "shadcn::scroll_area(cx, |_cx| [inner_rail])",
            "shadcn::scroll_area(cx, |_cx| [outer_body])",
        ],
        &[
            "fn row<H: UiHost>(cx: &mut ElementContext<'_, H>, i: usize) -> AnyElement",
            "shadcn::ScrollArea::new([inner_rail])",
            "shadcn::ScrollArea::new([outer_body])",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/diagnostics/scroll_area/expand_at_bottom.rs",
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

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/data_table/guide_demo.rs",
        &["fn guide_demo_content(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &["fn guide_demo_content(cx: &mut UiCx<'_>) -> Vec<AnyElement>"],
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
fn selected_radio_group_snippets_prefer_builder_preserving_helpers() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/radio_group/usage.rs",
        &["shadcn::radio_group_uncontrolled("],
        &["shadcn::RadioGroup::uncontrolled("],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/radio_group/label.rs",
        &[
            "shadcn::radio_group(",
            "shadcn::RadioGroupItem::new(\"free\", \"Free\")",
        ],
        &["shadcn::RadioGroup::new(value)"],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/form/upstream_demo.rs",
        &[
            "shadcn::radio_group(",
            "notify_type.clone()",
            "shadcn::RadioGroupItem::new(\"all\", \"All new messages\")",
        ],
        &["shadcn::RadioGroup::new(notify_type.clone())"],
    );
}

#[test]
fn field_page_usage_prefers_field_wrapper_family() {
    let page = read("src/ui/pages/field.rs");
    let usage = read("src/ui/snippets/field/usage.rs");
    assert!(
        usage.contains("shadcn::field_set(|cx| {"),
        "src/ui/snippets/field/usage.rs should keep the docs-aligned wrapper-family usage lane"
    );
    assert!(
        usage.contains("shadcn::field_group(|cx| {"),
        "src/ui/snippets/field/usage.rs should keep the grouped docs-aligned wrapper-family usage lane"
    );
    assert!(
        page.contains(".code_rust_from_file_region(snippets::usage::SOURCE, \"example\")"),
        "src/ui/pages/field.rs should show the Usage section from a real snippet file"
    );
    assert!(
        page.contains(".code_rust_from_file_region(snippets::anatomy::SOURCE, \"example\")"),
        "src/ui/pages/field.rs should show the Anatomy section from a real snippet file"
    );
    assert!(
        !page.contains(".code_rust("),
        "src/ui/pages/field.rs should avoid page-local hand-written Rust strings for docs-facing sections"
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
                "fn centered<B>(cx: &mut UiCx<'_>, body: B) -> impl UiChild + use<B> where B: UiChild",
            ],
            &[
                "fn centered<H: UiHost, B>(body: B) -> impl IntoUiElement<H> + use<H, B> where B: IntoUiElement<H>",
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
            &["pub fn preview(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
            &[
                "pub fn preview<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>,) -> impl IntoUiElement<H> + use<H>",
                "pub fn preview<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement",
            ],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/test_results_demo.rs",
        &["fn progress_section(cx: &mut UiCx<'_>) -> impl UiChild + use<>"],
        &[
            "fn progress_section<H: UiHost>(cx: &mut ElementContext<'_, H>) -> impl IntoUiElement<H> + use<H>",
            "fn progress_section<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/attachments_usage.rs",
        &[
            "fn render_grid_attachment(cx: &mut UiCx<'_>, data: ui_ai::AttachmentData,) -> impl UiChild + use<>",
        ],
        &[
            "fn render_grid_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData,) -> impl IntoUiElement<H> + use<H>",
            "fn render_grid_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/file_tree_demo.rs",
        &["fn invisible_marker(cx: &mut UiCx<'_>, test_id: &'static str) -> impl UiChild + use<>"],
        &[
            "fn invisible_marker<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
            "fn invisible_marker<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/file_tree_large.rs",
        &["fn invisible_marker(cx: &mut UiCx<'_>, test_id: &'static str) -> impl UiChild + use<>"],
        &[
            "fn invisible_marker<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str,) -> impl IntoUiElement<H> + use<H>",
            "fn invisible_marker<H: UiHost>(cx: &mut ElementContext<'_, H>, test_id: &'static str,) -> AnyElement",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/speech_input_demo.rs",
        &[
            "fn body_text(cx: &mut UiCx<'_>, text: impl Into<Arc<str>>, style: TextStyle, color: Color, align: TextAlign,) -> impl UiChild + use<>",
            "fn clear_action(cx: &mut UiCx<'_>, transcript: Model<String>) -> impl UiChild + use<>",
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
            "fn render_grid_attachment(cx: &mut UiCx<'_>, data: ui_ai::AttachmentData, on_remove: ui_ai::OnAttachmentRemove, test_id: Option<&'static str>, remove_test_id: Option<&'static str>,) -> impl UiChild + use<>"
        } else {
            "fn render_list_attachment(cx: &mut UiCx<'_>, data: ui_ai::AttachmentData, on_remove: ui_ai::OnAttachmentRemove, test_id: Option<&'static str>,) -> impl UiChild + use<>"
        };

        let old_helper = if relative_path.ends_with("attachments_grid.rs") {
            "fn render_grid_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData, on_remove: ui_ai::OnAttachmentRemove, test_id: Option<&'static str>, remove_test_id: Option<&'static str>,) -> AnyElement"
        } else {
            "fn render_list_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData, on_remove: ui_ai::OnAttachmentRemove, test_id: Option<&'static str>,) -> AnyElement"
        };

        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[helper],
            &[
                old_helper,
                if relative_path.ends_with("attachments_grid.rs") {
                    "fn render_grid_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData, on_remove: ui_ai::OnAttachmentRemove, test_id: Option<&'static str>, remove_test_id: Option<&'static str>,) -> impl IntoUiElement<H> + use<H>"
                } else {
                    "fn render_list_attachment<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, data: ui_ai::AttachmentData, on_remove: ui_ai::OnAttachmentRemove, test_id: Option<&'static str>,) -> impl IntoUiElement<H> + use<H>"
                },
            ],
        );
    }
}

#[test]
fn selected_breadcrumb_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/breadcrumb/dropdown.rs",
        &[
            "fn slash_separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> impl IntoUiElement<H> + use<H>",
        ],
        &["fn slash_separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
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
fn toggle_group_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/toggle_group/custom.rs",
            "src/ui/snippets/toggle_group/demo.rs",
            "src/ui/snippets/toggle_group/disabled.rs",
            "src/ui/snippets/toggle_group/flex_1_items.rs",
            "src/ui/snippets/toggle_group/full_width_items.rs",
            "src/ui/snippets/toggle_group/label.rs",
            "src/ui/snippets/toggle_group/large.rs",
            "src/ui/snippets/toggle_group/outline.rs",
            "src/ui/snippets/toggle_group/rtl.rs",
            "src/ui/snippets/toggle_group/single.rs",
            "src/ui/snippets/toggle_group/size.rs",
            "src/ui/snippets/toggle_group/small.rs",
            "src/ui/snippets/toggle_group/spacing.rs",
            "src/ui/snippets/toggle_group/usage.rs",
            "src/ui/snippets/toggle_group/vertical.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/toggle_group",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn toggle_group_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/toggle_group.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Outline\", outline)",
            "DocSection::build(cx, \"Size\", size)",
            "DocSection::build(cx, \"Spacing\", spacing)",
            "DocSection::build(cx, \"Vertical\", vertical)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"Custom\", custom)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Single (Fret)\", single)",
            "DocSection::build(cx, \"Small (Fret)\", small)",
            "DocSection::build(cx, \"Large (Fret)\", large)",
            "DocSection::build(cx, \"Label Association (Fret)\", label)",
            "DocSection::build(cx, \"Full Width Items (Fret)\", full_width_items)",
            "DocSection::build(cx, \"Flex-1 Items (Fret)\", stretch)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Outline\", outline)",
            "DocSection::new(\"Size\", size)",
            "DocSection::new(\"Spacing\", spacing)",
            "DocSection::new(\"Vertical\", vertical)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"Custom\", custom)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Single (Fret)\", single)",
            "DocSection::new(\"Small (Fret)\", small)",
            "DocSection::new(\"Large (Fret)\", large)",
            "DocSection::new(\"Label Association (Fret)\", label)",
            "DocSection::new(\"Full Width Items (Fret)\", full_width_items)",
            "DocSection::new(\"Flex-1 Items (Fret)\", stretch)",
        ],
    );
}

#[test]
fn switch_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/switch/airplane_mode.rs",
            "src/ui/snippets/switch/bluetooth.rs",
            "src/ui/snippets/switch/choice_card.rs",
            "src/ui/snippets/switch/description.rs",
            "src/ui/snippets/switch/disabled.rs",
            "src/ui/snippets/switch/invalid.rs",
            "src/ui/snippets/switch/label.rs",
            "src/ui/snippets/switch/rtl.rs",
            "src/ui/snippets/switch/sizes.rs",
            "src/ui/snippets/switch/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/switch",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn switch_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/switch.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Description\", description)",
            "DocSection::build(cx, \"Choice Card\", choice_card)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"Invalid\", invalid)",
            "DocSection::build(cx, \"Size\", sizes)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Label Association\", label)",
            "DocSection::build(cx, \"Style Override\", style_override)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Description\", description)",
            "DocSection::new(\"Choice Card\", choice_card)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"Invalid\", invalid)",
            "DocSection::new(\"Size\", sizes)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Label Association\", label)",
            "DocSection::new(\"Style Override\", style_override)",
        ],
    );
}

#[test]
fn checkbox_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/checkbox/basic.rs",
            "src/ui/snippets/checkbox/checked_state.rs",
            "src/ui/snippets/checkbox/demo.rs",
            "src/ui/snippets/checkbox/description.rs",
            "src/ui/snippets/checkbox/disabled.rs",
            "src/ui/snippets/checkbox/group.rs",
            "src/ui/snippets/checkbox/invalid_state.rs",
            "src/ui/snippets/checkbox/label.rs",
            "src/ui/snippets/checkbox/rtl.rs",
            "src/ui/snippets/checkbox/table.rs",
            "src/ui/snippets/checkbox/usage.rs",
            "src/ui/snippets/checkbox/with_title.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/checkbox",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn checkbox_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/checkbox.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Checked State\", checked_state)",
            "DocSection::build(cx, \"Invalid State\", invalid_state)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Description\", description_section)",
            "DocSection::build(cx, \"Disabled\", disabled_section)",
            "DocSection::build(cx, \"Group\", group)",
            "DocSection::build(cx, \"Table\", table)",
            "DocSection::build(cx, \"RTL\", rtl_section)",
            "DocSection::build(cx, \"Label Association (Fret)\", label)",
            "DocSection::build(cx, \"With Title (Fret)\", with_title_section)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Checked State\", checked_state)",
            "DocSection::new(\"Invalid State\", invalid_state)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Description\", description_section)",
            "DocSection::new(\"Disabled\", disabled_section)",
            "DocSection::new(\"Group\", group)",
            "DocSection::new(\"Table\", table)",
            "DocSection::new(\"RTL\", rtl_section)",
            "DocSection::new(\"Label Association (Fret)\", label)",
            "DocSection::new(\"With Title (Fret)\", with_title_section)",
        ],
    );
}

#[test]
fn toggle_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/toggle/demo.rs",
            "src/ui/snippets/toggle/disabled.rs",
            "src/ui/snippets/toggle/label.rs",
            "src/ui/snippets/toggle/outline.rs",
            "src/ui/snippets/toggle/rtl.rs",
            "src/ui/snippets/toggle/size.rs",
            "src/ui/snippets/toggle/usage.rs",
            "src/ui/snippets/toggle/with_text.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/toggle",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn selected_toggle_snippets_prefer_builder_preserving_helpers() {
    for relative_path in [
        "src/ui/snippets/toggle/demo.rs",
        "src/ui/snippets/toggle/disabled.rs",
        "src/ui/snippets/toggle/outline.rs",
        "src/ui/snippets/toggle/rtl.rs",
        "src/ui/snippets/toggle/size.rs",
        "src/ui/snippets/toggle/usage.rs",
        "src/ui/snippets/toggle/with_text.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::toggle_uncontrolled("],
            &["shadcn::Toggle::uncontrolled("],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/toggle/label.rs",
        &["shadcn::toggle("],
        &["shadcn::Toggle::new("],
    );
}

#[test]
fn toggle_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/toggle.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Outline\", outline)",
            "DocSection::build(cx, \"With Text\", with_text)",
            "DocSection::build(cx, \"Size\", size)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Label Association\", label)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Outline\", outline)",
            "DocSection::new(\"With Text\", with_text)",
            "DocSection::new(\"Size\", size)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Label Association\", label)",
        ],
    );
}

#[test]
fn radio_group_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/radio_group/choice_card.rs",
            "src/ui/snippets/radio_group/demo.rs",
            "src/ui/snippets/radio_group/description.rs",
            "src/ui/snippets/radio_group/disabled.rs",
            "src/ui/snippets/radio_group/fieldset.rs",
            "src/ui/snippets/radio_group/invalid.rs",
            "src/ui/snippets/radio_group/label.rs",
            "src/ui/snippets/radio_group/rtl.rs",
            "src/ui/snippets/radio_group/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/radio_group",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn radio_group_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/radio_group.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Description\", description)",
            "DocSection::build(cx, \"Choice Card\", choice_card)",
            "DocSection::build(cx, \"Fieldset\", fieldset)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"Invalid\", invalid)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Label Association (Fret)\", label)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Description\", description)",
            "DocSection::new(\"Choice Card\", choice_card)",
            "DocSection::new(\"Fieldset\", fieldset)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"Invalid\", invalid)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Label Association (Fret)\", label)",
        ],
    );
}

#[test]
fn slider_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/slider.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Label Association\", label)",
            "DocSection::build(cx, \"Extras\", extras)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Label Association\", label)",
            "DocSection::new(\"Extras\", extras)",
        ],
    );
}

#[test]
fn native_select_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/native_select.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Groups\", groups)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"Invalid\", invalid)",
            "DocSection::build(cx, \"RTL\", rtl)",
            "DocSection::build(cx, \"Label Association\", label)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Groups\", groups)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"Invalid\", invalid)",
            "DocSection::new(\"RTL\", rtl)",
            "DocSection::new(\"Label Association\", label)",
        ],
    );
}

#[test]
fn resizable_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/resizable/demo.rs",
            "src/ui/snippets/resizable/handle.rs",
            "src/ui/snippets/resizable/notes.rs",
            "src/ui/snippets/resizable/rtl.rs",
            "src/ui/snippets/resizable/usage.rs",
            "src/ui/snippets/resizable/vertical.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/resizable",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn resizable_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/resizable.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Handle\", handle)",
            "DocSection::build(cx, \"Vertical\", vertical)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Handle\", handle)",
            "DocSection::new(\"Vertical\", vertical)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn accordion_app_facing_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/accordion/basic.rs",
            "src/ui/snippets/accordion/borders.rs",
            "src/ui/snippets/accordion/card.rs",
            "src/ui/snippets/accordion/demo.rs",
            "src/ui/snippets/accordion/disabled.rs",
            "src/ui/snippets/accordion/extras.rs",
            "src/ui/snippets/accordion/multiple.rs",
            "src/ui/snippets/accordion/rtl.rs",
            "src/ui/snippets/accordion/usage.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/accordion",
        &["pub fn render(cx: &mut UiCx<'_>) -> AnyElement"],
    );
}

#[test]
fn selected_accordion_snippets_prefer_builder_preserving_helpers() {
    for relative_path in [
        "src/ui/snippets/accordion/basic.rs",
        "src/ui/snippets/accordion/borders.rs",
        "src/ui/snippets/accordion/demo.rs",
        "src/ui/snippets/accordion/disabled.rs",
        "src/ui/snippets/accordion/rtl.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::accordion_single_uncontrolled("],
            &["shadcn::Accordion::single_uncontrolled("],
        );
    }

    for relative_path in [
        "src/ui/snippets/accordion/card.rs",
        "src/ui/snippets/accordion/multiple.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::accordion_multiple_uncontrolled("],
            &["shadcn::Accordion::multiple_uncontrolled("],
        );
    }

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/accordion/extras.rs",
        &[
            "shadcn::accordion_single_uncontrolled(",
            "shadcn::accordion_multiple_uncontrolled(",
        ],
        &[
            "shadcn::Accordion::single_uncontrolled(",
            "shadcn::Accordion::multiple_uncontrolled(",
        ],
    );
}

#[test]
fn accordion_usage_snippet_keeps_the_composable_advanced_seam() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/accordion/usage.rs",
        &["acc::AccordionRoot::single_uncontrolled("],
        &["shadcn::Accordion::single_uncontrolled("],
    );
}

#[test]
fn accordion_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/accordion.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Basic\", basic)",
            "DocSection::build(cx, \"Multiple\", multiple)",
            "DocSection::build(cx, \"Disabled\", disabled)",
            "DocSection::build(cx, \"Borders\", borders)",
            "DocSection::build(cx, \"Card\", card)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Basic\", basic)",
            "DocSection::new(\"Multiple\", multiple)",
            "DocSection::new(\"Disabled\", disabled)",
            "DocSection::new(\"Borders\", borders)",
            "DocSection::new(\"Card\", card)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn selected_drawer_snippet_helpers_prefer_into_ui_element_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/drawer/demo.rs",
        &[
            "fn goal_adjust_button(cx: &mut UiCx<'_>, goal: Model<i32>, adjustment: i32, icon: &'static str, a11y_label: &'static str, disabled: bool, test_id: &'static str,) -> shadcn::Button",
            "fn goal_chart<H: UiHost>(cx: &mut ElementContext<'_, H>, goal: i32,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn goal_adjust_button(cx: &mut UiCx<'_>, goal: Model<i32>, adjustment: i32, icon: &'static str, a11y_label: &'static str, disabled: bool, test_id: &'static str,) -> AnyElement",
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
            "fn side_button<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, title: &'static str, direction: shadcn::DrawerDirection, open: Model<bool>, test_id_prefix: &'static str,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn side_button<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>, title: &'static str, direction: shadcn::DrawerDirection, open: Model<bool>, test_id_prefix: &'static str,) -> AnyElement",
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
fn separator_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/separator/demo.rs",
            "src/ui/snippets/separator/list.rs",
            "src/ui/snippets/separator/menu.rs",
            "src/ui/snippets/separator/rtl.rs",
            "src/ui/snippets/separator/usage.rs",
            "src/ui/snippets/separator/vertical.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/separator",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn separator_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/separator.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"Usage\", usage)",
            "DocSection::build(cx, \"Vertical\", vertical)",
            "DocSection::build(cx, \"Menu\", menu)",
            "DocSection::build(cx, \"List\", list)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"Usage\", usage)",
            "DocSection::new(\"Vertical\", vertical)",
            "DocSection::new(\"Menu\", menu)",
            "DocSection::new(\"List\", list)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn typography_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/typography/demo.rs",
            "src/ui/snippets/typography/h1.rs",
            "src/ui/snippets/typography/h2.rs",
            "src/ui/snippets/typography/h3.rs",
            "src/ui/snippets/typography/h4.rs",
            "src/ui/snippets/typography/p.rs",
            "src/ui/snippets/typography/blockquote.rs",
            "src/ui/snippets/typography/table.rs",
            "src/ui/snippets/typography/list.rs",
            "src/ui/snippets/typography/inline_code.rs",
            "src/ui/snippets/typography/lead.rs",
            "src/ui/snippets/typography/large.rs",
            "src/ui/snippets/typography/small.rs",
            "src/ui/snippets/typography/muted.rs",
            "src/ui/snippets/typography/rtl.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/typography",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn typography_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/typography.rs",
        &[
            "DocSection::build(cx, \"Demo\", demo)",
            "DocSection::build(cx, \"h1\", h1)",
            "DocSection::build(cx, \"h2\", h2)",
            "DocSection::build(cx, \"h3\", h3)",
            "DocSection::build(cx, \"h4\", h4)",
            "DocSection::build(cx, \"p\", p)",
            "DocSection::build(cx, \"blockquote\", blockquote)",
            "DocSection::build(cx, \"table\", table)",
            "DocSection::build(cx, \"list\", list)",
            "DocSection::build(cx, \"Inline Code\", inline_code)",
            "DocSection::build(cx, \"Lead\", lead)",
            "DocSection::build(cx, \"Large\", large)",
            "DocSection::build(cx, \"Small\", small)",
            "DocSection::build(cx, \"Muted\", muted)",
            "DocSection::build(cx, \"RTL\", rtl)",
        ],
        &[
            "DocSection::new(\"Demo\", demo)",
            "DocSection::new(\"h1\", h1)",
            "DocSection::new(\"h2\", h2)",
            "DocSection::new(\"h3\", h3)",
            "DocSection::new(\"h4\", h4)",
            "DocSection::new(\"p\", p)",
            "DocSection::new(\"blockquote\", blockquote)",
            "DocSection::new(\"table\", table)",
            "DocSection::new(\"list\", list)",
            "DocSection::new(\"Inline Code\", inline_code)",
            "DocSection::new(\"Lead\", lead)",
            "DocSection::new(\"Large\", large)",
            "DocSection::new(\"Small\", small)",
            "DocSection::new(\"Muted\", muted)",
            "DocSection::new(\"RTL\", rtl)",
        ],
    );
}

#[test]
fn shadcn_extras_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/shadcn_extras/announcement.rs",
            "src/ui/snippets/shadcn_extras/avatar_stack.rs",
            "src/ui/snippets/shadcn_extras/banner.rs",
            "src/ui/snippets/shadcn_extras/kanban.rs",
            "src/ui/snippets/shadcn_extras/marquee.rs",
            "src/ui/snippets/shadcn_extras/rating.rs",
            "src/ui/snippets/shadcn_extras/relative_time.rs",
            "src/ui/snippets/shadcn_extras/tags.rs",
            "src/ui/snippets/shadcn_extras/ticker.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing snippet surface",
    );

    assert_sources_absent(
        "src/ui/snippets/shadcn_extras",
        &["pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement"],
    );
}

#[test]
fn shadcn_extras_page_uses_typed_doc_sections_for_app_facing_snippets() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/shadcn_extras.rs",
        &[
            "DocSection::build(cx, \"Announcement\", announcement)",
            "DocSection::build(cx, \"Banner (dismissible)\", banner)",
            "DocSection::build(cx, \"Tags\", tags)",
            "DocSection::build(cx, \"Marquee (pause on hover)\", marquee)",
            "DocSection::build(cx, \"Kanban (drag & drop)\", kanban)",
            "DocSection::build(cx, \"Ticker\", ticker)",
            "DocSection::build(cx, \"Relative time\", relative_time)",
            "DocSection::build(cx, \"Rating\", rating)",
            "DocSection::build(cx, \"Avatar stack\", avatar_stack)",
        ],
        &[
            "DocSection::new(\"Announcement\", announcement)",
            "DocSection::new(\"Banner (dismissible)\", banner)",
            "DocSection::new(\"Tags\", tags)",
            "DocSection::new(\"Marquee (pause on hover)\", marquee)",
            "DocSection::new(\"Kanban (drag & drop)\", kanban)",
            "DocSection::new(\"Ticker\", ticker)",
            "DocSection::new(\"Relative time\", relative_time)",
            "DocSection::new(\"Rating\", rating)",
            "DocSection::new(\"Avatar stack\", avatar_stack)",
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
                "fn menu_button(cx: &mut UiCx<'_>, selected_model: Model<Arc<str>>, active_value: &Arc<str>, value: &'static str, label: &'static str, icon: &'static str, test_id: Arc<str>,) -> shadcn::SidebarMenuButton",
            ],
            &[
                "fn menu_button(cx: &mut UiCx<'_>, selected_model: Model<Arc<str>>, active_value: &Arc<str>, value: &'static str, label: &'static str, icon: &'static str, test_id: Arc<str>,) -> AnyElement",
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
            "fn render_frame<H: UiHost, E>(image: E) -> impl IntoUiElement<H> + use<H, E>",
            "pub fn render_preview<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> impl IntoUiElement<H> + use<H>",
        ],
        &[
            "fn render_frame<H: UiHost>(cx: &mut ElementContext<'_, H>, image: AnyElement,) -> impl IntoUiElement<H> + use<H>",
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
        let ratio_helper = if relative_path.ends_with("rtl.rs") {
            "fn ratio_example<H: UiHost>(cx: &mut ElementContext<'_, H>, ratio: f32, max_w: Px, test_id: &'static str, figure_test_id: &'static str, content_test_id: &'static str, caption_test_id: &'static str, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> impl IntoUiElement<H> + use<H>"
        } else {
            "fn ratio_example<H: UiHost>(cx: &mut ElementContext<'_, H>, ratio: f32, max_w: Px, test_id: &'static str, content_test_id: &'static str, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> impl IntoUiElement<H> + use<H>"
        };
        let ratio_helper_old = if relative_path.ends_with("rtl.rs") {
            "fn ratio_example<H: UiHost>(cx: &mut ElementContext<'_, H>, ratio: f32, max_w: Px, test_id: &'static str, figure_test_id: &'static str, content_test_id: &'static str, caption_test_id: &'static str, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> AnyElement"
        } else {
            "fn ratio_example<H: UiHost>(cx: &mut ElementContext<'_, H>, ratio: f32, max_w: Px, test_id: &'static str, content_test_id: &'static str, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> AnyElement"
        };

        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                image_helper,
                ratio_helper,
                "pub fn render_preview<H: UiHost>(cx: &mut ElementContext<'_, H>, demo_image: Option<Model<Option<fret_core::ImageId>>>,) -> impl IntoUiElement<H> + use<H>",
            ],
            &[
                image_helper_old,
                ratio_helper_old,
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
        "src/ui/snippets/item/size.rs",
        &[
            "shadcn::item_sized(cx, shadcn::ItemSize::Default, |cx|",
            "shadcn::item_sized(cx, shadcn::ItemSize::Sm, |cx|",
            "shadcn::item_sized(cx, shadcn::ItemSize::Xs, |cx|",
        ],
        &[
            ".size(shadcn::ItemSize::Default)",
            ".size(shadcn::ItemSize::Sm)",
            ".size(shadcn::ItemSize::Xs)",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/item/group.rs",
        &["shadcn::item_group(cx, |cx|"],
        &["shadcn::ItemGroup::new(children)"],
    );

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
fn selected_native_select_snippets_prefer_builder_preserving_helpers() {
    for relative_path in [
        "src/ui/snippets/native_select/demo.rs",
        "src/ui/snippets/native_select/disabled.rs",
        "src/ui/snippets/native_select/invalid.rs",
        "src/ui/snippets/native_select/label.rs",
        "src/ui/snippets/native_select/with_groups.rs",
        "src/ui/snippets/native_select/usage.rs",
        "src/ui/snippets/native_select/rtl.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::native_select("],
            &[
                "shadcn::NativeSelect::new(",
                "shadcn::NativeSelect::new_controllable(",
            ],
        );
    }
}

#[test]
fn selected_pages_prefer_builder_preserving_helper_family_in_copy() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/tabs.rs",
        &[
            "`tabs_uncontrolled(cx, default, |cx| ..)`",
            "`tabs(cx, model, |cx| ..)`",
        ],
        &[
            "Tabs already exposes composable `TabsRoot` / `TabsList` / `TabsTrigger` / `TabsContent`, so the main parity gap here is documentation clarity rather than missing authoring APIs.",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/avatar.rs",
        &["`avatar_sized(...)`"],
        &[
            "`Avatar::new([..])` and `Avatar::children([..])` are already sufficient for composable avatar content; no extra generic children or slot-merge API is needed here.",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/item.rs",
        &["`item_sized(...)`", "`item_group(...)`"],
        &[],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/scroll_area.rs",
        &["`scroll_area(...)`"],
        &[
            "ScrollArea already exposes both a compact builder and a Radix-shaped composable surface, so the main parity gap here is usage clarity rather than missing authoring APIs.",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/native_select.rs",
        &["`native_select(model, open)`", "`new_controllable(...)`"],
        &[
            "`NativeSelect::new(model, open)` and `new_controllable(...)` cover the controlled and default-value/open authoring paths.",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/navigation_menu.rs",
        &[
            "`navigation_menu(cx, model, |cx| ..)`",
            "`NavigationMenu::new(model)`",
        ],
        &[
            "Navigation Menu already exposes a shadcn-friendly builder surface, so the remaining drift is mainly public-surface/docs parity rather than mechanism coverage.",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/resizable.rs",
        &["`resizable_panel_group(...)`"],
        &[],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/slider.rs",
        &["`slider(model)`", "`new_controllable(...)`"],
        &[
            "Slider already exposes the important authoring surface (`new`, `new_controllable`, range/step/orientation/on_value_commit), so the main parity gap here is usage clarity rather than missing composition APIs.",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/radio_group.rs",
        &[
            "`radio_group_uncontrolled(default, items)`",
            "`radio_group(model, items)`",
        ],
        &[
            "`RadioGroup::uncontrolled(default)` and `RadioGroup::new(model)` cover the documented uncontrolled and controlled authoring paths.",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/toggle.rs",
        &[
            "`toggle_uncontrolled(cx, false, |cx| ..)`",
            "`toggle(cx, model, |cx| ..)`",
        ],
        &[
            "`Toggle::uncontrolled(false)` mirrors the upstream `<Toggle />` quick-start path; `variant(...)`, `size(...)`, `disabled(...)`, and `a11y_label(...)` cover the documented control surface.",
        ],
    );

    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/pages/accordion.rs",
        &[
            "`accordion_single_uncontrolled(cx, default, |cx| ..)`",
            "`accordion_multiple_uncontrolled(cx, default, |cx| ..)`",
        ],
        &[
            "The legacy builder-style API remains available as a compact Fret shorthand, but the docs `Usage` section now prefers the composable Radix-shaped surface for parity.",
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
            "fn trending_footer(cx: &mut UiCx<'_>, secondary: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            "fn chart_card(title: &'static str, description: &'static str, canvas: AnyElement, footer_secondary: &'static str, test_id: &'static str,) -> impl IntoUiElement<fret_app::App> + use<>",
            "shadcn::card(",
            "shadcn::card_header(",
            "shadcn::card_content(",
            "shadcn::card_footer(",
        ],
        &[
            "fn trending_footer(cx: &mut UiCx<'_>, secondary: &'static str,) -> AnyElement",
            "fn chart_card(title: &'static str, description: &'static str, canvas: AnyElement, footer_secondary: &'static str, test_id: &'static str,) -> AnyElement",
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
            "shadcn::slider(controlled_values)",
        ],
        &[
            "fn controlled<H: UiHost>(cx: &mut ElementContext<'_, H>, controlled_values: Model<Vec<f32>>,) -> AnyElement",
            "shadcn::Slider::new(controlled_values)",
        ],
    );

    for relative_path in [
        "src/ui/snippets/slider/usage.rs",
        "src/ui/snippets/slider/label.rs",
        "src/ui/snippets/field/slider.rs",
        "src/ui/snippets/progress/controlled.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &["shadcn::slider("],
            &["shadcn::Slider::new("],
        );
    }
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
fn material3_controls_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/material3/badge.rs",
            "src/ui/snippets/material3/button.rs",
            "src/ui/snippets/material3/checkbox.rs",
            "src/ui/snippets/material3/icon_button.rs",
            "src/ui/snippets/material3/radio.rs",
            "src/ui/snippets/material3/segmented_button.rs",
            "src/ui/snippets/material3/slider.rs",
            "src/ui/snippets/material3/switch.rs",
            "src/ui/snippets/material3/touch_targets.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing Material 3 controls snippet surface",
    );

    for relative_path in [
        "src/ui/snippets/material3/badge.rs",
        "src/ui/snippets/material3/button.rs",
        "src/ui/snippets/material3/icon_button.rs",
        "src/ui/snippets/material3/touch_targets.rs",
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains("ElementContext<'_,H>"),
            "{} reintroduced legacy host-bound helper parameters",
            path.display()
        );
    }
}

#[test]
fn material3_inputs_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/material3/autocomplete.rs",
            "src/ui/snippets/material3/date_picker.rs",
            "src/ui/snippets/material3/select.rs",
            "src/ui/snippets/material3/text_field.rs",
            "src/ui/snippets/material3/time_picker.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing Material 3 inputs snippet surface",
    );

    let select_path = manifest_path("src/ui/snippets/material3/select.rs");
    let select_source = read_path(&select_path);
    let select_normalized = select_source.split_whitespace().collect::<String>();
    assert!(
        !select_normalized.contains("ElementContext<'_,H>"),
        "{} reintroduced legacy host-bound helper parameters",
        select_path.display()
    );
}

#[test]
fn material3_navigation_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/material3/list.rs",
            "src/ui/snippets/material3/modal_navigation_drawer.rs",
            "src/ui/snippets/material3/navigation_bar.rs",
            "src/ui/snippets/material3/navigation_drawer.rs",
            "src/ui/snippets/material3/navigation_rail.rs",
            "src/ui/snippets/material3/tabs.rs",
            "src/ui/snippets/material3/top_app_bar.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing Material 3 navigation snippet surface",
    );

    for relative_path in [
        "src/ui/snippets/material3/list.rs",
        "src/ui/snippets/material3/top_app_bar.rs",
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains("ElementContext<'_,H>"),
            "{} reintroduced legacy host-bound helper parameters",
            path.display()
        );
    }
}

#[test]
fn material3_overlay_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/material3/bottom_sheet.rs",
            "src/ui/snippets/material3/dialog.rs",
            "src/ui/snippets/material3/menu.rs",
            "src/ui/snippets/material3/snackbar.rs",
            "src/ui/snippets/material3/tooltip.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(",
            "cx: &mut UiCx<'_>",
            "-> impl UiChild + use<>",
        ],
        "app-facing Material 3 overlay snippet surface",
    );

    for relative_path in [
        "src/ui/snippets/material3/bottom_sheet.rs",
        "src/ui/snippets/material3/dialog.rs",
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains("ElementContext<'_,H>"),
            "{} reintroduced legacy host-bound helper parameters",
            path.display()
        );
    }
}

#[test]
fn material3_composite_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/material3/gallery.rs",
            "src/ui/snippets/material3/state_matrix.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(",
            "cx: &mut UiCx<'_>",
            "-> impl UiChild + use<>",
        ],
        "app-facing Material 3 composite snippet surface",
    );

    for relative_path in [
        "src/ui/snippets/material3/gallery.rs",
        "src/ui/snippets/material3/state_matrix.rs",
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains("ElementContext<'_,H>"),
            "{} reintroduced legacy host-bound helper parameters",
            path.display()
        );
    }
}

#[test]
fn ai_curated_snippets_prefer_ui_cx_on_the_default_app_surface() {
    assert_curated_default_app_paths(
        &[
            "src/ui/snippets/ai/agent_demo.rs",
            "src/ui/snippets/ai/artifact_demo.rs",
            "src/ui/snippets/ai/artifact_code_display.rs",
            "src/ui/snippets/ai/attachments_empty.rs",
            "src/ui/snippets/ai/attachments_grid.rs",
            "src/ui/snippets/ai/attachments_inline.rs",
            "src/ui/snippets/ai/attachments_list.rs",
            "src/ui/snippets/ai/attachments_usage.rs",
            "src/ui/snippets/ai/audio_player_demo.rs",
            "src/ui/snippets/ai/canvas_world_layer_spike.rs",
            "src/ui/snippets/ai/chat_demo.rs",
            "src/ui/snippets/ai/chain_of_thought_composable.rs",
            "src/ui/snippets/ai/chain_of_thought_demo.rs",
            "src/ui/snippets/ai/checkpoint_demo.rs",
            "src/ui/snippets/ai/code_block_demo.rs",
            "src/ui/snippets/ai/commit_custom_children.rs",
            "src/ui/snippets/ai/commit_demo.rs",
            "src/ui/snippets/ai/commit_large_demo.rs",
            "src/ui/snippets/ai/confirmation_accepted.rs",
            "src/ui/snippets/ai/confirmation_demo.rs",
            "src/ui/snippets/ai/confirmation_rejected.rs",
            "src/ui/snippets/ai/confirmation_request.rs",
            "src/ui/snippets/ai/conversation_demo.rs",
            "src/ui/snippets/ai/context_default.rs",
            "src/ui/snippets/ai/context_demo.rs",
            "src/ui/snippets/ai/environment_variables_demo.rs",
            "src/ui/snippets/ai/file_tree_basic.rs",
            "src/ui/snippets/ai/file_tree_demo.rs",
            "src/ui/snippets/ai/file_tree_expanded.rs",
            "src/ui/snippets/ai/file_tree_large.rs",
            "src/ui/snippets/ai/image_demo.rs",
            "src/ui/snippets/ai/inline_citation_demo.rs",
            "src/ui/snippets/ai/message_branch_demo.rs",
            "src/ui/snippets/ai/message_demo.rs",
            "src/ui/snippets/ai/message_usage.rs",
            "src/ui/snippets/ai/mic_selector_demo.rs",
            "src/ui/snippets/ai/model_selector_demo.rs",
            "src/ui/snippets/ai/open_in_chat_demo.rs",
            "src/ui/snippets/ai/package_info_demo.rs",
            "src/ui/snippets/ai/tool_demo.rs",
            "src/ui/snippets/ai/plan_demo.rs",
            "src/ui/snippets/ai/persona_basic.rs",
            "src/ui/snippets/ai/persona_custom_styling.rs",
            "src/ui/snippets/ai/persona_custom_visual.rs",
            "src/ui/snippets/ai/persona_demo.rs",
            "src/ui/snippets/ai/persona_state_management.rs",
            "src/ui/snippets/ai/persona_variants.rs",
            "src/ui/snippets/ai/prompt_input_action_menu_demo.rs",
            "src/ui/snippets/ai/prompt_input_docs_demo.rs",
            "src/ui/snippets/ai/prompt_input_provider_demo.rs",
            "src/ui/snippets/ai/prompt_input_referenced_sources_demo.rs",
            "src/ui/snippets/ai/queue_demo.rs",
            "src/ui/snippets/ai/reasoning_demo.rs",
            "src/ui/snippets/ai/sandbox_demo.rs",
            "src/ui/snippets/ai/schema_display_demo.rs",
            "src/ui/snippets/ai/shimmer_demo.rs",
            "src/ui/snippets/ai/shimmer_duration_demo.rs",
            "src/ui/snippets/ai/shimmer_elements_demo.rs",
            "src/ui/snippets/ai/snippet_demo.rs",
            "src/ui/snippets/ai/snippet_plain.rs",
            "src/ui/snippets/ai/sources_demo.rs",
            "src/ui/snippets/ai/speech_input_demo.rs",
            "src/ui/snippets/ai/stack_trace_collapsed.rs",
            "src/ui/snippets/ai/stack_trace_demo.rs",
            "src/ui/snippets/ai/stack_trace_large_demo.rs",
            "src/ui/snippets/ai/stack_trace_no_internal.rs",
            "src/ui/snippets/ai/suggestions_demo.rs",
            "src/ui/snippets/ai/task_demo.rs",
            "src/ui/snippets/ai/terminal_demo.rs",
            "src/ui/snippets/ai/test_results_basic.rs",
            "src/ui/snippets/ai/test_results_demo.rs",
            "src/ui/snippets/ai/test_results_errors.rs",
            "src/ui/snippets/ai/test_results_large_demo.rs",
            "src/ui/snippets/ai/test_results_suites.rs",
            "src/ui/snippets/ai/transcript_torture.rs",
            "src/ui/snippets/ai/transcription_demo.rs",
            "src/ui/snippets/ai/voice_selector_demo.rs",
            "src/ui/snippets/ai/web_preview_demo.rs",
            "src/ui/snippets/ai/workflow_canvas_demo.rs",
            "src/ui/snippets/ai/workflow_chrome_demo.rs",
            "src/ui/snippets/ai/workflow_connection_demo.rs",
            "src/ui/snippets/ai/workflow_controls_demo.rs",
            "src/ui/snippets/ai/workflow_edge_demo.rs",
            "src/ui/snippets/ai/workflow_node_demo.rs",
            "src/ui/snippets/ai/workflow_node_graph_demo.rs",
            "src/ui/snippets/ai/workflow_panel_demo.rs",
            "src/ui/snippets/ai/workflow_toolbar_demo.rs",
        ],
        &[
            "use fret::{UiChild, UiCx};",
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        ],
        "app-facing AI leaf snippet surface",
    );

    for relative_path in [
        "src/ui/snippets/ai/agent_demo.rs",
        "src/ui/snippets/ai/artifact_demo.rs",
        "src/ui/snippets/ai/artifact_code_display.rs",
        "src/ui/snippets/ai/attachments_empty.rs",
        "src/ui/snippets/ai/attachments_grid.rs",
        "src/ui/snippets/ai/attachments_inline.rs",
        "src/ui/snippets/ai/attachments_list.rs",
        "src/ui/snippets/ai/attachments_usage.rs",
        "src/ui/snippets/ai/audio_player_demo.rs",
        "src/ui/snippets/ai/chat_demo.rs",
        "src/ui/snippets/ai/chain_of_thought_composable.rs",
        "src/ui/snippets/ai/chain_of_thought_demo.rs",
        "src/ui/snippets/ai/checkpoint_demo.rs",
        "src/ui/snippets/ai/code_block_demo.rs",
        "src/ui/snippets/ai/commit_custom_children.rs",
        "src/ui/snippets/ai/commit_demo.rs",
        "src/ui/snippets/ai/commit_large_demo.rs",
        "src/ui/snippets/ai/confirmation_accepted.rs",
        "src/ui/snippets/ai/confirmation_demo.rs",
        "src/ui/snippets/ai/confirmation_rejected.rs",
        "src/ui/snippets/ai/confirmation_request.rs",
        "src/ui/snippets/ai/context_default.rs",
        "src/ui/snippets/ai/context_demo.rs",
        "src/ui/snippets/ai/file_tree_basic.rs",
        "src/ui/snippets/ai/file_tree_demo.rs",
        "src/ui/snippets/ai/file_tree_expanded.rs",
        "src/ui/snippets/ai/file_tree_large.rs",
        "src/ui/snippets/ai/inline_citation_demo.rs",
        "src/ui/snippets/ai/message_demo.rs",
        "src/ui/snippets/ai/mic_selector_demo.rs",
        "src/ui/snippets/ai/model_selector_demo.rs",
        "src/ui/snippets/ai/open_in_chat_demo.rs",
        "src/ui/snippets/ai/package_info_demo.rs",
        "src/ui/snippets/ai/tool_demo.rs",
        "src/ui/snippets/ai/plan_demo.rs",
        "src/ui/snippets/ai/persona_basic.rs",
        "src/ui/snippets/ai/persona_custom_styling.rs",
        "src/ui/snippets/ai/persona_custom_visual.rs",
        "src/ui/snippets/ai/persona_demo.rs",
        "src/ui/snippets/ai/persona_state_management.rs",
        "src/ui/snippets/ai/persona_variants.rs",
        "src/ui/snippets/ai/prompt_input_action_menu_demo.rs",
        "src/ui/snippets/ai/prompt_input_docs_demo.rs",
        "src/ui/snippets/ai/prompt_input_provider_demo.rs",
        "src/ui/snippets/ai/prompt_input_referenced_sources_demo.rs",
        "src/ui/snippets/ai/reasoning_demo.rs",
        "src/ui/snippets/ai/schema_display_demo.rs",
        "src/ui/snippets/ai/shimmer_demo.rs",
        "src/ui/snippets/ai/shimmer_duration_demo.rs",
        "src/ui/snippets/ai/shimmer_elements_demo.rs",
        "src/ui/snippets/ai/snippet_demo.rs",
        "src/ui/snippets/ai/snippet_plain.rs",
        "src/ui/snippets/ai/sources_demo.rs",
        "src/ui/snippets/ai/stack_trace_collapsed.rs",
        "src/ui/snippets/ai/stack_trace_demo.rs",
        "src/ui/snippets/ai/stack_trace_large_demo.rs",
        "src/ui/snippets/ai/stack_trace_no_internal.rs",
        "src/ui/snippets/ai/task_demo.rs",
        "src/ui/snippets/ai/terminal_demo.rs",
        "src/ui/snippets/ai/test_results_basic.rs",
        "src/ui/snippets/ai/test_results_demo.rs",
        "src/ui/snippets/ai/test_results_errors.rs",
        "src/ui/snippets/ai/test_results_large_demo.rs",
        "src/ui/snippets/ai/test_results_suites.rs",
        "src/ui/snippets/ai/voice_selector_demo.rs",
        "src/ui/snippets/ai/web_preview_demo.rs",
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains("ElementContext<'_,H>"),
            "{} reintroduced legacy host-bound helper parameters",
            path.display()
        );
    }
}

#[test]
fn selected_ai_snippets_follow_selects_direct_recipe_root_lane() {
    for relative_path in [
        "src/ui/snippets/ai/code_block_demo.rs",
        "src/ui/snippets/ai/prompt_input_docs_demo.rs",
    ] {
        assert_selected_generic_helpers_prefer_into_ui_element(
            relative_path,
            &[
                "shadcn::Select::new(",
                ".trigger(",
                ".value(",
                ".content(",
                ".entries(",
                ".into_element(cx)",
            ],
            &[".into_element_parts("],
        );
    }
}

#[test]
fn selected_ai_snippet_helpers_prefer_typed_children_over_anyelement() {
    assert_selected_generic_helpers_prefer_into_ui_element(
        "src/ui/snippets/ai/shimmer_elements_demo.rs",
        &[
            "fn item<B>(label: &'static str, el: B) -> impl UiChild + use<B> where B: IntoUiElement<fret_app::App>",
        ],
        &["let item = |cx: &mut UiCx<'_>, label: &'static str, el: AnyElement| {"],
    );
}

#[test]
fn selected_ai_snippets_do_not_reintroduce_anyelement_type_annotations() {
    for relative_path in [
        "src/ui/snippets/ai/test_results_demo.rs",
        "src/ui/snippets/ai/test_results_errors.rs",
        "src/ui/snippets/ai/test_results_suites.rs",
        "src/ui/snippets/ai/conversation_demo.rs",
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            !normalized.contains("usefret_ui::element::AnyElement;"),
            "{} reintroduced an unnecessary AnyElement import",
            path.display()
        );
        assert!(
            !normalized.contains("Vec<AnyElement>"),
            "{} reintroduced an unnecessary Vec<AnyElement> annotation",
            path.display()
        );
    }
}

#[test]
fn material3_overlay_snippets_prefer_uncontrolled_copyable_roots() {
    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/menu.rs",
        &[
            "pub fn render( cx: &mut UiCx<'_>, last_action: Model<Arc<str>>, ) -> impl UiChild + use<> {",
            "material3::DropdownMenu::uncontrolled(cx)",
            "let open = dropdown.open_model();",
        ],
        &[
            "pub fn render<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Model<bool>, last_action: Model<Arc<str>>, ) -> AnyElement {",
            "let open = cx.local_model_keyed(\"open\", || false);",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/dialog.rs",
        &[
            "pub fn render( cx: &mut UiCx<'_>, last_action: Model<Arc<str>>, ) -> impl UiChild + use<> {",
            "let default_dialog = material3::Dialog::uncontrolled(cx);",
            "let open = default_dialog.open_model();",
            "material3::Select::uncontrolled(cx)",
        ],
        &[
            "pub fn render<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Model<bool>, last_action: Model<Arc<str>>, ) -> AnyElement {",
            "let open = cx.local_model_keyed(\"open\", || false);",
            "let selected = cx.local_model_keyed(\"selected\", || None::<Arc<str>>);",
            "let build_dialog = |cx: &mut UiCx<'_>, mut dialog: material3::Dialog, style: Option<material3::DialogStyle>, id_prefix: &'static str, open_action: OnActivate, close_action: OnActivate, confirm_action: OnActivate| -> AnyElement {",
            "let build_container = |cx: &mut UiCx<'_>, dialog: AnyElement| -> AnyElement {",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/bottom_sheet.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "material3::ModalBottomSheet::uncontrolled(cx)",
            "let open = sheet.open_model();",
        ],
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, open: Model<bool>) -> AnyElement {",
            "let open = cx.local_model_keyed(\"open\", || false);",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/state_matrix.rs",
        &[
            "fn render_search_view(cx: &mut UiCx<'_>) -> Vec<AnyElement> {",
            "material3::SearchView::uncontrolled(cx)",
        ],
        &[
            "let open = cx.local_model_keyed(\"open\", || false);",
            "let query = cx.local_model_keyed(\"query\", String::new);",
            "material3::SearchView::new(open, query)",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/modal_navigation_drawer.rs",
        &[
            "let modal = material3::ModalNavigationDrawer::uncontrolled(cx);",
            "let open = modal.open_model();",
        ],
        &[
            "pub fn render<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Model<bool>, value: Model<Arc<str>>, ) -> AnyElement {",
            "material3::ModalNavigationDrawer::new(open.clone())",
        ],
    );
}

#[test]
fn material3_autocomplete_snippet_prefers_uncontrolled_query_and_dialog_roots() {
    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/autocomplete.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let outlined_autocomplete = material3::Autocomplete::uncontrolled(cx);",
            "let value = outlined_autocomplete.query_model();",
            "let dialog = material3::Dialog::uncontrolled(cx);",
            "let dialog_open = dialog.open_model();",
            "let disabled_toggle = material3::Switch::uncontrolled(cx, false);",
            "let disabled = disabled_toggle.selected_model();",
            "let error_toggle = material3::Switch::uncontrolled(cx, false);",
            "let error = error_toggle.selected_model();",
            "let exposed_dropdown = material3::ExposedDropdown::new_controllable( cx, None, Some(Arc::<str>::from(\"beta\")), None, String::new(), );",
            "let exposed_selected_value = exposed_dropdown.selected_value_model();",
            "let exposed_query = exposed_dropdown.query_model();",
        ],
        &[
            "pub fn render<H: UiHost>( cx: &mut ElementContext<'_, H>, disabled: Model<bool>, error: Model<bool>, ) -> AnyElement {",
            "pub fn render<H: UiHost>( cx: &mut ElementContext<'_, H>, value: Model<String>, disabled: Model<bool>, error: Model<bool>, dialog_open: Model<bool>, ) -> AnyElement {",
            "material3::Dialog::new(dialog_open.clone())",
            "let exposed_selected_value = cx.local_model_keyed(\"exposed_selected_value\", || Some(Arc::<str>::from(\"beta\")));",
            "let exposed_query = cx.local_model_keyed(\"exposed_query\", String::new);",
        ],
    );
}

#[test]
fn material3_select_snippet_prefers_uncontrolled_value_roots() {
    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/select.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let default_select = material3::Select::uncontrolled(cx);",
            "let selected = default_select.value_model();",
            "let overridden = material3::Select::new(selected.clone())",
            "let unclamped = material3::Select::uncontrolled(cx)",
            "let typeahead_select = material3::Select::uncontrolled(cx)",
            "let rich_select = material3::Select::uncontrolled(cx)",
            "let transformed_select = material3::Select::uncontrolled(cx)",
        ],
        &[
            "let selected = cx.local_model_keyed(\"selected\", || None::<Arc<str>>);",
            "let selected_unclamped = cx.local_model_keyed(\"selected_unclamped\", || None::<Arc<str>>);",
            "let selected_typeahead = cx.local_model_keyed(\"selected_typeahead\", || None::<Arc<str>>);",
            "let selected_rich = cx.local_model_keyed(\"selected_rich\", || None::<Arc<str>>);",
            "let selected_transformed = cx.local_model_keyed(\"selected_transformed\", || None::<Arc<str>>);",
        ],
    );
}

#[test]
fn material3_date_and_time_picker_snippets_prefer_uncontrolled_dialog_roots() {
    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/date_picker.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let dialog = material3::DatePickerDialog::uncontrolled(cx);",
            "let open = dialog.open_model();",
            "let month = dialog.month_model();",
            "let selected = dialog.selected_model();",
        ],
        &[
            "pub fn render<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Model<bool>, month: Model<CalendarMonth>, selected: Model<Option<time::Date>>, ) -> AnyElement {",
            "material3::DatePickerDialog::new(open, month.clone(), selected.clone())",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/time_picker.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let dialog = material3::TimePickerDialog::uncontrolled(cx);",
            "let open = dialog.open_model();",
            "let selected = dialog.selected_model();",
        ],
        &[
            "pub fn render<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Model<bool>, selected: Model<time::Time>, ) -> AnyElement {",
            "material3::TimePickerDialog::new(open, selected.clone())",
        ],
    );
}

#[test]
fn material3_selection_and_field_snippets_prefer_uncontrolled_value_roots() {
    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/checkbox.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let checkbox = material3::Checkbox::uncontrolled(cx, false);",
            "let checked = checkbox.checked_model();",
            "let tristate = material3::Checkbox::uncontrolled_optional(cx, None);",
            "let tristate_model = tristate.optional_checked_model();",
        ],
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, checked: Model<bool>) -> AnyElement {",
            "let tristate = cx.local_model_keyed(\"tristate\", || None::<bool>);",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/switch.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let default_switch = material3::Switch::uncontrolled(cx, false);",
            "let selected = default_switch.selected_model();",
            "let icons_both_root = material3::Switch::uncontrolled(cx, false);",
            "let icons_selected_only_root = material3::Switch::uncontrolled(cx, false);",
        ],
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, selected: Model<bool>) -> AnyElement {",
            "let icons_both = cx.local_model_keyed(\"icons_both\", || false);",
            "let icons_selected_only = cx.local_model_keyed(\"icons_selected_only\", || false);",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/radio.rs",
        &[
            "let group = material3::RadioGroup::uncontrolled(cx, None::<Arc<str>>);",
            "let group_value = group.value_model();",
            "let standalone = material3::Radio::uncontrolled(cx, false);",
            "let standalone_selected = standalone.selected_model();",
        ],
        &[
            "pub fn render<H: UiHost>( cx: &mut ElementContext<'_, H>, group_value: Model<Option<Arc<str>>>, ) -> AnyElement {",
            "let standalone_selected = cx.local_model_keyed(\"standalone_selected\", || false);",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/slider.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let slider = material3::Slider::uncontrolled(cx, 0.3);",
            "let value = slider.value_model();",
        ],
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, value: Model<f32>) -> AnyElement {",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/tabs.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let tabs = material3::Tabs::uncontrolled(cx, \"overview\");",
            "let value = tabs.value_model();",
        ],
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, value: Model<Arc<str>>) -> AnyElement {",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/list.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let list = material3::List::uncontrolled(cx, \"alpha\");",
            "let value = list.value_model();",
        ],
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, value: Model<Arc<str>>) -> AnyElement {",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/navigation_bar.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let bar = material3::NavigationBar::uncontrolled(cx, \"search\");",
            "let value = bar.value_model();",
        ],
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, value: Model<Arc<str>>) -> AnyElement {",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/navigation_rail.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let rail = material3::NavigationRail::uncontrolled(cx, \"search\");",
            "let value = rail.value_model();",
        ],
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, value: Model<Arc<str>>) -> AnyElement {",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/navigation_drawer.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let drawer = material3::NavigationDrawer::uncontrolled(cx, \"search\");",
            "let value = drawer.value_model();",
        ],
        &[
            "pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, value: Model<Arc<str>>) -> AnyElement {",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/text_field.rs",
        &[
            "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {",
            "let demo_field = material3::TextField::uncontrolled(cx);",
            "let value = demo_field.value_model();",
            "let disabled_toggle = material3::Switch::uncontrolled(cx, false);",
            "let disabled = disabled_toggle.selected_model();",
            "let error_toggle = material3::Switch::uncontrolled(cx, false);",
            "let error = error_toggle.selected_model();",
        ],
        &[
            "pub fn render<H: UiHost>( cx: &mut ElementContext<'_, H>, disabled: Model<bool>, error: Model<bool>, ) -> AnyElement {",
            "pub fn render<H: UiHost>( cx: &mut ElementContext<'_, H>, value: Model<String>, disabled: Model<bool>, error: Model<bool>, ) -> AnyElement {",
        ],
    );
}

#[test]
fn material3_composite_snippets_prefer_local_uncontrolled_value_roots() {
    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/touch_targets.rs",
        &[
            "let checkbox_root = material3::Checkbox::uncontrolled(cx, false);",
            "let material3_checkbox = checkbox_root.checked_model();",
            "let switch_root = material3::Switch::uncontrolled(cx, false);",
            "let material3_switch = switch_root.selected_model();",
            "let radio_group_root = material3::RadioGroup::uncontrolled(cx, None::<Arc<str>>);",
            "let material3_radio_value = radio_group_root.value_model();",
            "let tabs_root = material3::Tabs::uncontrolled(cx, \"overview\");",
            "let material3_tabs_value = tabs_root.value_model();",
        ],
        &[
            "material3_checkbox: Model<bool>,",
            "material3_switch: Model<bool>,",
            "material3_radio_value: Model<Option<Arc<str>>>,",
            "material3_tabs_value: Model<Arc<str>>,",
            "let target_overlay = |cx: &mut UiCx<'_>, label: &'static str, chrome: Option<Size>, child: AnyElement| {",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/gallery.rs",
        &[
            "let checkbox_root = material3::Checkbox::uncontrolled(cx, false);",
            "let material3_checkbox = checkbox_root.checked_model();",
            "let switch_root = material3::Switch::uncontrolled(cx, false);",
            "let material3_switch = switch_root.selected_model();",
            "let radio_group_root = material3::RadioGroup::uncontrolled(cx, None::<Arc<str>>);",
            "let material3_radio_value = radio_group_root.value_model();",
            "let tabs_root = material3::Tabs::uncontrolled(cx, \"overview\");",
            "let list_root = material3::List::uncontrolled(cx, \"alpha\");",
            "let navigation_bar_root = material3::NavigationBar::uncontrolled(cx, \"search\");",
            "let text_field_root = material3::TextField::uncontrolled(cx);",
            "let text_field_disabled_root = material3::Switch::uncontrolled(cx, false);",
            "let text_field_error_root = material3::Switch::uncontrolled(cx, false);",
        ],
        &[
            "material3_checkbox: Model<bool>,",
            "material3_switch: Model<bool>,",
            "material3_radio_value: Model<Option<Arc<str>>>,",
            "material3_text_field_disabled: Model<bool>,",
            "material3_text_field_error: Model<bool>,",
            "material3_tabs_value: Model<Arc<str>>,",
            "material3_list_value: Model<Arc<str>>,",
            "material3_navigation_bar_value: Model<Arc<str>>,",
            "material3_text_field_value: Model<String>,",
        ],
    );

    assert_material3_snippet_prefers_copyable_root(
        "src/ui/snippets/material3/state_matrix.rs",
        &[
            "let checkbox_root = material3::Checkbox::uncontrolled(cx, false);",
            "let material3_checkbox = checkbox_root.checked_model();",
            "let switch_root = material3::Switch::uncontrolled(cx, false);",
            "let material3_switch = switch_root.selected_model();",
            "let radio_group_root = material3::RadioGroup::uncontrolled(cx, None::<Arc<str>>);",
            "let material3_radio_value = radio_group_root.value_model();",
            "let tabs_root = material3::Tabs::uncontrolled(cx, \"overview\");",
            "let navigation_bar_root = material3::NavigationBar::uncontrolled(cx, \"search\");",
            "let text_field_root = material3::TextField::uncontrolled(cx);",
            "let text_field_disabled_root = material3::Switch::uncontrolled(cx, false);",
            "let text_field_error_root = material3::Switch::uncontrolled(cx, false);",
            "let dropdown_root = material3::DropdownMenu::uncontrolled(cx).a11y_label(\"Material 3 Menu\");",
            "let open = dropdown_root.open_model();",
        ],
        &[
            "material3_checkbox: Model<bool>,",
            "material3_switch: Model<bool>,",
            "material3_radio_value: Model<Option<Arc<str>>>,",
            "material3_text_field_disabled: Model<bool>,",
            "material3_text_field_error: Model<bool>,",
            "material3_menu_open: Model<bool>,",
            "material3_tabs_value: Model<Arc<str>>,",
            "material3_navigation_bar_value: Model<Arc<str>>,",
            "material3_text_field_value: Model<String>,",
        ],
    );
}

#[test]
fn material3_pages_do_not_route_demo_only_runtime_models() {
    for (relative_path, required_markers, forbidden_markers) in [
        (
            "src/ui/pages/material3/controls.rs",
            vec![
                "pub(in crate::ui) fn preview_material3_touch_targets(cx: &mut UiCx<'_>) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_checkbox(cx: &mut UiCx<'_>) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_switch(cx: &mut UiCx<'_>) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_slider(cx: &mut UiCx<'_>) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_radio(cx: &mut UiCx<'_>) -> Vec<AnyElement> {",
            ],
            vec![
                "pub(in crate::ui) fn preview_material3_touch_targets( cx: &mut UiCx<'_>, material3_checkbox: Model<bool>, material3_switch: Model<bool>, material3_radio_value: Model<Option<Arc<str>>>, ) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_checkbox( cx: &mut UiCx<'_>, checked: Model<bool>, ) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_switch( cx: &mut UiCx<'_>, selected: Model<bool>, ) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_slider( cx: &mut UiCx<'_>, value: Model<f32>, ) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_radio( cx: &mut UiCx<'_>, group_value: Model<Option<Arc<str>>>, ) -> Vec<AnyElement> {",
            ],
        ),
        (
            "src/ui/pages/material3/gallery.rs",
            vec![
                "pub(in crate::ui) fn preview_material3_gallery( cx: &mut UiCx<'_>, last_action: Model<Arc<str>>, ) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_state_matrix( cx: &mut UiCx<'_>, last_action: Model<Arc<str>>, ) -> Vec<AnyElement> {",
            ],
            vec![
                "material3_checkbox: Model<bool>,",
                "material3_switch: Model<bool>,",
                "material3_radio_value: Model<Option<Arc<str>>>,",
                "material3_text_field_disabled: Model<bool>,",
                "material3_text_field_error: Model<bool>,",
                "material3_menu_open: Model<bool>,",
            ],
        ),
        (
            "src/ui/pages/material3/inputs.rs",
            vec![
                "pub(in crate::ui) fn preview_material3_autocomplete(cx: &mut UiCx<'_>) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_text_field(cx: &mut UiCx<'_>) -> Vec<AnyElement> {",
            ],
            vec![
                "pub(in crate::ui) fn preview_material3_autocomplete( cx: &mut UiCx<'_>, disabled: Model<bool>, error: Model<bool>, ) -> Vec<AnyElement> {",
                "pub(in crate::ui) fn preview_material3_text_field( cx: &mut UiCx<'_>, disabled: Model<bool>, error: Model<bool>, ) -> Vec<AnyElement> {",
            ],
        ),
        (
            "src/ui/content.rs",
            vec![
                "pages::material3::preview_material3_slider(cx)",
                "pages::material3::preview_material3_gallery(cx, last_action.clone())",
                "pages::material3::preview_material3_state_matrix(cx, last_action.clone())",
                "pages::material3::preview_material3_touch_targets(cx)",
                "pages::material3::preview_material3_checkbox(cx)",
                "pages::material3::preview_material3_switch(cx)",
                "pages::material3::preview_material3_radio(cx)",
                "pages::material3::preview_material3_autocomplete(cx)",
                "pages::material3::preview_material3_text_field(cx)",
            ],
            vec![
                "let material3_slider_value = models.material3_slider_value.clone();",
                "let material3_text_field_disabled = models.material3_text_field_disabled.clone();",
                "let material3_text_field_error = models.material3_text_field_error.clone();",
                "let material3_autocomplete_disabled = models.material3_autocomplete_disabled.clone();",
                "let material3_autocomplete_error = models.material3_autocomplete_error.clone();",
                "let material3_menu_open = models.material3_menu_open.clone();",
                "let material3_checkbox = models.material3_checkbox.clone();",
                "let material3_switch = models.material3_switch.clone();",
                "let material3_radio_value = models.material3_radio_value.clone();",
                "pages::material3::preview_material3_slider(cx, material3_slider_value)",
                "pages::material3::preview_material3_gallery( cx, material3_text_field_disabled, material3_text_field_error, last_action.clone(), )",
                "pages::material3::preview_material3_state_matrix( cx, material3_text_field_disabled, material3_text_field_error, material3_menu_open, last_action.clone(), )",
                "pages::material3::preview_material3_autocomplete( cx, material3_autocomplete_disabled, material3_autocomplete_error, )",
                "pages::material3::preview_material3_text_field( cx, material3_text_field_disabled, material3_text_field_error, )",
                "pages::material3::preview_material3_checkbox(cx, material3_checkbox)",
                "pages::material3::preview_material3_switch(cx, material3_switch)",
                "pages::material3::preview_material3_radio(cx, material3_radio_value)",
            ],
        ),
        (
            "src/ui/models.rs",
            vec![],
            vec![
                "pub(crate) material3_slider_value: Model<f32>,",
                "pub(crate) material3_text_field_disabled: Model<bool>,",
                "pub(crate) material3_text_field_error: Model<bool>,",
                "pub(crate) material3_autocomplete_disabled: Model<bool>,",
                "pub(crate) material3_autocomplete_error: Model<bool>,",
                "pub(crate) material3_menu_open: Model<bool>,",
            ],
        ),
        (
            "src/driver/runtime_driver.rs",
            vec![],
            vec![
                "material3_slider_value: Model<f32>,",
                "material3_text_field_disabled: Model<bool>,",
                "material3_text_field_error: Model<bool>,",
                "material3_autocomplete_disabled: Model<bool>,",
                "material3_autocomplete_error: Model<bool>,",
                "material3_menu_open: Model<bool>,",
                "material3_slider_value: self.material3_slider_value.clone(),",
                "material3_text_field_disabled: self.material3_text_field_disabled.clone(),",
                "material3_text_field_error: self.material3_text_field_error.clone(),",
                "material3_autocomplete_disabled: self.material3_autocomplete_disabled.clone(),",
                "material3_autocomplete_error: self.material3_autocomplete_error.clone(),",
                "material3_menu_open: self.material3_menu_open.clone(),",
            ],
        ),
        (
            "src/driver/window_bootstrap.rs",
            vec![],
            vec![
                "let material3_slider_value = app.models_mut().insert(0.3f32);",
                "let material3_text_field_disabled = app.models_mut().insert(false);",
                "let material3_text_field_error = app.models_mut().insert(false);",
                "let material3_autocomplete_disabled = app.models_mut().insert(false);",
                "let material3_autocomplete_error = app.models_mut().insert(false);",
                "let material3_menu_open = app.models_mut().insert(false);",
            ],
        ),
    ] {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        let normalized = source.split_whitespace().collect::<String>();

        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                normalized.contains(&marker),
                "{} is missing Material 3 choice-control authoring marker `{}`",
                path.display(),
                marker
            );
        }

        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "{} reintroduced demo-only Material 3 choice-control state marker `{}`",
                path.display(),
                marker
            );
        }
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
fn gallery_doc_layout_app_helpers_prefer_ui_child_over_anyelement() {
    assert_selected_page_helpers_prefer_ui_child(
        "src/ui/doc_layout.rs",
        &[
            "fn render_doc_page(cx: &mut UiCx<'_>, intro: Option<&'static str>, sections: Vec<DocSection>,) -> impl UiChild + use<>",
            "fn wrap_preview_page(cx: &mut UiCx<'_>, intro: Option<&'static str>, section_title: &'static str, elements: Vec<AnyElement>,) -> impl UiChild + use<>",
            "fn wrap_row<F>(cx: &mut UiCx<'_>, theme: &Theme, gap: Space, align: fret_ui::element::CrossAlign, children: F,) -> impl UiChild + use<F> where F: FnOnce(&mut UiCx<'_>) -> Vec<AnyElement>",
            "fn wrap_controls_row<F>(cx: &mut UiCx<'_>, theme: &Theme, gap: Space, children: F,) -> impl UiChild + use<F> where F: FnOnce(&mut UiCx<'_>) -> Vec<AnyElement>",
            "fn text_table<const N: usize, I>(cx: &mut UiCx<'_>, headers: [&'static str; N], rows: I, border_bottom: bool,) -> impl UiChild + use<N, I> where I: IntoIterator<Item = [&'static str; N]>",
            "fn muted_full_width<T>(cx: &mut UiCx<'_>, text: T) -> impl UiChild + use<T> where T: Into<Arc<str>>",
            "fn muted_inline<T>(cx: &mut UiCx<'_>, text: T) -> impl UiChild + use<T> where T: Into<Arc<str>>",
            "fn muted_flex_1_min_w_0<T>(cx: &mut UiCx<'_>, text: T) -> impl UiChild + use<T> where T: Into<Arc<str>>",
            "fn icon(cx: &mut UiCx<'_>, id: &'static str) -> impl UiChild + use<>",
            "fn render_section(cx: &mut UiCx<'_>, section: DocSection) -> impl UiChild + use<>",
            "fn preview_code_tabs(cx: &mut UiCx<'_>, test_id_prefix: Option<&str>, preview: AnyElement, max_w: Px, code: DocCodeBlock, #[cfg(feature = \"gallery-ai\")] tabs_sizing: DocTabsSizing, #[cfg(not(feature = \"gallery-ai\"))] _tabs_sizing: DocTabsSizing, shell: bool,) -> impl UiChild + use<>",
            "fn code_block_shell(cx: &mut UiCx<'_>, test_id_prefix: Option<&str>, max_w: Px, block: DocCodeBlock, shell: bool,) -> impl UiChild + use<>",
            "fn section_title(cx: &mut UiCx<'_>, title: &'static str) -> impl UiChild + use<>",
        ],
        &[
            "fn render_doc_page(cx: &mut UiCx<'_>, intro: Option<&'static str>, sections: Vec<DocSection>,) -> AnyElement",
            "fn wrap_preview_page(cx: &mut UiCx<'_>, intro: Option<&'static str>, section_title: &'static str, elements: Vec<AnyElement>,) -> AnyElement",
            "fn wrap_row(cx: &mut UiCx<'_>, theme: &Theme, gap: Space, align: fret_ui::element::CrossAlign, children: impl FnOnce(&mut UiCx<'_>) -> Vec<AnyElement>,) -> AnyElement",
            "fn wrap_controls_row(cx: &mut UiCx<'_>, theme: &Theme, gap: Space, children: impl FnOnce(&mut UiCx<'_>) -> Vec<AnyElement>,) -> AnyElement",
            "fn text_table<const N: usize>(cx: &mut UiCx<'_>, headers: [&'static str; N], rows: impl IntoIterator<Item = [&'static str; N]>, border_bottom: bool,) -> AnyElement",
            "fn muted_full_width<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
            "fn muted_inline<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
            "fn muted_flex_1_min_w_0<H: UiHost>(cx: &mut ElementContext<'_, H>, text: impl Into<Arc<str>>) -> AnyElement",
            "fn icon(cx: &mut UiCx<'_>, id: &'static str) -> AnyElement",
            "fn render_section(cx: &mut UiCx<'_>, section: DocSection) -> AnyElement",
            "fn preview_code_tabs(cx: &mut UiCx<'_>, test_id_prefix: Option<&str>, preview: AnyElement, max_w: Px, code: DocCodeBlock, #[cfg(feature = \"gallery-ai\")] tabs_sizing: DocTabsSizing, #[cfg(not(feature = \"gallery-ai\"))] _tabs_sizing: DocTabsSizing, shell: bool,) -> AnyElement",
            "fn code_block_shell(cx: &mut UiCx<'_>, test_id_prefix: Option<&str>, max_w: Px, block: DocCodeBlock, shell: bool,) -> AnyElement",
            "fn section_title(cx: &mut UiCx<'_>, title: &'static str) -> AnyElement",
        ],
    );
}

#[test]
fn gallery_doc_layout_retains_only_intentional_raw_boundaries() {
    let normalized = assert_normalized_markers_present(
        "src/ui/doc_layout.rs",
        &[
            "pub preview: AnyElement,",
            "pub(in crate::ui) fn new(title: &'static str, preview: AnyElement) -> Self",
            "pub(in crate::ui) fn gap_card(",
            ")-> (&'static str, AnyElement) {",
        ],
    );

    assert_eq!(
        normalized.matches("->AnyElement").count(),
        0,
        "src/ui/doc_layout.rs should keep exactly the audited raw-return boundaries until the page-collection lane migrates",
    );

    let source = read("src/ui/doc_layout.rs");
    assert_eq!(
        source.matches("Intentional raw boundary:").count(),
        1,
        "src/ui/doc_layout.rs should document every retained raw-return boundary with an explicit rationale",
    );
    assert!(source.contains(
        "Intentionally stored as a landed value because the doc scaffold still decorates preview"
    ));
}

#[test]
fn render_doc_page_callers_land_the_typed_doc_page_explicitly() {
    for path in rust_sources("src/ui/pages") {
        let source = read_path(&path);
        if !source.contains("render_doc_page(") {
            continue;
        }

        let mut saw_final_return_line = false;
        for line in source.lines() {
            let trimmed = line.trim();
            assert_ne!(
                trimmed,
                "vec![body]",
                "{} should not keep the legacy raw render_doc_page landing",
                path.display()
            );
            assert_ne!(
                trimmed,
                "vec![page]",
                "{} should not keep the legacy raw render_doc_page landing",
                path.display()
            );
            if trimmed.starts_with("vec![body") || trimmed.starts_with("vec![page") {
                saw_final_return_line = true;
                assert!(
                    trimmed.contains(".into_element(cx)"),
                    "{} should keep the final render_doc_page landing explicit at the page surface",
                    path.display()
                );
            }
        }
        assert!(
            saw_final_return_line,
            "{} should expose a final page return line for render_doc_page output",
            path.display()
        );
    }
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
