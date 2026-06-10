use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret_app::Effect;
use fret_ui::element::AnyElement;
use fret_ui_kit::IntoUiElementInExt as _;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::facade as shadcn;

use super::catalog::{WorkbenchQuickActionSpec, workbench_quick_action_command_bundle_text};

const TEST_ID_ACTION_COPY_SELECTED: &str = "imui-editor-workbench.action.copy-selected-command";
const TEST_ID_ACTION_COPY_BUNDLE: &str = "imui-editor-workbench.action.copy-command-bundle";
const TEST_ID_ACTION_COPY_STATUS: &str = "imui-editor-workbench.action-copy-status";

pub(super) fn initial_workbench_copy_status() -> String {
    "Ready to copy the selected command or the full command bundle.".to_string()
}

pub(super) fn render_workbench_quick_action_copy_buttons(
    cx: &mut AppUi<'_, '_>,
    active_spec: WorkbenchQuickActionSpec,
    copy_status_state: LocalState<String>,
) -> [AnyElement; 2] {
    let copy_selected = shadcn::Button::new("Copy command")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(workbench_copy_text_on_activate(
            active_spec.command.to_string(),
            copy_status_state.clone(),
            format!("Copied {} command.", active_spec.label),
        ))
        .test_id(TEST_ID_ACTION_COPY_SELECTED)
        .ui()
        .into_element_in(cx);

    let copy_bundle = shadcn::Button::new("Copy commands")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .on_activate(workbench_copy_text_on_activate(
            workbench_quick_action_command_bundle_text(),
            copy_status_state,
            "Copied Demo/Metrics/Debug command bundle.".to_string(),
        ))
        .test_id(TEST_ID_ACTION_COPY_BUNDLE)
        .ui()
        .into_element_in(cx);

    [copy_selected, copy_bundle]
}

pub(super) fn render_workbench_quick_action_copy_status<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    copy_status: String,
) -> AnyElement {
    decl_text::text_control_readout(cx, copy_status).test_id(TEST_ID_ACTION_COPY_STATUS)
}

fn workbench_copy_text_on_activate(
    text: String,
    copy_status: LocalState<String>,
    next_status: String,
) -> fret_ui::action::OnActivate {
    Arc::new(move |host, action_cx, _reason| {
        let token = host.next_clipboard_token();
        host.push_effect(Effect::ClipboardWriteText {
            window: action_cx.window,
            token,
            text: text.clone(),
        });
        let _ = copy_status.set_in(host.models_mut(), next_status.clone());
        host.request_redraw(action_cx.window);
    })
}
