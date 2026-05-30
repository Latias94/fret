use fret_runtime::{CommandId, Effect, Model};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::super::focus::sync_select_all_on_focus;
use super::super::policy_commands::install_textarea_policy_commands;
use super::super::text_model_changed_for;
use super::props::textarea_props;
use crate::imui::{
    KEY_SELECT_ALL_ON_FOCUS, ResponseExt, TextAreaOptions, imui_is_disabled,
    populate_response_lifecycle_from_active_state,
};

pub(super) fn textarea_model_element_with_options<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    options: TextAreaOptions,
    response: &mut ResponseExt,
) -> AnyElement {
    let enabled = options.enabled && !imui_is_disabled(cx);
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
        populate_response_lifecycle_from_active_state(cx, id, focused, changed, response);
        sync_select_all_on_focus(
            cx,
            id,
            focused,
            !current.is_empty(),
            options.select_all_on_focus,
        );
        let select_all_requested = cx.take_transient_for(id, KEY_SELECT_ALL_ON_FOCUS);
        if select_all_requested && options.select_all_on_focus && focused {
            cx.app.push_effect(Effect::Command {
                window: Some(cx.window),
                command: CommandId::from("edit.select_all"),
            });
        }

        let props = textarea_props(cx, model.clone(), enabled, &options);
        let mut element = cx.text_area(props);
        element.id = id;
        install_textarea_policy_commands(cx, id, &options);
        element
    })
}
