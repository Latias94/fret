//! Typed token access for Material 3 outlined segmented buttons.
//!
//! Material Web currently exposes segmented buttons as a labs component, but the v30 token set is
//! stable enough to drive an outcome-oriented implementation.

use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.outlined-segmented-button";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SegmentedButtonInteraction {
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.outlined-segmented-button.container.height")
        .unwrap_or(Px(40.0))
}

pub(crate) fn outline_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.outlined-segmented-button.outline.width")
        .unwrap_or(Px(1.0))
}

pub(crate) fn shape_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.outlined-segmented-button.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0))
}

pub(crate) fn icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.outlined-segmented-button.with-icon.icon.size")
        .unwrap_or(Px(18.0))
}

pub(crate) fn container_background(theme: &Theme, selected: bool) -> Option<Color> {
    if !selected {
        return None;
    }
    theme
        .color_by_key("md.comp.outlined-segmented-button.selected.container.color")
        .or_else(|| theme.color_by_key("md.sys.color.secondary-container"))
}

pub(crate) fn outline_color(theme: &Theme, enabled: bool) -> Color {
    let tokens = MaterialTokenResolver::new(theme);

    let mut color = if enabled {
        theme.color_by_key("md.comp.outlined-segmented-button.outline.color")
    } else {
        theme.color_by_key("md.comp.outlined-segmented-button.disabled.outline.color")
    }
    .or_else(|| theme.color_by_key("md.sys.color.outline"))
    .unwrap_or_else(|| tokens.color_sys("md.sys.color.outline"));

    if !enabled {
        let opacity = theme
            .number_by_key("md.comp.outlined-segmented-button.disabled.outline.opacity")
            .unwrap_or(0.12);
        color.a *= opacity;
    } else {
        color.a = 1.0;
    }

    color
}

pub(crate) fn label_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<SegmentedButtonInteraction>,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);

    if !enabled {
        let mut color = theme
            .color_by_key("md.comp.outlined-segmented-button.disabled.label-text.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| tokens.color_sys("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.outlined-segmented-button.disabled.label-text.opacity")
            .unwrap_or(0.38);
        color.a *= opacity;
        return color;
    }

    let base = if selected { "selected" } else { "unselected" };
    let default_key = format!("md.comp.outlined-segmented-button.{base}.label-text.color");
    let mut color = theme
        .color_by_key(&default_key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| tokens.color_sys("md.sys.color.on-surface"));

    if let Some(interaction) = interaction {
        let key = match interaction {
            SegmentedButtonInteraction::Hovered => {
                format!("md.comp.outlined-segmented-button.{base}.hover.label-text.color")
            }
            SegmentedButtonInteraction::Focused => {
                format!("md.comp.outlined-segmented-button.{base}.focus.label-text.color")
            }
            SegmentedButtonInteraction::Pressed => {
                format!("md.comp.outlined-segmented-button.{base}.pressed.label-text.color")
            }
        };
        color = theme.color_by_key(&key).unwrap_or(color);
    }

    color
}

pub(crate) fn icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<SegmentedButtonInteraction>,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);

    if !enabled {
        let mut color = theme
            .color_by_key("md.comp.outlined-segmented-button.disabled.icon.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| tokens.color_sys("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.outlined-segmented-button.disabled.icon.opacity")
            .unwrap_or(0.38);
        color.a *= opacity;
        return color;
    }

    let base = if selected { "selected" } else { "unselected" };
    let default_key = format!("md.comp.outlined-segmented-button.{base}.with-icon.icon.color");
    let mut color = theme
        .color_by_key(&default_key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| tokens.color_sys("md.sys.color.on-surface"));

    if let Some(interaction) = interaction {
        let key = match interaction {
            SegmentedButtonInteraction::Hovered => {
                format!("md.comp.outlined-segmented-button.{base}.hover.icon.color")
            }
            SegmentedButtonInteraction::Focused => {
                format!("md.comp.outlined-segmented-button.{base}.focus.icon.color")
            }
            SegmentedButtonInteraction::Pressed => {
                format!("md.comp.outlined-segmented-button.{base}.pressed.icon.color")
            }
        };
        color = theme.color_by_key(&key).unwrap_or(color);
    }

    color
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: SegmentedButtonInteraction,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let base = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        SegmentedButtonInteraction::Hovered => {
            format!("md.comp.outlined-segmented-button.{base}.hover.state-layer.color")
        }
        SegmentedButtonInteraction::Focused => {
            format!("md.comp.outlined-segmented-button.{base}.focus.state-layer.color")
        }
        SegmentedButtonInteraction::Pressed => {
            format!("md.comp.outlined-segmented-button.{base}.pressed.state-layer.color")
        }
    };

    theme
        .color_by_key(&key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| tokens.color_sys("md.sys.color.on-surface"))
}

pub(crate) fn state_layer_opacity(theme: &Theme, interaction: SegmentedButtonInteraction) -> f32 {
    let (comp_key, sys_key, fallback) = match interaction {
        SegmentedButtonInteraction::Hovered => (
            "md.comp.outlined-segmented-button.hover.state-layer.opacity",
            "md.sys.state.hover.state-layer-opacity",
            0.08,
        ),
        SegmentedButtonInteraction::Focused => (
            "md.comp.outlined-segmented-button.focus.state-layer.opacity",
            "md.sys.state.focus.state-layer-opacity",
            0.1,
        ),
        SegmentedButtonInteraction::Pressed => (
            "md.comp.outlined-segmented-button.pressed.state-layer.opacity",
            "md.sys.state.pressed.state-layer-opacity",
            0.1,
        ),
    };

    theme
        .number_by_key(comp_key)
        .or_else(|| theme.number_by_key(sys_key))
        .unwrap_or(fallback)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    theme
        .number_by_key("md.comp.outlined-segmented-button.pressed.state-layer.opacity")
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
}
