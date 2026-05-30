//! Shared compact readout styling for non-edit editor text.
//!
//! This keeps trailing value/outcome labels on one subdued baseline without forcing a shared
//! container geometry across controls and proof surfaces.

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, FontWeight, Px, TextAlign, TextStyle};
use fret_ui::Theme;
use fret_ui::element::{FlexItemStyle, LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui_kit::typography;

use super::colors::editor_muted_foreground;
use super::style::EditorStyle;
mod popup_list;
mod theme_preset;

pub(crate) use popup_list::{
    editor_popup_empty_text_props, editor_popup_list_centered_row_text_props,
    editor_popup_list_option_caption_text_props, editor_popup_list_row_text_props,
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

pub(crate) fn editor_property_group_header_text_props(
    text: Arc<str>,
    color: Color,
    header_height: Px,
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
            size: Px(12.0),
            weight: FontWeight::SEMIBOLD,
            line_height: Some(header_height),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    }
}

pub(crate) fn editor_inspector_panel_title_text_props(
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
            flex: FlexItemStyle {
                order: 0,
                grow: 1.0,
                shrink: 1.0,
                basis: Length::Px(Px(0.0)),
                align_self: None,
            },
            ..Default::default()
        },
        text,
        style: Some(typography::as_control_text(TextStyle {
            size: Px(12.0),
            weight: FontWeight::SEMIBOLD,
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

pub(crate) fn editor_property_row_label_text_props(
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
            size: Px(11.0),
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

pub(crate) fn editor_inline_control_label_text_props(
    text: Arc<str>,
    color: Color,
    text_px: Px,
    line_height: Px,
) -> TextProps {
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
            size: text_px,
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

pub(crate) fn editor_input_segment_text_props(
    text: Arc<str>,
    color: Color,
    text_px: Px,
    line_height: Px,
) -> TextProps {
    TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Fill,
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(typography::as_control_text(TextStyle {
            size: text_px,
            line_height: Some(line_height),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    }
}

pub(crate) fn editor_input_value_text_props(
    text: Arc<str>,
    color: Color,
    text_px: Px,
    line_height: Px,
    height: Length,
) -> TextProps {
    TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height,
                min_width: Some(Length::Px(Px(0.0))),
                ..Default::default()
            },
            flex: FlexItemStyle {
                order: 0,
                grow: 1.0,
                shrink: 1.0,
                basis: Length::Px(Px(0.0)),
                align_self: None,
            },
            ..Default::default()
        },
        text,
        style: Some(typography::as_control_text(TextStyle {
            size: text_px,
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

pub(crate) fn editor_axis_marker_text_props(
    text: Arc<str>,
    color: Color,
    line_height: Px,
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
            line_height: Some(line_height),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Center,
        ink_overflow: Default::default(),
    }
}

pub(crate) fn editor_property_row_reset_glyph_text_props(
    text: Arc<str>,
    color: Color,
    line_height: Px,
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
            size: Px(11.0),
            weight: FontWeight::SEMIBOLD,
            line_height: Some(line_height),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: TextAlign::Center,
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
mod tests {
    use std::sync::Arc;

    use super::{
        compact_readout_text_px, editor_axis_marker_text_props, editor_empty_state_text_props,
        editor_inline_control_label_text_props, editor_inline_error_text_props,
        editor_input_segment_text_props, editor_input_value_text_props,
        editor_inspector_panel_title_text_props, editor_preview_caption_text_props,
        editor_property_group_header_text_props, editor_property_row_label_text_props,
        editor_property_row_reset_glyph_text_props, editor_section_badge_text_props,
        editor_section_heading_text_props, editor_status_badge_text_props,
        editor_tooltip_readout_text_props, editor_validation_message_text_props,
    };
    use fret_core::{Color, FontWeight, Px, TextAlign, TextOverflow, TextStyle, TextWrap};
    use fret_ui::element::Length;

    #[test]
    fn compact_readout_text_px_keeps_floor_for_small_base_sizes() {
        assert_eq!(compact_readout_text_px(Px(10.0)), Px(11.0));
        assert_eq!(compact_readout_text_px(Px(11.0)), Px(11.0));
    }

    #[test]
    fn compact_readout_text_px_trims_one_step_from_primary_text() {
        assert_eq!(compact_readout_text_px(Px(12.0)), Px(11.0));
        assert_eq!(compact_readout_text_px(Px(14.0)), Px(13.0));
    }

    #[test]
    fn editor_status_badge_text_uses_compact_single_line_readout_role() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_status_badge_text_props(Arc::from("Loading"), color, Px(14.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Auto);
        assert_eq!(props.layout.size.height, Length::Px(Px(14.0)));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Center);

        let style = props.style.expect("status badge text should set style");
        assert_eq!(style.size, Px(9.0));
        assert_eq!(style.weight, FontWeight::MEDIUM);
        assert_eq!(style.line_height, Some(Px(14.0)));
    }

    #[test]
    fn editor_inline_error_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xFF_44_44);
        let props = editor_inline_error_text_props(Arc::from("Invalid hex color"), color, Px(20.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props.style.expect("inline error text should set style");
        assert_eq!(style.size, Px(10.0));
        assert_eq!(style.line_height, Some(Px(20.0)));
    }

    #[test]
    fn editor_validation_message_text_wraps_and_shrinks() {
        let color = Color::from_srgb_hex_rgb(0xFF_44_44);
        let props = editor_validation_message_text_props(
            Arc::from("Value must be between 0 and 1"),
            color,
            TextStyle {
                size: Px(12.0),
                line_height: Some(Px(16.0)),
                ..Default::default()
            },
        );

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(props.align, TextAlign::Start);

        let style = props
            .style
            .expect("validation message text should set style");
        assert_eq!(style.size, Px(12.0));
        assert_eq!(style.line_height, Some(Px(16.0)));
    }

    #[test]
    fn editor_section_badge_text_is_single_line_centered_badge_label() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_section_badge_text_props(Arc::from("P"), color, Px(22.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Fill);
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(props.align, TextAlign::Center);

        let style = props.style.expect("section badge text should set style");
        assert_eq!(style.size, Px(11.0));
        assert_eq!(style.weight, FontWeight::SEMIBOLD);
        assert_eq!(style.line_height, Some(Px(22.0)));
    }

    #[test]
    fn editor_section_heading_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_section_heading_text_props(Arc::from("Position"), color);

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props.style.expect("section heading text should set style");
        assert_eq!(style.size, Px(11.0));
        assert_eq!(style.weight, FontWeight::SEMIBOLD);
        assert_eq!(style.line_height, Some(Px(14.0)));
    }

    #[test]
    fn editor_property_group_header_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_property_group_header_text_props(
            Arc::from("Transform Controls"),
            color,
            Px(24.0),
        );

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props
            .style
            .expect("property group header text should set style");
        assert_eq!(style.size, Px(12.0));
        assert_eq!(style.weight, FontWeight::SEMIBOLD);
        assert_eq!(style.line_height, Some(Px(24.0)));
    }

    #[test]
    fn editor_inspector_panel_title_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_inspector_panel_title_text_props(
            Arc::from("Material Inspector With Long Asset Name"),
            color,
            Px(22.0),
        );

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props
            .style
            .expect("inspector panel title text should set style");
        assert_eq!(style.size, Px(12.0));
        assert_eq!(style.weight, FontWeight::SEMIBOLD);
        assert_eq!(style.line_height, Some(Px(22.0)));
    }

    #[test]
    fn editor_inline_control_label_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props =
            editor_inline_control_label_text_props(Arc::from("Uniform"), color, Px(10.0), Px(12.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Auto);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props
            .style
            .expect("inline control label text should set style");
        assert_eq!(style.size, Px(10.0));
        assert_eq!(style.line_height, Some(Px(12.0)));
    }

    #[test]
    fn editor_input_segment_text_keeps_fixed_segment_line_box() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_input_segment_text_props(Arc::from("m/s"), color, Px(10.0), Px(22.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Auto);
        assert_eq!(props.layout.size.height, Length::Fill);
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(props.align, TextAlign::Start);

        let style = props.style.expect("input segment text should set style");
        assert_eq!(style.size, Px(10.0));
        assert_eq!(style.line_height, Some(Px(22.0)));
    }

    #[test]
    fn editor_input_value_text_props_are_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_input_value_text_props(
            Arc::from("123456789.123456789"),
            color,
            Px(12.0),
            Px(22.0),
            Length::Fill,
        );

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.layout.flex.grow, 1.0);
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props.style.expect("input value text should set style");
        assert_eq!(style.size, Px(12.0));
        assert_eq!(style.line_height, Some(Px(22.0)));
    }

    #[test]
    fn editor_axis_marker_text_keeps_fixed_centered_line_box() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_axis_marker_text_props(Arc::from("X"), color, Px(22.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Fill);
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(props.align, TextAlign::Center);

        let style = props.style.expect("axis marker text should set style");
        assert_eq!(style.size, Px(11.0));
        assert_eq!(style.weight, FontWeight::SEMIBOLD);
        assert_eq!(style.line_height, Some(Px(22.0)));
    }

    #[test]
    fn editor_property_row_label_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_property_row_label_text_props(
            Arc::from("Very Long Property Label"),
            color,
            Px(20.0),
        );

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props
            .style
            .expect("property row label text should set style");
        assert_eq!(style.size, Px(11.0));
        assert_eq!(style.line_height, Some(Px(20.0)));
    }

    #[test]
    fn editor_property_row_reset_glyph_text_keeps_fixed_button_line_box() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_property_row_reset_glyph_text_props(Arc::from("R"), color, Px(18.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(props.align, TextAlign::Center);

        let style = props
            .style
            .expect("property row reset glyph text should set style");
        assert_eq!(style.size, Px(11.0));
        assert_eq!(style.weight, FontWeight::SEMIBOLD);
        assert_eq!(style.line_height, Some(Px(18.0)));
    }

    #[test]
    fn editor_preview_caption_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_preview_caption_text_props(Arc::from("Original"), color);

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props.style.expect("preview caption text should set style");
        assert_eq!(style.size, Px(10.0));
        assert_eq!(style.line_height, Some(Px(12.0)));
    }

    #[test]
    fn editor_empty_state_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_empty_state_text_props(Arc::from("No stops"), color, Px(20.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props.style.expect("empty state text should set style");
        assert_eq!(style.size, Px(11.0));
        assert_eq!(style.line_height, Some(Px(20.0)));
    }

    #[test]
    fn editor_tooltip_readout_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_tooltip_readout_text_props(Arc::from("#AABBCC"), color);

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Auto);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props.style.expect("tooltip readout text should set style");
        assert_eq!(style.size, Px(10.0));
        assert_eq!(style.line_height, Some(Px(13.0)));
    }
}
