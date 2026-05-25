//! Immediate-mode text input and textarea helpers.

use std::sync::Arc;

use fret_core::NodeId;
use fret_runtime::{CommandId, Effect};
use fret_ui::UiHost;
use fret_ui::element::Length;

use super::{InputTextMode, InputTextOptions, ResponseExt, TextAreaOptions, UiWriterImUiFacadeExt};

mod focus;
mod policy_commands;
mod style;

use focus::sync_select_all_on_focus;
use policy_commands::{install_input_text_policy_commands, install_textarea_policy_commands};
use style::{
    default_input_text_style_from_theme, imui_text_area_style_from_theme,
    imui_text_input_style_from_theme, input_text_layout,
};

fn text_model_changed_for<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    current: &str,
) -> bool {
    super::model_value_changed_for(cx, id, current.to_string())
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct InputTextAssistiveSemantics {
    pub active_descendant: Option<NodeId>,
    pub active_descendant_element: Option<u64>,
    pub controls_element: Option<u64>,
    pub expanded: Option<bool>,
}

pub(super) fn input_text_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
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

pub(super) fn input_text_model_element_with_options<H: UiHost>(
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

pub(super) fn input_text_model_element_with_options_and_semantics<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    model: fret_runtime::Model<String>,
    options: InputTextOptions,
    assistive_semantics: InputTextAssistiveSemantics,
    response: &mut ResponseExt,
) -> fret_ui::element::AnyElement {
    let enabled = options.enabled && !super::imui_is_disabled(cx);
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
        super::populate_response_lifecycle_from_active_state(cx, id, focused, changed, response);
        sync_select_all_on_focus(
            cx,
            id,
            focused,
            !current.is_empty(),
            options.select_all_on_focus,
        );
        let select_all_requested = cx.take_transient_for(id, super::KEY_SELECT_ALL_ON_FOCUS);
        if select_all_requested && options.select_all_on_focus && focused {
            cx.app.push_effect(Effect::Command {
                window: Some(cx.window),
                command: CommandId::from("edit.select_all"),
            });
        }

        let mut props = fret_ui::element::TextInputProps::new(model.clone());
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        props.read_only = options.read_only;
        props.obscure_text = matches!(options.mode, InputTextMode::Password);
        props.layout = input_text_layout();
        props.a11y_label = options.a11y_label.clone();
        props.a11y_role = options.a11y_role;
        props.active_descendant = assistive_semantics.active_descendant;
        props.active_descendant_element = assistive_semantics.active_descendant_element;
        props.controls_element = assistive_semantics.controls_element;
        props.expanded = assistive_semantics.expanded;
        props.test_id = options.test_id.clone();
        props.placeholder = options.placeholder.clone();
        props.submit_command = options.submit_command.clone();
        props.cancel_command = options.cancel_command.clone();
        if !options.filters.is_empty() || options.custom_filter.is_some() {
            let filters = options.filters;
            let custom_filter = options.custom_filter.clone();
            props.insert_filter = Some(Arc::new(move |text| {
                let filtered = filters.filter_text(text);
                match custom_filter.as_ref() {
                    Some(filter) => filter.filter_text(&filtered),
                    None => filtered,
                }
            }));
        }
        let (chrome, text_style) = {
            let theme = fret_ui::Theme::global(&*cx.app);
            (
                imui_text_input_style_from_theme(theme),
                default_input_text_style_from_theme(theme),
            )
        };
        props.chrome = chrome;
        props.text_style = text_style;

        let mut element = cx.text_input(props);
        element.id = id;
        install_input_text_policy_commands(cx, id, &options);
        element
    })
}

pub(super) fn textarea_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    model: &fret_runtime::Model<String>,
    options: TextAreaOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let enabled = options.enabled && !super::imui_is_disabled(cx);
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
            super::populate_response_lifecycle_from_active_state(
                cx,
                id,
                focused,
                changed,
                &mut response,
            );
            sync_select_all_on_focus(
                cx,
                id,
                focused,
                !current.is_empty(),
                options.select_all_on_focus,
            );
            let select_all_requested = cx.take_transient_for(id, super::KEY_SELECT_ALL_ON_FOCUS);
            if select_all_requested && options.select_all_on_focus && focused {
                cx.app.push_effect(Effect::Command {
                    window: Some(cx.window),
                    command: CommandId::from("edit.select_all"),
                });
            }

            let mut props = fret_ui::element::TextAreaProps::new(model.clone());
            props.enabled = enabled;
            props.focusable = enabled && options.focusable;
            props.read_only = options.read_only;
            props.allow_tab_input = options.allow_tab_input;
            props.layout.size.width = Length::Fill;
            props.a11y_label = options.a11y_label.clone();
            props.test_id = options.test_id.clone();
            props.min_height = options.min_height;
            let (chrome, text_style) = {
                let theme = fret_ui::Theme::global(&*cx.app);
                let chrome = imui_text_area_style_from_theme(theme);
                let text_style = if options.stable_line_boxes {
                    crate::typography::text_area_control_text_style(theme)
                } else {
                    crate::typography::text_area_content_text_style(theme)
                };
                (chrome, text_style)
            };
            props.chrome = chrome;
            props.text_style = text_style;

            let mut element = cx.text_area(props);
            element.id = id;
            install_textarea_policy_commands(cx, id, &options);
            element
        })
    });

    ui.add(element);
    response
}

#[cfg(test)]
mod tests;
