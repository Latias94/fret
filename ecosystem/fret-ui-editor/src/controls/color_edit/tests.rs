use std::collections::BTreeSet;
use std::sync::Arc;

use super::model::{
    ColorNumericInputMode, HsvColor, HueWheelDragTarget, color_from_rgb_preserving_alpha,
    color_numeric_input_modes, hsv_from_color, hsv_numeric_text, hsv_to_color_preserving_alpha,
    hsv_to_rgb, hsv_with_hue_wheel_position, hsv_with_sv_from_local_position, hue_from_local_y,
    hue_wheel_geometry, hue_wheel_rotated_triangle, hue_wheel_sv_cursor_position,
    hue_wheel_target_from_local_position, parse_color_numeric_input, rgb_numeric_text, rgb_to_hsv,
};
use super::popup::copy::{ColorEditCopyFormat, color_copy_entries};
use super::popup::picker::alpha::{alpha_from_local_x, alpha_from_local_y, alpha_percent_text};
use super::popup::preview::{
    SIDE_PREVIEW_SWATCH_HEIGHT, SIDE_PREVIEW_SWATCH_WIDTH, checkerboard_cell_color,
    opaque_preview_color, preview_color_for_alpha_visibility, restore_reference_color,
};
use super::popup::tooltip::color_tooltip_lines;
use super::*;

#[test]
fn color_presets_are_unique_and_hex_formattable() {
    let mut seen = BTreeSet::new();
    let palette = default_color_edit_palette();
    for entry in palette.iter() {
        assert!(
            seen.insert(entry.rgb),
            "duplicate preset rgb for {}",
            entry.name
        );
        let formatted = format_hex(Color::from_srgb_hex_rgb(entry.rgb), false);
        assert_eq!(formatted.len(), 7);
        assert!(formatted.starts_with('#'));
    }
    assert_eq!(palette.len(), 12);
}

#[test]
fn color_edit_options_default_to_the_builtin_palette_source() {
    let options = ColorEditOptions::default();

    assert_eq!(options.palette, default_color_edit_palette());
    assert_eq!(options.palette.len(), 12);
    assert!(options.history.is_empty());
}

#[test]
fn color_edit_palette_entries_are_app_owned_rgb_slots() {
    let custom: Arc<[ColorEditPaletteEntry]> = vec![
        ColorEditPaletteEntry::new("Brand Primary", 0x12_34_56),
        ColorEditPaletteEntry::new("Brand Accent", 0xab_cd_ef),
    ]
    .into();

    let options = ColorEditOptions {
        palette: custom.clone(),
        ..Default::default()
    };

    assert_eq!(options.palette, custom);
    assert_eq!(options.palette[0].name.as_ref(), "Brand Primary");
    assert_eq!(options.palette[1].rgb, 0xab_cd_ef);
}

#[test]
fn color_edit_history_entries_are_app_owned_recent_rgb_slots() {
    let history: Arc<[ColorEditPaletteEntry]> = vec![
        ColorEditPaletteEntry::new("Recent 1", 0xef_44_44),
        ColorEditPaletteEntry::new("Recent 2", 0x3b_82_f6),
    ]
    .into();

    let options = ColorEditOptions {
        history: history.clone(),
        ..Default::default()
    };

    assert_eq!(options.history, history);
    assert_eq!(options.history[0].name.as_ref(), "Recent 1");
}

#[test]
fn color_edit_palette_slot_drop_defaults_to_app_owned_callback_only() {
    let options = ColorEditOptions::default();

    assert!(options.on_palette_slot_drop.is_none());
}

#[test]
fn palette_slot_drop_event_replaces_rgb_and_preserves_slot_metadata() {
    let previous = ColorEditPaletteEntry::new("Saved Slot", 0x00_00_00);
    let mut source = Color::from_srgb_hex_rgb(0xef_44_44);
    source.a = 0.25;
    let payload = ColorEditDragDropPayload::from_color(source, true);

    let event = ColorEditPaletteSlotDrop::new(7, previous.clone(), payload);

    assert_eq!(event.index(), 7);
    assert_eq!(event.previous(), &previous);
    assert_eq!(event.payload(), payload);
    assert_eq!(event.next().name.as_ref(), "Saved Slot");
    assert_eq!(event.next().rgb, 0xef_44_44);
}

#[test]
fn palette_slot_drop_event_ignores_payload_alpha_because_palette_slots_are_rgb() {
    let previous = ColorEditPaletteEntry::new("Alpha Source", 0x12_34_56);
    let mut source = Color::from_srgb_hex_rgb(0x10_b9_81);
    source.a = 0.125;
    let payload = ColorEditDragDropPayload::from_color(source, true);

    let event = ColorEditPaletteSlotDrop::new(1, previous, payload);

    assert_eq!(event.next().rgb, source.to_srgb_hex_rgb());
}

#[test]
fn popup_options_default_to_imgui_like_hue_bar_surface() {
    let options = ColorEditPopupOptions::default();

    assert_eq!(options.picker, ColorEditPopupPicker::HsvHueBar);
    assert_eq!(
        options.numeric_inputs,
        ColorEditPopupNumericInputs::RgbAndHsv
    );
    assert_eq!(
        options.side_preview,
        ColorEditPopupSidePreview::CurrentAndOriginal
    );
    assert!(options.presets);
    assert!(options.alpha_bar);
    assert!(options.picker_options);
    assert!(options.has_visible_content_with_swatches(false, true, false));
    assert!(options.has_visible_content_with_swatches(true, true, false));
    assert!(!options.shows_alpha_bar(false));
    assert!(options.shows_alpha_bar(true));
    assert!(options.shows_picker_options(false));
    assert!(options.shows_picker_options(true));
}

#[test]
fn popup_side_preview_defaults_to_imgui_current_and_original() {
    let options = ColorEditOptions::default();

    assert_eq!(
        options.popup.side_preview,
        ColorEditPopupSidePreview::CurrentAndOriginal
    );
    assert!(options.popup.side_preview.has_visible_content());
    assert!(options.popup.side_preview.shows_original());
}

#[test]
fn popup_side_preview_uses_imgui_three_by_two_color_button_ratio() {
    let ratio = SIDE_PREVIEW_SWATCH_WIDTH.0 / SIDE_PREVIEW_SWATCH_HEIGHT.0;

    assert!((ratio - 1.5).abs() < f32::EPSILON);
}

#[test]
fn alpha_preview_options_cover_imgui_color_button_preview_modes() {
    let options = ColorEditOptions::default();
    assert_eq!(options.alpha_preview, ColorEditAlphaPreview::Checkerboard);
    assert_eq!(
        [
            ColorEditAlphaPreview::Checkerboard,
            ColorEditAlphaPreview::Opaque,
            ColorEditAlphaPreview::NoBackground,
            ColorEditAlphaPreview::Half,
        ]
        .len(),
        4
    );
}

#[test]
fn drag_drop_options_default_to_imgui_enabled_local_payloads() {
    let options = ColorEditOptions::default();

    assert!(options.drag_drop.enabled);
    assert!(!options.drag_drop.cross_window);
}

#[test]
fn tooltip_options_default_to_imgui_hover_preview_enabled() {
    let options = ColorEditOptions::default();

    assert!(options.tooltip.enabled);
    assert!(options.tooltip_test_id.is_none());
}

#[test]
fn copy_options_default_to_imgui_context_copy_enabled() {
    let options = ColorEditOptions::default();

    assert!(options.copy.enabled);
    assert!(options.copy_menu_test_id.is_none());
}

#[test]
fn eyedropper_defaults_to_app_owned_opt_in() {
    let options = ColorEditOptions::default();

    assert!(options.on_eyedropper.is_none());
    assert!(options.eyedropper_test_id.is_none());
}

#[test]
fn eyedropper_request_applies_sample_alpha_by_visibility() {
    let mut current = Color::from_srgb_hex_rgb(0x11_22_33);
    current.a = 0.25;
    let mut sampled = Color::from_srgb_hex_rgb(0xef_44_44);
    sampled.a = 0.75;

    let rgb_only = ColorEditEyedropperRequest::new(current, false).apply_sample(sampled);
    assert_eq!(rgb_only.to_srgb_hex_rgb(), 0xef_44_44);
    assert!((rgb_only.a - current.a).abs() < f32::EPSILON);

    let rgba = ColorEditEyedropperRequest::new(current, true).apply_sample(sampled);
    assert_eq!(rgba.to_srgb_hex_rgb(), 0xef_44_44);
    assert!((rgba.a - sampled.a).abs() < f32::EPSILON);
}

#[test]
fn color_tooltip_lines_match_imgui_hex_rgb_hsv_preview_text() {
    let mut color = Color::from_srgb_hex_rgb(0x33_66_99);
    color.a = 0.5;

    let rgb_lines = color_tooltip_lines(color, false);
    assert_eq!(rgb_lines.len(), 3);
    assert_eq!(rgb_lines[0].as_ref(), "#336699");
    assert_eq!(rgb_lines[1].as_ref(), "RGB 51 102 153");
    assert_eq!(rgb_lines[2].as_ref(), "HSV 210deg | S 67% | V 60%");

    let rgba_lines = color_tooltip_lines(color, true);
    assert_eq!(rgba_lines[0].as_ref(), "#33669980");
    assert_eq!(rgba_lines[1].as_ref(), "RGB 51 102 153 | A 50%");
}

#[test]
fn color_copy_entries_match_imgui_copy_as_payloads() {
    let mut color = Color::from_srgb_hex_rgb(0x33_66_99);
    color.a = 0.5;

    let rgb_entries = color_copy_entries(color, false);
    assert_eq!(rgb_entries.len(), 3);
    assert_eq!(rgb_entries[0].format, ColorEditCopyFormat::FloatTuple);
    assert!(rgb_entries[0].text.ends_with(", 1.000f)"));
    assert_eq!(rgb_entries[1].format, ColorEditCopyFormat::IntTuple);
    assert_eq!(rgb_entries[1].text.as_ref(), "(51,102,153,255)");
    assert_eq!(rgb_entries[2].format, ColorEditCopyFormat::HexRgb);
    assert_eq!(rgb_entries[2].text.as_ref(), "#336699");

    let rgba_entries = color_copy_entries(color, true);
    assert_eq!(rgba_entries.len(), 4);
    assert_eq!(rgba_entries[0].format, ColorEditCopyFormat::FloatTuple);
    assert!(rgba_entries[0].text.ends_with(", 0.500f)"));
    assert_eq!(rgba_entries[1].text.as_ref(), "(51,102,153,128)");
    assert_eq!(rgba_entries[2].text.as_ref(), "#336699");
    assert_eq!(rgba_entries[3].format, ColorEditCopyFormat::HexRgba);
    assert_eq!(rgba_entries[3].text.as_ref(), "#33669980");
}

#[test]
fn drag_drop_payload_shape_tracks_alpha_visibility() {
    let mut color = Color::from_srgb_hex_rgb(0x3b_82_f6);
    color.a = 0.25;

    let rgb = ColorEditDragDropPayload::from_color(color, false);
    assert_eq!(rgb.color(), color);
    assert_eq!(rgb.components(), ColorEditDragDropComponents::Rgb);

    let rgba = ColorEditDragDropPayload::from_color(color, true);
    assert_eq!(rgba.color(), color);
    assert_eq!(rgba.components(), ColorEditDragDropComponents::Rgba);
}

#[test]
fn drag_drop_payload_apply_matches_imgui_col3f_col4f_alpha_rules() {
    let mut target = Color::from_srgb_hex_rgb(0x11_22_33);
    target.a = 0.25;
    let mut source = Color::from_srgb_hex_rgb(0xef_44_44);
    source.a = 0.75;

    let rgb = ColorEditDragDropPayload::from_color(source, false);
    let rgb_applied = apply_color_drop_payload(rgb, target, true);
    assert_eq!(rgb_applied.to_srgb_hex_rgb(), source.to_srgb_hex_rgb());
    assert!((rgb_applied.a - target.a).abs() < f32::EPSILON);

    let rgba = ColorEditDragDropPayload::from_color(source, true);
    let rgba_to_rgb_target = apply_color_drop_payload(rgba, target, false);
    assert_eq!(
        rgba_to_rgb_target.to_srgb_hex_rgb(),
        source.to_srgb_hex_rgb()
    );
    assert!((rgba_to_rgb_target.a - target.a).abs() < f32::EPSILON);

    let rgba_to_rgba_target = apply_color_drop_payload(rgba, target, true);
    assert_eq!(
        rgba_to_rgba_target.to_srgb_hex_rgb(),
        source.to_srgb_hex_rgb()
    );
    assert!((rgba_to_rgba_target.a - source.a).abs() < f32::EPSILON);
}

#[test]
fn popup_options_can_hide_every_popup_affordance() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::Hidden,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: false,
        alpha_bar: false,
        picker_options: false,
    };

    assert!(!options.has_visible_content_with_swatches(false, false, false));
    assert!(!options.has_visible_content_with_swatches(true, false, false));
}

#[test]
fn empty_palette_does_not_count_as_visible_popup_content() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::Hidden,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: true,
        alpha_bar: false,
        picker_options: false,
    };

    assert!(!options.has_visible_content_with_swatches(false, false, false));
    assert!(options.has_visible_content_with_swatches(false, true, false));
}

#[test]
fn non_empty_history_counts_as_visible_popup_content_without_palette() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::Hidden,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: false,
        alpha_bar: false,
        picker_options: false,
    };

    assert!(options.has_visible_content_with_swatches(false, false, true));
}

#[test]
fn popup_runtime_options_are_local_overrides_until_defaults_change() {
    let options = ColorEditPopupOptions::default();
    let mut runtime = options.runtime_defaults();

    runtime.picker = ColorEditPopupPicker::HsvHueWheel;
    runtime.alpha_bar = false;
    runtime.sync_defaults(options.runtime_defaults());

    assert_eq!(runtime.picker, ColorEditPopupPicker::HsvHueWheel);
    assert!(!runtime.alpha_bar);

    let next_defaults = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::HsvHueBar,
        numeric_inputs: ColorEditPopupNumericInputs::Rgb,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: false,
        alpha_bar: false,
        picker_options: true,
    };
    runtime.sync_defaults(next_defaults.runtime_defaults());

    assert_eq!(runtime.picker, ColorEditPopupPicker::HsvHueWheel);
    assert!(!runtime.alpha_bar);

    let changed_picker_defaults = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::HsvHueWheel,
        ..next_defaults
    };
    runtime.sync_defaults(changed_picker_defaults.runtime_defaults());

    assert_eq!(runtime.picker, ColorEditPopupPicker::HsvHueWheel);
    assert!(!runtime.alpha_bar);
}

#[test]
fn popup_runtime_options_do_not_re_enable_hidden_picker_policy() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::Hidden,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: false,
        alpha_bar: true,
        picker_options: true,
    };
    let mut runtime = options.runtime_defaults();
    runtime.picker = ColorEditPopupPicker::HsvHueWheel;

    let effective = options.with_runtime_options(runtime);

    assert_eq!(effective.picker, ColorEditPopupPicker::Hidden);
    assert!(effective.shows_picker_options(true));
    assert!(!effective.shows_picker_options(false));
}

#[test]
fn popup_runtime_options_are_ignored_when_options_surface_is_disabled() {
    let options = ColorEditPopupOptions {
        picker: ColorEditPopupPicker::HsvHueBar,
        numeric_inputs: ColorEditPopupNumericInputs::Hidden,
        side_preview: ColorEditPopupSidePreview::Hidden,
        presets: false,
        alpha_bar: true,
        picker_options: false,
    };
    let mut runtime = options.runtime_defaults();
    runtime.picker = ColorEditPopupPicker::HsvHueWheel;
    runtime.alpha_bar = false;

    let effective = options.with_runtime_options(runtime);

    assert_eq!(effective.picker, ColorEditPopupPicker::HsvHueBar);
    assert!(effective.alpha_bar);
    assert!(!effective.shows_picker_options(true));
}

#[test]
fn popup_numeric_input_modes_are_explicit_and_ordered() {
    assert_eq!(
        color_numeric_input_modes(ColorEditPopupNumericInputs::RgbAndHsv),
        &[ColorNumericInputMode::Rgb, ColorNumericInputMode::Hsv]
    );
    assert_eq!(
        color_numeric_input_modes(ColorEditPopupNumericInputs::Rgb),
        &[ColorNumericInputMode::Rgb]
    );
    assert_eq!(
        color_numeric_input_modes(ColorEditPopupNumericInputs::Hsv),
        &[ColorNumericInputMode::Hsv]
    );
    assert!(color_numeric_input_modes(ColorEditPopupNumericInputs::Hidden).is_empty());
}

#[test]
fn rgb_hex_parse_preserves_alpha_when_alpha_is_not_explicit() {
    let mut current = Color::from_srgb_hex_rgb(0x11_22_33);
    current.a = 0.375;

    let parsed = parse_hex("#EF4444", false, current).expect("rgb hex should parse");

    assert_eq!(parsed.to_srgb_hex_rgb(), 0xef_44_44);
    assert!((parsed.a - current.a).abs() < 0.002);
}

#[test]
fn rgba_hex_parse_is_only_available_when_alpha_is_visible() {
    let current = Color::from_srgb_hex_rgb(0x11_22_33);

    assert!(parse_hex("#EF444480", false, current).is_none());

    let parsed =
        parse_hex("#EF444480", true, current).expect("rgba hex should parse when alpha shows");

    assert_eq!(parsed.to_srgb_hex_rgb(), 0xef_44_44);
    assert!((parsed.a - (0x80 as f32 / 255.0)).abs() < f32::EPSILON);
}

#[test]
fn rgb_presets_preserve_current_alpha() {
    let color = color_from_rgb_preserving_alpha(0x3b_82_f6, 0.25);

    assert_eq!(color.to_srgb_hex_rgb(), 0x3b_82_f6);
    assert_eq!(format_hex(color, true).as_ref(), "#3B82F640");
}

#[test]
fn numeric_readout_formats_rgb_hsv_and_optional_alpha() {
    let mut color = Color::from_srgb_hex_rgb(0x33_66_99);
    color.a = 0.5;

    assert_eq!(rgb_numeric_text(color, false).as_ref(), "RGB 51 102 153");
    assert_eq!(
        rgb_numeric_text(color, true).as_ref(),
        "RGB 51 102 153 | A 50%"
    );
    assert_eq!(
        hsv_numeric_text(color).as_ref(),
        "HSV 210deg | S 67% | V 60%"
    );
}

#[test]
fn rgb_numeric_input_parses_channels_and_optional_alpha_percent() {
    let mut current = Color::from_srgb_hex_rgb(0x11_22_33);
    current.a = 0.375;

    let rgb_only =
        parse_color_numeric_input(ColorNumericInputMode::Rgb, "RGB 255 128 0", true, current)
            .expect("rgb values should parse");
    assert_eq!(rgb_only.to_srgb_hex_rgb(), 0xff_80_00);
    assert!((rgb_only.a - current.a).abs() < f32::EPSILON);

    let rgba = parse_color_numeric_input(
        ColorNumericInputMode::Rgb,
        "RGB 255 128 0 | A 25%",
        true,
        current,
    )
    .expect("rgba values should parse");
    assert_eq!(rgba.to_srgb_hex_rgb(), 0xff_80_00);
    assert!((rgba.a - 0.25).abs() < f32::EPSILON);
}

#[test]
fn hsv_numeric_input_parses_degrees_and_percentages_preserving_alpha() {
    let mut current = Color::from_srgb_hex_rgb(0x11_22_33);
    current.a = 0.625;

    let parsed = parse_color_numeric_input(
        ColorNumericInputMode::Hsv,
        "HSV 120deg | S 50% | V 25%",
        false,
        current,
    )
    .expect("hsv values should parse");

    let hsv = hsv_from_color(parsed);
    assert_hsv_close(hsv, 120.0 / 360.0, 0.5, 0.25);
    assert!((parsed.a - current.a).abs() < f32::EPSILON);
}

#[test]
fn numeric_input_rejects_out_of_range_or_incomplete_values() {
    let current = Color::from_srgb_hex_rgb(0x11_22_33);

    assert!(
        parse_color_numeric_input(ColorNumericInputMode::Rgb, "RGB 256 0 0", false, current)
            .is_none()
    );
    assert!(
        parse_color_numeric_input(ColorNumericInputMode::Rgb, "RGB 255 0", false, current)
            .is_none()
    );
    assert!(
        parse_color_numeric_input(ColorNumericInputMode::Hsv, "HSV 361 50 50", false, current)
            .is_none()
    );
    assert!(
        parse_color_numeric_input(ColorNumericInputMode::Hsv, "HSV 120 101 50", false, current)
            .is_none()
    );
}

#[test]
fn hsv_conversion_matches_primary_colors() {
    let red = rgb_to_hsv(0xff_00_00);
    assert_hsv_close(red, 0.0, 1.0, 1.0);

    let green = rgb_to_hsv(0x00_ff_00);
    assert_hsv_close(green, 1.0 / 3.0, 1.0, 1.0);

    let blue = rgb_to_hsv(0x00_00_ff);
    assert_hsv_close(blue, 2.0 / 3.0, 1.0, 1.0);

    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 0.0,
            saturation: 1.0,
            value: 1.0,
        }),
        0xff_00_00
    );
    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 1.0 / 3.0,
            saturation: 1.0,
            value: 1.0,
        }),
        0x00_ff_00
    );
    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 2.0 / 3.0,
            saturation: 1.0,
            value: 1.0,
        }),
        0x00_00_ff
    );
}

#[test]
fn hsv_conversion_handles_grayscale_without_unstable_hue() {
    assert_hsv_close(rgb_to_hsv(0x00_00_00), 0.0, 0.0, 0.0);
    assert_hsv_close(rgb_to_hsv(0x80_80_80), 0.0, 0.0, 128.0 / 255.0);
    assert_hsv_close(rgb_to_hsv(0xff_ff_ff), 0.0, 0.0, 1.0);

    assert_eq!(
        hsv_to_rgb(HsvColor {
            hue: 0.42,
            saturation: 0.0,
            value: 128.0 / 255.0,
        }),
        0x80_80_80
    );
}

#[test]
fn hsv_conversion_roundtrips_color_presets() {
    for entry in default_color_edit_palette().iter() {
        let hsv = rgb_to_hsv(entry.rgb);
        assert_eq!(hsv_to_rgb(hsv), entry.rgb);
    }
}

#[test]
fn sv_picker_position_preserves_hue_and_clamps_sv() {
    let current = HsvColor {
        hue: 0.25,
        saturation: 0.4,
        value: 0.6,
    };

    let inside = hsv_with_sv_from_local_position(current, 25.0, 75.0, 100.0, 100.0);
    assert_hsv_close(inside, 0.25, 0.25, 0.25);

    let clamped = hsv_with_sv_from_local_position(current, 120.0, -10.0, 100.0, 100.0);
    assert_hsv_close(clamped, 0.25, 1.0, 1.0);
}

#[test]
fn vertical_hue_bar_position_maps_local_y_to_clamped_hue() {
    assert_eq!(hue_from_local_y(-10.0, 100.0), 0.0);
    assert!((hue_from_local_y(37.5, 100.0) - 0.375).abs() < f32::EPSILON);
    assert_eq!(hue_from_local_y(120.0, 100.0), 1.0);
    assert_eq!(hue_from_local_y(10.0, 0.0), 0.0);
}

#[test]
fn hue_wheel_ring_maps_screen_angle_to_hue() {
    let current = HsvColor {
        hue: 0.0,
        saturation: 0.4,
        value: 0.6,
    };
    let width = 138.0;
    let height = 120.0;
    let geometry = hue_wheel_geometry(width, height);
    let radius = (geometry.wheel_r_inner + geometry.wheel_r_outer) * 0.5;

    let right = (geometry.center_x + radius, geometry.center_y);
    assert_eq!(
        hue_wheel_target_from_local_position(current, right.0, right.1, width, height),
        Some(HueWheelDragTarget::Hue)
    );
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            right.0,
            right.1,
            width,
            height,
            HueWheelDragTarget::Hue,
        ),
        0.0,
        0.4,
        0.6,
    );

    let down = (geometry.center_x, geometry.center_y + radius);
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            down.0,
            down.1,
            width,
            height,
            HueWheelDragTarget::Hue,
        ),
        0.25,
        0.4,
        0.6,
    );

    let left = (geometry.center_x - radius, geometry.center_y);
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            left.0,
            left.1,
            width,
            height,
            HueWheelDragTarget::Hue,
        ),
        0.5,
        0.4,
        0.6,
    );

    let up = (geometry.center_x, geometry.center_y - radius);
    assert_hsv_close(
        hsv_with_hue_wheel_position(current, up.0, up.1, width, height, HueWheelDragTarget::Hue),
        0.75,
        0.4,
        0.6,
    );
}

#[test]
fn hue_wheel_triangle_maps_imgui_barycentric_sv() {
    let current = HsvColor {
        hue: 0.0,
        saturation: 0.5,
        value: 0.5,
    };
    let width = 138.0;
    let height = 120.0;
    let geometry = hue_wheel_geometry(width, height);
    let triangle = hue_wheel_rotated_triangle(geometry, current.hue);

    assert_eq!(
        hue_wheel_target_from_local_position(
            current,
            triangle.hue.0,
            triangle.hue.1,
            width,
            height
        ),
        Some(HueWheelDragTarget::SaturationValue)
    );
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            triangle.hue.0,
            triangle.hue.1,
            width,
            height,
            HueWheelDragTarget::SaturationValue,
        ),
        0.0,
        1.0,
        1.0,
    );

    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            triangle.white.0,
            triangle.white.1,
            width,
            height,
            HueWheelDragTarget::SaturationValue,
        ),
        0.0,
        0.0001,
        1.0,
    );
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            triangle.black.0,
            triangle.black.1,
            width,
            height,
            HueWheelDragTarget::SaturationValue,
        ),
        0.0,
        0.0001,
        0.0001,
    );

    let centroid = (
        (triangle.hue.0 + triangle.black.0 + triangle.white.0) / 3.0,
        (triangle.hue.1 + triangle.black.1 + triangle.white.1) / 3.0,
    );
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            current,
            centroid.0,
            centroid.1,
            width,
            height,
            HueWheelDragTarget::SaturationValue,
        ),
        0.0,
        0.5,
        2.0 / 3.0,
    );
}

#[test]
fn hue_wheel_triangle_rotates_with_hue() {
    let hsv = HsvColor {
        hue: 0.25,
        saturation: 1.0,
        value: 1.0,
    };
    let width = 138.0;
    let height = 120.0;
    let geometry = hue_wheel_geometry(width, height);
    let triangle = hue_wheel_rotated_triangle(geometry, hsv.hue);
    let cursor = hue_wheel_sv_cursor_position(hsv, width, height);

    assert_eq!(
        hue_wheel_target_from_local_position(hsv, triangle.hue.0, triangle.hue.1, width, height),
        Some(HueWheelDragTarget::SaturationValue)
    );
    assert!((cursor.0 - triangle.hue.0).abs() < 0.002);
    assert!((cursor.1 - triangle.hue.1).abs() < 0.002);
    assert_hsv_close(
        hsv_with_hue_wheel_position(
            hsv,
            triangle.hue.0,
            triangle.hue.1,
            width,
            height,
            HueWheelDragTarget::SaturationValue,
        ),
        0.25,
        1.0,
        1.0,
    );
}

#[test]
fn hue_wheel_target_rejects_outside_or_empty_geometry() {
    let current = HsvColor {
        hue: 0.0,
        saturation: 0.5,
        value: 0.5,
    };

    assert_eq!(
        hue_wheel_target_from_local_position(current, 0.0, 0.0, 138.0, 120.0),
        None
    );
    assert_eq!(
        hue_wheel_target_from_local_position(current, 10.0, 10.0, 0.0, 120.0),
        None
    );
}

#[test]
fn hsv_color_edits_preserve_current_alpha() {
    let color = hsv_to_color_preserving_alpha(
        HsvColor {
            hue: 1.0 / 3.0,
            saturation: 1.0,
            value: 1.0,
        },
        0.25,
    );

    assert_eq!(color.to_srgb_hex_rgb(), 0x00_ff_00);
    assert_eq!(format_hex(color, true).as_ref(), "#00FF0040");
}

#[test]
fn alpha_checkerboard_colors_are_stable_and_alternating() {
    let light = Color::from_srgb_hex_rgb(CHECKERBOARD_LIGHT_RGB);
    let dark = Color::from_srgb_hex_rgb(CHECKERBOARD_DARK_RGB);

    assert_ne!(light, dark);
    assert_eq!(checkerboard_cell_color(0, 0), light);
    assert_eq!(checkerboard_cell_color(0, 1), dark);
    assert_eq!(checkerboard_cell_color(1, 0), dark);
    assert_eq!(checkerboard_cell_color(1, 1), light);
}

#[test]
fn opaque_alpha_preview_keeps_rgb_and_forces_preview_alpha() {
    let mut color = Color::from_srgb_hex_rgb(0x40_80_c0);
    color.a = 0.25;
    let opaque = opaque_preview_color(color);

    assert_eq!(opaque.r, color.r);
    assert_eq!(opaque.g, color.g);
    assert_eq!(opaque.b, color.b);
    assert_eq!(opaque.a, 1.0);
}

#[test]
fn popup_preview_hides_alpha_when_alpha_editing_is_not_visible() {
    let mut color = Color::from_srgb_hex_rgb(0x40_80_c0);
    color.a = 0.25;

    assert_eq!(preview_color_for_alpha_visibility(color, true), color);

    let opaque = preview_color_for_alpha_visibility(color, false);
    assert_eq!(opaque.r, color.r);
    assert_eq!(opaque.g, color.g);
    assert_eq!(opaque.b, color.b);
    assert_eq!(opaque.a, 1.0);
}

#[test]
fn popup_original_restore_matches_imgui_component_count_rules() {
    let mut current = Color::from_srgb_hex_rgb(0x11_22_33);
    current.a = 0.25;
    let mut original = Color::from_srgb_hex_rgb(0xef_44_44);
    original.a = 0.75;

    let restored_rgb = restore_reference_color(original, current, false);
    assert_eq!(restored_rgb.to_srgb_hex_rgb(), original.to_srgb_hex_rgb());
    assert!((restored_rgb.a - current.a).abs() < f32::EPSILON);

    let restored_rgba = restore_reference_color(original, current, true);
    assert_eq!(restored_rgba, original);
}

#[test]
fn alpha_bar_position_maps_local_x_to_clamped_alpha() {
    assert_eq!(alpha_from_local_x(-10.0, 100.0), 0.0);
    assert_eq!(alpha_from_local_x(0.0, 100.0), 0.0);
    assert!((alpha_from_local_x(37.5, 100.0) - 0.375).abs() < f32::EPSILON);
    assert_eq!(alpha_from_local_x(120.0, 100.0), 1.0);
    assert_eq!(alpha_from_local_x(10.0, 0.0), 0.0);
}

#[test]
fn vertical_alpha_bar_position_maps_local_y_to_inverted_alpha() {
    assert_eq!(alpha_from_local_y(-10.0, 100.0), 1.0);
    assert_eq!(alpha_from_local_y(0.0, 100.0), 1.0);
    assert!((alpha_from_local_y(37.5, 100.0) - 0.625).abs() < f32::EPSILON);
    assert_eq!(alpha_from_local_y(120.0, 100.0), 0.0);
    assert_eq!(alpha_from_local_y(10.0, 0.0), 1.0);
}

#[test]
fn alpha_percent_text_rounds_for_a11y_value() {
    assert_eq!(alpha_percent_text(0.0).as_ref(), "0%");
    assert_eq!(alpha_percent_text(0.374).as_ref(), "37%");
    assert_eq!(alpha_percent_text(1.2).as_ref(), "100%");
}

fn assert_hsv_close(actual: HsvColor, hue: f32, saturation: f32, value: f32) {
    assert!(
        (actual.hue - hue).abs() < 0.002,
        "hue mismatch: actual {:?}, expected {hue}",
        actual
    );
    assert!(
        (actual.saturation - saturation).abs() < 0.002,
        "saturation mismatch: actual {:?}, expected {saturation}",
        actual
    );
    assert!(
        (actual.value - value).abs() < 0.002,
        "value mismatch: actual {:?}, expected {value}",
        actual
    );
}
