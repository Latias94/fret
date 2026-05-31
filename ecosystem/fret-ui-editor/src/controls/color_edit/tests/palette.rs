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
