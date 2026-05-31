//! Typed token access for Material 3 cards.
//!
//! This module centralizes token key mapping and fallback chains so card variants remain
//! consistent and drift-resistant during refactors.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::card::CardVariant;
use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};

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

pub(crate) fn container_shape(theme: &Theme, variant: CardVariant) -> Corners {
    theme
        .metric_by_key(&format!("{}.container.shape", component_prefix(variant)))
        .map(Corners::all)
        .or_else(|| {
            theme
                .metric_by_key("md.sys.shape.corner.medium")
                .map(Corners::all)
        })
        .unwrap_or_else(|| Corners::all(Px(12.0)))
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
        return theme
            .metric_by_key(&format!("{prefix}.disabled.container.elevation"))
            .unwrap_or(Px(0.0));
    }

    let key = match interaction {
        Some(PressableInteraction::Pressed) => "pressed.container.elevation",
        Some(PressableInteraction::Focused) => "focus.container.elevation",
        Some(PressableInteraction::Hovered) => "hover.container.elevation",
        None => "container.elevation",
    };

    theme
        .metric_by_key(&format!("{prefix}.{key}"))
        .unwrap_or(Px(0.0))
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
    let width = theme
        .metric_by_key(&format!("{prefix}.outline.width"))
        .unwrap_or(Px(1.0));

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

    let key = match interaction {
        Some(PressableInteraction::Pressed) => "pressed.outline.color",
        Some(PressableInteraction::Focused) => "focus.outline.color",
        Some(PressableInteraction::Hovered) => "hover.outline.color",
        None => "outline.color",
    };

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
    let key = match interaction {
        Some(PressableInteraction::Pressed) => "pressed.state-layer.color",
        Some(PressableInteraction::Focused) => "focus.state-layer.color",
        Some(PressableInteraction::Hovered) => "hover.state-layer.color",
        None => "hover.state-layer.color",
    };

    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(&format!("{prefix}.{key}"), "md.sys.color.on-surface")
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    variant: CardVariant,
    interaction: Option<PressableInteraction>,
) -> f32 {
    let prefix = component_prefix(variant);
    let key = match interaction {
        Some(PressableInteraction::Pressed) => "pressed.state-layer.opacity",
        Some(PressableInteraction::Focused) => "focus.state-layer.opacity",
        Some(PressableInteraction::Hovered) => "hover.state-layer.opacity",
        None => return 0.0,
    };

    let Some(interaction) = material_state_layer_interaction(interaction) else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme)
        .state_layer_opacity(&format!("{prefix}.{key}"), interaction)
        .clamp(0.0, 1.0)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, variant: CardVariant) -> f32 {
    MaterialTokenResolver::new(theme)
        .state_layer_opacity(
            &format!("{}.pressed.state-layer.opacity", component_prefix(variant)),
            MaterialStateLayerInteraction::Pressed,
        )
        .clamp(0.0, 1.0)
}

fn material_state_layer_interaction(
    interaction: Option<PressableInteraction>,
) -> Option<MaterialStateLayerInteraction> {
    match interaction {
        Some(PressableInteraction::Pressed) => Some(MaterialStateLayerInteraction::Pressed),
        Some(PressableInteraction::Focused) => Some(MaterialStateLayerInteraction::Focused),
        Some(PressableInteraction::Hovered) => Some(MaterialStateLayerInteraction::Hovered),
        None => None,
    }
}
