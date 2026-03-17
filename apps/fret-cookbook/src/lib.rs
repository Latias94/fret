//! Cookbook examples crate.
//!
//! This crate intentionally keeps a tiny surface:
//! - helpers shared by `examples/`,
//! - no reusable product APIs (those belong in ecosystem crates).

use fret::app::prelude::*;

pub mod scaffold;

pub fn install_cookbook_defaults(app: &mut App) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        shadcn::themes::ShadcnColorScheme::Light,
    );
}

#[cfg(test)]
mod authoring_surface_policy_tests {
    use std::path::{Path, PathBuf};

    const ROOT_README: &str = include_str!("../../../README.md");
    const GOLDEN_PATH_DOC: &str = include_str!("../../../docs/examples/todo-app-golden-path.md");
    const COMMANDS_KEYMAP_EXAMPLE: &str = include_str!("../examples/commands_keymap_basics.rs");
    const DATA_TABLE_EXAMPLE: &str = include_str!("../examples/data_table_basics.rs");
    const DATE_PICKER_EXAMPLE: &str = include_str!("../examples/date_picker_basics.rs");
    const DRAG_EXAMPLE: &str = include_str!("../examples/drag_basics.rs");
    const DOCKING_EXAMPLE: &str = include_str!("../examples/docking_basics.rs");
    const DROP_SHADOW_EXAMPLE: &str = include_str!("../examples/drop_shadow_basics.rs");
    const EMBEDDED_VIEWPORT_EXAMPLE: &str = include_str!("../examples/embedded_viewport_basics.rs");
    const EFFECTS_LAYER_EXAMPLE: &str = include_str!("../examples/effects_layer_basics.rs");
    const EXTERNAL_TEXTURE_IMPORT_EXAMPLE: &str =
        include_str!("../examples/external_texture_import_basics.rs");
    const FORM_EXAMPLE: &str = include_str!("../examples/form_basics.rs");
    const GIZMO_EXAMPLE: &str = include_str!("../examples/gizmo_basics.rs");
    const IMUI_ACTION_EXAMPLE: &str = include_str!("../examples/imui_action_basics.rs");
    const ICONS_AND_ASSETS_EXAMPLE: &str = include_str!("../examples/icons_and_assets_basics.rs");
    const SCAFFOLD: &str = include_str!("scaffold.rs");
    const HELLO_EXAMPLE: &str = include_str!("../examples/hello.rs");
    const HELLO_COUNTER_EXAMPLE: &str = include_str!("../examples/hello_counter.rs");
    const MARKDOWN_AND_CODE_EXAMPLE: &str = include_str!("../examples/markdown_and_code_basics.rs");
    const OVERLAY_EXAMPLE: &str = include_str!("../examples/overlay_basics.rs");
    const PAYLOAD_ACTIONS_EXAMPLE: &str = include_str!("../examples/payload_actions_basics.rs");
    const QUERY_EXAMPLE: &str = include_str!("../examples/query_basics.rs");
    const ROUTER_EXAMPLE: &str = include_str!("../examples/router_basics.rs");
    const SIMPLE_TODO_EXAMPLE: &str = include_str!("../examples/simple_todo.rs");
    const SIMPLE_TODO_V2_TARGET_EXAMPLE: &str =
        include_str!("../examples/simple_todo_v2_target.rs");
    const APP_OWNED_BUNDLE_ASSETS_EXAMPLE: &str =
        include_str!("../examples/app_owned_bundle_assets_basics.rs");
    const ASYNC_INBOX_EXAMPLE: &str = include_str!("../examples/async_inbox_basics.rs");
    const ASSETS_RELOAD_EPOCH_EXAMPLE: &str =
        include_str!("../examples/assets_reload_epoch_basics.rs");
    const CANVAS_PAN_ZOOM_EXAMPLE: &str = include_str!("../examples/canvas_pan_zoom_basics.rs");
    const CHART_INTERACTIONS_EXAMPLE: &str =
        include_str!("../examples/chart_interactions_basics.rs");
    const THEME_SWITCHING_EXAMPLE: &str = include_str!("../examples/theme_switching_basics.rs");
    const CUSTOM_V1_EXAMPLE: &str = include_str!("../examples/customv1_basics.rs");
    const TEXT_INPUT_EXAMPLE: &str = include_str!("../examples/text_input_basics.rs");
    const TOAST_EXAMPLE: &str = include_str!("../examples/toast_basics.rs");
    const TOGGLE_EXAMPLE: &str = include_str!("../examples/toggle_basics.rs");
    const UNDO_EXAMPLE: &str = include_str!("../examples/undo_basics.rs");
    const UTILITY_WINDOW_MATERIALS_EXAMPLE: &str =
        include_str!("../examples/utility_window_materials_windows.rs");
    const VIRTUAL_LIST_EXAMPLE: &str = include_str!("../examples/virtual_list_basics.rs");

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

    fn cookbook_rust_sources() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        collect_rust_sources(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("examples"),
            &mut paths,
        );
        paths.sort();
        paths
    }

    fn assert_uses_app_surface(src: &str) {
        assert!(src.contains("use fret::app::prelude::*;"));
        assert!(src.contains("&mut App"));
        assert!(src.contains("WindowId"));
        assert!(!src.contains("AppWindowId"));
        assert!(!src.contains("KernelApp"));
        assert!(src.contains("AppUi<'_, '_>"));
        assert!(!src.contains("AppUi<'_, '_, KernelApp>"));
        assert!(src.contains(") -> Ui {"));
        assert!(!src.contains("use fret::prelude::*;"));
        assert!(!src.contains("ViewCx<'_, '_, App>"));
        assert!(!src.contains(") -> Elements {"));
        assert!(!src.contains("cx.use_local"));
        assert!(!src.contains("cx.on_action_notify_"));
        assert!(!src.contains("cx.on_payload_action_notify_"));
    }

    fn assert_avoids_legacy_conversion_names(src: &str) {
        assert!(!src.contains("UiIntoElement"));
        assert!(!src.contains("UiHostBoundIntoElement"));
        assert!(!src.contains("UiChildIntoElement"));
        assert!(!src.contains("UiBuilderHostBoundIntoElementExt"));
    }

    fn assert_uses_app_surface_doc(src: &str) {
        assert!(src.contains("use fret::app::prelude::*;"));
        assert!(src.contains("AppUi<'_, '_>"));
        assert!(!src.contains("AppUi<'_, '_, KernelApp>"));
        assert!(!src.contains("KernelApp"));
        assert!(!src.contains("use fret::prelude::*;"));
        assert!(!src.contains("ViewCx<'_, '_, App>"));
    }

    fn assert_uses_advanced_surface(src: &str) {
        assert!(src.contains("advanced::prelude::*"));
        assert!(src.contains("KernelApp"));
        assert!(
            src.contains("AppUi<'_, '_>")
                || src.contains("ViewCx<'_, '_, KernelApp>")
                || src.contains("ElementContext<'_, KernelApp>")
        );
        assert!(
            src.contains(") -> Ui {")
                || src.contains(") -> Elements {")
                || src.contains(") -> ViewElements {")
        );
        assert!(!src.contains("use fret::prelude::*;"));
        assert!(!src.contains("use fret::app::prelude::*;"));
        assert!(!src.contains("AppUi<'_, '_, KernelApp>"));
        assert!(!src.contains("cx.use_local"));
        assert!(!src.contains("cx.on_action_notify_"));
    }

    fn assert_advanced_view_runtime_example_uses_app_ui_aliases(src: &str) {
        assert!(src.contains("fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui"));
        assert!(
            !src.contains("fn render(&mut self, cx: &mut ViewCx<'_, '_, KernelApp>) -> Elements")
        );
    }

    fn assert_prefers_view_builder_then_run(src: &str) {
        let normalized = src.split_whitespace().collect::<String>();
        assert!(normalized.contains(".view::<"));
        assert!(normalized.contains(".run()"));
        assert!(!normalized.contains(".run_view::<"));
    }

    fn assert_setup_surface_keeps_inline_closures_off_setup(src: &str) {
        let normalized = src.split_whitespace().collect::<String>();
        assert!(!normalized.contains(".setup(|"));
        assert!(!normalized.contains(".setup(move|"));
        assert!(!normalized.contains(".setup_with("));
    }

    fn assert_advanced_helpers_prefer_uicx(
        src: &str,
        required_markers: &[&str],
        forbidden_markers: &[&str],
    ) {
        let normalized = src.split_whitespace().collect::<String>();
        assert!(normalized.contains("UiCx<'_>"));
        for marker in required_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for marker in forbidden_markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "legacy marker still present: {marker}"
            );
        }
    }

    fn assert_intentional_raw_retained_seam(src: &str, markers: &[&str], forbidden: &[&str]) {
        let normalized = src.split_whitespace().collect::<String>();
        for marker in markers {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(normalized.contains(&marker), "missing marker: {marker}");
        }
        for marker in forbidden {
            let marker = marker.split_whitespace().collect::<String>();
            assert!(
                !normalized.contains(&marker),
                "unexpected non-raw marker present: {marker}"
            );
        }
    }

    fn assert_promoted_card_wrapper_family_only(name: &str, src: &str) {
        for forbidden in [
            "shadcn::Card::build(",
            "shadcn::CardHeader::build(",
            "shadcn::CardContent::build(",
        ] {
            assert!(
                !src.contains(forbidden),
                "{name} reintroduced legacy card teaching surface: {forbidden}"
            );
        }
    }

    #[test]
    fn onboarding_examples_use_the_new_app_surface() {
        assert_uses_app_surface(HELLO_EXAMPLE);
        assert_uses_app_surface(SIMPLE_TODO_EXAMPLE);
        assert_uses_app_surface(SIMPLE_TODO_V2_TARGET_EXAMPLE);
        assert!(HELLO_EXAMPLE.contains(
            "fn hello_page(render_marker: &'static str, count_value: u32) -> impl UiChild"
        ));
        assert!(!HELLO_EXAMPLE.contains("fn hello_page(cx: &mut UiCx<'_>,"));
        assert!(HELLO_EXAMPLE.contains("ui::single(cx, hello_page(render_marker, count_value))"));
        assert!(HELLO_EXAMPLE.contains("cx.state().local::<u32>()"));
        assert!(HELLO_EXAMPLE.contains(".local_update::<act::Click, u32>("));
        assert!(!HELLO_EXAMPLE.contains("root.into_element(cx).into()"));
        assert!(SIMPLE_TODO_EXAMPLE.contains("cx.state().local::<String>()"));
        assert!(SIMPLE_TODO_EXAMPLE.contains("cx.actions().locals::<act::Add>"));
        assert!(
            SIMPLE_TODO_EXAMPLE.contains(".payload_local_update_if::<act::Toggle, Vec<TodoRow>>(")
        );
        assert!(SIMPLE_TODO_EXAMPLE.contains(
            ".payload_local_update_if::<act::Toggle, Vec<TodoRow>>(&todos_state, |rows, id| {"
        ));
        assert!(SIMPLE_TODO_EXAMPLE.contains("impl UiChild"));
        assert!(SIMPLE_TODO_V2_TARGET_EXAMPLE.contains("impl UiChild"));
        assert!(SIMPLE_TODO_V2_TARGET_EXAMPLE.contains("cx.actions().locals::<act::Add>"));
        assert_avoids_legacy_conversion_names(SIMPLE_TODO_V2_TARGET_EXAMPLE);
    }

    #[test]
    fn migrated_basics_examples_use_the_new_app_surface() {
        assert_uses_app_surface(HELLO_COUNTER_EXAMPLE);
        assert_uses_app_surface(TEXT_INPUT_EXAMPLE);
        assert_uses_app_surface(TOGGLE_EXAMPLE);
        assert_uses_app_surface(PAYLOAD_ACTIONS_EXAMPLE);
        assert_uses_app_surface(FORM_EXAMPLE);
        assert_uses_app_surface(DATE_PICKER_EXAMPLE);
        assert_uses_app_surface(COMMANDS_KEYMAP_EXAMPLE);
        assert_uses_app_surface(OVERLAY_EXAMPLE);
        assert_uses_app_surface(THEME_SWITCHING_EXAMPLE);
        assert_uses_app_surface(TOAST_EXAMPLE);
        assert_uses_app_surface(VIRTUAL_LIST_EXAMPLE);
        assert_uses_app_surface(ASYNC_INBOX_EXAMPLE);
        assert_uses_app_surface(QUERY_EXAMPLE);
        assert_uses_app_surface(ROUTER_EXAMPLE);
        assert_uses_app_surface(DATA_TABLE_EXAMPLE);
        assert_uses_app_surface(UNDO_EXAMPLE);
        assert_uses_app_surface(MARKDOWN_AND_CODE_EXAMPLE);
        assert_uses_app_surface(IMUI_ACTION_EXAMPLE);

        assert!(HELLO_COUNTER_EXAMPLE.contains("cx.state().local_init(|| 0i64)"));
        assert!(HELLO_COUNTER_EXAMPLE.contains("cx.actions().locals::<act::Inc>"));
        assert!(HELLO_COUNTER_EXAMPLE.contains("cx.actions().local_set::<act::Reset, i64>"));

        assert!(TEXT_INPUT_EXAMPLE.contains("cx.actions().locals::<act::Submit>"));
        assert!(TEXT_INPUT_EXAMPLE.contains("cx.actions().availability::<act::Submit>"));

        assert!(TOGGLE_EXAMPLE.contains("toggle_local_bool::<act::ToggleBookmark>"));

        assert!(PAYLOAD_ACTIONS_EXAMPLE.contains("cx.state().local_init(|| {"));
        assert!(PAYLOAD_ACTIONS_EXAMPLE.contains("payload::<act::Remove>()"));
        assert!(PAYLOAD_ACTIONS_EXAMPLE.contains("local_update_if::<Vec<Row>>(&rows_state"));

        assert!(FORM_EXAMPLE.contains("locals::<act::Submit>"));
        assert!(FORM_EXAMPLE.contains("availability::<act::Submit>"));

        assert!(DATE_PICKER_EXAMPLE.contains("cx.state().local_init(|| false)"));
        assert!(DATE_PICKER_EXAMPLE.contains("watch(&selected_state)"));

        assert!(COMMANDS_KEYMAP_EXAMPLE.contains("locals::<act::TogglePanel>"));
        assert!(COMMANDS_KEYMAP_EXAMPLE.contains("toggle_local_bool::<act::ToggleAllowCommand>"));

        assert!(OVERLAY_EXAMPLE.contains("local_set::<act::OpenDialog, bool>"));
        assert!(OVERLAY_EXAMPLE.contains("local_update::<act::BumpUnderlay, u32>"));

        assert!(THEME_SWITCHING_EXAMPLE.contains("use fret_app::Effect;"));
        assert!(THEME_SWITCHING_EXAMPLE.contains("local_init(|| Some::<Arc<str>>"));

        assert!(TOAST_EXAMPLE.contains("on_action_notify::<act::DefaultToast>"));

        assert!(VIRTUAL_LIST_EXAMPLE.contains("use fret_runtime::Model;"));
        assert!(VIRTUAL_LIST_EXAMPLE.contains(".items"));
        assert!(VIRTUAL_LIST_EXAMPLE.contains(".watch(cx)"));
        assert!(VIRTUAL_LIST_EXAMPLE.contains("models::<act::RotateItems>"));

        assert!(ASYNC_INBOX_EXAMPLE.contains("use fret_runtime::Model;"));
        assert!(ASYNC_INBOX_EXAMPLE.contains("models::<act::Cancel>"));
        assert!(ASYNC_INBOX_EXAMPLE.contains("on_action_notify::<act::Start>"));

        assert!(QUERY_EXAMPLE.contains("cx.data().query("));
        assert!(QUERY_EXAMPLE.contains("let state = handle.read_layout(cx);"));
        assert!(!QUERY_EXAMPLE.contains("handle.layout(cx).value_or_default()"));
        assert!(!QUERY_EXAMPLE.contains("cx.use_query("));
        assert!(!QUERY_EXAMPLE.contains("fret_query::ui::QueryElementContextExt"));
        assert!(QUERY_EXAMPLE.contains("toggle_local_bool::<act::ToggleErrorMode>"));

        assert!(ROUTER_EXAMPLE.contains("use fret::router::{"));
        assert!(ROUTER_EXAMPLE.contains("RouteCodec"));
        assert!(!ROUTER_EXAMPLE.contains("use fret_router::{"));
        assert!(!ROUTER_EXAMPLE.contains("use fret_router_ui::{"));
        assert!(ROUTER_EXAMPLE.contains("router_link_to_typed_route_with_test_id"));
        assert!(ROUTER_EXAMPLE.contains("models::<act::ClearIntents>"));
        assert!(ROUTER_EXAMPLE.contains("on_action_notify::<act::RouterBack>"));
        assert!(ROUTER_EXAMPLE.contains("self.store.back_on_action()"));
        assert!(ROUTER_EXAMPLE.contains("self.store.forward_on_action()"));
        assert!(!ROUTER_EXAMPLE.contains("set_router_command_availability(window"));
        assert!(ROUTER_EXAMPLE.contains(".setup(fret::router::app::install)"));
        assert!(!ROUTER_EXAMPLE.contains(".setup(fret::router::install_app)"));
        assert!(ROUTER_EXAMPLE.contains(".into_element_by_leaf("));
        assert!(!ROUTER_EXAMPLE.contains(".into_element_by_leaf_ui("));
        assert!(!ROUTER_EXAMPLE.contains("router_outlet_ui("));

        assert!(DATA_TABLE_EXAMPLE.contains("use fret_runtime::Model;"));
        assert!(DATA_TABLE_EXAMPLE.contains("fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui"));

        assert!(UNDO_EXAMPLE.contains("use fret_app::Effect;"));
        assert!(UNDO_EXAMPLE.contains("models::<act::Inc>"));
        assert!(UNDO_EXAMPLE.contains("on_action_notify::<act::Undo>"));

        assert!(MARKDOWN_AND_CODE_EXAMPLE.contains("MarkdownComponents::<App>::default()"));
        assert!(MARKDOWN_AND_CODE_EXAMPLE.contains("local_set::<act::Reset, String>"));

        assert!(
            IMUI_ACTION_EXAMPLE
                .contains("use fret_runtime::{CommandId, CommandMeta, CommandScope, Model};")
        );
        assert!(IMUI_ACTION_EXAMPLE.contains("local_update::<act::Inc, u32>"));
    }

    #[test]
    fn shared_scaffold_prefers_app_surface_for_cookbook_page_shells() {
        assert!(SCAFFOLD.contains("use fret::app::prelude::*;"));
        assert!(SCAFFOLD.contains("use fret::style::{ColorRef, Space, Theme};"));
        assert!(SCAFFOLD.contains("&mut UiCx<'_>"));
        assert!(SCAFFOLD.contains("B: UiChild"));
        assert!(SCAFFOLD.contains("ui::single(cx, surface)"));
        assert!(!SCAFFOLD.contains("ui::children![cx; surface]"));
        assert!(!SCAFFOLD.contains("&mut ComponentCx<'_, H>"));
        assert!(!SCAFFOLD.contains("B: IntoUiElement<H>"));
        assert!(!SCAFFOLD.contains("use fret::prelude::*;"));
        assert!(!SCAFFOLD.contains("surface: AnyElement"));
        assert!(!SCAFFOLD.contains("surface.into_element(cx);"));
        assert!(!SCAFFOLD.contains("use fret::component::prelude::*;"));
    }

    #[test]
    fn utility_window_example_uses_ui_single_for_single_surface_shells() {
        assert!(UTILITY_WINDOW_MATERIALS_EXAMPLE.contains("ui::single(cx, surface)"));
        assert!(!UTILITY_WINDOW_MATERIALS_EXAMPLE.contains("ui::children![cx; surface]"));
    }

    #[test]
    fn cookbook_examples_use_unified_centered_page_helpers() {
        for src in [
            UNDO_EXAMPLE,
            DRAG_EXAMPLE,
            EXTERNAL_TEXTURE_IMPORT_EXAMPLE,
            CUSTOM_V1_EXAMPLE,
            ASSETS_RELOAD_EPOCH_EXAMPLE,
            DROP_SHADOW_EXAMPLE,
            GIZMO_EXAMPLE,
            DATE_PICKER_EXAMPLE,
            COMMANDS_KEYMAP_EXAMPLE,
            FORM_EXAMPLE,
            DOCKING_EXAMPLE,
            EFFECTS_LAYER_EXAMPLE,
            CHART_INTERACTIONS_EXAMPLE,
            SIMPLE_TODO_EXAMPLE,
            TOAST_EXAMPLE,
            ICONS_AND_ASSETS_EXAMPLE,
            HELLO_COUNTER_EXAMPLE,
            CANVAS_PAN_ZOOM_EXAMPLE,
            TOGGLE_EXAMPLE,
            TEXT_INPUT_EXAMPLE,
            ASYNC_INBOX_EXAMPLE,
            QUERY_EXAMPLE,
            PAYLOAD_ACTIONS_EXAMPLE,
            MARKDOWN_AND_CODE_EXAMPLE,
            THEME_SWITCHING_EXAMPLE,
            SIMPLE_TODO_V2_TARGET_EXAMPLE,
            DATA_TABLE_EXAMPLE,
            ROUTER_EXAMPLE,
            VIRTUAL_LIST_EXAMPLE,
        ] {
            assert!(!src.contains("centered_page_background_ui("));
            assert!(!src.contains("centered_page_muted_ui("));
        }
    }

    #[test]
    fn canonical_compare_set_uses_ui_returning_cookbook_scaffold() {
        for src in [
            HELLO_COUNTER_EXAMPLE,
            SIMPLE_TODO_EXAMPLE,
            SIMPLE_TODO_V2_TARGET_EXAMPLE,
        ] {
            assert!(src.contains("centered_page_muted(cx, TEST_ID_ROOT, card)"));
            assert!(!src.contains("centered_page_muted(cx, TEST_ID_ROOT, card).into()"));
        }
    }

    #[test]
    fn advanced_examples_use_the_explicit_advanced_surface() {
        assert_uses_advanced_surface(DRAG_EXAMPLE);
        assert_uses_advanced_surface(EFFECTS_LAYER_EXAMPLE);
        assert_uses_advanced_surface(DROP_SHADOW_EXAMPLE);
        assert_uses_advanced_surface(ICONS_AND_ASSETS_EXAMPLE);
        assert_uses_advanced_surface(ASSETS_RELOAD_EPOCH_EXAMPLE);
        assert_uses_advanced_surface(CANVAS_PAN_ZOOM_EXAMPLE);
        assert_uses_advanced_surface(CHART_INTERACTIONS_EXAMPLE);
        assert_uses_advanced_surface(CUSTOM_V1_EXAMPLE);
        assert_uses_advanced_surface(DOCKING_EXAMPLE);
        assert_uses_advanced_surface(EMBEDDED_VIEWPORT_EXAMPLE);
        assert_uses_advanced_surface(EXTERNAL_TEXTURE_IMPORT_EXAMPLE);
        assert_uses_advanced_surface(GIZMO_EXAMPLE);
        assert_uses_advanced_surface(UTILITY_WINDOW_MATERIALS_EXAMPLE);

        assert!(DRAG_EXAMPLE.contains("use fret::{FretApp, advanced::prelude::*, shadcn};"));
        assert!(DRAG_EXAMPLE.contains("UiPointerActionHost"));

        assert!(EFFECTS_LAYER_EXAMPLE.contains("UiCx<'_>"));
        assert!(EFFECTS_LAYER_EXAMPLE.contains("cx.actions().models::<act::Pixelate>"));

        assert!(DROP_SHADOW_EXAMPLE.contains("UiCx<'_>"));
        assert!(DROP_SHADOW_EXAMPLE.contains("DropShadowV1"));
        assert!(DROP_SHADOW_EXAMPLE.contains("cx.state().local_init(|| true)"));
        assert!(DROP_SHADOW_EXAMPLE.contains("cx.state().watch(&enabled_state)"));

        assert!(ICONS_AND_ASSETS_EXAMPLE.contains("icon::IconSvgPreloadDiagnostics"));
        assert!(ICONS_AND_ASSETS_EXAMPLE.contains("integration::InstallIntoApp"));
        assert!(ICONS_AND_ASSETS_EXAMPLE.contains("impl InstallIntoApp for IconsAndAssetsBundle"));
        assert!(
            ICONS_AND_ASSETS_EXAMPLE
                .contains("PACKAGE_ASSET_BUNDLE_NAME: &str = \"cookbook-icons-demo\"")
        );
        assert!(
            ICONS_AND_ASSETS_EXAMPLE.contains("AssetBundleId::package(PACKAGE_ASSET_BUNDLE_NAME)")
        );
        assert!(ICONS_AND_ASSETS_EXAMPLE.contains("assets::register_bundle_entries"));
        assert!(
            ICONS_AND_ASSETS_EXAMPLE
                .contains("the app never replays low-level icon or asset registration manually")
        );
        assert!(
            ICONS_AND_ASSETS_EXAMPLE
                .contains("App setup bundle: composes transitive icon + asset installers")
        );
        assert!(
            ICONS_AND_ASSETS_EXAMPLE
                .contains("Low-level registration stays internal to the dependency")
        );
        assert!(ICONS_AND_ASSETS_EXAMPLE.contains(
            "Hand-written bundle wrapper: use when the crate also composes icons or app defaults"
        ));
        assert!(ICONS_AND_ASSETS_EXAMPLE.contains(
            "This is the hand-written wrapper lane to teach when a crate composes more than raw shipped bytes"
        ));
        assert!(ICONS_AND_ASSETS_EXAMPLE.contains(".ui_assets_budgets("));
        assert!(!ICONS_AND_ASSETS_EXAMPLE.contains("UiAssets::configure("));
        assert!(!ICONS_AND_ASSETS_EXAMPLE.contains("AssetBundleId::app(\"fret-cookbook\")"));
        let icons_and_assets_normalized = ICONS_AND_ASSETS_EXAMPLE
            .split_whitespace()
            .collect::<String>();
        assert!(
            icons_and_assets_normalized
                .contains(".setup((IconsAndAssetsBundle,fret_cookbook::install_cookbook_defaults")
        );
        assert!(!icons_and_assets_normalized.contains(
            ".setup((fret_cookbook::install_cookbook_defaults,fret_icons_lucide::app::install))"
        ));
        assert!(
            !icons_and_assets_normalized
                .contains(".setup((shadcn::app::install,fret_icons_lucide::app::install))")
        );
        assert!(
            APP_OWNED_BUNDLE_ASSETS_EXAMPLE
                .contains("Scaffold equivalent: `generated_assets::mount(builder)`")
        );
        assert!(
            APP_OWNED_BUNDLE_ASSETS_EXAMPLE
                .contains("Generated module is enough when the crate only publishes shipped bytes")
        );
        assert!(
            APP_OWNED_BUNDLE_ASSETS_EXAMPLE
                .contains("`BundleAsset` is the public lookup lane; `Embedded` stays lower-level")
        );
        assert!(APP_OWNED_BUNDLE_ASSETS_EXAMPLE.contains(
            "This is the generated-module lane to teach when a crate only publishes shipped bytes."
        ));
        assert!(APP_OWNED_BUNDLE_ASSETS_EXAMPLE.contains("`FretApp::asset_startup(...)`"));
        assert!(
            APP_OWNED_BUNDLE_ASSETS_EXAMPLE.contains("`AssetStartupPlan::packaged_entries(...)`")
        );
        assert!(APP_OWNED_BUNDLE_ASSETS_EXAMPLE.contains("`FretApp::asset_entries(...)`"));
        assert!(
            APP_OWNED_BUNDLE_ASSETS_EXAMPLE.contains("without native-only file path assumptions.")
        );
        assert!(!APP_OWNED_BUNDLE_ASSETS_EXAMPLE.contains("ImageSource::from_file_path"));
        assert!(
            ASSETS_RELOAD_EPOCH_EXAMPLE
                .contains("use fret::{FretApp, advanced::prelude::*, shadcn};")
        );
        assert!(ASSETS_RELOAD_EPOCH_EXAMPLE.contains("fret::assets::bump_asset_reload_epoch"));
        assert!(ASSETS_RELOAD_EPOCH_EXAMPLE.contains("fret::assets::asset_reload_epoch(&*cx.app)"));
        assert!(
            !ASSETS_RELOAD_EPOCH_EXAMPLE.contains("fret_ui_assets::bump_ui_assets_reload_epoch")
        );
        assert!(!ASSETS_RELOAD_EPOCH_EXAMPLE.contains("UiAssetsReloadEpoch"));
        assert!(ASSETS_RELOAD_EPOCH_EXAMPLE.contains("Effect::RequestAnimationFrame"));
        assert!(ASSETS_RELOAD_EPOCH_EXAMPLE.contains("cx.state().local::<u64>()"));
        assert!(
            ASSETS_RELOAD_EPOCH_EXAMPLE
                .contains("cx.actions()\n            .local_update::<act::BumpReload, u64>")
        );

        assert!(
            CANVAS_PAN_ZOOM_EXAMPLE.contains("use fret::{FretApp, advanced::prelude::*, shadcn};")
        );
        assert!(CANVAS_PAN_ZOOM_EXAMPLE.contains("PanZoomCanvasSurfacePanelProps"));
        assert!(CANVAS_PAN_ZOOM_EXAMPLE.contains("CanvasPainter"));
        assert!(CANVAS_PAN_ZOOM_EXAMPLE.contains("cx.actions().models::<act::ResetView>"));

        assert!(CHART_INTERACTIONS_EXAMPLE.contains("use fret::{advanced::prelude::*, shadcn};"));
        assert!(CHART_INTERACTIONS_EXAMPLE.contains("ChartCanvas"));
        assert!(CHART_INTERACTIONS_EXAMPLE.contains("RetainedSubtreeProps::new::<KernelApp>"));
        assert!(
            CHART_INTERACTIONS_EXAMPLE
                .contains(".setup((shadcn::app::install, fret_icons_lucide::app::install))")
        );
        assert!(!CHART_INTERACTIONS_EXAMPLE.contains(".setup(shadcn::install_app)"));

        assert!(CUSTOM_V1_EXAMPLE.contains("use fret::{FretApp, advanced::prelude::*, shadcn};"));
        assert!(CUSTOM_V1_EXAMPLE.contains("EffectStep::CustomV1"));
        assert!(CUSTOM_V1_EXAMPLE.contains(".install_custom_effects(install_custom_effect)"));
        assert!(CUSTOM_V1_EXAMPLE.contains("cx.state().local_init(|| true)"));
        assert!(
            CUSTOM_V1_EXAMPLE
                .contains("cx.actions()\n            .toggle_local_bool::<act::ToggleEnabled>")
        );

        assert!(DOCKING_EXAMPLE.contains("use fret::{"));
        assert!(DOCKING_EXAMPLE.contains("advanced::prelude::*"));
        assert!(DOCKING_EXAMPLE.contains("integration::InstallIntoApp"));
        assert!(DOCKING_EXAMPLE.contains("docking::{"));
        assert!(!DOCKING_EXAMPLE.contains("use fret_docking::{"));
        assert!(DOCKING_EXAMPLE.contains("DockPanelFactory<KernelApp>"));
        assert!(DOCKING_EXAMPLE.contains("DockPanelRegistryBuilder::new()"));
        assert!(DOCKING_EXAMPLE.contains("docking::handle_dock_op"));
        assert!(DOCKING_EXAMPLE.contains("impl InstallIntoApp for DockingBasicsBundle"));
        assert!(
            DOCKING_EXAMPLE
                .contains(".setup((DockingBasicsBundle, fret_icons_lucide::app::install))")
        );
        assert!(!DOCKING_EXAMPLE.contains(".setup(shadcn::install_app)"));
        assert!(DOCKING_EXAMPLE.contains("RetainedSubtreeProps::new::<KernelApp>"));

        assert!(
            EMBEDDED_VIEWPORT_EXAMPLE
                .contains("use fret::advanced::interop::embedded_viewport as embedded;")
        );
        assert!(EMBEDDED_VIEWPORT_EXAMPLE.contains("ui_app_with_hooks("));
        assert!(
            EMBEDDED_VIEWPORT_EXAMPLE
                .contains(".setup((shadcn::app::install, fret_icons_lucide::app::install))")
        );
        assert!(
            EMBEDDED_VIEWPORT_EXAMPLE.contains("UiAppDriver<EmbeddedViewportBasicsWindowState>")
        );

        assert!(
            EXTERNAL_TEXTURE_IMPORT_EXAMPLE.contains("use fret::{advanced::prelude::*, shadcn};")
        );
        assert!(EXTERNAL_TEXTURE_IMPORT_EXAMPLE.contains("ui_app_with_hooks("));
        assert!(
            EXTERNAL_TEXTURE_IMPORT_EXAMPLE
                .contains(".setup((shadcn::app::install, fret_icons_lucide::app::install))")
        );
        assert!(!EXTERNAL_TEXTURE_IMPORT_EXAMPLE.contains(".setup(shadcn::install_app)"));
        assert!(
            EXTERNAL_TEXTURE_IMPORT_EXAMPLE
                .contains("UiAppDriver<ExternalTextureImportBasicsState>")
        );

        assert!(GIZMO_EXAMPLE.contains("use fret::{advanced::prelude::*, shadcn};"));
        assert!(GIZMO_EXAMPLE.contains("GizmoInput"));
        assert!(GIZMO_EXAMPLE.contains("ui_app_with_hooks("));
        assert!(
            GIZMO_EXAMPLE
                .contains(".setup((shadcn::app::install, fret_icons_lucide::app::install))")
        );
        assert!(!GIZMO_EXAMPLE.contains(".setup(shadcn::install_app)"));

        assert!(
            UTILITY_WINDOW_MATERIALS_EXAMPLE.contains("use fret::{advanced::prelude::*, shadcn};")
        );
        assert!(UTILITY_WINDOW_MATERIALS_EXAMPLE.contains("ui_app_with_hooks("));
        assert!(UTILITY_WINDOW_MATERIALS_EXAMPLE.contains("status: Model<Arc<str>>"));
    }

    #[test]
    fn theme_examples_use_curated_shadcn_theme_surface() {
        assert!(THEME_SWITCHING_EXAMPLE.contains("shadcn::themes::apply_shadcn_new_york("));
        assert!(THEME_SWITCHING_EXAMPLE.contains("shadcn::themes::ShadcnBaseColor::Slate"));
        assert!(THEME_SWITCHING_EXAMPLE.contains("shadcn::themes::ShadcnColorScheme::Dark"));
        assert!(!THEME_SWITCHING_EXAMPLE.contains("shadcn::shadcn_themes::"));
    }

    #[test]
    fn advanced_view_examples_prefer_app_ui_and_ui_aliases() {
        for src in [
            DRAG_EXAMPLE,
            EFFECTS_LAYER_EXAMPLE,
            DROP_SHADOW_EXAMPLE,
            ICONS_AND_ASSETS_EXAMPLE,
            ASSETS_RELOAD_EPOCH_EXAMPLE,
            CANVAS_PAN_ZOOM_EXAMPLE,
            CUSTOM_V1_EXAMPLE,
        ] {
            assert_advanced_view_runtime_example_uses_app_ui_aliases(src);
        }
    }

    #[test]
    fn advanced_helper_contexts_prefer_uicx_aliases() {
        assert_advanced_helpers_prefer_uicx(
            EFFECTS_LAYER_EXAMPLE,
            &[
                "let button = |_cx: &mut UiCx<'_>,",
                "let tile = |_cx: &mut UiCx<'_>, color: ColorRef|",
            ],
            &[
                "let button = |_cx: &mut ElementContext<'_, KernelApp>,",
                "let tile = |_cx: &mut ElementContext<'_, KernelApp>, color: ColorRef|",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            DROP_SHADOW_EXAMPLE,
            &[
                "fn shadow_card(",
                "title: String,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
            ],
            &[
                "let card = |cx: &mut ElementContext<'_, KernelApp>, title: String| -> AnyElement",
                "let card = |cx: &mut UiCx<'_>, title: String| -> AnyElement",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            ICONS_AND_ASSETS_EXAMPLE,
            &[
                "ui::v_flex(|cx: &mut UiCx<'_>| {",
                "fn render_image_preview(",
                "image: Option<ImageId>,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
            ],
            &[
                "ui::v_flex(|cx: &mut ElementContext<'_, KernelApp>| {",
                "let render_image = |cx: &mut ElementContext<'_, KernelApp>,",
                "let render_image = |cx: &mut UiCx<'_>,",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            ASSETS_RELOAD_EPOCH_EXAMPLE,
            &[
                "fn render_image_panel(_cx: &mut UiCx<'_>,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "fn render_svg_panel(_cx: &mut UiCx<'_>,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
            ],
            &[
                "fn render_image_panel(cx: &mut ElementContext<'_, KernelApp>,",
                "fn render_svg_panel(cx: &mut ElementContext<'_, KernelApp>,",
                "fn render_image_panel(cx: &mut UiCx<'_>, theme: &ThemeSnapshot, st: fret_ui_assets::ImageSourceState,) -> AnyElement",
                "fn render_svg_panel(cx: &mut UiCx<'_>, theme: &ThemeSnapshot, st: fret_ui_assets::SvgFileState,) -> AnyElement",
            ],
        );

        assert_advanced_helpers_prefer_uicx(
            CHART_INTERACTIONS_EXAMPLE,
            &["fn chart_canvas(cx: &mut UiCx<'_>,"],
            &["fn chart_canvas(cx: &mut ElementContext<'_, KernelApp>,"],
        );

        assert_advanced_helpers_prefer_uicx(
            CUSTOM_V1_EXAMPLE,
            &[
                "fn panel_shell<B>(",
                "title: &'static str,",
                ") -> impl IntoUiElement<KernelApp> + use<B>",
                "where B: IntoUiElement<KernelApp>",
                "fn preview_content(",
                "label: &'static str,",
                ") -> impl IntoUiElement<KernelApp> + use<>",
                "let swatch = |_cx: &mut UiCx<'_>, rgb: u32|",
            ],
            &[
                "UiChildIntoElement<KernelApp>",
                "fn panel_shell(cx: &mut ElementContext<'_, KernelApp>, title: &str, body: impl IntoUiElement<KernelApp>) -> AnyElement",
                "fn preview_content(cx: &mut ElementContext<'_, KernelApp>, label: &str) -> AnyElement",
                "let swatch = |_cx: &mut ElementContext<'_, KernelApp>, rgb: u32|",
            ],
        );
    }

    #[test]
    fn retained_canvas_helpers_keep_raw_landing_seams() {
        assert_intentional_raw_retained_seam(
            CHART_INTERACTIONS_EXAMPLE,
            &[
                "fn chart_canvas(cx: &mut UiCx<'_>, st: &ChartInteractionsWindowState) -> AnyElement",
                "RetainedSubtreeProps::new::<KernelApp>",
                "cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true),",
                "vec![cx.retained_subtree(props)]",
            ],
            &[
                "fn chart_canvas(cx: &mut UiCx<'_>, st: &ChartInteractionsWindowState) -> impl IntoUiElement<KernelApp>",
            ],
        );
    }

    #[test]
    fn cookbook_examples_keep_card_wrapper_family_as_the_only_card_teaching_surface() {
        for (name, src) in [
            ("commands_keymap_basics", COMMANDS_KEYMAP_EXAMPLE),
            ("data_table_basics", DATA_TABLE_EXAMPLE),
            ("date_picker_basics", DATE_PICKER_EXAMPLE),
            ("drag_basics", DRAG_EXAMPLE),
            ("docking_basics", DOCKING_EXAMPLE),
            ("drop_shadow_basics", DROP_SHADOW_EXAMPLE),
            ("embedded_viewport_basics", EMBEDDED_VIEWPORT_EXAMPLE),
            ("effects_layer_basics", EFFECTS_LAYER_EXAMPLE),
            (
                "external_texture_import_basics",
                EXTERNAL_TEXTURE_IMPORT_EXAMPLE,
            ),
            ("form_basics", FORM_EXAMPLE),
            ("gizmo_basics", GIZMO_EXAMPLE),
            ("icons_and_assets_basics", ICONS_AND_ASSETS_EXAMPLE),
            ("markdown_and_code_basics", MARKDOWN_AND_CODE_EXAMPLE),
            ("overlay_basics", OVERLAY_EXAMPLE),
            ("payload_actions_basics", PAYLOAD_ACTIONS_EXAMPLE),
            ("query_basics", QUERY_EXAMPLE),
            ("router_basics", ROUTER_EXAMPLE),
            ("simple_todo", SIMPLE_TODO_EXAMPLE),
            ("simple_todo_v2_target", SIMPLE_TODO_V2_TARGET_EXAMPLE),
            ("async_inbox_basics", ASYNC_INBOX_EXAMPLE),
            ("assets_reload_epoch_basics", ASSETS_RELOAD_EPOCH_EXAMPLE),
            ("canvas_pan_zoom_basics", CANVAS_PAN_ZOOM_EXAMPLE),
            ("chart_interactions_basics", CHART_INTERACTIONS_EXAMPLE),
            ("theme_switching_basics", THEME_SWITCHING_EXAMPLE),
            ("customv1_basics", CUSTOM_V1_EXAMPLE),
            ("hello_counter", HELLO_COUNTER_EXAMPLE),
            ("text_input_basics", TEXT_INPUT_EXAMPLE),
            ("toast_basics", TOAST_EXAMPLE),
            ("toggle_basics", TOGGLE_EXAMPLE),
            ("undo_basics", UNDO_EXAMPLE),
            (
                "utility_window_materials_windows",
                UTILITY_WINDOW_MATERIALS_EXAMPLE,
            ),
            ("virtual_list_basics", VIRTUAL_LIST_EXAMPLE),
        ] {
            assert_promoted_card_wrapper_family_only(name, src);
        }
    }

    #[test]
    fn cookbook_examples_limit_raw_shadcn_escape_hatches() {
        for (name, src) in [
            ("commands_keymap_basics", COMMANDS_KEYMAP_EXAMPLE),
            ("data_table_basics", DATA_TABLE_EXAMPLE),
            ("date_picker_basics", DATE_PICKER_EXAMPLE),
            ("drag_basics", DRAG_EXAMPLE),
            ("docking_basics", DOCKING_EXAMPLE),
            ("drop_shadow_basics", DROP_SHADOW_EXAMPLE),
            ("embedded_viewport_basics", EMBEDDED_VIEWPORT_EXAMPLE),
            ("effects_layer_basics", EFFECTS_LAYER_EXAMPLE),
            (
                "external_texture_import_basics",
                EXTERNAL_TEXTURE_IMPORT_EXAMPLE,
            ),
            ("form_basics", FORM_EXAMPLE),
            ("gizmo_basics", GIZMO_EXAMPLE),
            ("imui_action_basics", IMUI_ACTION_EXAMPLE),
            ("icons_and_assets_basics", ICONS_AND_ASSETS_EXAMPLE),
            ("hello", HELLO_EXAMPLE),
            ("hello_counter", HELLO_COUNTER_EXAMPLE),
            ("markdown_and_code_basics", MARKDOWN_AND_CODE_EXAMPLE),
            ("overlay_basics", OVERLAY_EXAMPLE),
            ("payload_actions_basics", PAYLOAD_ACTIONS_EXAMPLE),
            ("query_basics", QUERY_EXAMPLE),
            ("router_basics", ROUTER_EXAMPLE),
            ("simple_todo", SIMPLE_TODO_EXAMPLE),
            ("simple_todo_v2_target", SIMPLE_TODO_V2_TARGET_EXAMPLE),
            ("async_inbox_basics", ASYNC_INBOX_EXAMPLE),
            ("assets_reload_epoch_basics", ASSETS_RELOAD_EPOCH_EXAMPLE),
            ("canvas_pan_zoom_basics", CANVAS_PAN_ZOOM_EXAMPLE),
            ("chart_interactions_basics", CHART_INTERACTIONS_EXAMPLE),
            ("theme_switching_basics", THEME_SWITCHING_EXAMPLE),
            ("customv1_basics", CUSTOM_V1_EXAMPLE),
            ("text_input_basics", TEXT_INPUT_EXAMPLE),
            ("toast_basics", TOAST_EXAMPLE),
            ("toggle_basics", TOGGLE_EXAMPLE),
            ("undo_basics", UNDO_EXAMPLE),
            (
                "utility_window_materials_windows",
                UTILITY_WINDOW_MATERIALS_EXAMPLE,
            ),
            ("virtual_list_basics", VIRTUAL_LIST_EXAMPLE),
        ] {
            for (line_idx, line) in src.lines().enumerate() {
                let trimmed = line.trim();
                if !(trimmed.contains("shadcn::raw::") || trimmed.contains("fret::shadcn::raw::")) {
                    continue;
                }

                let allowed = trimmed.contains("fret::shadcn::raw::prelude::");
                assert!(
                    allowed,
                    "{name}:{} used an undocumented shadcn raw escape hatch: {}",
                    line_idx + 1,
                    trimmed
                );
            }
        }
    }

    #[test]
    fn cookbook_examples_keep_setup_on_named_installers() {
        for path in cookbook_rust_sources() {
            let source = std::fs::read_to_string(&path).unwrap();
            assert_setup_surface_keeps_inline_closures_off_setup(&source);
        }
    }

    #[test]
    fn docs_and_examples_prefer_builder_then_run() {
        for src in [
            ROOT_README,
            GOLDEN_PATH_DOC,
            HELLO_EXAMPLE,
            SIMPLE_TODO_EXAMPLE,
            SIMPLE_TODO_V2_TARGET_EXAMPLE,
            HELLO_COUNTER_EXAMPLE,
            TEXT_INPUT_EXAMPLE,
            TOGGLE_EXAMPLE,
            PAYLOAD_ACTIONS_EXAMPLE,
            FORM_EXAMPLE,
            DATE_PICKER_EXAMPLE,
            COMMANDS_KEYMAP_EXAMPLE,
            OVERLAY_EXAMPLE,
            THEME_SWITCHING_EXAMPLE,
            TOAST_EXAMPLE,
            VIRTUAL_LIST_EXAMPLE,
            ASYNC_INBOX_EXAMPLE,
            QUERY_EXAMPLE,
            ROUTER_EXAMPLE,
            DATA_TABLE_EXAMPLE,
            UNDO_EXAMPLE,
            MARKDOWN_AND_CODE_EXAMPLE,
            IMUI_ACTION_EXAMPLE,
            DRAG_EXAMPLE,
            EFFECTS_LAYER_EXAMPLE,
            DROP_SHADOW_EXAMPLE,
            ICONS_AND_ASSETS_EXAMPLE,
            ASSETS_RELOAD_EPOCH_EXAMPLE,
            CANVAS_PAN_ZOOM_EXAMPLE,
            CUSTOM_V1_EXAMPLE,
        ] {
            assert_prefers_view_builder_then_run(src);
        }
    }

    #[test]
    fn onboarding_docs_use_the_new_app_surface() {
        assert_uses_app_surface(ROOT_README);
        assert_uses_app_surface_doc(GOLDEN_PATH_DOC);
        assert!(ROOT_README.contains("cx.state().local::<String>()"));
        assert!(ROOT_README.contains("cx.actions().local_set::<act::Add, String>"));
        assert!(GOLDEN_PATH_DOC.contains("cx.state().local::<String>()"));
        assert!(GOLDEN_PATH_DOC.contains("cx.actions().local_set::<act::Add, String>"));
    }
}
