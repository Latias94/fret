//! Cookbook examples crate.
//!
//! This crate intentionally keeps a tiny surface:
//! - helpers shared by `examples/`,
//! - no reusable product APIs (those belong in ecosystem crates).

use fret::app::prelude::*;

pub mod scaffold;

pub fn install_cookbook_defaults(app: &mut KernelApp) {
    shadcn::shadcn_themes::apply_shadcn_new_york(
        app,
        shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}

#[cfg(test)]
mod authoring_surface_policy_tests {
    const ROOT_README: &str = include_str!("../../../README.md");
    const GOLDEN_PATH_DOC: &str = include_str!("../../../docs/examples/todo-app-golden-path.md");
    const COMMANDS_KEYMAP_EXAMPLE: &str = include_str!("../examples/commands_keymap_basics.rs");
    const DATA_TABLE_EXAMPLE: &str = include_str!("../examples/data_table_basics.rs");
    const DATE_PICKER_EXAMPLE: &str = include_str!("../examples/date_picker_basics.rs");
    const DRAG_EXAMPLE: &str = include_str!("../examples/drag_basics.rs");
    const DROP_SHADOW_EXAMPLE: &str = include_str!("../examples/drop_shadow_basics.rs");
    const EFFECTS_LAYER_EXAMPLE: &str = include_str!("../examples/effects_layer_basics.rs");
    const FORM_EXAMPLE: &str = include_str!("../examples/form_basics.rs");
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
    const ASYNC_INBOX_EXAMPLE: &str = include_str!("../examples/async_inbox_basics.rs");
    const THEME_SWITCHING_EXAMPLE: &str = include_str!("../examples/theme_switching_basics.rs");
    const TEXT_INPUT_EXAMPLE: &str = include_str!("../examples/text_input_basics.rs");
    const TOAST_EXAMPLE: &str = include_str!("../examples/toast_basics.rs");
    const TOGGLE_EXAMPLE: &str = include_str!("../examples/toggle_basics.rs");
    const UNDO_EXAMPLE: &str = include_str!("../examples/undo_basics.rs");
    const VIRTUAL_LIST_EXAMPLE: &str = include_str!("../examples/virtual_list_basics.rs");

    fn assert_uses_app_surface(src: &str) {
        assert!(src.contains("use fret::app::prelude::*;"));
        assert!(src.contains("KernelApp"));
        assert!(src.contains("AppUi<'_, '_, KernelApp>"));
        assert!(src.contains("-> Ui"));
        assert!(!src.contains("use fret::prelude::*;"));
        assert!(!src.contains("ViewCx<'_, '_, App>"));
        assert!(!src.contains("fn init(_app: &mut App"));
        assert!(!src.contains("-> Elements"));
        assert!(!src.contains("cx.use_local"));
        assert!(!src.contains("cx.on_action_notify_"));
        assert!(!src.contains("cx.on_payload_action_notify_"));
    }

    fn assert_uses_advanced_surface(src: &str) {
        assert!(src.contains("advanced::prelude::*"));
        assert!(src.contains("KernelApp"));
        assert!(src.contains("ViewCx<'_, '_, KernelApp>"));
        assert!(src.contains("-> Elements"));
        assert!(!src.contains("use fret::prelude::*;"));
        assert!(!src.contains("use fret::app::prelude::*;"));
        assert!(!src.contains("AppUi<'_, '_, KernelApp>"));
        assert!(!src.contains("-> Ui"));
    }

    #[test]
    fn onboarding_examples_use_the_new_app_surface() {
        assert_uses_app_surface(HELLO_EXAMPLE);
        assert_uses_app_surface(SIMPLE_TODO_EXAMPLE);
        assert_uses_app_surface(SIMPLE_TODO_V2_TARGET_EXAMPLE);
        assert!(HELLO_EXAMPLE.contains("cx.state().local::<u32>()"));
        assert!(HELLO_EXAMPLE.contains(".local_update::<act::Click, u32>("));
        assert!(SIMPLE_TODO_EXAMPLE.contains("cx.state().local::<String>()"));
        assert!(SIMPLE_TODO_EXAMPLE.contains("cx.actions().locals::<act::Add>"));
        assert!(SIMPLE_TODO_EXAMPLE.contains(".payload::<act::Toggle>()"));
        assert!(
            SIMPLE_TODO_EXAMPLE
                .contains(".local_update_if::<Vec<TodoRow>>(&todos_state, |rows, id| {")
        );
        assert!(SIMPLE_TODO_EXAMPLE.contains("impl UiChild"));
        assert!(SIMPLE_TODO_V2_TARGET_EXAMPLE.contains("impl UiChild"));
        assert!(SIMPLE_TODO_V2_TARGET_EXAMPLE.contains("cx.actions().locals::<act::Add>"));
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
        assert!(QUERY_EXAMPLE.contains("toggle_local_bool::<act::ToggleErrorMode>"));

        assert!(ROUTER_EXAMPLE.contains("models::<act::ClearIntents>"));
        assert!(ROUTER_EXAMPLE.contains("on_action_notify::<act::RouterBack>"));

        assert!(DATA_TABLE_EXAMPLE.contains("use fret_runtime::Model;"));
        assert!(
            DATA_TABLE_EXAMPLE
                .contains("fn render(&mut self, cx: &mut AppUi<'_, '_, KernelApp>) -> Ui")
        );

        assert!(UNDO_EXAMPLE.contains("use fret_app::Effect;"));
        assert!(UNDO_EXAMPLE.contains("models::<act::Inc>"));
        assert!(UNDO_EXAMPLE.contains("on_action_notify::<act::Undo>"));

        assert!(MARKDOWN_AND_CODE_EXAMPLE.contains("MarkdownComponents::<KernelApp>::default()"));
        assert!(MARKDOWN_AND_CODE_EXAMPLE.contains("local_set::<act::Reset, String>"));

        assert!(
            IMUI_ACTION_EXAMPLE
                .contains("use fret_runtime::{CommandId, CommandMeta, CommandScope, Model};")
        );
        assert!(IMUI_ACTION_EXAMPLE.contains("local_update::<act::Inc, u32>"));
    }

    #[test]
    fn shared_scaffold_uses_component_surface_instead_of_legacy_prelude() {
        assert!(SCAFFOLD.contains("use fret::component::prelude::*;"));
        assert!(!SCAFFOLD.contains("use fret::prelude::*;"));
    }

    #[test]
    fn advanced_examples_use_the_explicit_advanced_surface() {
        assert_uses_advanced_surface(DRAG_EXAMPLE);
        assert_uses_advanced_surface(EFFECTS_LAYER_EXAMPLE);
        assert_uses_advanced_surface(DROP_SHADOW_EXAMPLE);
        assert_uses_advanced_surface(ICONS_AND_ASSETS_EXAMPLE);

        assert!(DRAG_EXAMPLE.contains("use fret::{FretApp, advanced::prelude::*, shadcn};"));
        assert!(DRAG_EXAMPLE.contains("UiPointerActionHost"));

        assert!(EFFECTS_LAYER_EXAMPLE.contains("ElementContext<'_, KernelApp>"));
        assert!(EFFECTS_LAYER_EXAMPLE.contains("on_action_notify_model_set::<act::Pixelate"));

        assert!(DROP_SHADOW_EXAMPLE.contains("ElementContext<'_, KernelApp>"));
        assert!(DROP_SHADOW_EXAMPLE.contains("DropShadowV1"));

        assert!(ICONS_AND_ASSETS_EXAMPLE.contains("icon::IconSvgPreloadDiagnostics"));
        assert!(ICONS_AND_ASSETS_EXAMPLE.contains("Effect::RequestAnimationFrame"));
    }

    #[test]
    fn onboarding_docs_use_the_new_app_surface() {
        assert_uses_app_surface(ROOT_README);
        assert_uses_app_surface(GOLDEN_PATH_DOC);
        assert!(ROOT_README.contains("cx.state().local::<String>()"));
        assert!(ROOT_README.contains("cx.actions().local_set::<act::Add, String>"));
        assert!(GOLDEN_PATH_DOC.contains("cx.state().local::<String>()"));
        assert!(GOLDEN_PATH_DOC.contains("cx.actions().local_set::<act::Add, String>"));
    }
}
