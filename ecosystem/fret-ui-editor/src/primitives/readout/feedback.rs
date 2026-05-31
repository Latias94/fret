use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, FontWeight, Px, TextAlign, TextStyle};
use fret_ui::element::{LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui_kit::typography;

pub(crate) fn editor_status_badge_text_props(
    text: Arc<str>,
    color: Color,
    badge_h: Px,
) -> TextProps {
    TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(badge_h),
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(typography::as_control_text(TextStyle {
            size: Px(9.0),
            weight: FontWeight::MEDIUM,
            line_height: Some(badge_h),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Center,
        ink_overflow: Default::default(),
    }
}

pub(crate) fn editor_inline_error_text_props(
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
        style: Some(typography::as_control_text(TextStyle {
            size: Px(10.0),
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

pub(crate) fn editor_validation_message_text_props(
    text: Arc<str>,
    color: Color,
    text_style: TextStyle,
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
        style: Some(typography::as_content_text(text_style)),
        color: Some(color),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    }
}
