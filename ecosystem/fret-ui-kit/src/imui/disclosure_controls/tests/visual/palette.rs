use super::*;

#[test]
fn tree_node_hover_palette_prefers_accent_chrome_over_popover_fill() {
    let mut app = App::new();
    Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("list.active.background".to_string(), "#224466".to_string());
        cfg.colors
            .insert("accent".to_string(), "#335577".to_string());
        cfg.colors
            .insert("accent-foreground".to_string(), "#fefefe".to_string());
        cfg.colors
            .insert("foreground".to_string(), "#f5f6f7".to_string());
        cfg.colors.insert("card".to_string(), "#101418".to_string());
        theme.apply_config_patch(&cfg);
    });

    let theme = Theme::global(&app);
    let spec = DisclosureSpec::tree_node(Arc::from("Scene"), TreeNodeOptions::default());

    let hovered = resolve_disclosure_palette(
        theme,
        &spec,
        PressableState {
            hovered: true,
            ..Default::default()
        },
    );
    assert_eq!(
        hovered.background,
        Some(Color::from_srgb_hex_rgb(0x33_55_77))
    );
    assert_eq!(hovered.foreground, Color::from_srgb_hex_rgb(0xfe_fe_fe));

    let idle = resolve_disclosure_palette(theme, &spec, PressableState::default());
    assert_eq!(idle.background, None);
    assert_eq!(idle.foreground, Color::from_srgb_hex_rgb(0xf5_f6_f7));
}
