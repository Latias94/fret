use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, Px, TextAlign, TextStyle};
use fret_ui::element::{LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui_kit::typography;

pub(crate) fn editor_preview_caption_text_props(text: Arc<str>, color: Color) -> TextProps {
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
            line_height: Some(Px(12.0)),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    }
}

pub(crate) fn editor_empty_state_text_props(
    text: Arc<str>,
    color: Color,
    line_height: Px,
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
            size: Px(11.0),
            line_height: Some(line_height),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    }
}

pub(crate) fn editor_tooltip_readout_text_props(text: Arc<str>, color: Color) -> TextProps {
    TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Auto,
                min_width: Some(Length::Px(Px(0.0))),
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(typography::as_control_text(TextStyle {
            size: Px(10.0),
            line_height: Some(Px(13.0)),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    }
}
