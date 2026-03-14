use fret_core::{Color, Corners, Px};
use fret_ui::Theme;
use fret_ui::element::{ShadowLayerStyle, ShadowStyle};

use super::EditorTokenKeys;
use super::chrome::sanitize_editor_surface_bg;

const DEFAULT_EDITOR_POPUP_RADIUS: Px = Px(8.0);
const DEFAULT_EDITOR_POPUP_SHADOW_OFFSET_Y: Px = Px(6.0);
const DEFAULT_EDITOR_POPUP_SHADOW_BLUR: Px = Px(16.0);
const DEFAULT_EDITOR_POPUP_SHADOW_SPREAD: Px = Px(-4.0);

#[derive(Debug, Clone)]
pub(crate) struct EditorPopupSurfaceChrome {
    pub(crate) bg: Color,
    pub(crate) border: Color,
    pub(crate) radius: Px,
    pub(crate) shadow: Option<ShadowStyle>,
}

pub(crate) fn resolve_editor_popup_surface_chrome(
    theme: &Theme,
    is_overlay_surface: bool,
) -> EditorPopupSurfaceChrome {
    let bg = theme
        .color_by_key(EditorTokenKeys::POPUP_BG)
        .or_else(|| theme.color_by_key("card"))
        .or_else(|| theme.color_by_key("component.card.bg"))
        .or_else(|| theme.color_by_key("component.text_field.bg"))
        .or_else(|| theme.color_by_key("popover"))
        .unwrap_or_else(|| theme.color_token("card"));
    let border = theme
        .color_by_key(EditorTokenKeys::POPUP_BORDER)
        .or_else(|| theme.color_by_key("component.card.border"))
        .or_else(|| theme.color_by_key("border"))
        .or_else(|| theme.color_by_key("component.text_field.border"))
        .unwrap_or_else(|| theme.color_token("border"));
    let radius = theme
        .metric_by_key(EditorTokenKeys::POPUP_RADIUS)
        .unwrap_or(DEFAULT_EDITOR_POPUP_RADIUS);
    let shadow = is_overlay_surface.then(|| ShadowStyle {
        primary: ShadowLayerStyle {
            color: theme.color_token("muted"),
            offset_x: Px(0.0),
            offset_y: theme
                .metric_by_key(EditorTokenKeys::POPUP_SHADOW_OFFSET_Y)
                .unwrap_or(DEFAULT_EDITOR_POPUP_SHADOW_OFFSET_Y),
            blur: theme
                .metric_by_key(EditorTokenKeys::POPUP_SHADOW_BLUR)
                .unwrap_or(DEFAULT_EDITOR_POPUP_SHADOW_BLUR),
            spread: theme
                .metric_by_key(EditorTokenKeys::POPUP_SHADOW_SPREAD)
                .unwrap_or(DEFAULT_EDITOR_POPUP_SHADOW_SPREAD),
        },
        secondary: None,
        corner_radii: Corners::all(radius),
    });

    EditorPopupSurfaceChrome {
        bg: sanitize_editor_surface_bg(theme, bg),
        border,
        radius,
        shadow,
    }
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::{Color, Px};
    use fret_ui::{Theme, ThemeConfig};

    use super::resolve_editor_popup_surface_chrome;
    use crate::primitives::EditorTokenKeys;
    use crate::theme::{EditorThemePresetV1, apply_editor_theme_preset_v1};

    #[test]
    fn overlay_popup_surface_adds_shadow() {
        let app = App::new();
        let chrome = resolve_editor_popup_surface_chrome(Theme::global(&app), true);
        assert!(chrome.shadow.is_some());
    }

    #[test]
    fn inline_popup_surface_skips_shadow() {
        let app = App::new();
        let chrome = resolve_editor_popup_surface_chrome(Theme::global(&app), false);
        assert!(chrome.shadow.is_none());
    }

    #[test]
    fn editor_popup_surface_prefers_editor_owned_popup_tokens() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.colors
                .insert("popover".to_string(), "#ffffff".to_string());
            theme.apply_config_patch(&cfg);
        });
        apply_editor_theme_preset_v1(&mut app, EditorThemePresetV1::Default);

        let chrome = resolve_editor_popup_surface_chrome(Theme::global(&app), true);
        assert_eq!(
            chrome.bg,
            Color::from_srgb_hex_rgb(0x13_1b_25),
            "editor popup background should not fall back to host popover"
        );
        assert_eq!(
            chrome.border,
            Theme::global(&app)
                .color_by_key(EditorTokenKeys::POPUP_BORDER)
                .unwrap()
        );
    }

    #[test]
    fn popup_surface_respects_editor_popup_radius_and_shadow_metrics() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut cfg = ThemeConfig::default();
            cfg.metrics
                .insert(EditorTokenKeys::POPUP_RADIUS.to_string(), 5.0);
            cfg.metrics
                .insert(EditorTokenKeys::POPUP_SHADOW_OFFSET_Y.to_string(), 3.0);
            cfg.metrics
                .insert(EditorTokenKeys::POPUP_SHADOW_BLUR.to_string(), 9.0);
            cfg.metrics
                .insert(EditorTokenKeys::POPUP_SHADOW_SPREAD.to_string(), -2.0);
            theme.apply_config_patch(&cfg);
        });

        let chrome = resolve_editor_popup_surface_chrome(Theme::global(&app), true);
        assert_eq!(chrome.radius, Px(5.0));
        let shadow = chrome.shadow.expect("overlay popup should keep shadow");
        assert_eq!(shadow.primary.offset_y, Px(3.0));
        assert_eq!(shadow.primary.blur, Px(9.0));
        assert_eq!(shadow.primary.spread, Px(-2.0));
    }

    #[test]
    fn dense_preset_uses_tighter_popup_radius_than_default() {
        let mut default_app = App::new();
        apply_editor_theme_preset_v1(&mut default_app, EditorThemePresetV1::Default);
        let default_chrome = resolve_editor_popup_surface_chrome(Theme::global(&default_app), true);

        let mut dense_app = App::new();
        apply_editor_theme_preset_v1(&mut dense_app, EditorThemePresetV1::ImguiLikeDense);
        let dense_chrome = resolve_editor_popup_surface_chrome(Theme::global(&dense_app), true);

        assert!(dense_chrome.radius.0 < default_chrome.radius.0);
        let default_shadow = default_chrome.shadow.expect("default overlay shadow");
        let dense_shadow = dense_chrome.shadow.expect("dense overlay shadow");
        assert!(dense_shadow.primary.blur.0 < default_shadow.primary.blur.0);
    }
}
