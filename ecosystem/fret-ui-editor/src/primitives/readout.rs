//! Shared compact readout styling for non-edit editor text.
//!
//! This keeps trailing value/outcome labels on one subdued baseline without forcing a shared
//! container geometry across controls and proof surfaces.

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, FontWeight, Px, TextAlign, TextStyle};
use fret_ui::Theme;
use fret_ui::element::{LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui_kit::typography;

use super::colors::editor_muted_foreground;
use super::style::EditorStyle;
mod feedback;
mod input;
mod popup_list;
mod property;
mod theme_preset;

pub(crate) use feedback::{
    editor_inline_error_text_props, editor_status_badge_text_props,
    editor_validation_message_text_props,
};
pub(crate) use input::{
    editor_axis_marker_text_props, editor_inline_control_label_text_props,
    editor_input_segment_text_props, editor_input_value_text_props,
};
pub(crate) use popup_list::{
    editor_popup_empty_text_props, editor_popup_list_centered_row_text_props,
    editor_popup_list_option_caption_text_props, editor_popup_list_row_text_props,
};
pub(crate) use property::{
    editor_inspector_panel_title_text_props, editor_property_group_header_text_props,
    editor_property_row_label_text_props, editor_property_row_reset_glyph_text_props,
};
pub(crate) use theme_preset::{
    editor_theme_preset_picker_header_text_props, editor_theme_preset_picker_row_label_text_props,
    editor_theme_preset_picker_row_status_text_props,
};

/// Resolve the compact readout text size from a base control text size.
///
/// The editor baseline keeps readouts one step quieter than primary editable text, but clamps at
/// a conservative floor so dense presets do not become illegible.
pub fn compact_readout_text_px(base_text_px: Px) -> Px {
    Px((base_text_px.0 - 1.0).max(11.0))
}

fn compact_readout_fg(theme: &Theme) -> Color {
    editor_muted_foreground(theme)
}

/// Shared compact non-edit readout text styling.
#[derive(Debug, Clone, Copy)]
pub struct EditorCompactReadoutStyle {
    pub text_px: Px,
    pub line_height: Px,
    pub color: Color,
}

impl EditorCompactReadoutStyle {
    pub fn resolve(theme: &Theme, line_height: Px) -> Self {
        let base_text_px = EditorStyle::resolve(theme).frame_chrome_small().text_px;
        Self {
            text_px: compact_readout_text_px(base_text_px),
            line_height,
            color: compact_readout_fg(theme),
        }
    }

    pub fn text_props(
        self,
        text: Arc<str>,
        layout: LayoutStyle,
        align: TextAlign,
        overflow: TextOverflow,
    ) -> TextProps {
        TextProps {
            layout,
            text,
            style: Some(typography::as_control_text(TextStyle {
                size: self.text_px,
                line_height: Some(self.line_height),
                ..Default::default()
            })),
            color: Some(self.color),
            wrap: TextWrap::None,
            overflow,
            align,
            ink_overflow: Default::default(),
        }
    }
}

pub(crate) fn editor_section_badge_text_props(
    text: Arc<str>,
    color: Color,
    row_height: Px,
) -> TextProps {
    TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(typography::as_control_text(TextStyle {
            size: Px(11.0),
            weight: FontWeight::SEMIBOLD,
            line_height: Some(row_height),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Center,
        ink_overflow: Default::default(),
    }
}

pub(crate) fn editor_section_heading_text_props(text: Arc<str>, color: Color) -> TextProps {
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
            weight: FontWeight::SEMIBOLD,
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

#[cfg(test)]
mod tests;
