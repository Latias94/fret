//! Immediate-mode text input and textarea helpers.

use fret_ui::UiHost;
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

use super::{InputTextMode, InputTextOptions, ResponseExt, TextAreaOptions, UiWriterImUiFacadeExt};

fn text_model_changed_for<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    current: &str,
) -> bool {
    super::model_value_changed_for(cx, id, current.to_string())
}

fn default_text_area_style_from_theme(theme: &fret_ui::Theme) -> fret_ui::TextAreaStyle {
    let input_style = crate::recipes::input::default_text_input_style(theme);
    let mut preedit_bg_color = input_style.selection_color;
    preedit_bg_color.a = (preedit_bg_color.a * 0.35).clamp(0.0, 1.0);

    fret_ui::TextAreaStyle {
        padding_x: input_style.padding.left,
        padding_y: input_style.padding.top,
        background: input_style.background,
        border: input_style.border,
        border_color: input_style.border_color,
        border_color_focused: input_style.border_color_focused,
        focus_ring: input_style.focus_ring,
        corner_radii: input_style.corner_radii,
        text_color: input_style.text_color,
        placeholder_color: input_style.placeholder_color,
        selection_color: input_style.selection_color,
        caret_color: input_style.caret_color,
        preedit_bg_color,
        preedit_underline_color: input_style.preedit_color,
    }
}

fn default_input_text_style_from_theme(theme: &fret_ui::Theme) -> fret_core::TextStyle {
    crate::typography::control_text_style_for_font_size(
        theme,
        fret_core::FontId::ui(),
        theme
            .metric_by_key("font.size")
            .unwrap_or_else(|| theme.metric_token("font.size")),
    )
}

fn input_text_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Px(super::control_chrome::FIELD_MIN_HEIGHT),
            min_height: Some(Length::Px(super::control_chrome::FIELD_MIN_HEIGHT)),
            max_height: Some(Length::Px(super::control_chrome::FIELD_MIN_HEIGHT)),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub(super) fn input_text_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    model: &fret_runtime::Model<String>,
    options: InputTextOptions,
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

            response.id = Some(id);
            response.enabled = enabled;
            response.core.focused = enabled && cx.is_focused_element(id);
            response.core.changed = enabled && text_model_changed_for(cx, id, &current);
            response.core.rect = cx.last_bounds_for_element(id);
            super::populate_response_lifecycle_from_active_state(
                cx,
                id,
                response.core.focused,
                response.core.changed,
                &mut response,
            );

            let mut props = fret_ui::element::TextInputProps::new(model.clone());
            props.enabled = enabled;
            props.focusable = enabled && options.focusable;
            props.obscure_text = matches!(options.mode, InputTextMode::Password);
            props.layout = input_text_layout();
            props.a11y_label = options.a11y_label.clone();
            props.a11y_role = options.a11y_role;
            props.test_id = options.test_id.clone();
            props.placeholder = options.placeholder.clone();
            props.submit_command = options.submit_command.clone();
            props.cancel_command = options.cancel_command.clone();
            let (chrome, text_style) = {
                let theme = fret_ui::Theme::global(&*cx.app);
                (
                    crate::recipes::input::default_text_input_style(theme),
                    default_input_text_style_from_theme(theme),
                )
            };
            props.chrome = chrome;
            props.text_style = text_style;

            let mut element = cx.text_input(props);
            element.id = id;
            element
        })
    });

    ui.add(element);
    response
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

            response.id = Some(id);
            response.enabled = enabled;
            response.core.focused = enabled && cx.is_focused_element(id);
            response.core.changed = enabled && text_model_changed_for(cx, id, &current);
            response.core.rect = cx.last_bounds_for_element(id);
            super::populate_response_lifecycle_from_active_state(
                cx,
                id,
                response.core.focused,
                response.core.changed,
                &mut response,
            );

            let mut props = fret_ui::element::TextAreaProps::new(model.clone());
            props.enabled = enabled;
            props.focusable = enabled && options.focusable;
            props.layout.size.width = Length::Fill;
            props.a11y_label = options.a11y_label.clone();
            props.test_id = options.test_id.clone();
            props.min_height = options.min_height;
            let (chrome, text_style) = {
                let theme = fret_ui::Theme::global(&*cx.app);
                let chrome = default_text_area_style_from_theme(theme);
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
            element
        })
    });

    ui.add(element);
    response
}
