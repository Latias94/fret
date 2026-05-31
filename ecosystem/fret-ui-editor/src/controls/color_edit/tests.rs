use std::collections::BTreeSet;
use std::sync::Arc;

use super::drag_drop::apply_color_drop_payload;
use super::model::{
    ColorNumericInputMode, HsvColor, HueWheelDragTarget, color_from_rgb_preserving_alpha,
    color_numeric_input_modes, hsv_from_color, hsv_numeric_text, hsv_to_color_preserving_alpha,
    hsv_to_rgb, hsv_with_hue_wheel_position, hsv_with_sv_from_local_position, hue_from_local_y,
    hue_wheel_geometry, hue_wheel_rotated_triangle, hue_wheel_sv_cursor_position,
    hue_wheel_target_from_local_position, parse_color_numeric_input, parse_hex, rgb_numeric_text,
    rgb_to_hsv,
};
use super::popup::copy::{ColorEditCopyFormat, color_copy_entries};
use super::popup::picker::alpha::{alpha_from_local_x, alpha_from_local_y, alpha_percent_text};
use super::popup::preview::{
    SIDE_PREVIEW_SWATCH_HEIGHT, SIDE_PREVIEW_SWATCH_WIDTH, checkerboard_cell_color,
    opaque_preview_color, preview_color_for_alpha_visibility, restore_reference_color,
};
use super::popup::tooltip::color_tooltip_lines;
use super::*;

mod numeric;
mod picker;
mod popup_policy;

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
fn drag_drop_options_default_to_imgui_enabled_local_payloads() {
    let options = ColorEditOptions::default();

    assert!(options.drag_drop.enabled);
    assert!(!options.drag_drop.cross_window);
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
