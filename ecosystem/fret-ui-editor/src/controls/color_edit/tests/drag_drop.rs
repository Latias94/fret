use super::super::drag_drop::{
    ColorDragDropStore, prune_color_drag_drop_store, take_delivered_color_drop,
};
use super::*;

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
fn empty_drag_drop_prune_does_not_bump_store_revision() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let store = app.models_mut().insert(ColorDragDropStore::default());
    let revision = store.revision(&app);
    let store_for_render = store.clone();

    let _ = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "color-edit-drag-prune-noop",
        |cx| {
            prune_color_drag_drop_store(cx, &store_for_render);
            cx.spacer(Default::default())
        },
    );

    assert_eq!(store.revision(&app), revision);
}

#[test]
fn missing_delivered_drop_does_not_bump_store_revision() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let store = app.models_mut().insert(ColorDragDropStore::default());
    let revision = store.revision(&app);
    let store_for_render = store.clone();
    let mut delivered = None;

    let _ = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "color-edit-drop-take-noop",
        |cx| {
            delivered =
                take_delivered_color_drop(cx, &store_for_render, fret_ui::GlobalElementId(42));
            cx.spacer(Default::default())
        },
    );

    assert_eq!(delivered, None);
    assert_eq!(store.revision(&app), revision);
}
