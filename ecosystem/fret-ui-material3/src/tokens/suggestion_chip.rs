//! Typed token access for Material 3 suggestion chips.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;
pub(crate) use crate::tokens::chip_common::ChipOutline;
use crate::tokens::chip_common::{self, ChipOutlineKeys};

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.suggestion-chip";

pub(crate) fn container_height(theme: &Theme) -> Px {
    chip_common::container_height(theme, COMPONENT_PREFIX)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    chip_common::container_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn leading_icon_size(theme: &Theme) -> Px {
    chip_common::icon_size(
        theme,
        "md.comp.suggestion-chip.with-leading-icon.leading-icon.size",
    )
}

pub(crate) fn elevated_container_background(theme: &Theme, enabled: bool) -> Color {
    if enabled {
        MaterialTokenResolver::new(theme).color_comp_or_sys(
            "md.comp.suggestion-chip.elevated.container.color",
            "md.sys.color.surface-container-low",
        )
    } else {
        disabled_on_surface_color(
            theme,
            "md.comp.suggestion-chip.elevated.disabled.container.color",
            "md.comp.suggestion-chip.elevated.disabled.container.opacity",
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

pub(crate) fn label_color(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.suggestion-chip.disabled.label-text.color",
            "md.comp.suggestion-chip.disabled.label-text.opacity",
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
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn label_text_style(theme: &Theme) -> TextStyle {
    chip_common::label_text_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn state_layer_color(theme: &Theme, interaction: Option<PressableInteraction>) -> Color {
    chip_common::state_layer_color(
        theme,
        COMPONENT_PREFIX,
        None,
        interaction,
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn state_layer_opacity(theme: &Theme, interaction: Option<PressableInteraction>) -> f32 {
    chip_common::state_layer_opacity(theme, COMPONENT_PREFIX, None, interaction)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    chip_common::pressed_state_layer_opacity(theme, COMPONENT_PREFIX, None)
}

pub(crate) fn leading_icon_color(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.suggestion-chip.with-leading-icon.disabled.leading-icon.color",
            "md.comp.suggestion-chip.with-leading-icon.disabled.leading-icon.opacity",
            0.38,
        );
    }

    let key = match interaction {
        Some(PressableInteraction::Pressed) => "with-leading-icon.pressed.leading-icon.color",
        Some(PressableInteraction::Focused) => "with-leading-icon.focus.leading-icon.color",
        Some(PressableInteraction::Hovered) => "with-leading-icon.hover.leading-icon.color",
        None => "with-leading-icon.leading-icon.color",
    };

    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(&format!("{COMPONENT_PREFIX}.{key}"), "md.sys.color.primary")
}

pub(crate) fn flat_outline(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> ChipOutline {
    chip_common::outline(
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
    )
}

fn disabled_on_surface_color(
    theme: &Theme,
    color_key: &str,
    opacity_key: &str,
    fallback_opacity: f32,
) -> Color {
    chip_common::disabled_on_surface_color(theme, color_key, opacity_key, fallback_opacity)
}
