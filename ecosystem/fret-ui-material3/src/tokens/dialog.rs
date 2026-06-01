//! Typed token access for Material 3 dialogs.
//!
//! This module centralizes token key mapping and fallback chains so dialog outcomes remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Corners, Edges, Px, TextStyle};
use fret_ui::{Theme, theme::CubicBezier};
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{MaterialStateLayerInteraction, MaterialTokenResolver};
use crate::tokens::typography;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DialogActionInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn scrim_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_sys("md.sys.color.scrim")
}

pub(crate) fn scrim_opacity(theme: &Theme, fallback: f32) -> f32 {
    MaterialTokenResolver::new(theme)
        .number_optional(Some("md.sys.fret.material.dialog.scrim.opacity"), fallback)
        .clamp(0.0, 1.0)
}

pub(crate) fn container_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.dialog.container.color",
        "md.sys.color.surface-container-high",
    )
}

fn dialog_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

fn dialog_metric_chain(theme: &Theme, keys: &[&'static str], fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(keys, fallback)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    Corners::all(dialog_metric_chain(
        theme,
        &[
            "md.comp.dialog.container.shape",
            "md.sys.shape.corner.extra-large",
        ],
        Px(28.0),
    ))
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    dialog_metric(theme, "md.comp.dialog.container.elevation", Px(0.0))
}

pub(crate) fn container_shadow_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.dialog.container.shadow-color",
        "md.sys.color.shadow",
    )
}

pub(crate) fn headline_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys("md.comp.dialog.headline.color", "md.sys.color.on-surface")
}

pub(crate) fn supporting_text_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.dialog.supporting-text.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn headline_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        None,
        "md.sys.typescale.headline-small",
        Some("md.comp.dialog.headline.weight"),
        TextIntent::Content,
    )
}

pub(crate) fn supporting_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        None,
        "md.sys.typescale.body-medium",
        Some("md.comp.dialog.supporting-text.weight"),
        TextIntent::Content,
    )
}

pub(crate) fn default_open_duration_ms(theme: &Theme) -> u32 {
    MaterialTokenResolver::new(theme).duration_ms_sys("md.sys.motion.duration.medium2", 300)
}

pub(crate) fn default_close_duration_ms(theme: &Theme) -> u32 {
    MaterialTokenResolver::new(theme).duration_ms_sys("md.sys.motion.duration.medium2", 300)
}

pub(crate) fn easing(theme: &Theme, easing_key: Option<&str>) -> CubicBezier {
    let key = easing_key.unwrap_or("md.sys.motion.easing.emphasized");
    MaterialTokenResolver::new(theme).easing_optional_or_linear(Some(key))
}

pub(crate) fn panel_padding(theme: &Theme) -> Edges {
    let _ = theme;
    Edges::all(Px(24.0))
}

pub(crate) fn viewport_margin(theme: &Theme) -> Edges {
    let _ = theme;
    Edges::all(Px(24.0))
}

pub(crate) fn container_min_width(theme: &Theme) -> Px {
    let _ = theme;
    Px(280.0)
}

pub(crate) fn container_max_width(theme: &Theme) -> Px {
    let _ = theme;
    Px(560.0)
}

pub(crate) fn action_height(theme: &Theme) -> Px {
    let _ = theme;
    Px(40.0)
}

pub(crate) fn action_padding(theme: &Theme) -> Edges {
    let _ = theme;
    Edges {
        left: Px(12.0),
        right: Px(12.0),
        top: Px(0.0),
        bottom: Px(0.0),
    }
}

pub(crate) fn action_corner_radii(theme: &Theme) -> Corners {
    let _ = theme;
    Corners::all(Px(9999.0))
}

pub(crate) fn action_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        None,
        "md.sys.typescale.label-large",
        Some("md.comp.dialog.action.label-text.weight"),
        TextIntent::Control,
    )
}

fn action_label_color_key(interaction: DialogActionInteraction) -> &'static str {
    match interaction {
        DialogActionInteraction::Pressed => "md.comp.dialog.action.pressed.label-text.color",
        DialogActionInteraction::Hovered => "md.comp.dialog.action.hover.label-text.color",
        DialogActionInteraction::Focused => "md.comp.dialog.action.focus.label-text.color",
        DialogActionInteraction::Default => "md.comp.dialog.action.label-text.color",
    }
}

pub(crate) fn action_label_color(theme: &Theme, interaction: DialogActionInteraction) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(action_label_color_key(interaction), "md.sys.color.primary")
}

fn action_state_layer_color_key(interaction: DialogActionInteraction) -> &'static str {
    match interaction {
        DialogActionInteraction::Pressed => "md.comp.dialog.action.pressed.state-layer.color",
        DialogActionInteraction::Hovered => "md.comp.dialog.action.hover.state-layer.color",
        DialogActionInteraction::Focused | DialogActionInteraction::Default => {
            "md.comp.dialog.action.focus.state-layer.color"
        }
    }
}

pub(crate) fn action_state_layer_color(
    theme: &Theme,
    interaction: DialogActionInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        action_state_layer_color_key(interaction),
        "md.sys.color.primary",
    )
}

fn action_state_layer_opacity_key(interaction: DialogActionInteraction) -> Option<&'static str> {
    match interaction {
        DialogActionInteraction::Pressed => {
            Some("md.comp.dialog.action.pressed.state-layer.opacity")
        }
        DialogActionInteraction::Hovered => Some("md.comp.dialog.action.hover.state-layer.opacity"),
        DialogActionInteraction::Focused => Some("md.comp.dialog.action.focus.state-layer.opacity"),
        DialogActionInteraction::Default => None,
    }
}

pub(crate) fn action_state_layer_target_opacity(
    theme: &Theme,
    interaction: DialogActionInteraction,
) -> f32 {
    let Some(key) = action_state_layer_opacity_key(interaction) else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme)
        .state_layer_opacity(key, material_state_layer_interaction(interaction))
}

pub(crate) fn action_pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        "md.comp.dialog.action.pressed.state-layer.opacity",
        MaterialStateLayerInteraction::Pressed,
    )
}

fn material_state_layer_interaction(
    interaction: DialogActionInteraction,
) -> MaterialStateLayerInteraction {
    match interaction {
        DialogActionInteraction::Pressed => MaterialStateLayerInteraction::Pressed,
        DialogActionInteraction::Hovered => MaterialStateLayerInteraction::Hovered,
        DialogActionInteraction::Focused => MaterialStateLayerInteraction::Focused,
        DialogActionInteraction::Default => MaterialStateLayerInteraction::Hovered,
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
    fn dialog_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(container_shape(theme), Corners::all(Px(28.0)));
        assert_eq!(container_elevation(theme), Px(0.0));
    }

    #[test]
    fn dialog_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.dialog.container.shape".to_string(), 24.0);
        patch
            .metrics
            .insert("md.comp.dialog.container.elevation".to_string(), 2.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(container_shape(&theme), Corners::all(Px(24.0)));
        assert_eq!(container_elevation(&theme), Px(2.0));
    }

    #[test]
    fn dialog_shape_uses_system_fallback() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.shape.corner.extra-large".to_string(), 26.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(container_shape(&theme), Corners::all(Px(26.0)));
    }
}
