use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, FontWeight, Px, TextAlign, TextStyle};
use fret_ui::element::{LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui_kit::typography;

use super::compact_readout_text_px;

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        editor_theme_preset_picker_header_text_props,
        editor_theme_preset_picker_row_label_text_props,
        editor_theme_preset_picker_row_status_text_props,
    };
    use fret_core::{Color, FontWeight, Px, TextAlign, TextOverflow, TextWrap};
    use fret_ui::element::Length;

    #[test]
    fn editor_theme_preset_picker_header_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_theme_preset_picker_header_text_props(
            Arc::from("Editor theme preset"),
            color,
            Px(12.0),
        );

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props.style.expect("theme picker header should set style");
        assert_eq!(style.size, Px(11.0));
        assert_eq!(style.line_height, Some(Px(14.0)));
    }

    #[test]
    fn editor_theme_preset_picker_row_label_text_keeps_fixed_row_line_box() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_theme_preset_picker_row_label_text_props(
            Arc::from("ImGui-like dense"),
            color,
            Px(22.0),
            Px(12.0),
        );

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props
            .style
            .expect("theme picker row label should set style");
        assert_eq!(style.size, Px(12.0));
        assert_eq!(style.weight, FontWeight::MEDIUM);
        assert_eq!(style.line_height, Some(Px(22.0)));
    }

    #[test]
    fn editor_theme_preset_picker_row_status_text_keeps_fixed_slot() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_theme_preset_picker_row_status_text_props(
            Arc::from("On"),
            color,
            Px(22.0),
            Px(12.0),
        );

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Px(Px(28.0)));
        assert_eq!(props.layout.size.height, Length::Fill);
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(props.align, TextAlign::End);

        let style = props
            .style
            .expect("theme picker row status should set style");
        assert_eq!(style.size, Px(11.0));
        assert_eq!(style.line_height, Some(Px(22.0)));
    }
}
