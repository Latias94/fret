//! Typed token access for Material 3 icon buttons.
//!
//! This module exists to reduce ad-hoc `format!` key building in components and centralize:
//! - key mapping across variants and toggle states,
//! - fallback chains (component token -> sys token -> required sys token),
//! - derived values like disabled alpha application.

use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};
use crate::icon_button::IconButtonSize;
use crate::icon_button::IconButtonVariant;
use crate::motion::SpringSpec;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.icon-button";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IconButtonInteraction {
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn selected_container_shape_radius(theme: &Theme) -> f32 {
    theme
        .metric_by_key("md.comp.icon-button.selected.container.shape.round")
        .or_else(|| theme.metric_by_key("md.comp.icon-button.container.shape.round"))
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0))
        .0
}

pub(crate) fn icon_color(
    theme: &Theme,
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    enabled: bool,
    interaction: Option<IconButtonInteraction>,
) -> Color {
    let base_key = icon_color_key(variant, toggle, selected, interaction);
    let tokens = MaterialTokenResolver::new(theme);
    if enabled {
        tokens.color_comp_or_sys_chain(
            base_key,
            &["md.sys.color.on-surface-variant", "md.sys.color.on-surface"],
        )
    } else {
        let color = tokens.color_comp_or_sys_chain(
            disabled_icon_color_key(variant),
            &["md.sys.color.on-surface-variant", "md.sys.color.on-surface"],
        );
        let opacity = tokens.number_optional(Some(disabled_icon_opacity_key(variant)), 0.38);
        alpha_mul(color, opacity)
    }
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    enabled: bool,
    interaction: Option<IconButtonInteraction>,
) -> Color {
    let pressed_key = state_layer_color_key(
        variant,
        toggle,
        selected,
        Some(IconButtonInteraction::Pressed),
    );
    let tokens = MaterialTokenResolver::new(theme);
    let mut color = tokens.color_comp_or_sys_or(
        pressed_key,
        "md.sys.color.on-surface-variant",
        icon_color(theme, variant, toggle, selected, enabled, None),
    );

    if let Some(interaction) = interaction {
        let key = state_layer_color_key(variant, toggle, selected, Some(interaction));
        color = tokens.color_comp_or_fallback(key, color);
    }

    color
}

pub(crate) fn pressed_state_layer_opacity(
    theme: &Theme,
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
) -> f32 {
    state_layer_opacity(
        theme,
        variant,
        toggle,
        selected,
        IconButtonInteraction::Pressed,
    )
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    interaction: IconButtonInteraction,
) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        state_layer_opacity_key(variant, toggle, selected, interaction),
        material_state_layer_interaction(interaction),
    )
}

pub(crate) fn container_background(
    theme: &Theme,
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    enabled: bool,
    icon_fallback: Color,
) -> Option<Color> {
    match variant {
        IconButtonVariant::Standard => None,
        IconButtonVariant::Filled => {
            if enabled {
                MaterialTokenResolver::new(theme)
                    .color_comp_chain(&[container_color_key_filled(toggle, selected)])
            } else {
                Some(disabled_container_color(
                    theme,
                    "md.comp.icon-button.filled.disabled.container.color",
                    "md.comp.icon-button.filled.disabled.container.opacity",
                    icon_fallback,
                ))
            }
        }
        IconButtonVariant::Tonal => {
            if enabled {
                MaterialTokenResolver::new(theme)
                    .color_comp_chain(&[container_color_key_tonal(toggle, selected)])
            } else {
                Some(disabled_container_color(
                    theme,
                    "md.comp.icon-button.tonal.disabled.container.color",
                    "md.comp.icon-button.tonal.disabled.container.opacity",
                    icon_fallback,
                ))
            }
        }
        IconButtonVariant::Outlined => {
            if !toggle || !selected {
                None
            } else if enabled {
                MaterialTokenResolver::new(theme)
                    .color_comp_chain(&["md.comp.icon-button.outlined.selected.container.color"])
            } else {
                Some(disabled_container_color(
                    theme,
                    "md.comp.icon-button.outlined.selected.disabled.container.color",
                    "md.comp.icon-button.outlined.selected.disabled.container.opacity",
                    icon_fallback,
                ))
            }
        }
    }
}

pub(crate) fn outlined_outline_color(theme: &Theme, enabled: bool) -> Color {
    let comp_key = if !enabled {
        "md.comp.icon-button.outlined.disabled.outline.color"
    } else {
        "md.comp.icon-button.outlined.outline.color"
    };
    let mut color = MaterialTokenResolver::new(theme).color_comp_or_sys_chain(
        comp_key,
        &["md.sys.color.outline-variant", "md.sys.color.outline"],
    );

    color.a = 1.0;
    color
}

fn disabled_container_color(
    theme: &Theme,
    color_key: &'static str,
    opacity_key: &'static str,
    fallback: Color,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let color = tokens.color_comp_or_sys_or(color_key, "md.sys.color.on-surface", fallback);
    let opacity = tokens.number_optional(Some(opacity_key), 0.1);
    alpha_mul(color, opacity)
}

fn material_state_layer_interaction(
    interaction: IconButtonInteraction,
) -> MaterialStateLayerInteraction {
    match interaction {
        IconButtonInteraction::Hovered => MaterialStateLayerInteraction::Hovered,
        IconButtonInteraction::Focused => MaterialStateLayerInteraction::Focused,
        IconButtonInteraction::Pressed => MaterialStateLayerInteraction::Pressed,
    }
}

pub(crate) fn container_shape_radius(theme: &Theme) -> f32 {
    theme
        .metric_by_key("md.comp.icon-button.container.shape.round")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0))
        .0
}

pub(crate) fn pressed_container_shape_radius(theme: &Theme) -> f32 {
    theme
        .metric_by_key("md.comp.icon-button.pressed.container.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.small"))
        .unwrap_or(Px(8.0))
        .0
}

pub(crate) fn pressed_container_corner_spring(
    theme: &Theme,
    scheme_fallback: SpringSpec,
) -> SpringSpec {
    let tokens = MaterialTokenResolver::new(theme);
    SpringSpec {
        damping: tokens.number_comp_or_sys(
            "md.comp.icon-button.pressed.container.corner-size.motion.spring.damping",
            "md.sys.motion.spring.fast.spatial.damping",
            scheme_fallback.damping,
        ),
        stiffness: tokens.number_comp_or_sys(
            "md.comp.icon-button.pressed.container.corner-size.motion.spring.stiffness",
            "md.sys.motion.spring.fast.spatial.stiffness",
            scheme_fallback.stiffness,
        ),
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IconButtonSizeTokens {
    pub container: Px,
    pub pad_left: Px,
    pub pad_right: Px,
    pub icon_size: Px,
    pub outline_width: Px,
}

pub(crate) fn size_tokens(theme: &Theme, size: IconButtonSize) -> IconButtonSizeTokens {
    match size {
        IconButtonSize::Small => IconButtonSizeTokens {
            container: theme
                .metric_by_key("md.comp.icon-button.small.container.height")
                .unwrap_or(Px(40.0)),
            pad_left: theme
                .metric_by_key("md.comp.icon-button.small.default.leading-space")
                .unwrap_or(Px(8.0)),
            pad_right: theme
                .metric_by_key("md.comp.icon-button.small.default.trailing-space")
                .unwrap_or(Px(8.0)),
            icon_size: theme
                .metric_by_key("md.comp.icon-button.small.icon.size")
                .unwrap_or(Px(24.0)),
            outline_width: theme
                .metric_by_key("md.comp.icon-button.small.outlined.outline.width")
                .unwrap_or(Px(1.0)),
        },
    }
}

fn state_layer_opacity_key(
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    interaction: IconButtonInteraction,
) -> &'static str {
    if toggle && selected {
        match interaction {
            IconButtonInteraction::Hovered => {
                "md.comp.icon-button.selected.hover.state-layer.opacity"
            }
            IconButtonInteraction::Focused => {
                "md.comp.icon-button.selected.focus.state-layer.opacity"
            }
            IconButtonInteraction::Pressed => {
                "md.comp.icon-button.selected.pressed.state-layer.opacity"
            }
        }
    } else {
        match (variant, interaction) {
            (IconButtonVariant::Standard, IconButtonInteraction::Hovered) => {
                "md.comp.icon-button.standard.hovered.state-layer.opacity"
            }
            (IconButtonVariant::Standard, IconButtonInteraction::Focused) => {
                "md.comp.icon-button.standard.focused.state-layer.opacity"
            }
            (IconButtonVariant::Standard, IconButtonInteraction::Pressed) => {
                "md.comp.icon-button.standard.pressed.state-layer.opacity"
            }
            (IconButtonVariant::Filled, IconButtonInteraction::Hovered) => {
                "md.comp.icon-button.filled.hovered.state-layer.opacity"
            }
            (IconButtonVariant::Filled, IconButtonInteraction::Focused) => {
                "md.comp.icon-button.filled.focused.state-layer.opacity"
            }
            (IconButtonVariant::Filled, IconButtonInteraction::Pressed) => {
                "md.comp.icon-button.filled.pressed.state-layer.opacity"
            }
            (IconButtonVariant::Tonal, IconButtonInteraction::Hovered) => {
                "md.comp.icon-button.tonal.hovered.state-layer.opacity"
            }
            (IconButtonVariant::Tonal, IconButtonInteraction::Focused) => {
                "md.comp.icon-button.tonal.focused.state-layer.opacity"
            }
            (IconButtonVariant::Tonal, IconButtonInteraction::Pressed) => {
                "md.comp.icon-button.tonal.pressed.state-layer.opacity"
            }
            (IconButtonVariant::Outlined, IconButtonInteraction::Hovered) => {
                "md.comp.icon-button.outlined.hovered.state-layer.opacity"
            }
            (IconButtonVariant::Outlined, IconButtonInteraction::Focused) => {
                "md.comp.icon-button.outlined.focused.state-layer.opacity"
            }
            (IconButtonVariant::Outlined, IconButtonInteraction::Pressed) => {
                "md.comp.icon-button.outlined.pressed.state-layer.opacity"
            }
        }
    }
}

fn container_color_key_filled(toggle: bool, selected: bool) -> &'static str {
    if !toggle || selected {
        "md.comp.icon-button.filled.container.color"
    } else {
        "md.comp.icon-button.filled.unselected.container.color"
    }
}

fn container_color_key_tonal(toggle: bool, selected: bool) -> &'static str {
    if toggle && selected {
        "md.comp.icon-button.tonal.selected.container.color"
    } else {
        "md.comp.icon-button.tonal.container.color"
    }
}

fn icon_color_key(
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    interaction: Option<IconButtonInteraction>,
) -> &'static str {
    match (variant, toggle, selected, interaction) {
        (IconButtonVariant::Standard, false, _, None) => "md.comp.icon-button.standard.icon.color",
        (IconButtonVariant::Standard, false, _, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.standard.hovered.icon.color"
        }
        (IconButtonVariant::Standard, false, _, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.standard.focused.icon.color"
        }
        (IconButtonVariant::Standard, false, _, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.standard.pressed.icon.color"
        }
        (IconButtonVariant::Standard, true, false, None) => {
            "md.comp.icon-button.standard.icon.color"
        }
        (IconButtonVariant::Standard, true, false, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.standard.hovered.icon.color"
        }
        (IconButtonVariant::Standard, true, false, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.standard.focused.icon.color"
        }
        (IconButtonVariant::Standard, true, false, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.standard.pressed.icon.color"
        }
        (IconButtonVariant::Standard, true, true, None) => {
            "md.comp.icon-button.standard.selected.icon.color"
        }
        (IconButtonVariant::Standard, true, true, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.standard.selected.hovered.icon.color"
        }
        (IconButtonVariant::Standard, true, true, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.standard.selected.focused.icon.color"
        }
        (IconButtonVariant::Standard, true, true, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.standard.selected.pressed.icon.color"
        }
        (IconButtonVariant::Filled, false, _, None) => "md.comp.icon-button.filled.icon.color",
        (IconButtonVariant::Filled, false, _, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.filled.hovered.icon.color"
        }
        (IconButtonVariant::Filled, false, _, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.filled.focused.icon.color"
        }
        (IconButtonVariant::Filled, false, _, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.filled.pressed.icon.color"
        }
        // Filled: base tokens are the "selected" look; unselected differs.
        (IconButtonVariant::Filled, true, true, None) => "md.comp.icon-button.filled.icon.color",
        (IconButtonVariant::Filled, true, true, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.filled.hovered.icon.color"
        }
        (IconButtonVariant::Filled, true, true, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.filled.focused.icon.color"
        }
        (IconButtonVariant::Filled, true, true, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.filled.pressed.icon.color"
        }
        (IconButtonVariant::Filled, true, false, None) => {
            "md.comp.icon-button.filled.unselected.icon.color"
        }
        (IconButtonVariant::Filled, true, false, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.filled.unselected.hovered.icon.color"
        }
        (IconButtonVariant::Filled, true, false, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.filled.unselected.focused.icon.color"
        }
        (IconButtonVariant::Filled, true, false, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.filled.unselected.pressed.icon.color"
        }
        (IconButtonVariant::Tonal, false, _, None) => "md.comp.icon-button.tonal.icon.color",
        (IconButtonVariant::Tonal, false, _, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.tonal.hovered.icon.color"
        }
        (IconButtonVariant::Tonal, false, _, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.tonal.focused.icon.color"
        }
        (IconButtonVariant::Tonal, false, _, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.tonal.pressed.icon.color"
        }
        (IconButtonVariant::Tonal, true, false, None) => "md.comp.icon-button.tonal.icon.color",
        (IconButtonVariant::Tonal, true, false, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.tonal.hovered.icon.color"
        }
        (IconButtonVariant::Tonal, true, false, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.tonal.focused.icon.color"
        }
        (IconButtonVariant::Tonal, true, false, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.tonal.pressed.icon.color"
        }
        (IconButtonVariant::Tonal, true, true, None) => {
            "md.comp.icon-button.tonal.selected.icon.color"
        }
        (IconButtonVariant::Tonal, true, true, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.tonal.selected.hovered.icon.color"
        }
        (IconButtonVariant::Tonal, true, true, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.tonal.selected.focused.icon.color"
        }
        (IconButtonVariant::Tonal, true, true, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.tonal.selected.pressed.icon.color"
        }
        (IconButtonVariant::Outlined, false, _, None) => "md.comp.icon-button.outlined.icon.color",
        (IconButtonVariant::Outlined, false, _, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.outlined.hovered.icon.color"
        }
        (IconButtonVariant::Outlined, false, _, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.outlined.focused.icon.color"
        }
        (IconButtonVariant::Outlined, false, _, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.outlined.pressed.icon.color"
        }
        (IconButtonVariant::Outlined, true, false, None) => {
            "md.comp.icon-button.outlined.icon.color"
        }
        (IconButtonVariant::Outlined, true, false, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.outlined.hovered.icon.color"
        }
        (IconButtonVariant::Outlined, true, false, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.outlined.focused.icon.color"
        }
        (IconButtonVariant::Outlined, true, false, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.outlined.pressed.icon.color"
        }
        (IconButtonVariant::Outlined, true, true, None) => {
            "md.comp.icon-button.outlined.selected.icon.color"
        }
        (IconButtonVariant::Outlined, true, true, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.outlined.selected.hovered.icon.color"
        }
        (IconButtonVariant::Outlined, true, true, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.outlined.selected.focused.icon.color"
        }
        (IconButtonVariant::Outlined, true, true, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.outlined.selected.pressed.icon.color"
        }
    }
}

fn state_layer_color_key(
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    interaction: Option<IconButtonInteraction>,
) -> &'static str {
    match (variant, toggle, selected, interaction) {
        (IconButtonVariant::Standard, false, _, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.standard.hovered.state-layer.color"
        }
        (IconButtonVariant::Standard, false, _, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.standard.focused.state-layer.color"
        }
        (IconButtonVariant::Standard, false, _, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.standard.pressed.state-layer.color"
        }
        (IconButtonVariant::Standard, false, _, None) => {
            "md.comp.icon-button.standard.pressed.state-layer.color"
        }

        (IconButtonVariant::Standard, true, false, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.standard.hovered.state-layer.color"
        }
        (IconButtonVariant::Standard, true, false, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.standard.focused.state-layer.color"
        }
        (IconButtonVariant::Standard, true, false, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.standard.pressed.state-layer.color"
        }
        (IconButtonVariant::Standard, true, false, None) => {
            "md.comp.icon-button.standard.pressed.state-layer.color"
        }
        (IconButtonVariant::Standard, true, true, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.standard.selected.hovered.state-layer.color"
        }
        (IconButtonVariant::Standard, true, true, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.standard.selected.focused.state-layer.color"
        }
        (IconButtonVariant::Standard, true, true, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.standard.selected.pressed.state-layer.color"
        }
        (IconButtonVariant::Standard, true, true, None) => {
            "md.comp.icon-button.standard.selected.pressed.state-layer.color"
        }

        (IconButtonVariant::Filled, false, _, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.filled.hovered.state-layer.color"
        }
        (IconButtonVariant::Filled, false, _, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.filled.focused.state-layer.color"
        }
        (IconButtonVariant::Filled, false, _, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.filled.pressed.state-layer.color"
        }
        (IconButtonVariant::Filled, false, _, None) => {
            "md.comp.icon-button.filled.pressed.state-layer.color"
        }
        (IconButtonVariant::Filled, true, true, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.filled.hovered.state-layer.color"
        }
        (IconButtonVariant::Filled, true, true, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.filled.focused.state-layer.color"
        }
        (IconButtonVariant::Filled, true, true, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.filled.pressed.state-layer.color"
        }
        (IconButtonVariant::Filled, true, true, None) => {
            "md.comp.icon-button.filled.pressed.state-layer.color"
        }
        (IconButtonVariant::Filled, true, false, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.filled.unselected.hovered.state-layer.color"
        }
        (IconButtonVariant::Filled, true, false, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.filled.unselected.focused.state-layer.color"
        }
        (IconButtonVariant::Filled, true, false, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.filled.unselected.pressed.state-layer.color"
        }
        (IconButtonVariant::Filled, true, false, None) => {
            "md.comp.icon-button.filled.unselected.pressed.state-layer.color"
        }

        (IconButtonVariant::Tonal, false, _, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.tonal.hovered.state-layer.color"
        }
        (IconButtonVariant::Tonal, false, _, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.tonal.focused.state-layer.color"
        }
        (IconButtonVariant::Tonal, false, _, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.tonal.pressed.state-layer.color"
        }
        (IconButtonVariant::Tonal, false, _, None) => {
            "md.comp.icon-button.tonal.pressed.state-layer.color"
        }
        (IconButtonVariant::Tonal, true, false, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.tonal.hovered.state-layer.color"
        }
        (IconButtonVariant::Tonal, true, false, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.tonal.focused.state-layer.color"
        }
        (IconButtonVariant::Tonal, true, false, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.tonal.pressed.state-layer.color"
        }
        (IconButtonVariant::Tonal, true, false, None) => {
            "md.comp.icon-button.tonal.pressed.state-layer.color"
        }
        (IconButtonVariant::Tonal, true, true, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.tonal.selected.hovered.state-layer.color"
        }
        (IconButtonVariant::Tonal, true, true, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.tonal.selected.focused.state-layer.color"
        }
        (IconButtonVariant::Tonal, true, true, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.tonal.selected.pressed.state-layer.color"
        }
        (IconButtonVariant::Tonal, true, true, None) => {
            "md.comp.icon-button.tonal.selected.pressed.state-layer.color"
        }

        (IconButtonVariant::Outlined, false, _, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.outlined.hovered.state-layer.color"
        }
        (IconButtonVariant::Outlined, false, _, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.outlined.focused.state-layer.color"
        }
        (IconButtonVariant::Outlined, false, _, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.outlined.pressed.state-layer.color"
        }
        (IconButtonVariant::Outlined, false, _, None) => {
            "md.comp.icon-button.outlined.pressed.state-layer.color"
        }
        (IconButtonVariant::Outlined, true, false, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.outlined.hovered.state-layer.color"
        }
        (IconButtonVariant::Outlined, true, false, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.outlined.focused.state-layer.color"
        }
        (IconButtonVariant::Outlined, true, false, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.outlined.pressed.state-layer.color"
        }
        (IconButtonVariant::Outlined, true, false, None) => {
            "md.comp.icon-button.outlined.pressed.state-layer.color"
        }
        (IconButtonVariant::Outlined, true, true, Some(IconButtonInteraction::Hovered)) => {
            "md.comp.icon-button.outlined.selected.hovered.state-layer.color"
        }
        (IconButtonVariant::Outlined, true, true, Some(IconButtonInteraction::Focused)) => {
            "md.comp.icon-button.outlined.selected.focused.state-layer.color"
        }
        (IconButtonVariant::Outlined, true, true, Some(IconButtonInteraction::Pressed)) => {
            "md.comp.icon-button.outlined.selected.pressed.state-layer.color"
        }
        (IconButtonVariant::Outlined, true, true, None) => {
            "md.comp.icon-button.outlined.selected.pressed.state-layer.color"
        }
    }
}

fn disabled_icon_color_key(variant: IconButtonVariant) -> &'static str {
    match variant {
        IconButtonVariant::Standard => "md.comp.icon-button.standard.disabled.icon.color",
        IconButtonVariant::Filled => "md.comp.icon-button.filled.disabled.icon.color",
        IconButtonVariant::Tonal => "md.comp.icon-button.tonal.disabled.icon.color",
        IconButtonVariant::Outlined => "md.comp.icon-button.outlined.disabled.icon.color",
    }
}

fn disabled_icon_opacity_key(variant: IconButtonVariant) -> &'static str {
    match variant {
        IconButtonVariant::Standard => "md.comp.icon-button.standard.disabled.icon.opacity",
        IconButtonVariant::Filled => "md.comp.icon-button.filled.disabled.icon.opacity",
        IconButtonVariant::Tonal => "md.comp.icon-button.tonal.disabled.icon.opacity",
        IconButtonVariant::Outlined => "md.comp.icon-button.outlined.disabled.icon.opacity",
    }
}
