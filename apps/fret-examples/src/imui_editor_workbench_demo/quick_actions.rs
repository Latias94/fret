use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret_app::Effect;
use fret_ui::element::AnyElement;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::{IntoUiElementInExt as _, Space};
use fret_ui_shadcn::facade as shadcn;

mod catalog;

use catalog::{
    WORKBENCH_QUICK_ACTIONS, WorkbenchQuickAction, install_workbench_quick_action_commands,
    workbench_quick_action_command, workbench_quick_action_command_bundle_text,
    workbench_quick_action_spec,
};

const TEST_ID_ACTION_STRIP: &str = "imui-editor-workbench.action-strip";
const TEST_ID_ACTION_BUTTONS: &str = "imui-editor-workbench.action-buttons";
const TEST_ID_ACTION_STATUS: &str = "imui-editor-workbench.action-status";
const TEST_ID_ACTION_COMMAND: &str = "imui-editor-workbench.action-command";
const TEST_ID_ACTION_COPY_SELECTED: &str = "imui-editor-workbench.action.copy-selected-command";
const TEST_ID_ACTION_COPY_BUNDLE: &str = "imui-editor-workbench.action.copy-command-bundle";
const TEST_ID_ACTION_COPY_STATUS: &str = "imui-editor-workbench.action-copy-status";

pub(super) fn render_workbench_quick_action_strip(cx: &mut AppUi<'_, '_>) -> AnyElement {
    let active_action = cx.state().local_init(WorkbenchQuickAction::default);
    let copy_status_state = cx.state().local_init(|| {
        "Ready to copy the selected command or the full command bundle.".to_string()
    });
    install_workbench_quick_action_commands(cx, &active_action);

    let active_action = cx.state().watch(&active_action).layout().value_or_default();
    let copy_status = copy_status_state.layout_value(cx);

    render_workbench_quick_action_strip_with_state(
        cx,
        active_action,
        copy_status_state,
        copy_status,
    )
}

fn workbench_text_section<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_section_chrome_label(cx, text)
}

fn workbench_text_paragraph<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_paragraph(cx, text)
}

fn workbench_text_readout<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_control_readout(cx, text)
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

fn render_workbench_quick_action_strip_with_state(
    cx: &mut AppUi<'_, '_>,
    active: WorkbenchQuickAction,
    copy_status_state: LocalState<String>,
    copy_status: String,
) -> AnyElement {
    let active_spec = *workbench_quick_action_spec(active);
    let mut action_buttons = WORKBENCH_QUICK_ACTIONS
        .iter()
        .map(|spec| {
            let selected = spec.action == active;
            let variant = if selected {
                shadcn::ButtonVariant::Default
            } else if spec.primary {
                shadcn::ButtonVariant::Secondary
            } else {
                shadcn::ButtonVariant::Outline
            };
            shadcn::Button::new(spec.label)
                .variant(variant)
                .size(shadcn::ButtonSize::Sm)
                .on_click(workbench_quick_action_command(spec.action))
                .test_id(spec.test_id)
                .ui()
                .into_element_in(cx)
        })
        .collect::<Vec<_>>();
    action_buttons.push(
        shadcn::Button::new("Copy command")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .on_activate(workbench_copy_text_on_activate(
                active_spec.command.to_string(),
                copy_status_state.clone(),
                format!("Copied {} command.", active_spec.label),
            ))
            .test_id(TEST_ID_ACTION_COPY_SELECTED)
            .ui()
            .into_element_in(cx),
    );
    action_buttons.push(
        shadcn::Button::new("Copy commands")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .on_activate(workbench_copy_text_on_activate(
                workbench_quick_action_command_bundle_text(),
                copy_status_state,
                "Copied Demo/Metrics/Debug command bundle.".to_string(),
            ))
            .test_id(TEST_ID_ACTION_COPY_BUNDLE)
            .ui()
            .into_element_in(cx),
    );
    let action_buttons = ui::h_flex(move |_cx| action_buttons)
        .gap(Space::N2)
        .items_center()
        .into_element_in(cx)
        .test_id(TEST_ID_ACTION_BUTTONS);

    let title = ui::v_flex(|cx| {
        ui::children![
            cx;
            workbench_text_section(cx, "Demo / Metrics / Debug"),
            workbench_text_paragraph(
                cx,
                "Canonical editor workbench actions stay visible next to the editor workflow; DevTools and fretboard own execution.",
            ),
        ]
    })
    .gap(Space::N1)
    .flex_1()
    .min_w_0()
    .into_element_in(cx);

    let status = ui::v_flex(move |cx| {
        ui::children![
            cx;
            workbench_text_readout(
                cx,
                format!(
                    "selected: {} | category: {} | primary: {}",
                    active_spec.label, active_spec.category, active_spec.primary
                ),
            ),
            workbench_text_readout(cx, active_spec.purpose),
            workbench_text_readout(cx, active_spec.command).test_id(TEST_ID_ACTION_COMMAND),
            workbench_text_readout(cx, copy_status).test_id(TEST_ID_ACTION_COPY_STATUS),
        ]
    })
    .gap(Space::N1)
    .flex_1()
    .min_w_0()
    .into_element_in(cx)
    .test_id(TEST_ID_ACTION_STATUS);

    ui::h_flex(|cx| ui::children![cx; title, action_buttons, status])
        .gap(Space::N4)
        .items_center()
        .justify_between()
        .w_full()
        .p(Space::N3)
        .rounded_md()
        .border_1()
        .into_element_in(cx)
        .test_id(TEST_ID_ACTION_STRIP)
}
