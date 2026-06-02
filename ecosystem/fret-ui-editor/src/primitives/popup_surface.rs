use fret_core::{Color, Corners, Px};
use fret_ui::Theme;
use fret_ui::element::{ShadowLayerStyle, ShadowStyle};

use super::EditorTokenKeys;
use super::chrome::sanitize_editor_surface_bg;
use super::colors::{editor_popup_background, editor_popup_border};

#[cfg(test)]
mod tests;

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
    let bg = editor_popup_background(theme);
    let border = editor_popup_border(theme);
    let radius = theme
        .metric_by_key(EditorTokenKeys::POPUP_RADIUS)
        .unwrap_or(DEFAULT_EDITOR_POPUP_RADIUS);
    let shadow = is_overlay_surface.then(|| ShadowStyle {
        primary: ShadowLayerStyle {
            color: theme
                .color_by_key(EditorTokenKeys::POPUP_SHADOW_COLOR)
                .unwrap_or_else(|| theme.color_token("muted")),
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
