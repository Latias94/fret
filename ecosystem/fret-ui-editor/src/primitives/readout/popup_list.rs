use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, Px, TextAlign, TextStyle};
use fret_ui::element::{LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui_kit::typography;

#[cfg(test)]
mod tests;

fn editor_popup_list_row_text_style(row_height: Px) -> TextStyle {
    typography::as_control_text(TextStyle {
        size: Px(12.0),
        line_height: Some(row_height),
        ..Default::default()
    })
}

fn editor_popup_list_text_props(
    text: Arc<str>,
    color: Color,
    row_height: Px,
    height: Length,
    align: TextAlign,
) -> TextProps {
    TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height,
                min_width: Some(Length::Px(Px(0.0))),
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(editor_popup_list_row_text_style(row_height)),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align,
        ink_overflow: Default::default(),
    }
}

pub(crate) fn editor_popup_list_row_text_props(
    text: Arc<str>,
    color: Color,
    row_height: Px,
) -> TextProps {
    editor_popup_list_text_props(text, color, row_height, Length::Fill, TextAlign::Start)
}

pub(crate) fn editor_popup_list_centered_row_text_props(
    text: Arc<str>,
    color: Color,
    row_height: Px,
) -> TextProps {
    editor_popup_list_text_props(text, color, row_height, Length::Fill, TextAlign::Center)
}

pub(crate) fn editor_popup_list_option_caption_text_props(
    text: Arc<str>,
    color: Color,
    row_height: Px,
) -> TextProps {
    editor_popup_list_text_props(
        text,
        color,
        row_height,
        Length::Px(row_height),
        TextAlign::Center,
    )
}

pub(crate) fn editor_popup_empty_text_props(
    text: Arc<str>,
    color: Color,
    row_height: Px,
) -> TextProps {
    TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                min_width: Some(Length::Px(Px(0.0))),
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(editor_popup_list_row_text_style(row_height)),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    }
}
