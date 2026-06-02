use super::*;

#[test]
fn selectable_palette_prefers_selected_background_hover_foreground_and_disabled_muted() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("list.active.background".to_string(), "#224466".to_string());
        cfg.colors
            .insert("accent".to_string(), "#335577".to_string());
        cfg.colors
            .insert("foreground".to_string(), "#f5f6f7".to_string());
        cfg.colors
            .insert("accent-foreground".to_string(), "#fefefe".to_string());
        cfg.colors
            .insert("muted-foreground".to_string(), "#8899aa".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    let selected = resolve_selectable_palette(theme, true, true, false, false);
    assert_eq!(selected.bg, Some(Color::from_srgb_hex_rgb(0x22_44_66)));
    assert_eq!(selected.fg, Color::from_srgb_hex_rgb(0xf5_f6_f7));

    let hovered = resolve_selectable_palette(theme, true, false, true, false);
    assert_eq!(hovered.bg, Some(Color::from_srgb_hex_rgb(0x33_55_77)));
    assert_eq!(hovered.fg, Color::from_srgb_hex_rgb(0xfe_fe_fe));

    let disabled = resolve_selectable_palette(theme, false, false, false, false);
    assert_eq!(disabled.bg, None);
    assert_eq!(disabled.fg, Color::from_srgb_hex_rgb(0x88_99_aa));
}

#[test]
fn selectable_palette_highlight_uses_hover_style_without_selected_semantics() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("list.active.background".to_string(), "#224466".to_string());
        cfg.colors
            .insert("list.hover.background".to_string(), "#335577".to_string());
        cfg.colors
            .insert("foreground".to_string(), "#f5f6f7".to_string());
        cfg.colors
            .insert("accent-foreground".to_string(), "#fefefe".to_string());
        cfg.colors
            .insert("muted-foreground".to_string(), "#8899aa".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    let highlighted = resolve_selectable_palette(theme, true, false, true, false);
    assert_eq!(highlighted.bg, Some(Color::from_srgb_hex_rgb(0x33_55_77)));
    assert_eq!(highlighted.fg, Color::from_srgb_hex_rgb(0xfe_fe_fe));

    let selected_highlighted = resolve_selectable_palette(theme, true, true, true, false);
    assert_eq!(
        selected_highlighted.bg,
        Some(Color::from_srgb_hex_rgb(0x22_44_66))
    );
    assert_eq!(
        selected_highlighted.fg,
        Color::from_srgb_hex_rgb(0xf5_f6_f7)
    );

    let disabled_highlighted = resolve_selectable_palette(theme, false, false, true, false);
    assert_eq!(disabled_highlighted.bg, None);
    assert_eq!(
        disabled_highlighted.fg,
        Color::from_srgb_hex_rgb(0x88_99_aa)
    );
}
