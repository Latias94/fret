//! Typed token access for Material 3 switches.
//!
//! This module centralizes token key mapping and fallback chains so switch visuals remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SwitchInteraction {
    None,
    Hovered,
    Focused,
    Pressed,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SwitchChrome {
    pub(crate) track_color: Color,
    pub(crate) outline_color: Option<Color>,
    pub(crate) handle_color: Color,
}

pub(crate) fn icon_size(theme: &Theme, selected: bool) -> Px {
    let key = if selected {
        "md.comp.switch.selected.icon.size"
    } else {
        "md.comp.switch.unselected.icon.size"
    };
    theme.metric_by_key(key).unwrap_or(Px(16.0))
}

pub(crate) fn icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: SwitchInteraction,
) -> Color {
    if !enabled {
        let base = MaterialTokenResolver::new(theme)
            .color_comp_or_sys(disabled_icon_color_key(selected), "md.sys.color.on-surface");
        let opacity = theme
            .number_by_key(disabled_icon_opacity_key(selected))
            .unwrap_or(0.38);
        return alpha_mul(base, opacity);
    }

    let sys_key = if selected {
        "md.sys.color.on-primary"
    } else {
        "md.sys.color.on-surface-variant"
    };
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(icon_color_key(selected, interaction), sys_key)
}

fn shape_or_full(theme: &Theme, key: &str) -> Corners {
    theme
        .corners_by_key(key)
        .or_else(|| theme.metric_by_key(key).map(Corners::all))
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or_else(|| Corners::all(Px(9999.0)))
}

pub(crate) fn track_shape(theme: &Theme) -> Corners {
    shape_or_full(theme, "md.comp.switch.track.shape")
}

pub(crate) fn handle_shape(theme: &Theme) -> Corners {
    shape_or_full(theme, "md.comp.switch.handle.shape")
}

pub(crate) fn state_layer_shape(theme: &Theme) -> Corners {
    shape_or_full(theme, "md.comp.switch.state-layer.shape")
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: SwitchInteraction,
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
        state_layer_opacity_key(selected, SwitchInteraction::Pressed),
        MaterialStateLayerInteraction::Pressed,
    )
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: SwitchInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        state_layer_color_key(selected, interaction),
        "md.sys.color.primary",
    )
}

pub(crate) fn chrome(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: SwitchInteraction,
) -> SwitchChrome {
    if !enabled {
        return disabled_chrome(theme, selected);
    }

    let track_key = track_color_key(selected, interaction);
    let handle_key = handle_color_key(selected, interaction);

    let track_sys_key = if selected {
        "md.sys.color.primary"
    } else {
        "md.sys.color.surface-container-highest"
    };
    let track_color = MaterialTokenResolver::new(theme).color_comp_or_sys(track_key, track_sys_key);

    let handle_sys_key = if selected {
        "md.sys.color.on-primary"
    } else {
        "md.sys.color.outline"
    };
    let handle_color =
        MaterialTokenResolver::new(theme).color_comp_or_sys(handle_key, handle_sys_key);

    let outline_color = if selected {
        None
    } else {
        Some(
            MaterialTokenResolver::new(theme)
                .color_comp_or_sys(track_outline_color_key(interaction), "md.sys.color.outline"),
        )
    };

    SwitchChrome {
        track_color,
        outline_color,
        handle_color,
    }
}

fn disabled_chrome(theme: &Theme, selected: bool) -> SwitchChrome {
    let track_base = if selected {
        theme.color_by_key("md.comp.switch.disabled.selected.track.color")
    } else {
        theme.color_by_key("md.comp.switch.disabled.unselected.track.color")
    }
    .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"));

    let track_opacity = theme
        .number_by_key("md.comp.switch.disabled.track.opacity")
        .unwrap_or(0.12);
    let track_color = alpha_mul(track_base, track_opacity);

    let handle_base = if selected {
        theme
            .color_by_key("md.comp.switch.disabled.selected.handle.color")
            .or_else(|| theme.color_by_key("md.sys.color.surface"))
    } else {
        theme
            .color_by_key("md.comp.switch.disabled.unselected.handle.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
    }
    .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"));

    let handle_opacity = if selected {
        theme.number_by_key("md.comp.switch.disabled.selected.handle.opacity")
    } else {
        theme.number_by_key("md.comp.switch.disabled.unselected.handle.opacity")
    }
    .unwrap_or(0.38);
    let handle_color = alpha_mul(handle_base, handle_opacity);

    let outline_color = if selected {
        None
    } else {
        theme
            .color_by_key("md.comp.switch.disabled.unselected.track.outline.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .map(|c| alpha_mul(c, handle_opacity))
    };

    SwitchChrome {
        track_color,
        outline_color,
        handle_color,
    }
}

fn material_state_layer_interaction(
    interaction: SwitchInteraction,
) -> Option<MaterialStateLayerInteraction> {
    match interaction {
        SwitchInteraction::Pressed => Some(MaterialStateLayerInteraction::Pressed),
        SwitchInteraction::Focused => Some(MaterialStateLayerInteraction::Focused),
        SwitchInteraction::Hovered => Some(MaterialStateLayerInteraction::Hovered),
        SwitchInteraction::None => None,
    }
}

fn icon_color_key(selected: bool, interaction: SwitchInteraction) -> &'static str {
    match (selected, interaction) {
        (true, SwitchInteraction::Pressed) => "md.comp.switch.selected.pressed.icon.color",
        (true, SwitchInteraction::Focused) => "md.comp.switch.selected.focus.icon.color",
        (true, SwitchInteraction::Hovered) => "md.comp.switch.selected.hover.icon.color",
        (true, SwitchInteraction::None) => "md.comp.switch.selected.icon.color",
        (false, SwitchInteraction::Pressed) => "md.comp.switch.unselected.pressed.icon.color",
        (false, SwitchInteraction::Focused) => "md.comp.switch.unselected.focus.icon.color",
        (false, SwitchInteraction::Hovered) => "md.comp.switch.unselected.hover.icon.color",
        (false, SwitchInteraction::None) => "md.comp.switch.unselected.icon.color",
    }
}

fn disabled_icon_color_key(selected: bool) -> &'static str {
    if selected {
        "md.comp.switch.disabled.selected.icon.color"
    } else {
        "md.comp.switch.disabled.unselected.icon.color"
    }
}

fn disabled_icon_opacity_key(selected: bool) -> &'static str {
    if selected {
        "md.comp.switch.disabled.selected.icon.opacity"
    } else {
        "md.comp.switch.disabled.unselected.icon.opacity"
    }
}

fn state_layer_opacity_key(selected: bool, interaction: SwitchInteraction) -> &'static str {
    match (selected, interaction) {
        (true, SwitchInteraction::Pressed) => "md.comp.switch.selected.pressed.state-layer.opacity",
        (true, SwitchInteraction::Focused) => "md.comp.switch.selected.focus.state-layer.opacity",
        (true, SwitchInteraction::Hovered) => "md.comp.switch.selected.hover.state-layer.opacity",
        (false, SwitchInteraction::Pressed) => {
            "md.comp.switch.unselected.pressed.state-layer.opacity"
        }
        (false, SwitchInteraction::Focused) => {
            "md.comp.switch.unselected.focus.state-layer.opacity"
        }
        (false, SwitchInteraction::Hovered) => {
            "md.comp.switch.unselected.hover.state-layer.opacity"
        }
        (_, SwitchInteraction::None) => "md.comp.switch.unselected.hover.state-layer.opacity",
    }
}

fn state_layer_color_key(selected: bool, interaction: SwitchInteraction) -> &'static str {
    match (selected, interaction) {
        (true, SwitchInteraction::Pressed) => "md.comp.switch.selected.pressed.state-layer.color",
        (true, SwitchInteraction::Focused) => "md.comp.switch.selected.focus.state-layer.color",
        (true, SwitchInteraction::Hovered) => "md.comp.switch.selected.hover.state-layer.color",
        (true, SwitchInteraction::None) => "md.comp.switch.selected.hover.state-layer.color",
        (false, SwitchInteraction::Pressed) => {
            "md.comp.switch.unselected.pressed.state-layer.color"
        }
        (false, SwitchInteraction::Focused) => "md.comp.switch.unselected.focus.state-layer.color",
        (false, SwitchInteraction::Hovered) => "md.comp.switch.unselected.hover.state-layer.color",
        (false, SwitchInteraction::None) => "md.comp.switch.unselected.hover.state-layer.color",
    }
}

fn track_color_key(selected: bool, interaction: SwitchInteraction) -> &'static str {
    match (selected, interaction) {
        (true, SwitchInteraction::None) => "md.comp.switch.selected.track.color",
        (true, SwitchInteraction::Hovered) => "md.comp.switch.selected.hover.track.color",
        (true, SwitchInteraction::Focused) => "md.comp.switch.selected.focus.track.color",
        (true, SwitchInteraction::Pressed) => "md.comp.switch.selected.pressed.track.color",
        (false, SwitchInteraction::None) => "md.comp.switch.unselected.track.color",
        (false, SwitchInteraction::Hovered) => "md.comp.switch.unselected.hover.track.color",
        (false, SwitchInteraction::Focused) => "md.comp.switch.unselected.focus.track.color",
        (false, SwitchInteraction::Pressed) => "md.comp.switch.unselected.pressed.track.color",
    }
}

fn handle_color_key(selected: bool, interaction: SwitchInteraction) -> &'static str {
    match (selected, interaction) {
        (true, SwitchInteraction::None) => "md.comp.switch.selected.handle.color",
        (true, SwitchInteraction::Hovered) => "md.comp.switch.selected.hover.handle.color",
        (true, SwitchInteraction::Focused) => "md.comp.switch.selected.focus.handle.color",
        (true, SwitchInteraction::Pressed) => "md.comp.switch.selected.pressed.handle.color",
        (false, SwitchInteraction::None) => "md.comp.switch.unselected.handle.color",
        (false, SwitchInteraction::Hovered) => "md.comp.switch.unselected.hover.handle.color",
        (false, SwitchInteraction::Focused) => "md.comp.switch.unselected.focus.handle.color",
        (false, SwitchInteraction::Pressed) => "md.comp.switch.unselected.pressed.handle.color",
    }
}

fn track_outline_color_key(interaction: SwitchInteraction) -> &'static str {
    match interaction {
        SwitchInteraction::None => "md.comp.switch.unselected.track.outline.color",
        SwitchInteraction::Hovered => "md.comp.switch.unselected.hover.track.outline.color",
        SwitchInteraction::Focused => "md.comp.switch.unselected.focus.track.outline.color",
        SwitchInteraction::Pressed => "md.comp.switch.unselected.pressed.track.outline.color",
    }
}
