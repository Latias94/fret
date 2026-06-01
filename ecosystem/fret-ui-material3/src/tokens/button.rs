//! Typed token access for Material 3 buttons.
//!
//! This module centralizes token key mapping and fallback chains so button variants remain
//! consistent and drift-resistant during refactors.

use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::button::{ButtonSize, ButtonVariant};
use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};

// Keep the Material button root on a stable minimum width so snapshots and layout do not depend on
// underconstrained wrapper fill resolution.
const BUTTON_MIN_WIDTH: Px = Px(64.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonInteraction {
    Hovered,
    Focused,
    Pressed,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ButtonSizeTokens {
    pub(crate) min_width: Px,
    pub(crate) container_height: Px,
    pub(crate) leading_space: Px,
    pub(crate) trailing_space: Px,
    pub(crate) icon_size: Px,
    pub(crate) icon_label_space: Px,
    pub(crate) outlined_outline_width: Px,
}

#[derive(Debug, Clone, Copy)]
struct ButtonSizeMetricKeys {
    container_height: &'static str,
    leading_space: &'static str,
    trailing_space: &'static str,
    icon_size: &'static str,
    icon_label_space: &'static str,
    outlined_outline_width: &'static str,
}

fn button_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

fn button_metric_chain(theme: &Theme, keys: &[&'static str], fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(keys, fallback)
}

pub(crate) fn size_tokens(theme: &Theme, size: ButtonSize) -> ButtonSizeTokens {
    let keys = size_metric_keys(size);
    let defaults = size_metric_defaults(size);

    ButtonSizeTokens {
        min_width: BUTTON_MIN_WIDTH,
        container_height: button_metric(theme, keys.container_height, defaults.container_height),
        leading_space: button_metric(theme, keys.leading_space, defaults.leading_space),
        trailing_space: button_metric(theme, keys.trailing_space, defaults.trailing_space),
        icon_size: button_metric(theme, keys.icon_size, defaults.icon_size),
        icon_label_space: button_metric(theme, keys.icon_label_space, defaults.icon_label_space),
        outlined_outline_width: button_metric(
            theme,
            keys.outlined_outline_width,
            defaults.outlined_outline_width,
        ),
    }
}

pub(crate) fn container_shape_radius(theme: &Theme, size: ButtonSize) -> Px {
    button_metric_chain(
        theme,
        &[container_shape_round_key(size), "md.sys.shape.corner.full"],
        Px(999.0),
    )
}

pub(crate) fn pressed_container_shape_radius(theme: &Theme, size: ButtonSize) -> Px {
    button_metric_chain(
        theme,
        &[
            pressed_container_shape_key(size),
            "md.sys.shape.corner.small",
        ],
        Px(8.0),
    )
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
    button_metric(theme, key, Px(fallback))
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

pub(crate) fn outlined_outline_color(theme: &Theme, enabled: bool) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let color = if enabled {
        tokens.color_comp_or_sys_chain(
            "md.comp.button.outlined.outline.color",
            &["md.sys.color.outline-variant", "md.sys.color.outline"],
        )
    } else {
        tokens.color_comp_or_sys_chain(
            "md.comp.button.outlined.disabled.outline.color",
            &["md.sys.color.outline-variant", "md.sys.color.outline"],
        )
    };

    Color { a: 1.0, ..color }
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

fn size_metric_keys(size: ButtonSize) -> ButtonSizeMetricKeys {
    match size {
        ButtonSize::XSmall => ButtonSizeMetricKeys {
            container_height: "md.comp.button.xsmall.container.height",
            leading_space: "md.comp.button.xsmall.leading-space",
            trailing_space: "md.comp.button.xsmall.trailing-space",
            icon_size: "md.comp.button.xsmall.icon.size",
            icon_label_space: "md.comp.button.xsmall.icon-label-space",
            outlined_outline_width: "md.comp.button.xsmall.outlined.outline.width",
        },
        ButtonSize::Small => ButtonSizeMetricKeys {
            container_height: "md.comp.button.small.container.height",
            leading_space: "md.comp.button.small.leading-space",
            trailing_space: "md.comp.button.small.trailing-space",
            icon_size: "md.comp.button.small.icon.size",
            icon_label_space: "md.comp.button.small.icon-label-space",
            outlined_outline_width: "md.comp.button.small.outlined.outline.width",
        },
        ButtonSize::Medium => ButtonSizeMetricKeys {
            container_height: "md.comp.button.medium.container.height",
            leading_space: "md.comp.button.medium.leading-space",
            trailing_space: "md.comp.button.medium.trailing-space",
            icon_size: "md.comp.button.medium.icon.size",
            icon_label_space: "md.comp.button.medium.icon-label-space",
            outlined_outline_width: "md.comp.button.medium.outlined.outline.width",
        },
        ButtonSize::Large => ButtonSizeMetricKeys {
            container_height: "md.comp.button.large.container.height",
            leading_space: "md.comp.button.large.leading-space",
            trailing_space: "md.comp.button.large.trailing-space",
            icon_size: "md.comp.button.large.icon.size",
            icon_label_space: "md.comp.button.large.icon-label-space",
            outlined_outline_width: "md.comp.button.large.outlined.outline.width",
        },
        ButtonSize::XLarge => ButtonSizeMetricKeys {
            container_height: "md.comp.button.xlarge.container.height",
            leading_space: "md.comp.button.xlarge.leading-space",
            trailing_space: "md.comp.button.xlarge.trailing-space",
            icon_size: "md.comp.button.xlarge.icon.size",
            icon_label_space: "md.comp.button.xlarge.icon-label-space",
            outlined_outline_width: "md.comp.button.xlarge.outlined.outline.width",
        },
    }
}

fn size_metric_defaults(size: ButtonSize) -> ButtonSizeTokens {
    match size {
        ButtonSize::XSmall => ButtonSizeTokens {
            min_width: BUTTON_MIN_WIDTH,
            container_height: Px(32.0),
            leading_space: Px(12.0),
            trailing_space: Px(12.0),
            icon_size: Px(20.0),
            icon_label_space: Px(8.0),
            outlined_outline_width: Px(1.0),
        },
        ButtonSize::Small => ButtonSizeTokens {
            min_width: BUTTON_MIN_WIDTH,
            container_height: Px(40.0),
            leading_space: Px(16.0),
            trailing_space: Px(16.0),
            icon_size: Px(20.0),
            icon_label_space: Px(8.0),
            outlined_outline_width: Px(1.0),
        },
        ButtonSize::Medium => ButtonSizeTokens {
            min_width: BUTTON_MIN_WIDTH,
            container_height: Px(56.0),
            leading_space: Px(24.0),
            trailing_space: Px(24.0),
            icon_size: Px(24.0),
            icon_label_space: Px(8.0),
            outlined_outline_width: Px(1.0),
        },
        ButtonSize::Large => ButtonSizeTokens {
            min_width: BUTTON_MIN_WIDTH,
            container_height: Px(96.0),
            leading_space: Px(48.0),
            trailing_space: Px(48.0),
            icon_size: Px(32.0),
            icon_label_space: Px(12.0),
            outlined_outline_width: Px(2.0),
        },
        ButtonSize::XLarge => ButtonSizeTokens {
            min_width: BUTTON_MIN_WIDTH,
            container_height: Px(136.0),
            leading_space: Px(64.0),
            trailing_space: Px(64.0),
            icon_size: Px(40.0),
            icon_label_space: Px(16.0),
            outlined_outline_width: Px(3.0),
        },
    }
}

fn container_shape_round_key(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::XSmall => "md.comp.button.xsmall.container.shape.round",
        ButtonSize::Small => "md.comp.button.small.container.shape.round",
        ButtonSize::Medium => "md.comp.button.medium.container.shape.round",
        ButtonSize::Large => "md.comp.button.large.container.shape.round",
        ButtonSize::XLarge => "md.comp.button.xlarge.container.shape.round",
    }
}

fn pressed_container_shape_key(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::XSmall => "md.comp.button.xsmall.pressed.container.shape",
        ButtonSize::Small => "md.comp.button.small.pressed.container.shape",
        ButtonSize::Medium => "md.comp.button.medium.pressed.container.shape",
        ButtonSize::Large => "md.comp.button.large.pressed.container.shape",
        ButtonSize::XLarge => "md.comp.button.xlarge.pressed.container.shape",
    }
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
    fn button_size_defaults_match_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        let xsmall = size_tokens(theme, ButtonSize::XSmall);
        assert_eq!(xsmall.container_height, Px(32.0));
        assert_eq!(xsmall.leading_space, Px(12.0));
        assert_eq!(xsmall.icon_size, Px(20.0));
        assert_eq!(xsmall.outlined_outline_width, Px(1.0));

        let large = size_tokens(theme, ButtonSize::Large);
        assert_eq!(large.container_height, Px(96.0));
        assert_eq!(large.leading_space, Px(48.0));
        assert_eq!(large.icon_size, Px(32.0));
        assert_eq!(large.outlined_outline_width, Px(2.0));

        let xlarge = size_tokens(theme, ButtonSize::XLarge);
        assert_eq!(xlarge.container_height, Px(136.0));
        assert_eq!(xlarge.trailing_space, Px(64.0));
        assert_eq!(xlarge.icon_label_space, Px(16.0));
        assert_eq!(xlarge.outlined_outline_width, Px(3.0));
    }

    #[test]
    fn button_metric_chains_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.button.medium.container.height".to_string(), 60.0);
        patch
            .metrics
            .insert("md.comp.button.medium.leading-space".to_string(), 28.0);
        patch.metrics.insert(
            "md.comp.button.medium.outlined.outline.width".to_string(),
            2.0,
        );
        patch
            .metrics
            .insert("md.sys.shape.corner.full".to_string(), 88.0);
        patch.metrics.insert(
            "md.comp.button.medium.pressed.container.shape".to_string(),
            12.0,
        );
        patch.metrics.insert(
            "md.comp.button.filled.hovered.container.elevation".to_string(),
            4.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        let medium = size_tokens(&theme, ButtonSize::Medium);
        assert_eq!(medium.container_height, Px(60.0));
        assert_eq!(medium.leading_space, Px(28.0));
        assert_eq!(medium.outlined_outline_width, Px(2.0));
        assert_eq!(container_shape_radius(&theme, ButtonSize::Medium), Px(88.0));
        assert_eq!(
            pressed_container_shape_radius(&theme, ButtonSize::Medium),
            Px(12.0)
        );
        assert_eq!(
            container_elevation(
                &theme,
                ButtonVariant::Filled,
                true,
                Some(ButtonInteraction::Hovered),
            ),
            Px(4.0)
        );
    }
}
