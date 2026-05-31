//! Typed token access for Material 3 buttons.
//!
//! This module centralizes token key mapping and fallback chains so button variants remain
//! consistent and drift-resistant during refactors.

use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::button::ButtonVariant;
use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonInteraction {
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn label_color(theme: &Theme, variant: ButtonVariant, enabled: bool) -> Color {
    let tokens = MaterialTokenResolver::new(theme);

    if enabled {
        tokens.color_comp_or_sys_chain(label_color_key(variant), label_sys_keys(variant))
    } else {
        let base =
            tokens.color_comp_or_sys(disabled_label_color_key(variant), "md.sys.color.on-surface");
        alpha_mul(base, disabled_label_opacity(theme, variant))
    }
}

fn label_sys_keys(variant: ButtonVariant) -> &'static [&'static str] {
    match variant {
        ButtonVariant::Filled => &["md.sys.color.on-primary", "md.sys.color.on-surface"],
        ButtonVariant::Tonal => &[
            "md.sys.color.on-secondary-container",
            "md.sys.color.on-surface",
        ],
        ButtonVariant::Elevated | ButtonVariant::Text => {
            &["md.sys.color.primary", "md.sys.color.on-surface"]
        }
        ButtonVariant::Outlined => &["md.sys.color.on-surface-variant", "md.sys.color.on-surface"],
    }
}

pub(crate) fn container_background(
    theme: &Theme,
    variant: ButtonVariant,
    enabled: bool,
    label_fallback: Color,
) -> Option<Color> {
    let (container_key, sys_key) = enabled_container_color_keys(variant)?;
    let tokens = MaterialTokenResolver::new(theme);

    if enabled {
        Some(tokens.color_comp_or_sys(container_key, sys_key))
    } else {
        Some(disabled_container_color(
            theme,
            variant,
            disabled_container_color_key(variant),
            label_fallback,
        ))
    }
}

pub(crate) fn container_elevation(
    theme: &Theme,
    variant: ButtonVariant,
    enabled: bool,
    interaction: Option<ButtonInteraction>,
) -> Px {
    let key = container_elevation_key(variant, enabled, interaction);
    let fallback = container_elevation_fallback(variant, enabled, interaction);
    theme.metric_by_key(key).unwrap_or(Px(fallback))
}

pub(crate) fn container_shadow_color(theme: &Theme, variant: ButtonVariant) -> Color {
    MaterialTokenResolver::new(theme).color_comp_chain_or_sys(
        &[
            container_shadow_color_key(variant),
            "md.comp.button.container.shadow-color",
        ],
        "md.sys.color.shadow",
    )
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    variant: ButtonVariant,
    label_fallback: Color,
    interaction: Option<ButtonInteraction>,
) -> Color {
    let Some(interaction) = interaction else {
        return label_fallback;
    };

    MaterialTokenResolver::new(theme)
        .color_comp_or_fallback(state_layer_color_key(variant, interaction), label_fallback)
}

pub(crate) fn icon_color(
    theme: &Theme,
    variant: ButtonVariant,
    enabled: bool,
    label_fallback: Color,
    interaction: Option<ButtonInteraction>,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);

    if !enabled {
        let base = tokens.color_comp_chain_or_sys_or(
            &[
                disabled_icon_color_key(variant),
                "md.comp.button.disabled.icon.color",
            ],
            "md.sys.color.on-surface",
            label_fallback,
        );
        return alpha_mul(base, disabled_icon_opacity(theme, variant));
    }

    if let Some(interaction) = interaction
        && let Some(c) = tokens.color_comp_chain(&[
            interaction_icon_color_key(variant, interaction),
            interaction_icon_color_key_any(interaction),
        ])
    {
        return c;
    }

    tokens.color_comp_chain_or_fallback(
        &[icon_color_key(variant), "md.comp.button.icon.color"],
        label_fallback,
    )
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    variant: ButtonVariant,
    interaction: ButtonInteraction,
) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        state_layer_opacity_key(variant, interaction),
        material_state_layer_interaction(interaction),
    )
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, variant: ButtonVariant) -> f32 {
    state_layer_opacity(theme, variant, ButtonInteraction::Pressed)
}

fn disabled_label_opacity(theme: &Theme, variant: ButtonVariant) -> f32 {
    MaterialTokenResolver::new(theme)
        .number_optional(Some(disabled_label_opacity_key(variant)), 0.38)
}

fn disabled_icon_opacity(theme: &Theme, variant: ButtonVariant) -> f32 {
    MaterialTokenResolver::new(theme).number_chain(
        &[
            disabled_icon_opacity_key(variant),
            "md.comp.button.disabled.icon.opacity",
        ],
        0.38,
    )
}

fn disabled_container_opacity(theme: &Theme, variant: ButtonVariant) -> f32 {
    MaterialTokenResolver::new(theme)
        .number_optional(Some(disabled_container_opacity_key(variant)), 0.1)
}

fn disabled_container_color(
    theme: &Theme,
    variant: ButtonVariant,
    token_key: &'static str,
    fallback: Color,
) -> Color {
    let base = MaterialTokenResolver::new(theme).color_comp_or_sys_or(
        token_key,
        "md.sys.color.on-surface",
        fallback,
    );
    alpha_mul(base, disabled_container_opacity(theme, variant))
}

fn material_state_layer_interaction(
    interaction: ButtonInteraction,
) -> MaterialStateLayerInteraction {
    match interaction {
        ButtonInteraction::Hovered => MaterialStateLayerInteraction::Hovered,
        ButtonInteraction::Focused => MaterialStateLayerInteraction::Focused,
        ButtonInteraction::Pressed => MaterialStateLayerInteraction::Pressed,
    }
}

fn enabled_container_color_keys(variant: ButtonVariant) -> Option<(&'static str, &'static str)> {
    match variant {
        ButtonVariant::Filled => Some((
            "md.comp.button.filled.container.color",
            "md.sys.color.primary",
        )),
        ButtonVariant::Tonal => Some((
            "md.comp.button.tonal.container.color",
            "md.sys.color.secondary-container",
        )),
        ButtonVariant::Elevated => Some((
            "md.comp.button.elevated.container.color",
            "md.sys.color.surface-container-low",
        )),
        ButtonVariant::Outlined | ButtonVariant::Text => None,
    }
}

fn disabled_container_color_key(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "md.comp.button.filled.disabled.container.color",
        ButtonVariant::Tonal => "md.comp.button.tonal.disabled.container.color",
        ButtonVariant::Elevated => "md.comp.button.elevated.disabled.container.color",
        ButtonVariant::Outlined => "md.comp.button.outlined.disabled.container.color",
        ButtonVariant::Text => "md.comp.button.text.disabled.container.color",
    }
}

fn container_elevation_key(
    variant: ButtonVariant,
    enabled: bool,
    interaction: Option<ButtonInteraction>,
) -> &'static str {
    if !enabled {
        return match variant {
            ButtonVariant::Filled => "md.comp.button.filled.disabled.container.elevation",
            ButtonVariant::Tonal => "md.comp.button.tonal.disabled.container.elevation",
            ButtonVariant::Elevated => "md.comp.button.elevated.disabled.container.elevation",
            ButtonVariant::Outlined => "md.comp.button.outlined.disabled.container.elevation",
            ButtonVariant::Text => "md.comp.button.text.disabled.container.elevation",
        };
    }

    match (variant, interaction) {
        (ButtonVariant::Filled, Some(ButtonInteraction::Hovered)) => {
            "md.comp.button.filled.hovered.container.elevation"
        }
        (ButtonVariant::Filled, Some(ButtonInteraction::Focused)) => {
            "md.comp.button.filled.focused.container.elevation"
        }
        (ButtonVariant::Filled, Some(ButtonInteraction::Pressed)) => {
            "md.comp.button.filled.pressed.container.elevation"
        }
        (ButtonVariant::Filled, None) => "md.comp.button.filled.container.elevation",

        (ButtonVariant::Tonal, Some(ButtonInteraction::Hovered)) => {
            "md.comp.button.tonal.hovered.container.elevation"
        }
        (ButtonVariant::Tonal, Some(ButtonInteraction::Focused)) => {
            "md.comp.button.tonal.focused.container.elevation"
        }
        (ButtonVariant::Tonal, Some(ButtonInteraction::Pressed)) => {
            "md.comp.button.tonal.pressed.container.elevation"
        }
        (ButtonVariant::Tonal, None) => "md.comp.button.tonal.container.elevation",

        (ButtonVariant::Elevated, Some(ButtonInteraction::Hovered)) => {
            "md.comp.button.elevated.hovered.container.elevation"
        }
        (ButtonVariant::Elevated, Some(ButtonInteraction::Focused)) => {
            "md.comp.button.elevated.focused.container.elevation"
        }
        (ButtonVariant::Elevated, Some(ButtonInteraction::Pressed)) => {
            "md.comp.button.elevated.pressed.container.elevation"
        }
        (ButtonVariant::Elevated, None) => "md.comp.button.elevated.container.elevation",

        (ButtonVariant::Outlined, Some(ButtonInteraction::Hovered)) => {
            "md.comp.button.outlined.hovered.container.elevation"
        }
        (ButtonVariant::Outlined, Some(ButtonInteraction::Focused)) => {
            "md.comp.button.outlined.focused.container.elevation"
        }
        (ButtonVariant::Outlined, Some(ButtonInteraction::Pressed)) => {
            "md.comp.button.outlined.pressed.container.elevation"
        }
        (ButtonVariant::Outlined, None) => "md.comp.button.outlined.container.elevation",

        (ButtonVariant::Text, Some(ButtonInteraction::Hovered)) => {
            "md.comp.button.text.hovered.container.elevation"
        }
        (ButtonVariant::Text, Some(ButtonInteraction::Focused)) => {
            "md.comp.button.text.focused.container.elevation"
        }
        (ButtonVariant::Text, Some(ButtonInteraction::Pressed)) => {
            "md.comp.button.text.pressed.container.elevation"
        }
        (ButtonVariant::Text, None) => "md.comp.button.text.container.elevation",
    }
}

fn container_elevation_fallback(
    variant: ButtonVariant,
    enabled: bool,
    interaction: Option<ButtonInteraction>,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    match (variant, interaction) {
        (ButtonVariant::Elevated, Some(ButtonInteraction::Hovered)) => 3.0,
        (ButtonVariant::Elevated, _) => 1.0,
        (ButtonVariant::Filled | ButtonVariant::Tonal, Some(ButtonInteraction::Hovered)) => 1.0,
        (ButtonVariant::Filled | ButtonVariant::Tonal, _) => 0.0,
        (ButtonVariant::Outlined | ButtonVariant::Text, _) => 0.0,
    }
}

fn container_shadow_color_key(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "md.comp.button.filled.container.shadow-color",
        ButtonVariant::Tonal => "md.comp.button.tonal.container.shadow-color",
        ButtonVariant::Elevated => "md.comp.button.elevated.container.shadow-color",
        ButtonVariant::Outlined => "md.comp.button.outlined.container.shadow-color",
        ButtonVariant::Text => "md.comp.button.text.container.shadow-color",
    }
}

fn label_color_key(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "md.comp.button.filled.label-text.color",
        ButtonVariant::Tonal => "md.comp.button.tonal.label-text.color",
        ButtonVariant::Elevated => "md.comp.button.elevated.label-text.color",
        ButtonVariant::Outlined => "md.comp.button.outlined.label-text.color",
        ButtonVariant::Text => "md.comp.button.text.label-text.color",
    }
}

fn disabled_label_color_key(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "md.comp.button.filled.disabled.label-text.color",
        ButtonVariant::Tonal => "md.comp.button.tonal.disabled.label-text.color",
        ButtonVariant::Elevated => "md.comp.button.elevated.disabled.label-text.color",
        ButtonVariant::Outlined => "md.comp.button.outlined.disabled.label-text.color",
        ButtonVariant::Text => "md.comp.button.text.disabled.label-text.color",
    }
}

fn disabled_icon_color_key(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "md.comp.button.filled.disabled.icon.color",
        ButtonVariant::Tonal => "md.comp.button.tonal.disabled.icon.color",
        ButtonVariant::Elevated => "md.comp.button.elevated.disabled.icon.color",
        ButtonVariant::Outlined => "md.comp.button.outlined.disabled.icon.color",
        ButtonVariant::Text => "md.comp.button.text.disabled.icon.color",
    }
}

fn disabled_label_opacity_key(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "md.comp.button.filled.disabled.label-text.opacity",
        ButtonVariant::Tonal => "md.comp.button.tonal.disabled.label-text.opacity",
        ButtonVariant::Elevated => "md.comp.button.elevated.disabled.label-text.opacity",
        ButtonVariant::Outlined => "md.comp.button.outlined.disabled.label-text.opacity",
        ButtonVariant::Text => "md.comp.button.text.disabled.label-text.opacity",
    }
}

fn disabled_icon_opacity_key(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "md.comp.button.filled.disabled.icon.opacity",
        ButtonVariant::Tonal => "md.comp.button.tonal.disabled.icon.opacity",
        ButtonVariant::Elevated => "md.comp.button.elevated.disabled.icon.opacity",
        ButtonVariant::Outlined => "md.comp.button.outlined.disabled.icon.opacity",
        ButtonVariant::Text => "md.comp.button.text.disabled.icon.opacity",
    }
}

fn disabled_container_opacity_key(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "md.comp.button.filled.disabled.container.opacity",
        ButtonVariant::Tonal => "md.comp.button.tonal.disabled.container.opacity",
        ButtonVariant::Elevated => "md.comp.button.elevated.disabled.container.opacity",
        ButtonVariant::Outlined => "md.comp.button.outlined.disabled.container.opacity",
        ButtonVariant::Text => "md.comp.button.text.disabled.container.opacity",
    }
}

fn icon_color_key(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Filled => "md.comp.button.filled.icon.color",
        ButtonVariant::Tonal => "md.comp.button.tonal.icon.color",
        ButtonVariant::Elevated => "md.comp.button.elevated.icon.color",
        ButtonVariant::Outlined => "md.comp.button.outlined.icon.color",
        ButtonVariant::Text => "md.comp.button.text.icon.color",
    }
}

fn interaction_icon_color_key(
    variant: ButtonVariant,
    interaction: ButtonInteraction,
) -> &'static str {
    match (variant, interaction) {
        (ButtonVariant::Filled, ButtonInteraction::Hovered) => {
            "md.comp.button.filled.hovered.icon.color"
        }
        (ButtonVariant::Filled, ButtonInteraction::Focused) => {
            "md.comp.button.filled.focused.icon.color"
        }
        (ButtonVariant::Filled, ButtonInteraction::Pressed) => {
            "md.comp.button.filled.pressed.icon.color"
        }

        (ButtonVariant::Tonal, ButtonInteraction::Hovered) => {
            "md.comp.button.tonal.hovered.icon.color"
        }
        (ButtonVariant::Tonal, ButtonInteraction::Focused) => {
            "md.comp.button.tonal.focused.icon.color"
        }
        (ButtonVariant::Tonal, ButtonInteraction::Pressed) => {
            "md.comp.button.tonal.pressed.icon.color"
        }

        (ButtonVariant::Elevated, ButtonInteraction::Hovered) => {
            "md.comp.button.elevated.hovered.icon.color"
        }
        (ButtonVariant::Elevated, ButtonInteraction::Focused) => {
            "md.comp.button.elevated.focused.icon.color"
        }
        (ButtonVariant::Elevated, ButtonInteraction::Pressed) => {
            "md.comp.button.elevated.pressed.icon.color"
        }

        (ButtonVariant::Outlined, ButtonInteraction::Hovered) => {
            "md.comp.button.outlined.hovered.icon.color"
        }
        (ButtonVariant::Outlined, ButtonInteraction::Focused) => {
            "md.comp.button.outlined.focused.icon.color"
        }
        (ButtonVariant::Outlined, ButtonInteraction::Pressed) => {
            "md.comp.button.outlined.pressed.icon.color"
        }

        (ButtonVariant::Text, ButtonInteraction::Hovered) => {
            "md.comp.button.text.hovered.icon.color"
        }
        (ButtonVariant::Text, ButtonInteraction::Focused) => {
            "md.comp.button.text.focused.icon.color"
        }
        (ButtonVariant::Text, ButtonInteraction::Pressed) => {
            "md.comp.button.text.pressed.icon.color"
        }
    }
}

fn interaction_icon_color_key_any(interaction: ButtonInteraction) -> &'static str {
    match interaction {
        ButtonInteraction::Hovered => "md.comp.button.hovered.icon.color",
        ButtonInteraction::Focused => "md.comp.button.focused.icon.color",
        ButtonInteraction::Pressed => "md.comp.button.pressed.icon.color",
    }
}

fn state_layer_color_key(variant: ButtonVariant, interaction: ButtonInteraction) -> &'static str {
    match (variant, interaction) {
        (ButtonVariant::Filled, ButtonInteraction::Hovered) => {
            "md.comp.button.filled.hovered.state-layer.color"
        }
        (ButtonVariant::Filled, ButtonInteraction::Focused) => {
            "md.comp.button.filled.focused.state-layer.color"
        }
        (ButtonVariant::Filled, ButtonInteraction::Pressed) => {
            "md.comp.button.filled.pressed.state-layer.color"
        }

        (ButtonVariant::Tonal, ButtonInteraction::Hovered) => {
            "md.comp.button.tonal.hovered.state-layer.color"
        }
        (ButtonVariant::Tonal, ButtonInteraction::Focused) => {
            "md.comp.button.tonal.focused.state-layer.color"
        }
        (ButtonVariant::Tonal, ButtonInteraction::Pressed) => {
            "md.comp.button.tonal.pressed.state-layer.color"
        }

        (ButtonVariant::Elevated, ButtonInteraction::Hovered) => {
            "md.comp.button.elevated.hovered.state-layer.color"
        }
        (ButtonVariant::Elevated, ButtonInteraction::Focused) => {
            "md.comp.button.elevated.focused.state-layer.color"
        }
        (ButtonVariant::Elevated, ButtonInteraction::Pressed) => {
            "md.comp.button.elevated.pressed.state-layer.color"
        }

        (ButtonVariant::Outlined, ButtonInteraction::Hovered) => {
            "md.comp.button.outlined.hovered.state-layer.color"
        }
        (ButtonVariant::Outlined, ButtonInteraction::Focused) => {
            "md.comp.button.outlined.focused.state-layer.color"
        }
        (ButtonVariant::Outlined, ButtonInteraction::Pressed) => {
            "md.comp.button.outlined.pressed.state-layer.color"
        }

        (ButtonVariant::Text, ButtonInteraction::Hovered) => {
            "md.comp.button.text.hovered.state-layer.color"
        }
        (ButtonVariant::Text, ButtonInteraction::Focused) => {
            "md.comp.button.text.focused.state-layer.color"
        }
        (ButtonVariant::Text, ButtonInteraction::Pressed) => {
            "md.comp.button.text.pressed.state-layer.color"
        }
    }
}

fn state_layer_opacity_key(variant: ButtonVariant, interaction: ButtonInteraction) -> &'static str {
    match (variant, interaction) {
        (ButtonVariant::Filled, ButtonInteraction::Hovered) => {
            "md.comp.button.filled.hovered.state-layer.opacity"
        }
        (ButtonVariant::Filled, ButtonInteraction::Focused) => {
            "md.comp.button.filled.focused.state-layer.opacity"
        }
        (ButtonVariant::Filled, ButtonInteraction::Pressed) => {
            "md.comp.button.filled.pressed.state-layer.opacity"
        }

        (ButtonVariant::Tonal, ButtonInteraction::Hovered) => {
            "md.comp.button.tonal.hovered.state-layer.opacity"
        }
        (ButtonVariant::Tonal, ButtonInteraction::Focused) => {
            "md.comp.button.tonal.focused.state-layer.opacity"
        }
        (ButtonVariant::Tonal, ButtonInteraction::Pressed) => {
            "md.comp.button.tonal.pressed.state-layer.opacity"
        }

        (ButtonVariant::Elevated, ButtonInteraction::Hovered) => {
            "md.comp.button.elevated.hovered.state-layer.opacity"
        }
        (ButtonVariant::Elevated, ButtonInteraction::Focused) => {
            "md.comp.button.elevated.focused.state-layer.opacity"
        }
        (ButtonVariant::Elevated, ButtonInteraction::Pressed) => {
            "md.comp.button.elevated.pressed.state-layer.opacity"
        }

        (ButtonVariant::Outlined, ButtonInteraction::Hovered) => {
            "md.comp.button.outlined.hovered.state-layer.opacity"
        }
        (ButtonVariant::Outlined, ButtonInteraction::Focused) => {
            "md.comp.button.outlined.focused.state-layer.opacity"
        }
        (ButtonVariant::Outlined, ButtonInteraction::Pressed) => {
            "md.comp.button.outlined.pressed.state-layer.opacity"
        }

        (ButtonVariant::Text, ButtonInteraction::Hovered) => {
            "md.comp.button.text.hovered.state-layer.opacity"
        }
        (ButtonVariant::Text, ButtonInteraction::Focused) => {
            "md.comp.button.text.focused.state-layer.opacity"
        }
        (ButtonVariant::Text, ButtonInteraction::Pressed) => {
            "md.comp.button.text.pressed.state-layer.opacity"
        }
    }
}
