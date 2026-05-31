//! Typed token access for Material 3 assist chips.
//!
//! Note: Material Web's v30 sassvars do not currently include the padding/spacing tokens used by
//! the chip recipe (`leading-space`, `trailing-space`, etc.). We keep those as component-level
//! layout constants in `chip.rs` instead of inventing new `md.*` keys.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;
pub(crate) use crate::tokens::chip_common::ChipOutline;
use crate::tokens::chip_common::{self, ChipOutlineKeys};

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.assist-chip";

pub(crate) fn container_height(theme: &Theme) -> Px {
    chip_common::container_height(theme, COMPONENT_PREFIX)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    chip_common::container_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn label_color(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.assist-chip.disabled.label-text.color",
            "md.comp.assist-chip.disabled.label-text.opacity",
            0.38,
        );
    }

    let key = match interaction {
        Some(PressableInteraction::Pressed) => "pressed.label-text.color",
        Some(PressableInteraction::Focused) => "focus.label-text.color",
        Some(PressableInteraction::Hovered) => "hover.label-text.color",
        None => "label-text.color",
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{COMPONENT_PREFIX}.{key}"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn label_text_style(theme: &Theme) -> TextStyle {
    chip_common::label_text_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn leading_icon_size(theme: &Theme) -> Px {
    chip_common::icon_size(theme, "md.comp.assist-chip.with-icon.icon.size")
}

pub(crate) fn leading_icon_color(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.assist-chip.with-icon.disabled.icon.color",
            "md.comp.assist-chip.with-icon.disabled.icon.opacity",
            0.38,
        );
    }

    let key = match interaction {
        Some(PressableInteraction::Pressed) => "with-icon.pressed.icon.color",
        Some(PressableInteraction::Focused) => "with-icon.focus.icon.color",
        Some(PressableInteraction::Hovered) => "with-icon.hover.icon.color",
        None => "with-icon.icon.color",
    };

    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(&format!("{COMPONENT_PREFIX}.{key}"), "md.sys.color.primary")
}

pub(crate) fn state_layer_color(theme: &Theme, interaction: Option<PressableInteraction>) -> Color {
    let key = match interaction {
        Some(PressableInteraction::Pressed) => "pressed.state-layer.color",
        Some(PressableInteraction::Focused) => "focus.state-layer.color",
        Some(PressableInteraction::Hovered) => "hover.state-layer.color",
        None => "hover.state-layer.color",
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{COMPONENT_PREFIX}.{key}"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn state_layer_opacity(theme: &Theme, interaction: Option<PressableInteraction>) -> f32 {
    chip_common::state_layer_opacity(theme, COMPONENT_PREFIX, None, interaction)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    chip_common::pressed_state_layer_opacity(theme, COMPONENT_PREFIX, None)
}

pub(crate) fn elevated_container_background(theme: &Theme, enabled: bool) -> Color {
    if enabled {
        MaterialTokenResolver::new(theme).color_comp_or_sys(
            "md.comp.assist-chip.elevated.container.color",
            "md.sys.color.surface-container-low",
        )
    } else {
        disabled_on_surface_color(
            theme,
            "md.comp.assist-chip.elevated.disabled.container.color",
            "md.comp.assist-chip.elevated.disabled.container.opacity",
            0.12,
        )
    }
}

pub(crate) fn elevated_container_elevation(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Px {
    chip_common::elevated_container_elevation(theme, COMPONENT_PREFIX, enabled, interaction)
}

pub(crate) fn elevated_container_shadow_color(theme: &Theme) -> Color {
    chip_common::elevated_container_shadow_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn flat_outline(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Option<ChipOutline> {
    Some(chip_common::outline(
        theme,
        COMPONENT_PREFIX,
        enabled,
        interaction,
        ChipOutlineKeys {
            width: "flat.outline.width",
            disabled_color: "flat.disabled.outline.color",
            disabled_opacity: "flat.disabled.outline.opacity",
            focus_color: "flat.focus.outline.color",
            color: "flat.outline.color",
        },
    ))
}

fn disabled_on_surface_color(
    theme: &Theme,
    color_key: &str,
    opacity_key: &str,
    fallback_opacity: f32,
) -> Color {
    chip_common::disabled_on_surface_color(theme, color_key, opacity_key, fallback_opacity)
}
