//! Typed token access for Material 3 icon buttons.
//!
//! This module exists to reduce ad-hoc `format!` key building in components and centralize:
//! - key mapping across variants and toggle states,
//! - fallback chains (component token -> sys token -> required sys token),
//! - derived values like disabled alpha application.

use fret_core::Color;
use fret_ui::Theme;

use crate::icon_button::IconButtonVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IconButtonInteraction {
    Hovered,
    Focused,
    Pressed,
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
    let mut color = if enabled {
        theme.color_by_key(base_key)
    } else {
        theme.color_by_key(disabled_icon_color_key(variant))
    }
    .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
    .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

    if !enabled {
        let opacity = theme
            .number_by_key(disabled_icon_opacity_key(variant))
            .unwrap_or(0.38);
        color.a *= opacity;
    }

    color
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
    let mut color = theme
        .color_by_key(pressed_key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| icon_color(theme, variant, toggle, selected, enabled, None));

    if let Some(interaction) = interaction {
        let key = state_layer_color_key(variant, toggle, selected, Some(interaction));
        color = theme.color_by_key(key).unwrap_or(color);
    }

    color
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, variant: IconButtonVariant) -> f32 {
    state_layer_opacity(theme, variant, IconButtonInteraction::Pressed)
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    variant: IconButtonVariant,
    interaction: IconButtonInteraction,
) -> f32 {
    let fallback = match interaction {
        IconButtonInteraction::Pressed => 0.1,
        IconButtonInteraction::Focused => 0.1,
        IconButtonInteraction::Hovered => 0.08,
    };

    theme
        .number_by_key(state_layer_opacity_key(variant, interaction))
        .or_else(|| match interaction {
            IconButtonInteraction::Pressed => {
                theme.number_by_key("md.sys.state.pressed.state-layer-opacity")
            }
            IconButtonInteraction::Focused => {
                theme.number_by_key("md.sys.state.focus.state-layer-opacity")
            }
            IconButtonInteraction::Hovered => {
                theme.number_by_key("md.sys.state.hover.state-layer-opacity")
            }
        })
        .unwrap_or(fallback)
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
                theme.color_by_key(container_color_key_filled(toggle, selected))
            } else {
                let mut c = theme
                    .color_by_key("md.comp.icon-button.filled.disabled.container.color")
                    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                    .unwrap_or(icon_fallback);
                let opacity = theme
                    .number_by_key("md.comp.icon-button.filled.disabled.container.opacity")
                    .unwrap_or(0.1);
                c.a *= opacity;
                Some(c)
            }
        }
        IconButtonVariant::Tonal => {
            if enabled {
                theme.color_by_key(container_color_key_tonal(toggle, selected))
            } else {
                let mut c = theme
                    .color_by_key("md.comp.icon-button.tonal.disabled.container.color")
                    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                    .unwrap_or(icon_fallback);
                let opacity = theme
                    .number_by_key("md.comp.icon-button.tonal.disabled.container.opacity")
                    .unwrap_or(0.1);
                c.a *= opacity;
                Some(c)
            }
        }
        IconButtonVariant::Outlined => {
            if !toggle || !selected {
                None
            } else if enabled {
                theme.color_by_key("md.comp.icon-button.outlined.selected.container.color")
            } else {
                let mut c = theme
                    .color_by_key("md.comp.icon-button.outlined.selected.disabled.container.color")
                    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                    .unwrap_or(icon_fallback);
                let opacity = theme
                    .number_by_key(
                        "md.comp.icon-button.outlined.selected.disabled.container.opacity",
                    )
                    .unwrap_or(0.1);
                c.a *= opacity;
                Some(c)
            }
        }
    }
}

pub(crate) fn outlined_outline_color(theme: &Theme, enabled: bool) -> Color {
    let mut color = if !enabled {
        theme.color_by_key("md.comp.icon-button.outlined.disabled.outline.color")
    } else {
        theme.color_by_key("md.comp.icon-button.outlined.outline.color")
    }
    .or_else(|| theme.color_by_key("md.sys.color.outline-variant"))
    .or_else(|| theme.color_by_key("md.sys.color.outline"))
    .unwrap_or_else(|| theme.color_required("md.sys.color.outline"));

    color.a = 1.0;
    color
}

fn state_layer_opacity_key(
    variant: IconButtonVariant,
    interaction: IconButtonInteraction,
) -> &'static str {
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
