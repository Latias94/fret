//! Typed token access for Material 3 filter chips.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;
pub(crate) use crate::tokens::chip_common::ChipOutline;
use crate::tokens::chip_common::{self, ChipOutlineKeys};

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.filter-chip";

pub(crate) fn container_height(theme: &Theme) -> Px {
    chip_common::container_height(theme, COMPONENT_PREFIX)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    chip_common::container_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn leading_icon_size(theme: &Theme) -> Px {
    chip_common::icon_size(theme, "md.comp.filter-chip.with-icon.icon.size")
}

pub(crate) fn trailing_icon_size(theme: &Theme) -> Px {
    chip_common::icon_size(theme, "md.comp.filter-chip.with-icon.icon.size")
}

pub(crate) fn flat_selected_container_background(theme: &Theme, enabled: bool) -> Color {
    if enabled {
        MaterialTokenResolver::new(theme).color_comp_or_sys(
            "md.comp.filter-chip.flat.selected.container.color",
            "md.sys.color.secondary-container",
        )
    } else {
        disabled_on_surface_color(
            theme,
            "md.comp.filter-chip.flat.disabled.selected.container.color",
            "md.comp.filter-chip.flat.disabled.selected.container.opacity",
            0.12,
        )
    }
}

pub(crate) fn elevated_container_background(theme: &Theme, selected: bool, enabled: bool) -> Color {
    if enabled {
        let key = if selected {
            "elevated.selected.container.color"
        } else {
            "elevated.unselected.container.color"
        };
        let sys_key = if selected {
            "md.sys.color.secondary-container"
        } else {
            "md.sys.color.surface-container-low"
        };
        let tokens = MaterialTokenResolver::new(theme);
        let fallback = tokens.color_sys("md.sys.color.surface-container-low");
        tokens.color_comp_or_sys_or(&format!("{COMPONENT_PREFIX}.{key}"), sys_key, fallback)
    } else {
        disabled_on_surface_color(
            theme,
            "md.comp.filter-chip.elevated.disabled.container.color",
            "md.comp.filter-chip.elevated.disabled.container.opacity",
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
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.filter-chip.disabled.label-text.color",
            "md.comp.filter-chip.disabled.label-text.opacity",
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
            "md.comp.filter-chip.with-leading-icon.disabled.leading-icon.color",
            "md.comp.filter-chip.with-leading-icon.disabled.leading-icon.opacity",
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
            "md.comp.filter-chip.with-trailing-icon.disabled.trailing-icon.color",
            "md.comp.filter-chip.with-trailing-icon.disabled.trailing-icon.opacity",
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

    MaterialTokenResolver::new(theme).color_comp_or_sys(&key, "md.sys.color.primary")
}

pub(crate) fn flat_unselected_outline(
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
            width: "flat.unselected.outline.width",
            disabled_color: "flat.disabled.unselected.outline.color",
            disabled_opacity: "flat.disabled.unselected.outline.opacity",
            focus_color: "flat.unselected.focus.outline.color",
            color: "flat.unselected.outline.color",
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
