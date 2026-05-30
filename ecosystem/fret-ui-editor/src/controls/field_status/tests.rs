use std::sync::Arc;

use fret_app::App;
use fret_ui::Theme;

use super::{FieldStatus, status_badge_palette};
use crate::theme::{EditorThemePresetV1, apply_editor_theme_preset_v1};

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
    apply_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);
    let theme = Theme::global(&app);

    let (bg, border, fg, _) = status_badge_palette(theme, &FieldStatus::Loading);

    assert!(relative_luma(bg) < relative_luma(fg));
    assert!(relative_luma(border) < relative_luma(theme.color_token("foreground")));
}

fn relative_luma(color: fret_core::Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}
