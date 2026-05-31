//! Typed token access for Material 3 outlined segmented buttons.
//!
//! Material Web currently exposes segmented buttons as a labs component, but the v30 token set is
//! stable enough to drive an outcome-oriented implementation.

use fret_core::{Color, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};

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
    Some(MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.outlined-segmented-button.selected.container.color",
        "md.sys.color.secondary-container",
    ))
}

pub(crate) fn outline_color(theme: &Theme, enabled: bool) -> Color {
    let tokens = MaterialTokenResolver::new(theme);

    let comp_key = if enabled {
        "md.comp.outlined-segmented-button.outline.color"
    } else {
        "md.comp.outlined-segmented-button.disabled.outline.color"
    };
    let mut color = tokens.color_comp_or_sys(comp_key, "md.sys.color.outline");

    if !enabled {
        let opacity = tokens.number_optional(
            Some("md.comp.outlined-segmented-button.disabled.outline.opacity"),
            0.12,
        );
        color = alpha_mul(color, opacity);
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
        let color = tokens.color_comp_or_sys(
            "md.comp.outlined-segmented-button.disabled.label-text.color",
            "md.sys.color.on-surface",
        );
        let opacity = tokens.number_optional(
            Some("md.comp.outlined-segmented-button.disabled.label-text.opacity"),
            0.38,
        );
        return alpha_mul(color, opacity);
    }

    let base = if selected { "selected" } else { "unselected" };
    let default_key = format!("md.comp.outlined-segmented-button.{base}.label-text.color");
    let mut color = tokens.color_comp_or_sys(&default_key, "md.sys.color.on-surface");

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
        color = tokens.color_comp_or_fallback(&key, color);
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
        let color = tokens.color_comp_or_sys(
            "md.comp.outlined-segmented-button.disabled.icon.color",
            "md.sys.color.on-surface",
        );
        let opacity = tokens.number_optional(
            Some("md.comp.outlined-segmented-button.disabled.icon.opacity"),
            0.38,
        );
        return alpha_mul(color, opacity);
    }

    let base = if selected { "selected" } else { "unselected" };
    let default_key = format!("md.comp.outlined-segmented-button.{base}.with-icon.icon.color");
    let mut color = tokens.color_comp_or_sys(&default_key, "md.sys.color.on-surface");

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
        color = tokens.color_comp_or_fallback(&key, color);
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

    tokens.color_comp_or_sys(&key, "md.sys.color.on-surface")
}

pub(crate) fn state_layer_opacity(theme: &Theme, interaction: SegmentedButtonInteraction) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        state_layer_opacity_key(interaction),
        material_state_layer_interaction(interaction),
    )
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    state_layer_opacity(theme, SegmentedButtonInteraction::Pressed)
}

fn material_state_layer_interaction(
    interaction: SegmentedButtonInteraction,
) -> MaterialStateLayerInteraction {
    match interaction {
        SegmentedButtonInteraction::Hovered => MaterialStateLayerInteraction::Hovered,
        SegmentedButtonInteraction::Focused => MaterialStateLayerInteraction::Focused,
        SegmentedButtonInteraction::Pressed => MaterialStateLayerInteraction::Pressed,
    }
}

fn state_layer_opacity_key(interaction: SegmentedButtonInteraction) -> &'static str {
    match interaction {
        SegmentedButtonInteraction::Hovered => {
            "md.comp.outlined-segmented-button.hover.state-layer.opacity"
        }
        SegmentedButtonInteraction::Focused => {
            "md.comp.outlined-segmented-button.focus.state-layer.opacity"
        }
        SegmentedButtonInteraction::Pressed => {
            "md.comp.outlined-segmented-button.pressed.state-layer.opacity"
        }
    }
}
