use super::{
    resolve_slider_chrome, resolve_slider_geometry, resolve_slider_paint, slider_thumb_props,
    slider_track_flex_props, slider_track_segment_props,
};
use crate::primitives::EditorTokenKeys;
use fret_app::App;
use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_ui::element::{CrossAlign, Length, MainAlign, SpacingLength};
use fret_ui::{Theme, ThemeConfig};

fn install_slider_colors(app: &mut App) {
    Theme::with_global_mut(app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.colors.insert(
            EditorTokenKeys::SLIDER_TRACK_BG.to_string(),
            "#171d26".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::SLIDER_FILL_BG.to_string(),
            "#355a86".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::SLIDER_THUMB_BG.to_string(),
            "#141b24".to_string(),
        );
        cfg.colors.insert(
            EditorTokenKeys::SLIDER_THUMB_BORDER.to_string(),
            "#3b4758".to_string(),
        );
        cfg.colors
            .insert("muted".to_string(), "#ff0000".to_string());
        cfg.colors
            .insert("primary".to_string(), "#00ff00".to_string());
        cfg.colors
            .insert("background".to_string(), "#0000ff".to_string());
        cfg.colors
            .insert("border".to_string(), "#ffffff".to_string());
        theme.apply_config_patch(&cfg);
    });
}

fn install_slider_metrics(app: &mut App, track_h: f32, thumb_d: f32) {
    Theme::with_global_mut(app, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.metrics
            .insert(EditorTokenKeys::SLIDER_TRACK_HEIGHT.to_string(), track_h);
        cfg.metrics
            .insert(EditorTokenKeys::SLIDER_THUMB_DIAMETER.to_string(), thumb_d);
        theme.apply_config_patch(&cfg);
    });
}

fn assert_color_close(actual: Color, expected: Color) {
    const EPSILON: f32 = 0.000_001;
    assert!((actual.r - expected.r).abs() <= EPSILON);
    assert!((actual.g - expected.g).abs() <= EPSILON);
    assert!((actual.b - expected.b).abs() <= EPSILON);
    assert!((actual.a - expected.a).abs() <= EPSILON);
}

#[test]
fn slider_geometry_uses_default_track_and_thumb_metrics() {
    let app = App::new();
    let theme = Theme::global(&app);
    let geometry = resolve_slider_geometry(theme);

    assert_eq!(geometry.track_h, Px(4.0));
    assert_eq!(geometry.thumb_d, Px(12.0));
    assert_eq!(geometry.track_radius, Px(2.0));
    assert_eq!(geometry.thumb_radius, Px(6.0));
}

#[test]
fn slider_geometry_clamps_track_and_keeps_thumb_at_least_track_height() {
    let mut app = App::new();
    install_slider_metrics(&mut app, 0.25, 0.5);

    let theme = Theme::global(&app);
    let geometry = resolve_slider_geometry(theme);

    assert_eq!(geometry.track_h, Px(1.0));
    assert_eq!(geometry.thumb_d, Px(1.0));
    assert_eq!(geometry.track_radius, Px(0.5));
    assert_eq!(geometry.thumb_radius, Px(0.5));

    let mut app = App::new();
    install_slider_metrics(&mut app, 8.0, 3.0);

    let theme = Theme::global(&app);
    let geometry = resolve_slider_geometry(theme);

    assert_eq!(geometry.track_h, Px(8.0));
    assert_eq!(geometry.thumb_d, Px(8.0));
    assert_eq!(geometry.track_radius, Px(4.0));
    assert_eq!(geometry.thumb_radius, Px(4.0));
}

#[test]
fn slider_track_props_encode_fill_track_layout_and_shape() {
    let app = App::new();
    let theme = Theme::global(&app);
    let geometry = resolve_slider_geometry(theme);
    let bg = Color::from_srgb_hex_rgb(0x35_5a_86);

    let flex = slider_track_flex_props();
    assert_eq!(flex.layout.size.width, Length::Fill);
    assert_eq!(flex.layout.size.height, Length::Fill);
    assert_eq!(flex.layout.flex.grow, 1.0);
    assert_eq!(flex.layout.flex.basis, Length::Px(Px(0.0)));
    assert_eq!(flex.direction, Axis::Horizontal);
    assert_eq!(flex.gap, SpacingLength::Px(Px(0.0)));
    assert_eq!(flex.justify, MainAlign::Start);
    assert_eq!(flex.align, CrossAlign::Center);
    assert!(!flex.wrap);

    let left = slider_track_segment_props(geometry, 0.35, bg, true);
    assert_eq!(left.layout.size.width, Length::Auto);
    assert_eq!(left.layout.size.height, Length::Px(geometry.track_h));
    assert_eq!(left.layout.flex.grow, 0.35);
    assert_eq!(left.layout.flex.shrink, 1.0);
    assert_eq!(left.layout.flex.basis, Length::Px(Px(0.0)));
    assert_eq!(left.background, Some(bg));
    assert_eq!(
        left.corner_radii,
        Corners {
            top_left: geometry.track_radius,
            bottom_left: geometry.track_radius,
            top_right: Px(0.0),
            bottom_right: Px(0.0),
        }
    );

    let right = slider_track_segment_props(geometry, 0.65, bg, false);
    assert_eq!(right.layout.flex.grow, 0.65);
    assert_eq!(
        right.corner_radii,
        Corners {
            top_left: Px(0.0),
            bottom_left: Px(0.0),
            top_right: geometry.track_radius,
            bottom_right: geometry.track_radius,
        }
    );
}

#[test]
fn slider_thumb_props_encode_fixed_diameter_border_and_shape() {
    let mut app = App::new();
    install_slider_colors(&mut app);

    let theme = Theme::global(&app);
    let geometry = resolve_slider_geometry(theme);
    let paint = resolve_slider_paint(theme, true, true, false, false);
    let props = slider_thumb_props(geometry, paint);

    assert_eq!(props.layout.size.width, Length::Px(geometry.thumb_d));
    assert_eq!(props.layout.size.height, Length::Px(geometry.thumb_d));
    assert_eq!(props.layout.flex.grow, 0.0);
    assert_eq!(props.layout.flex.shrink, 0.0);
    assert_eq!(props.layout.flex.basis, Length::Px(geometry.thumb_d));
    assert_eq!(props.background, Some(paint.thumb_bg));
    assert_eq!(props.border, Edges::all(Px(1.0)));
    assert_eq!(props.border_color, Some(paint.thumb_border));
    assert_eq!(props.corner_radii, Corners::all(geometry.thumb_radius));
}

#[test]
fn slider_chrome_prefers_editor_owned_tokens_over_generic_palette() {
    let mut app = App::new();
    install_slider_colors(&mut app);

    let theme = Theme::global(&app);
    let chrome = resolve_slider_chrome(theme);

    assert_eq!(chrome.track_bg, Color::from_srgb_hex_rgb(0x17_1d_26));
    assert_eq!(chrome.fill_bg, Color::from_srgb_hex_rgb(0x35_5a_86));
    assert_eq!(chrome.thumb_bg, Color::from_srgb_hex_rgb(0x14_1b_24));
    assert_eq!(chrome.thumb_border, Color::from_srgb_hex_rgb(0x3b_47_58));
}

#[test]
fn slider_paint_applies_disabled_alpha_to_all_chrome_channels() {
    let mut app = App::new();
    install_slider_colors(&mut app);

    let theme = Theme::global(&app);
    let enabled = resolve_slider_paint(theme, true, true, false, false);
    let disabled = resolve_slider_paint(theme, false, false, false, false);

    assert_color_close(
        disabled.track_bg,
        Color {
            a: enabled.track_bg.a * 0.55,
            ..enabled.track_bg
        },
    );
    assert_color_close(
        disabled.fill_bg,
        Color {
            a: enabled.fill_bg.a * 0.55,
            ..enabled.fill_bg
        },
    );
    assert_color_close(
        disabled.thumb_bg,
        Color {
            a: enabled.thumb_bg.a * 0.55,
            ..enabled.thumb_bg
        },
    );
    assert_color_close(
        disabled.thumb_border,
        Color {
            a: enabled.thumb_border.a * 0.55,
            ..enabled.thumb_border
        },
    );
}

#[test]
fn slider_paint_mixes_hover_and_pressed_track_fill_when_enabled() {
    let mut app = App::new();
    install_slider_colors(&mut app);

    let theme = Theme::global(&app);
    let base = resolve_slider_paint(theme, true, true, false, false);
    let hovered = resolve_slider_paint(theme, true, true, true, false);
    let pressed = resolve_slider_paint(theme, true, true, true, true);
    let disabled_hovered = resolve_slider_paint(theme, false, false, true, true);

    assert_ne!(hovered.track_bg, base.track_bg);
    assert_ne!(hovered.fill_bg, base.fill_bg);
    assert_ne!(pressed.track_bg, hovered.track_bg);
    assert_ne!(pressed.fill_bg, hovered.fill_bg);

    assert_color_close(
        disabled_hovered.track_bg,
        Color {
            a: base.track_bg.a * 0.55,
            ..base.track_bg
        },
    );
    assert_color_close(
        disabled_hovered.fill_bg,
        Color {
            a: base.fill_bg.a * 0.55,
            ..base.fill_bg
        },
    );
}
