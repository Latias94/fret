use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Color, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, Length, PressableState};
use fret_ui::elements;
use fret_ui::{Theme, ThemeConfig};

use super::visual::{resolve_selectable_palette, selectable_row_element};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

fn first_text(root: &AnyElement) -> Option<&AnyElement> {
    match &root.kind {
        ElementKind::Text(_) => Some(root),
        _ => root.children.iter().find_map(first_text),
    }
}

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

#[test]
fn selectable_row_label_uses_shared_list_row_text_role() {
    let window = AppWindowId::default();
    let mut app = App::new();

    let el = elements::with_element_cx(&mut app, window, test_bounds(), "test", |cx| {
        selectable_row_element(
            cx,
            Arc::from("Long selectable row label"),
            true,
            false,
            false,
            PressableState::default(),
        )
    });
    let expected_palette =
        resolve_selectable_palette(Theme::global(&app), true, false, false, false);

    let text = first_text(&el).expect("expected selectable row text");
    let ElementKind::Text(props) = &text.kind else {
        panic!("expected selectable row label to be text");
    };

    assert!(props.style.is_none());
    assert!(props.color.is_none());
    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.flex.shrink, 1.0);
    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
    assert_eq!(props.wrap, TextWrap::None);
    assert_eq!(props.overflow, TextOverflow::Ellipsis);
    assert!(text.inherited_text_style.is_some());
    assert_eq!(text.inherited_foreground, Some(expected_palette.fg));
}
