//! Typed token access for Material 3 input chips.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;
pub(crate) use crate::tokens::chip_common::ChipOutline;
use crate::tokens::chip_common::{self, ChipOutlineKeys};

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.input-chip";

pub(crate) fn container_height(theme: &Theme) -> Px {
    chip_common::container_height(theme, COMPONENT_PREFIX)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    chip_common::container_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn leading_icon_size(theme: &Theme) -> Px {
    chip_common::icon_size(
        theme,
        "md.comp.input-chip.with-leading-icon.leading-icon.size",
    )
}

pub(crate) fn trailing_icon_size(theme: &Theme) -> Px {
    chip_common::icon_size(
        theme,
        "md.comp.input-chip.with-trailing-icon.trailing-icon.size",
    )
}

pub(crate) fn selected_container_background(theme: &Theme, enabled: bool) -> Color {
    if enabled {
        MaterialTokenResolver::new(theme).color_comp_or_sys(
            "md.comp.input-chip.selected.container.color",
            "md.sys.color.secondary-container",
        )
    } else {
        disabled_on_surface_color(
            theme,
            "md.comp.input-chip.disabled.selected.container.color",
            "md.comp.input-chip.disabled.selected.container.opacity",
            0.12,
        )
    }
}

pub(crate) fn unselected_outline(
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
            width: "unselected.outline.width",
            disabled_color: "disabled.unselected.outline.color",
            disabled_opacity: "disabled.unselected.outline.opacity",
            focus_color: "unselected.focus.outline.color",
            color: "unselected.outline.color",
        },
    )
}

pub(crate) fn label_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.input-chip.disabled.label-text.color",
            "md.comp.input-chip.disabled.label-text.opacity",
            0.38,
        );
    }

    let state = if selected { "selected" } else { "unselected" };
    let key = chip_common::interaction_key(
        COMPONENT_PREFIX,
        Some(state),
        interaction,
        "label-text.color",
    );

    MaterialTokenResolver::new(theme).color_comp_or_sys(&key, "md.sys.color.on-surface-variant")
}

pub(crate) fn label_text_style(theme: &Theme) -> TextStyle {
    chip_common::label_text_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    let state = if selected { "selected" } else { "unselected" };
    chip_common::state_layer_color(
        theme,
        COMPONENT_PREFIX,
        Some(state),
        interaction,
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> f32 {
    let state = if selected { "selected" } else { "unselected" };
    chip_common::state_layer_opacity(theme, COMPONENT_PREFIX, Some(state), interaction)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, selected: bool) -> f32 {
    let state = if selected { "selected" } else { "unselected" };
    chip_common::pressed_state_layer_opacity(theme, COMPONENT_PREFIX, Some(state))
}

pub(crate) fn leading_icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.input-chip.with-leading-icon.disabled.leading-icon.color",
            "md.comp.input-chip.with-leading-icon.disabled.leading-icon.opacity",
            0.38,
        );
    }

    let state = if selected { "selected" } else { "unselected" };
    let icon_state = format!("with-leading-icon.{state}");
    let key = chip_common::interaction_key(
        COMPONENT_PREFIX,
        Some(icon_state.as_str()),
        interaction,
        "leading-icon.color",
    );

    MaterialTokenResolver::new(theme).color_comp_or_sys(&key, "md.sys.color.primary")
}

pub(crate) fn trailing_icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.input-chip.with-trailing-icon.disabled.trailing-icon.color",
            "md.comp.input-chip.with-trailing-icon.disabled.trailing-icon.opacity",
            0.38,
        );
    }

    let state = if selected { "selected" } else { "unselected" };
    let icon_state = format!("with-trailing-icon.{state}");
    let key = chip_common::interaction_key(
        COMPONENT_PREFIX,
        Some(icon_state.as_str()),
        interaction,
        "trailing-icon.color",
    );

    MaterialTokenResolver::new(theme).color_comp_or_sys(&key, "md.sys.color.on-surface-variant")
}

fn disabled_on_surface_color(
    theme: &Theme,
    color_key: &str,
    opacity_key: &str,
    fallback_opacity: f32,
) -> Color {
    chip_common::disabled_on_surface_color(theme, color_key, opacity_key, fallback_opacity)
}
