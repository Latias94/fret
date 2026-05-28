use fret_runtime::{CommandId, Effect};
use fret_ui::UiHost;

mod props;

pub(in crate::imui) use props::InputTextAssistiveSemantics;
use props::input_text_props;

use super::super::{InputTextOptions, ResponseExt, UiWriterImUiFacadeExt};
use super::focus::sync_select_all_on_focus;
use super::policy_commands::install_input_text_policy_commands;

pub(super) fn text_model_changed_for<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    current: &str,
) -> bool {
    super::super::model_value_changed_for(cx, id, current.to_string())
}

pub(in crate::imui) fn input_text_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    model: &fret_runtime::Model<String>,
    options: InputTextOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();
    let element = ui
        .with_cx_mut(|cx| input_text_model_element_with_options(cx, model, options, &mut response));

    ui.add(element);
    response
}

fn input_text_model_element_with_options<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    model: fret_runtime::Model<String>,
    options: InputTextOptions,
    response: &mut ResponseExt,
) -> fret_ui::element::AnyElement {
    input_text_model_element_with_options_and_semantics(
        cx,
        model,
        options,
        InputTextAssistiveSemantics::default(),
        response,
    )
}

pub(in crate::imui) fn input_text_model_element_with_options_and_semantics<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    model: fret_runtime::Model<String>,
    options: InputTextOptions,
    assistive_semantics: InputTextAssistiveSemantics,
    response: &mut ResponseExt,
) -> fret_ui::element::AnyElement {
    let enabled = options.enabled && !super::super::imui_is_disabled(cx);
    cx.scope(|cx| {
        let id = cx.root_id();
        let current = cx
            .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
            .unwrap_or_default();

        response.set_id(Some(id));
        response.set_enabled(enabled);
        let focused = enabled && cx.is_focused_element(id);
        let changed = enabled && text_model_changed_for(cx, id, &current);
        response.set_core_focused(focused);
        response.set_core_changed(changed);
        response.set_core_rect(cx.last_bounds_for_element(id));
        super::super::populate_response_lifecycle_from_active_state(
            cx, id, focused, changed, response,
        );
        sync_select_all_on_focus(
            cx,
            id,
            focused,
            !current.is_empty(),
            options.select_all_on_focus,
        );
        let select_all_requested = cx.take_transient_for(id, super::super::KEY_SELECT_ALL_ON_FOCUS);
        if select_all_requested && options.select_all_on_focus && focused {
            cx.app.push_effect(Effect::Command {
                window: Some(cx.window),
                command: CommandId::from("edit.select_all"),
            });
        }

        let props = input_text_props(cx, model.clone(), enabled, &options, assistive_semantics);
        let mut element = cx.text_input(props);
        element.id = id;
        install_input_text_policy_commands(cx, id, &options);
        element
    })
}
