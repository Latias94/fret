//! Typed token access for Material 3 switches.
//!
//! This module centralizes token key mapping and fallback chains so switch visuals remain stable
//! and drift-resistant during refactors.

use fret_core::Color;
use fret_ui::Theme;

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

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: SwitchInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    match interaction {
        SwitchInteraction::None => 0.0,
        SwitchInteraction::Pressed => theme
            .number_by_key(state_layer_opacity_key(
                selected,
                SwitchInteraction::Pressed,
            ))
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        SwitchInteraction::Focused => theme
            .number_by_key(state_layer_opacity_key(
                selected,
                SwitchInteraction::Focused,
            ))
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        SwitchInteraction::Hovered => theme
            .number_by_key(state_layer_opacity_key(
                selected,
                SwitchInteraction::Hovered,
            ))
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
    }
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, selected: bool) -> f32 {
    theme
        .number_by_key(state_layer_opacity_key(
            selected,
            SwitchInteraction::Pressed,
        ))
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: SwitchInteraction,
) -> Color {
    theme
        .color_by_key(state_layer_color_key(selected, interaction))
        .unwrap_or_else(|| {
            theme
                .color_by_key("md.sys.color.primary")
                .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
        })
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

    let track_color = theme.color_by_key(track_key).unwrap_or_else(|| {
        if selected {
            theme
                .color_by_key("md.sys.color.primary")
                .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
        } else {
            theme
                .color_by_key("md.sys.color.surface-container-highest")
                .unwrap_or_else(|| theme.color_required("md.sys.color.surface-container-highest"))
        }
    });

    let handle_color = theme.color_by_key(handle_key).unwrap_or_else(|| {
        if selected {
            theme
                .color_by_key("md.sys.color.on-primary")
                .unwrap_or_else(|| theme.color_required("md.sys.color.on-primary"))
        } else {
            theme
                .color_by_key("md.sys.color.outline")
                .unwrap_or_else(|| theme.color_required("md.sys.color.outline"))
        }
    });

    let outline_color = if selected {
        None
    } else {
        Some(
            theme
                .color_by_key(track_outline_color_key(interaction))
                .or_else(|| theme.color_by_key("md.sys.color.outline"))
                .unwrap_or_else(|| theme.color_required("md.sys.color.outline")),
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
    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
    .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

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
    .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

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

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
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
