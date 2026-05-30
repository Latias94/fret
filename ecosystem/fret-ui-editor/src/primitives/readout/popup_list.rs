use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, Px, TextAlign, TextStyle};
use fret_ui::element::{LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui_kit::typography;

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        editor_popup_empty_text_props, editor_popup_list_centered_row_text_props,
        editor_popup_list_option_caption_text_props, editor_popup_list_row_text_props,
    };
    use fret_core::{Color, Px, TextAlign, TextOverflow, TextWrap};
    use fret_ui::element::Length;

    #[test]
    fn popup_list_row_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props =
            editor_popup_list_row_text_props(Arc::from("Material / Matcap"), color, Px(28.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props.style.expect("popup list row text should set style");
        assert_eq!(style.size, Px(12.0));
        assert_eq!(style.line_height, Some(Px(28.0)));
    }

    #[test]
    fn popup_empty_text_is_single_line_and_shrinkable() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props = editor_popup_empty_text_props(Arc::from("No matches"), color, Px(28.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Auto);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Start);

        let style = props.style.expect("popup empty text should set style");
        assert_eq!(style.size, Px(12.0));
        assert_eq!(style.line_height, Some(Px(28.0)));
    }

    #[test]
    fn popup_list_centered_row_text_keeps_row_fill_and_center_alignment() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props =
            editor_popup_list_centered_row_text_props(Arc::from("Hue Bar"), color, Px(28.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Center);
    }

    #[test]
    fn popup_list_option_caption_text_keeps_fixed_caption_line_box() {
        let color = Color::from_srgb_hex_rgb(0xAA_BB_CC);
        let props =
            editor_popup_list_option_caption_text_props(Arc::from("Hue Wheel"), color, Px(28.0));

        assert_eq!(props.color, Some(color));
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Px(Px(28.0)));
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
        assert_eq!(props.align, TextAlign::Center);
    }
}
