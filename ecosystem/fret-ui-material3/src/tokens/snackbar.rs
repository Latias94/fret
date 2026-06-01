//! Typed token access for Material 3 snackbars.
//!
//! This module centralizes token key mapping and fallback chains so snackbar outcomes remain
//! stable and drift-resistant during refactors.

#[cfg(test)]
use fret_core::TextStyle;
use fret_core::{Color, Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::theme::CubicBezier;
#[cfg(test)]
use fret_ui_kit::typography::TextIntent;
use fret_ui_kit::{
    ToastButtonStyle, ToastIconButtonStyle, ToastVariantColors, ToastVariantPalette,
};

use crate::foundation::elevation::shadow_for_elevation_with_color;
#[cfg(test)]
use crate::foundation::token_resolver::MaterialStateLayerInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;
#[cfg(test)]
use crate::tokens::typography;

fn snackbar_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

fn snackbar_metric_value(theme: &Theme, key: &'static str) -> Option<Px> {
    MaterialTokenResolver::new(theme).metric_value(key)
}

pub(crate) fn icon_size(theme: &Theme) -> Px {
    snackbar_metric(theme, "md.comp.snackbar.icon.size", Px(24.0))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    MaterialTokenResolver::new(theme)
        .corners_chain_or(&["md.comp.snackbar.container.shape"], Corners::all(Px(4.0)))
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    snackbar_metric(theme, "md.comp.snackbar.container.elevation", Px(0.0))
}

pub(crate) fn container_shadow_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.snackbar.container.shadow-color",
        "md.sys.color.shadow",
    )
}

pub(crate) fn container_shadow(theme: &Theme) -> Option<fret_ui::element::ShadowStyle> {
    let elevation = container_elevation(theme);
    let corner_radii = container_shape(theme);
    let shadow_color = container_shadow_color(theme);
    shadow_for_elevation_with_color(theme, elevation, Some(shadow_color), corner_radii)
}

#[cfg(test)]
pub(crate) fn container_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.snackbar.container.color",
        "md.sys.color.inverse-surface",
    )
}

#[cfg(test)]
pub(crate) fn supporting_text_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.snackbar.supporting-text.color",
        "md.sys.color.inverse-on-surface",
    )
}

#[cfg(test)]
pub(crate) fn supporting_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.snackbar.supporting-text"),
        "md.sys.typescale.body-medium",
        Some("md.comp.snackbar.supporting-text.weight"),
        TextIntent::Content,
    )
}

pub(crate) fn open_duration_ms(theme: &Theme) -> u32 {
    MaterialTokenResolver::new(theme).duration_ms_sys("md.sys.motion.duration.short4", 200)
}

pub(crate) fn close_duration_ms(theme: &Theme) -> u32 {
    MaterialTokenResolver::new(theme).duration_ms_sys("md.sys.motion.duration.short2", 100)
}

pub(crate) fn easing(theme: &Theme) -> Option<CubicBezier> {
    MaterialTokenResolver::new(theme).easing_chain(&[
        "md.sys.motion.easing.emphasized",
        "md.sys.motion.easing.standard",
    ])
}

pub(crate) fn single_line_min_height(theme: &Theme) -> Option<Px> {
    snackbar_metric_value(theme, "md.comp.snackbar.with-single-line.container.height")
}

pub(crate) fn two_line_min_height(theme: &Theme) -> Option<Px> {
    snackbar_metric_value(theme, "md.comp.snackbar.with-two-lines.container.height")
}

pub(crate) fn host_margin(theme: &Theme) -> Px {
    let _ = theme;
    Px(12.0)
}

pub(crate) fn container_max_width(theme: &Theme) -> Px {
    let _ = theme;
    Px(600.0)
}

pub(crate) fn closed_scale(theme: &Theme) -> f32 {
    let _ = theme;
    0.8
}

pub(crate) fn palette() -> ToastVariantPalette {
    ToastVariantPalette {
        default: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        destructive: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        success: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        info: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        warning: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        error: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        loading: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
    }
}

pub(crate) fn container_padding(theme: &Theme) -> Edges {
    let _ = theme;
    // Token source does not define padding; keep a conservative default that fits the fixed
    // container heights.
    Edges {
        left: Px(16.0),
        right: Px(16.0),
        top: Px(8.0),
        bottom: Px(8.0),
    }
}

fn number_or_sys(theme: &Theme, key: &str, sys_key: &str, fallback: f32) -> f32 {
    MaterialTokenResolver::new(theme).number_comp_or_sys(key, sys_key, fallback)
}

pub(crate) fn action_button_style(theme: &Theme) -> ToastButtonStyle {
    let hover_opacity = number_or_sys(
        theme,
        "md.comp.snackbar.action.hover.state-layer.opacity",
        "md.sys.state.hover.state-layer-opacity",
        0.08,
    );
    let focus_opacity = number_or_sys(
        theme,
        "md.comp.snackbar.action.focus.state-layer.opacity",
        "md.sys.state.focus.state-layer-opacity",
        0.1,
    );
    let pressed_opacity = number_or_sys(
        theme,
        "md.comp.snackbar.action.pressed.state-layer.opacity",
        "md.sys.state.pressed.state-layer-opacity",
        0.1,
    );

    ToastButtonStyle {
        label_style_key: Some("md.comp.snackbar.action.label-text".to_string()),
        label_color_key: Some("md.comp.snackbar.action.label-text.color".to_string()),
        label_color: None,
        state_layer_color_key: Some("md.comp.snackbar.action.hover.state-layer.color".to_string()),
        state_layer_color: None,
        hover_state_layer_opacity_key: Some(
            "md.comp.snackbar.action.hover.state-layer.opacity".to_string(),
        ),
        focus_state_layer_opacity_key: Some(
            "md.comp.snackbar.action.focus.state-layer.opacity".to_string(),
        ),
        pressed_state_layer_opacity_key: Some(
            "md.comp.snackbar.action.pressed.state-layer.opacity".to_string(),
        ),
        hover_state_layer_opacity: hover_opacity,
        focus_state_layer_opacity: focus_opacity,
        pressed_state_layer_opacity: pressed_opacity,
        padding: Edges {
            left: Px(12.0),
            right: Px(12.0),
            top: Px(4.0),
            bottom: Px(4.0),
        },
        radius: Px(4.0),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) enum SnackbarActionInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

#[cfg(test)]
pub(crate) fn action_label_color(theme: &Theme, interaction: SnackbarActionInteraction) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        snackbar_action_label_color_key(interaction),
        "md.sys.color.inverse-primary",
    )
}

#[cfg(test)]
pub(crate) fn action_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.snackbar.action.label-text"),
        "md.sys.typescale.label-large",
        Some("md.comp.snackbar.action.label-text.weight"),
        TextIntent::Control,
    )
}

#[cfg(test)]
pub(crate) fn action_state_layer_color(
    theme: &Theme,
    interaction: SnackbarActionInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        snackbar_action_state_layer_color_key(interaction),
        "md.sys.color.inverse-primary",
    )
}

#[cfg(test)]
pub(crate) fn action_state_layer_opacity(
    theme: &Theme,
    interaction: SnackbarActionInteraction,
) -> f32 {
    let Some(key) = snackbar_action_state_layer_opacity_key(interaction) else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme)
        .state_layer_opacity(key, snackbar_state_layer_interaction(interaction))
}

#[cfg(test)]
pub(crate) fn icon_color(theme: &Theme, interaction: SnackbarActionInteraction) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        snackbar_icon_color_key(interaction),
        "md.sys.color.inverse-on-surface",
    )
}

#[cfg(test)]
pub(crate) fn icon_state_layer_color(
    theme: &Theme,
    interaction: SnackbarActionInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        snackbar_icon_state_layer_color_key(interaction),
        "md.sys.color.inverse-on-surface",
    )
}

#[cfg(test)]
pub(crate) fn icon_state_layer_opacity(
    theme: &Theme,
    interaction: SnackbarActionInteraction,
) -> f32 {
    let Some(key) = snackbar_icon_state_layer_opacity_key(interaction) else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme)
        .state_layer_opacity(key, snackbar_state_layer_interaction(interaction))
}

#[cfg(test)]
fn snackbar_action_label_color_key(interaction: SnackbarActionInteraction) -> &'static str {
    match interaction {
        SnackbarActionInteraction::Pressed => "md.comp.snackbar.action.pressed.label-text.color",
        SnackbarActionInteraction::Hovered => "md.comp.snackbar.action.hover.label-text.color",
        SnackbarActionInteraction::Focused => "md.comp.snackbar.action.focus.label-text.color",
        SnackbarActionInteraction::Default => "md.comp.snackbar.action.label-text.color",
    }
}

#[cfg(test)]
fn snackbar_action_state_layer_color_key(interaction: SnackbarActionInteraction) -> &'static str {
    match interaction {
        SnackbarActionInteraction::Pressed => "md.comp.snackbar.action.pressed.state-layer.color",
        SnackbarActionInteraction::Hovered => "md.comp.snackbar.action.hover.state-layer.color",
        SnackbarActionInteraction::Focused | SnackbarActionInteraction::Default => {
            "md.comp.snackbar.action.focus.state-layer.color"
        }
    }
}

#[cfg(test)]
fn snackbar_action_state_layer_opacity_key(
    interaction: SnackbarActionInteraction,
) -> Option<&'static str> {
    match interaction {
        SnackbarActionInteraction::Pressed => {
            Some("md.comp.snackbar.action.pressed.state-layer.opacity")
        }
        SnackbarActionInteraction::Hovered => {
            Some("md.comp.snackbar.action.hover.state-layer.opacity")
        }
        SnackbarActionInteraction::Focused => {
            Some("md.comp.snackbar.action.focus.state-layer.opacity")
        }
        SnackbarActionInteraction::Default => None,
    }
}

#[cfg(test)]
fn snackbar_icon_color_key(interaction: SnackbarActionInteraction) -> &'static str {
    match interaction {
        SnackbarActionInteraction::Hovered => "md.comp.snackbar.icon.hover.icon.color",
        SnackbarActionInteraction::Focused => "md.comp.snackbar.icon.focus.icon.color",
        SnackbarActionInteraction::Pressed => "md.comp.snackbar.icon.pressed.icon.color",
        SnackbarActionInteraction::Default => "md.comp.snackbar.icon.color",
    }
}

#[cfg(test)]
fn snackbar_icon_state_layer_color_key(interaction: SnackbarActionInteraction) -> &'static str {
    match interaction {
        SnackbarActionInteraction::Hovered => "md.comp.snackbar.icon.hover.state-layer.color",
        SnackbarActionInteraction::Focused | SnackbarActionInteraction::Default => {
            "md.comp.snackbar.icon.focus.state-layer.color"
        }
        SnackbarActionInteraction::Pressed => "md.comp.snackbar.icon.pressed.state-layer.color",
    }
}

#[cfg(test)]
fn snackbar_icon_state_layer_opacity_key(
    interaction: SnackbarActionInteraction,
) -> Option<&'static str> {
    match interaction {
        SnackbarActionInteraction::Hovered => {
            Some("md.comp.snackbar.icon.hover.state-layer.opacity")
        }
        SnackbarActionInteraction::Focused => {
            Some("md.comp.snackbar.icon.focus.state-layer.opacity")
        }
        SnackbarActionInteraction::Pressed => {
            Some("md.comp.snackbar.icon.pressed.state-layer.opacity")
        }
        SnackbarActionInteraction::Default => None,
    }
}

#[cfg(test)]
fn snackbar_state_layer_interaction(
    interaction: SnackbarActionInteraction,
) -> MaterialStateLayerInteraction {
    match interaction {
        SnackbarActionInteraction::Hovered => MaterialStateLayerInteraction::Hovered,
        SnackbarActionInteraction::Focused => MaterialStateLayerInteraction::Focused,
        SnackbarActionInteraction::Pressed => MaterialStateLayerInteraction::Pressed,
        SnackbarActionInteraction::Default => MaterialStateLayerInteraction::Hovered,
    }
}

pub(crate) fn close_icon_button_style(theme: &Theme) -> ToastIconButtonStyle {
    let hover_opacity = number_or_sys(
        theme,
        "md.comp.snackbar.icon.hover.state-layer.opacity",
        "md.sys.state.hover.state-layer-opacity",
        0.08,
    );
    let focus_opacity = number_or_sys(
        theme,
        "md.comp.snackbar.icon.focus.state-layer.opacity",
        "md.sys.state.focus.state-layer-opacity",
        0.1,
    );
    let pressed_opacity = number_or_sys(
        theme,
        "md.comp.snackbar.icon.pressed.state-layer.opacity",
        "md.sys.state.pressed.state-layer-opacity",
        0.1,
    );

    ToastIconButtonStyle {
        icon_color_key: Some("md.comp.snackbar.icon.color".to_string()),
        icon_color: None,
        state_layer_color_key: Some("md.comp.snackbar.icon.hover.state-layer.color".to_string()),
        state_layer_color: None,
        hover_state_layer_opacity_key: Some(
            "md.comp.snackbar.icon.hover.state-layer.opacity".to_string(),
        ),
        focus_state_layer_opacity_key: Some(
            "md.comp.snackbar.icon.focus.state-layer.opacity".to_string(),
        ),
        pressed_state_layer_opacity_key: Some(
            "md.comp.snackbar.icon.pressed.state-layer.opacity".to_string(),
        ),
        hover_state_layer_opacity: hover_opacity,
        focus_state_layer_opacity: focus_opacity,
        pressed_state_layer_opacity: pressed_opacity,
        padding: Edges {
            left: Px(8.0),
            right: Px(8.0),
            top: Px(8.0),
            bottom: Px(8.0),
        },
        radius: Px(4.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn snackbar_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(icon_size(theme), Px(24.0));
        assert_eq!(container_shape(theme), Corners::all(Px(4.0)));
        assert_eq!(container_elevation(theme), Px(0.0));
        assert_eq!(single_line_min_height(theme), None);
        assert_eq!(two_line_min_height(theme), None);
    }

    #[test]
    fn snackbar_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.snackbar.icon.size".to_string(), 26.0);
        patch
            .metrics
            .insert("md.comp.snackbar.container.shape".to_string(), 6.0);
        patch
            .metrics
            .insert("md.comp.snackbar.container.elevation".to_string(), 3.0);
        patch.metrics.insert(
            "md.comp.snackbar.with-single-line.container.height".to_string(),
            48.0,
        );
        patch.metrics.insert(
            "md.comp.snackbar.with-two-lines.container.height".to_string(),
            68.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(icon_size(&theme), Px(26.0));
        assert_eq!(container_shape(&theme), Corners::all(Px(6.0)));
        assert_eq!(container_elevation(&theme), Px(3.0));
        assert_eq!(single_line_min_height(&theme), Some(Px(48.0)));
        assert_eq!(two_line_min_height(&theme), Some(Px(68.0)));
    }

    #[test]
    fn snackbar_shape_prefers_structured_corners_over_uniform_metric() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.snackbar.container.shape".to_string(), 6.0);
        patch.corners.insert(
            "md.comp.snackbar.container.shape".to_string(),
            Corners {
                top_left: Px(1.0),
                top_right: Px(2.0),
                bottom_right: Px(3.0),
                bottom_left: Px(4.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme),
            Corners {
                top_left: Px(1.0),
                top_right: Px(2.0),
                bottom_right: Px(3.0),
                bottom_left: Px(4.0),
            }
        );
    }
}
