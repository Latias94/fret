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
    let props =
        editor_property_group_header_text_props(Arc::from("Transform Controls"), color, Px(24.0));

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
