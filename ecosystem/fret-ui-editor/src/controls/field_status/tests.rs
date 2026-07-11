use std::sync::Arc;

use fret_app::App;
use fret_core::Px;
use fret_ui::Theme;

use super::{FieldStatus, FieldStatusBadgeOptions, status_badge_palette};
use crate::theme::{EditorThemePreset, apply_editor_theme_preset};

#[test]
fn error_badge_palette_keeps_short_visible_label() {
    let app = App::new();
    let theme = Theme::global(&app);

    let (_, _, _, label) = status_badge_palette(theme, &FieldStatus::Error(Arc::from("stub")));

    assert_eq!(label.as_ref(), "Error");
}

#[test]
fn loading_badge_palette_uses_short_label() {
    let app = App::new();
    let theme = Theme::global(&app);

    let (_, _, _, label) = status_badge_palette(theme, &FieldStatus::Loading);

    assert_eq!(label.as_ref(), "Loading");
}

#[test]
fn loading_badge_palette_stays_darker_than_editor_foreground() {
    let mut app = App::new();
    apply_editor_theme_preset(&mut app, EditorThemePreset::Default);
    let theme = Theme::global(&app);

    let (bg, border, fg, _) = status_badge_palette(theme, &FieldStatus::Loading);

    assert!(relative_luma(bg) < relative_luma(fg));
    assert!(relative_luma(border) < relative_luma(theme.color_token("foreground")));
}

#[test]
fn field_status_badge_defaults_keep_padding_local() {
    let options = FieldStatusBadgeOptions::default();

    assert_eq!(options.padding.top, Px(0.0));
    assert_eq!(options.padding.bottom, Px(0.0));
    assert_eq!(options.padding.left, Px(5.0));
    assert_eq!(options.padding.right, Px(5.0));
}

fn relative_luma(color: fret_core::Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}
