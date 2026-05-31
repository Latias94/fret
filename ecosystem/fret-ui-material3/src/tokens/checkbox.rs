//! Typed token access for Material 3 checkboxes.
//!
//! This module centralizes token key mapping and fallback chains so checkbox visuals remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckboxInteraction {
    None,
    Hovered,
    Focused,
    Pressed,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CheckboxSizeTokens {
    pub(crate) container: Px,
    pub(crate) icon: Px,
    pub(crate) state_layer: Px,
    pub(crate) container_corner: Px,
}

pub(crate) fn size_tokens(theme: &Theme) -> CheckboxSizeTokens {
    let container = theme
        .metric_by_key("md.comp.checkbox.container.size")
        .unwrap_or(Px(18.0));
    let icon = theme
        .metric_by_key("md.comp.checkbox.icon.size")
        .unwrap_or(container);
    let state_layer = theme
        .metric_by_key("md.comp.checkbox.state-layer.size")
        .unwrap_or(Px(40.0));
    let container_corner = theme
        .metric_by_key("md.comp.checkbox.container.shape")
        .unwrap_or(Px(2.0));

    CheckboxSizeTokens {
        container,
        icon,
        state_layer,
        container_corner,
    }
}

fn state_layer_opacity_key(selected: bool, interaction: CheckboxInteraction) -> &'static str {
    match (selected, interaction) {
        (true, CheckboxInteraction::Pressed) => {
            "md.comp.checkbox.selected.pressed.state-layer.opacity"
        }
        (true, CheckboxInteraction::Focused) => {
            "md.comp.checkbox.selected.focus.state-layer.opacity"
        }
        (true, CheckboxInteraction::Hovered) => {
            "md.comp.checkbox.selected.hover.state-layer.opacity"
        }
        (false, CheckboxInteraction::Pressed) => {
            "md.comp.checkbox.unselected.pressed.state-layer.opacity"
        }
        (false, CheckboxInteraction::Focused) => {
            "md.comp.checkbox.unselected.focus.state-layer.opacity"
        }
        (false, CheckboxInteraction::Hovered) => {
            "md.comp.checkbox.unselected.hover.state-layer.opacity"
        }
        (true, CheckboxInteraction::None) => "md.comp.checkbox.selected.hover.state-layer.opacity",
        (false, CheckboxInteraction::None) => {
            "md.comp.checkbox.unselected.hover.state-layer.opacity"
        }
    }
}

fn material_state_layer_interaction(
    interaction: CheckboxInteraction,
) -> Option<MaterialStateLayerInteraction> {
    match interaction {
        CheckboxInteraction::Pressed => Some(MaterialStateLayerInteraction::Pressed),
        CheckboxInteraction::Focused => Some(MaterialStateLayerInteraction::Focused),
        CheckboxInteraction::Hovered => Some(MaterialStateLayerInteraction::Hovered),
        CheckboxInteraction::None => None,
    }
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: CheckboxInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    let Some(material_interaction) = material_state_layer_interaction(interaction) else {
        return 0.0;
    };

    MaterialTokenResolver::new(theme).state_layer_opacity(
        state_layer_opacity_key(selected, interaction),
        material_interaction,
    )
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, selected: bool) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        state_layer_opacity_key(selected, CheckboxInteraction::Pressed),
        MaterialStateLayerInteraction::Pressed,
    )
}

fn state_layer_color_key(selected: bool, interaction: CheckboxInteraction) -> &'static str {
    match (selected, interaction) {
        (true, CheckboxInteraction::Pressed) => {
            "md.comp.checkbox.selected.pressed.state-layer.color"
        }
        (true, CheckboxInteraction::Focused) => "md.comp.checkbox.selected.focus.state-layer.color",
        (true, CheckboxInteraction::Hovered) => "md.comp.checkbox.selected.hover.state-layer.color",
        (true, CheckboxInteraction::None) => "md.comp.checkbox.selected.hover.state-layer.color",
        (false, CheckboxInteraction::Pressed) => {
            "md.comp.checkbox.unselected.pressed.state-layer.color"
        }
        (false, CheckboxInteraction::Focused) => {
            "md.comp.checkbox.unselected.focus.state-layer.color"
        }
        (false, CheckboxInteraction::Hovered) => {
            "md.comp.checkbox.unselected.hover.state-layer.color"
        }
        (false, CheckboxInteraction::None) => "md.comp.checkbox.unselected.hover.state-layer.color",
    }
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: CheckboxInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        state_layer_color_key(selected, interaction),
        "md.sys.color.primary",
    )
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CheckboxChrome {
    pub(crate) container_bg: Option<Color>,
    pub(crate) outline_width: Px,
    pub(crate) outline_color: Option<Color>,
    pub(crate) icon_color: Color,
}

fn disabled_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).disabled_state_layer_opacity()
}

fn selected_outline_width_key(interaction: CheckboxInteraction) -> &'static str {
    match interaction {
        CheckboxInteraction::Pressed => "md.comp.checkbox.selected.pressed.outline.width",
        CheckboxInteraction::Focused => "md.comp.checkbox.selected.focus.outline.width",
        CheckboxInteraction::Hovered => "md.comp.checkbox.selected.hover.outline.width",
        CheckboxInteraction::None => "md.comp.checkbox.selected.outline.width",
    }
}

fn unselected_outline_width_key(interaction: CheckboxInteraction) -> &'static str {
    match interaction {
        CheckboxInteraction::Pressed => "md.comp.checkbox.unselected.pressed.outline.width",
        CheckboxInteraction::Focused => "md.comp.checkbox.unselected.focus.outline.width",
        CheckboxInteraction::Hovered => "md.comp.checkbox.unselected.hover.outline.width",
        CheckboxInteraction::None => "md.comp.checkbox.unselected.outline.width",
    }
}

fn unselected_outline_color_key(interaction: CheckboxInteraction) -> &'static str {
    match interaction {
        CheckboxInteraction::Pressed => "md.comp.checkbox.unselected.pressed.outline.color",
        CheckboxInteraction::Focused => "md.comp.checkbox.unselected.focus.outline.color",
        CheckboxInteraction::Hovered => "md.comp.checkbox.unselected.hover.outline.color",
        CheckboxInteraction::None => "md.comp.checkbox.unselected.outline.color",
    }
}

fn selected_container_color_key(interaction: CheckboxInteraction) -> &'static str {
    match interaction {
        CheckboxInteraction::Pressed => "md.comp.checkbox.selected.pressed.container.color",
        CheckboxInteraction::Focused => "md.comp.checkbox.selected.focus.container.color",
        CheckboxInteraction::Hovered => "md.comp.checkbox.selected.hover.container.color",
        CheckboxInteraction::None => "md.comp.checkbox.selected.container.color",
    }
}

fn selected_icon_color_key(interaction: CheckboxInteraction) -> &'static str {
    match interaction {
        CheckboxInteraction::Pressed => "md.comp.checkbox.selected.pressed.icon.color",
        CheckboxInteraction::Focused => "md.comp.checkbox.selected.focus.icon.color",
        CheckboxInteraction::Hovered => "md.comp.checkbox.selected.hover.icon.color",
        CheckboxInteraction::None => "md.comp.checkbox.selected.icon.color",
    }
}

pub(crate) fn chrome(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: CheckboxInteraction,
) -> CheckboxChrome {
    if selected {
        let mut container = theme
            .color_by_key(selected_container_color_key(interaction))
            .or_else(|| theme.color_by_key("md.comp.checkbox.selected.container.color"))
            .or_else(|| theme.color_by_key("md.sys.color.primary"))
            .unwrap_or_else(|| theme.color_token("md.sys.color.primary"));

        let mut icon_color = theme
            .color_by_key(selected_icon_color_key(interaction))
            .or_else(|| theme.color_by_key("md.comp.checkbox.selected.icon.color"))
            .or_else(|| theme.color_by_key("md.sys.color.on-primary"))
            .unwrap_or_else(|| theme.color_token("md.sys.color.on-primary"));

        let outline_width = if enabled {
            theme
                .metric_by_key(selected_outline_width_key(interaction))
                .or_else(|| theme.metric_by_key("md.comp.checkbox.selected.outline.width"))
                .unwrap_or(Px(0.0))
        } else {
            theme
                .metric_by_key("md.comp.checkbox.selected.disabled.container.outline.width")
                .unwrap_or(Px(0.0))
        };

        if !enabled {
            let opacity = theme
                .number_by_key("md.comp.checkbox.selected.disabled.container.opacity")
                .unwrap_or_else(|| disabled_opacity(theme));
            let disabled_container = theme
                .color_by_key("md.comp.checkbox.selected.disabled.container.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                .unwrap_or(container);
            container = alpha_mul(disabled_container, opacity);

            icon_color = theme
                .color_by_key("md.comp.checkbox.selected.disabled.icon.color")
                .or_else(|| theme.color_by_key("md.sys.color.surface"))
                .unwrap_or(icon_color);

            let icon_opacity = theme
                .number_by_key("md.comp.checkbox.disabled.selected.icon.opacity")
                .unwrap_or_else(|| disabled_opacity(theme));
            icon_color = alpha_mul(icon_color, icon_opacity);
        }

        CheckboxChrome {
            container_bg: Some(container),
            outline_width,
            outline_color: None,
            icon_color,
        }
    } else {
        let outline_width = if enabled {
            theme
                .metric_by_key(unselected_outline_width_key(interaction))
                .or_else(|| theme.metric_by_key("md.comp.checkbox.unselected.outline.width"))
                .unwrap_or(Px(2.0))
        } else {
            theme
                .metric_by_key("md.comp.checkbox.unselected.disabled.outline.width")
                .unwrap_or(Px(2.0))
        };

        let outline_color = if enabled {
            theme
                .color_by_key(unselected_outline_color_key(interaction))
                .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        } else {
            let base = theme
                .color_by_key("md.comp.checkbox.unselected.disabled.outline.color")
                .or_else(|| theme.color_by_key("md.sys.color.on-surface"));
            let opacity = theme
                .number_by_key("md.comp.checkbox.unselected.disabled.container.opacity")
                .unwrap_or_else(|| disabled_opacity(theme));
            base.map(|c| alpha_mul(c, opacity))
        };

        let icon_color = theme
            .color_by_key("md.comp.checkbox.selected.icon.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-primary"))
            .unwrap_or_else(|| theme.color_token("md.sys.color.on-primary"));

        CheckboxChrome {
            container_bg: None,
            outline_width,
            outline_color,
            icon_color,
        }
    }
}
