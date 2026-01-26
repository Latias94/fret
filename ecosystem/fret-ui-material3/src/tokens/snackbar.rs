//! Typed token access for Material 3 snackbars.
//!
//! This module centralizes token key mapping and fallback chains so snackbar outcomes remain
//! stable and drift-resistant during refactors.

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::theme::CubicBezier;
use fret_ui_kit::{
    ToastButtonStyle, ToastIconButtonStyle, ToastVariantColors, ToastVariantPalette,
};

use crate::foundation::elevation::shadow_for_elevation_with_color;

pub(crate) fn icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.snackbar.icon.size")
        .unwrap_or(Px(24.0))
}

pub(crate) fn container_shape_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.snackbar.container.shape")
        .unwrap_or(Px(4.0))
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.snackbar.container.elevation")
        .unwrap_or(Px(0.0))
}

pub(crate) fn container_shadow_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.snackbar.container.shadow-color")
        .or_else(|| theme.color_by_key("md.sys.color.shadow"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.shadow"))
}

pub(crate) fn container_shadow(theme: &Theme) -> Option<fret_ui::element::ShadowStyle> {
    let elevation = container_elevation(theme);
    let r = container_shape_radius(theme);
    let shadow_color = container_shadow_color(theme);
    shadow_for_elevation_with_color(theme, elevation, Some(shadow_color), Corners::all(r))
}

pub(crate) fn open_duration_ms(theme: &Theme) -> u32 {
    theme
        .duration_ms_by_key("md.sys.motion.duration.short4")
        .unwrap_or(200)
}

pub(crate) fn close_duration_ms(theme: &Theme) -> u32 {
    theme
        .duration_ms_by_key("md.sys.motion.duration.short2")
        .unwrap_or(100)
}

pub(crate) fn easing(theme: &Theme) -> Option<CubicBezier> {
    theme
        .easing_by_key("md.sys.motion.easing.emphasized")
        .or_else(|| theme.easing_by_key("md.sys.motion.easing.standard"))
}

pub(crate) fn single_line_min_height(theme: &Theme) -> Option<Px> {
    theme.metric_by_key("md.comp.snackbar.with-single-line.container.height")
}

pub(crate) fn two_line_min_height(theme: &Theme) -> Option<Px> {
    theme.metric_by_key("md.comp.snackbar.with-two-lines.container.height")
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
    theme
        .number_by_key(key)
        .or_else(|| theme.number_by_key(sys_key))
        .unwrap_or(fallback)
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
        label_style_key: Some("md.sys.typescale.label-large".to_string()),
        label_color_key: Some("md.comp.snackbar.action.label-text.color".to_string()),
        state_layer_color_key: Some("md.comp.snackbar.action.hover.state-layer.color".to_string()),
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
        state_layer_color_key: Some("md.comp.snackbar.icon.hover.state-layer.color".to_string()),
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
