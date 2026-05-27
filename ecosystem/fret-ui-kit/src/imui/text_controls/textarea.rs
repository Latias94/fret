use fret_runtime::{CommandId, Effect};
use fret_ui::UiHost;
use fret_ui::element::Length;

use super::focus::sync_select_all_on_focus;
use super::policy_commands::install_textarea_policy_commands;
use super::style::imui_text_area_style_from_theme;
use crate::imui::{ResponseExt, TextAreaOptions, UiWriterImUiFacadeExt};

pub(in crate::imui) fn textarea_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    model: &fret_runtime::Model<String>,
    options: TextAreaOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let enabled = options.enabled && !super::super::imui_is_disabled(cx);
        cx.scope(|cx| {
            let id = cx.root_id();
            let current = cx
                .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| v.clone())
                .unwrap_or_default();

            response.set_id(Some(id));
            response.set_enabled(enabled);
            let focused = enabled && cx.is_focused_element(id);
            let changed = enabled && super::text_model_changed_for(cx, id, &current);
            response.set_core_focused(focused);
            response.set_core_changed(changed);
            response.set_core_rect(cx.last_bounds_for_element(id));
            super::super::populate_response_lifecycle_from_active_state(
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
            let select_all_requested =
                cx.take_transient_for(id, super::super::KEY_SELECT_ALL_ON_FOCUS);
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
