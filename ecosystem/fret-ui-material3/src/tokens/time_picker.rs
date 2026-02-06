//! Typed token access for Material 3 time picker primitives.
//!
//! Reference: Material Web v30 `md.comp.time-picker.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.time-picker";

fn token_key(suffix: &str) -> String {
    format!("{COMPONENT_PREFIX}.{suffix}")
}

pub(crate) fn container_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("container.color"),
        "md.sys.color.surface-container-high",
    )
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("container.elevation"))
        .unwrap_or(Px(3.0))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key(&token_key("container.shape"))
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-large"))
        .unwrap_or(Corners::all(Px(28.0)))
}

pub(crate) fn headline_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key(&token_key("headline"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.label-medium"))
        .unwrap_or_default()
}

pub(crate) fn headline_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("headline.color"),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn clock_dial_size(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("clock-dial.container.size"))
        .unwrap_or(Px(256.0))
}

pub(crate) fn clock_dial_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("clock-dial.color"),
        "md.sys.color.surface-container-highest",
    )
}

pub(crate) fn clock_dial_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key(&token_key("clock-dial.shape"))
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Corners::all(Px(9999.0)))
}

pub(crate) fn clock_dial_label_text_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key(&token_key("clock-dial.label-text"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.body-large"))
        .unwrap_or_default()
}

pub(crate) fn clock_dial_label_text_color(theme: &Theme, selected: bool) -> Color {
    let suffix = if selected {
        "clock-dial.selected.label-text.color"
    } else {
        "clock-dial.unselected.label-text.color"
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(suffix),
        if selected {
            "md.sys.color.on-primary"
        } else {
            "md.sys.color.on-surface"
        },
    )
}

pub(crate) fn clock_dial_handle_size(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("clock-dial.selector.handle.container.size"))
        .unwrap_or(Px(48.0))
}

pub(crate) fn clock_dial_handle_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("clock-dial.selector.handle.container.color"),
        "md.sys.color.primary",
    )
}

pub(crate) fn clock_dial_handle_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key(&token_key("clock-dial.selector.handle.container.shape"))
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Corners::all(Px(9999.0)))
}

pub(crate) fn time_selector_container_width(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("time-selector.container.width"))
        .unwrap_or(Px(96.0))
}

pub(crate) fn time_selector_container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("time-selector.container.height"))
        .unwrap_or(Px(80.0))
}

pub(crate) fn time_selector_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key(&token_key("time-selector.container.shape"))
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.small"))
        .unwrap_or(Corners::all(Px(8.0)))
}

pub(crate) fn time_selector_container_color(theme: &Theme, selected: bool) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(if selected {
            "time-selector.selected.container.color"
        } else {
            "time-selector.unselected.container.color"
        }),
        if selected {
            "md.sys.color.primary-container"
        } else {
            "md.sys.color.surface-container-highest"
        },
    )
}

pub(crate) fn time_selector_label_text_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key(&token_key("time-selector.label-text"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.display-large"))
        .unwrap_or_default()
}

pub(crate) fn time_selector_separator_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key(&token_key("time-selector.separator"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.display-large"))
        .unwrap_or_default()
}

pub(crate) fn time_selector_separator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("time-selector.separator.color"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn time_selector_label_color(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    let suffix = match (selected, interaction) {
        (true, Some(PressableInteraction::Focused)) => {
            "time-selector.selected.focus.label-text.color"
        }
        (true, Some(PressableInteraction::Hovered)) => {
            "time-selector.selected.hover.label-text.color"
        }
        (true, Some(PressableInteraction::Pressed)) => {
            "time-selector.selected.pressed.label-text.color"
        }
        (true, None) => "time-selector.selected.label-text.color",
        (false, Some(PressableInteraction::Focused)) => {
            "time-selector.unselected.focus.label-text.color"
        }
        (false, Some(PressableInteraction::Hovered)) => {
            "time-selector.unselected.hover.label-text.color"
        }
        (false, Some(PressableInteraction::Pressed)) => {
            "time-selector.unselected.pressed.label-text.color"
        }
        (false, None) => "time-selector.unselected.label-text.color",
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(suffix),
        if selected {
            "md.sys.color.on-primary-container"
        } else {
            "md.sys.color.on-surface"
        },
    )
}

pub(crate) fn time_selector_state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: PressableInteraction,
) -> Color {
    let suffix = match (selected, interaction) {
        (true, PressableInteraction::Focused) => "time-selector.selected.focus.state-layer.color",
        (true, PressableInteraction::Hovered) => "time-selector.selected.hover.state-layer.color",
        (true, PressableInteraction::Pressed) => "time-selector.selected.pressed.state-layer.color",
        (false, PressableInteraction::Focused) => {
            "time-selector.unselected.focus.state-layer.color"
        }
        (false, PressableInteraction::Hovered) => {
            "time-selector.unselected.hover.state-layer.color"
        }
        (false, PressableInteraction::Pressed) => {
            "time-selector.unselected.pressed.state-layer.color"
        }
    };
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(&token_key(suffix), "md.sys.color.on-surface")
}

pub(crate) fn time_selector_state_layer_opacity(
    theme: &Theme,
    interaction: PressableInteraction,
) -> f32 {
    let (suffix, fallback) = match interaction {
        PressableInteraction::Focused => (
            "time-selector.focus.state-layer.opacity",
            "md.sys.state.focus.state-layer-opacity",
        ),
        PressableInteraction::Hovered => (
            "time-selector.hover.state-layer.opacity",
            "md.sys.state.hover.state-layer-opacity",
        ),
        PressableInteraction::Pressed => (
            "time-selector.pressed.state-layer.opacity",
            "md.sys.state.pressed.state-layer-opacity",
        ),
    };
    theme
        .number_by_key(&token_key(suffix))
        .or_else(|| theme.number_by_key(fallback))
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}

pub(crate) fn period_selector_container_width(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("period-selector.vertical.container.width"))
        .unwrap_or(Px(52.0))
}

pub(crate) fn period_selector_container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("period-selector.vertical.container.height"))
        .unwrap_or(Px(80.0))
}

pub(crate) fn period_selector_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key(&token_key("period-selector.container.shape"))
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.small"))
        .unwrap_or(Corners::all(Px(8.0)))
}

pub(crate) fn period_selector_outline_width(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("period-selector.outline.width"))
        .unwrap_or(Px(1.0))
}

pub(crate) fn period_selector_outline_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("period-selector.outline.color"),
        "md.sys.color.outline",
    )
}

pub(crate) fn period_selector_selected_container_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("period-selector.selected.container.color"),
        "md.sys.color.tertiary-container",
    )
}

pub(crate) fn period_selector_label_text_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key(&token_key("period-selector.label-text"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.title-medium"))
        .unwrap_or_default()
}

pub(crate) fn period_selector_label_color(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    let suffix = match (selected, interaction) {
        (true, Some(PressableInteraction::Focused)) => {
            "period-selector.selected.focus.label-text.color"
        }
        (true, Some(PressableInteraction::Hovered)) => {
            "period-selector.selected.hover.label-text.color"
        }
        (true, Some(PressableInteraction::Pressed)) => {
            "period-selector.selected.pressed.label-text.color"
        }
        (true, None) => "period-selector.selected.label-text.color",
        (false, Some(PressableInteraction::Focused)) => {
            "period-selector.unselected.focus.label-text.color"
        }
        (false, Some(PressableInteraction::Hovered)) => {
            "period-selector.unselected.hover.label-text.color"
        }
        (false, Some(PressableInteraction::Pressed)) => {
            "period-selector.unselected.pressed.label-text.color"
        }
        (false, None) => "period-selector.unselected.label-text.color",
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(suffix),
        if selected {
            "md.sys.color.on-tertiary-container"
        } else {
            "md.sys.color.on-surface-variant"
        },
    )
}

pub(crate) fn period_selector_state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: PressableInteraction,
) -> Color {
    let suffix = match (selected, interaction) {
        (true, PressableInteraction::Focused) => "period-selector.selected.focus.state-layer.color",
        (true, PressableInteraction::Hovered) => "period-selector.selected.hover.state-layer.color",
        (true, PressableInteraction::Pressed) => {
            "period-selector.selected.pressed.state-layer.color"
        }
        (false, PressableInteraction::Focused) => {
            "period-selector.unselected.focus.state-layer.color"
        }
        (false, PressableInteraction::Hovered) => {
            "period-selector.unselected.hover.state-layer.color"
        }
        (false, PressableInteraction::Pressed) => {
            "period-selector.unselected.pressed.state-layer.color"
        }
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(suffix),
        if selected {
            "md.sys.color.on-tertiary-container"
        } else {
            "md.sys.color.on-surface-variant"
        },
    )
}

pub(crate) fn period_selector_state_layer_opacity(
    theme: &Theme,
    interaction: PressableInteraction,
) -> f32 {
    let (suffix, fallback) = match interaction {
        PressableInteraction::Focused => (
            "period-selector.focus.state-layer.opacity",
            "md.sys.state.focus.state-layer-opacity",
        ),
        PressableInteraction::Hovered => (
            "period-selector.hover.state-layer.opacity",
            "md.sys.state.hover.state-layer-opacity",
        ),
        PressableInteraction::Pressed => (
            "period-selector.pressed.state-layer.opacity",
            "md.sys.state.pressed.state-layer-opacity",
        ),
    };
    theme
        .number_by_key(&token_key(suffix))
        .or_else(|| theme.number_by_key(fallback))
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}
