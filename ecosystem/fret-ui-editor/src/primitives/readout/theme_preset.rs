use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, FontWeight, Px, TextAlign, TextStyle};
use fret_ui::element::{LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui_kit::typography;

use super::compact_readout_text_px;

#[cfg(test)]
mod tests;

pub(crate) fn editor_theme_preset_picker_header_text_props(
    text: Arc<str>,
    color: Color,
    text_px: Px,
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
        style: Some(typography::as_control_text(TextStyle {
            size: compact_readout_text_px(text_px),
            line_height: Some(Px(14.0)),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    }
}

pub(crate) fn editor_theme_preset_picker_row_label_text_props(
    text: Arc<str>,
    color: Color,
    row_height: Px,
    text_px: Px,
) -> TextProps {
    TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                min_width: Some(Length::Px(Px(0.0))),
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(typography::as_control_text(TextStyle {
            size: text_px,
            weight: FontWeight::MEDIUM,
            line_height: Some(row_height),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    }
}

pub(crate) fn editor_theme_preset_picker_row_status_text_props(
    text: Arc<str>,
    color: Color,
    row_height: Px,
    text_px: Px,
) -> TextProps {
    TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Px(Px(28.0)),
                height: Length::Fill,
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(typography::as_control_text(TextStyle {
            size: Px((text_px.0 - 1.0).max(10.0)),
            line_height: Some(row_height),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::End,
        ink_overflow: Default::default(),
    }
}
