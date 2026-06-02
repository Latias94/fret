//! Typed token access for Material 3 cards.
//!
//! This module centralizes token key mapping and fallback chains so card variants remain
//! consistent and drift-resistant during refactors.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::card::CardVariant;
use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul};

pub(crate) const FILLED_COMPONENT_PREFIX: &str = "md.comp.filled-card";
pub(crate) const ELEVATED_COMPONENT_PREFIX: &str = "md.comp.elevated-card";
pub(crate) const OUTLINED_COMPONENT_PREFIX: &str = "md.comp.outlined-card";

#[derive(Debug, Clone, Copy)]
pub(crate) struct CardOutline {
    pub width: Px,
    pub color: Color,
}

pub(crate) fn component_prefix(variant: CardVariant) -> &'static str {
    match variant {
        CardVariant::Filled => FILLED_COMPONENT_PREFIX,
        CardVariant::Elevated => ELEVATED_COMPONENT_PREFIX,
        CardVariant::Outlined => OUTLINED_COMPONENT_PREFIX,
    }
}

fn card_metric(theme: &Theme, key: impl AsRef<str>, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key.as_ref()), fallback)
}

pub(crate) fn container_shape(theme: &Theme, variant: CardVariant) -> Corners {
    let component_key = format!("{}.container.shape", component_prefix(variant));
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[component_key.as_str(), "md.sys.shape.corner.medium"],
        Corners::all(Px(12.0)),
    )
}

pub(crate) fn container_shadow_color(theme: &Theme, variant: CardVariant) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{}.container.shadow-color", component_prefix(variant)),
        "md.sys.color.shadow",
    )
}

pub(crate) fn container_background(theme: &Theme, variant: CardVariant, enabled: bool) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    if enabled {
        tokens.color_comp_or_sys(
            &format!("{}.container.color", component_prefix(variant)),
            "md.sys.color.surface-container-low",
        )
    } else {
        let base = tokens.color_comp_or_sys(
            &format!("{}.disabled.container.color", component_prefix(variant)),
            "md.sys.color.on-surface",
        );

        let opacity_key = format!("{}.disabled.container.opacity", component_prefix(variant));
        let opacity = tokens.number_optional(Some(opacity_key.as_str()), 0.12);

        alpha_mul(base, opacity)
    }
}

pub(crate) fn container_elevation(
    theme: &Theme,
    variant: CardVariant,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Px {
    let prefix = component_prefix(variant);

    if !enabled {
        return card_metric(
            theme,
            format!("{prefix}.disabled.container.elevation"),
            Px(0.0),
        );
    }

    let key = interaction
        .map(|interaction| format!("{}.container.elevation", interaction.token_state()))
        .unwrap_or_else(|| "container.elevation".to_string());

    card_metric(theme, format!("{prefix}.{key}"), Px(0.0))
}

pub(crate) fn outline(
    theme: &Theme,
    variant: CardVariant,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Option<CardOutline> {
    if variant != CardVariant::Outlined {
        return None;
    }

    let prefix = OUTLINED_COMPONENT_PREFIX;
    let width = card_metric(theme, format!("{prefix}.outline.width"), Px(1.0));

    if !enabled {
        let tokens = MaterialTokenResolver::new(theme);
        let base = tokens.color_comp_or_sys(
            &format!("{prefix}.disabled.outline.color"),
            "md.sys.color.on-surface",
        );
        let opacity_key = format!("{prefix}.disabled.outline.opacity");
        let opacity = tokens.number_optional(Some(opacity_key.as_str()), 0.12);
        let c = alpha_mul(base, opacity);
        return Some(CardOutline { width, color: c });
    }

    let key = interaction
        .map(|interaction| format!("{}.outline.color", interaction.token_state()))
        .unwrap_or_else(|| "outline.color".to_string());

    let state_key = format!("{prefix}.{key}");
    let default_key = format!("{prefix}.outline.color");
    let mut color = MaterialTokenResolver::new(theme).color_comp_chain_or_sys(
        &[state_key.as_str(), default_key.as_str()],
        "md.sys.color.outline",
    );
    color.a = 1.0;

    Some(CardOutline { width, color })
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    variant: CardVariant,
    interaction: Option<PressableInteraction>,
) -> Color {
    let prefix = component_prefix(variant);
    let key = interaction
        .map(|interaction| format!("{}.state-layer.color", interaction.token_state()))
        .unwrap_or_else(|| "hover.state-layer.color".to_string());

    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(&format!("{prefix}.{key}"), "md.sys.color.on-surface")
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    variant: CardVariant,
    interaction: Option<PressableInteraction>,
) -> f32 {
    let prefix = component_prefix(variant);
    let Some(interaction) = interaction else {
        return 0.0;
    };

    let key = format!("{}.state-layer.opacity", interaction.token_state());
    MaterialTokenResolver::new(theme)
        .pressable_state_layer_opacity(&format!("{prefix}.{key}"), interaction)
        .clamp(0.0, 1.0)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, variant: CardVariant) -> f32 {
    MaterialTokenResolver::new(theme)
        .pressable_state_layer_opacity(
            &format!("{}.pressed.state-layer.opacity", component_prefix(variant)),
            PressableInteraction::Pressed,
        )
        .clamp(0.0, 1.0)
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
    fn card_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(
            container_shape(theme, CardVariant::Filled),
            Corners::all(Px(12.0))
        );
        assert_eq!(
            container_elevation(theme, CardVariant::Elevated, true, None),
            Px(0.0)
        );
        assert_eq!(
            outline(theme, CardVariant::Outlined, true, None)
                .expect("outlined card has outline")
                .width,
            Px(1.0)
        );
    }

    #[test]
    fn card_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.filled-card.container.shape".to_string(), 14.0);
        patch.metrics.insert(
            "md.comp.elevated-card.hover.container.elevation".to_string(),
            5.0,
        );
        patch
            .metrics
            .insert("md.comp.outlined-card.outline.width".to_string(), 2.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme, CardVariant::Filled),
            Corners::all(Px(14.0))
        );
        assert_eq!(
            container_elevation(
                &theme,
                CardVariant::Elevated,
                true,
                Some(PressableInteraction::Hovered),
            ),
            Px(5.0)
        );
        assert_eq!(
            outline(&theme, CardVariant::Outlined, true, None)
                .expect("outlined card has outline")
                .width,
            Px(2.0)
        );
    }

    #[test]
    fn card_shape_uses_system_fallback() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.shape.corner.medium".to_string(), 10.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme, CardVariant::Filled),
            Corners::all(Px(10.0))
        );
    }

    #[test]
    fn card_shape_prefers_structured_corners_over_uniform_metric() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.filled-card.container.shape".to_string(), 14.0);
        patch.corners.insert(
            "md.comp.filled-card.container.shape".to_string(),
            Corners {
                top_left: Px(2.0),
                top_right: Px(4.0),
                bottom_right: Px(6.0),
                bottom_left: Px(8.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme, CardVariant::Filled),
            Corners {
                top_left: Px(2.0),
                top_right: Px(4.0),
                bottom_right: Px(6.0),
                bottom_left: Px(8.0),
            }
        );
    }
}
