//! Typed token access for Material 3 time picker primitives.
//!
//! Reference: Material Web v30 `md.comp.time-picker.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::{shape, time_period_common, typography};

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
    let key = token_key("container.shape");
    shape::corners_or_metric(theme, &key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-large"))
        .unwrap_or(Corners::all(Px(28.0)))
}

pub(crate) fn headline_style(theme: &Theme) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key("headline")),
        "md.sys.typescale.label-medium",
        TextIntent::Control,
    )
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
    let key = token_key("clock-dial.shape");
    shape::corners_or_metric(theme, &key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Corners::all(Px(9999.0)))
}

pub(crate) fn clock_dial_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key("clock-dial.label-text")),
        "md.sys.typescale.body-large",
        TextIntent::Control,
    )
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
    let key = token_key("clock-dial.selector.handle.container.shape");
    shape::corners_or_metric(theme, &key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Corners::all(Px(9999.0)))
}

pub(crate) fn clock_dial_selector_center_size(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("clock-dial.selector.center.container.size"))
        .unwrap_or(Px(8.0))
}

pub(crate) fn clock_dial_selector_center_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("clock-dial.selector.center.container.color"),
        "md.sys.color.primary",
    )
}

pub(crate) fn clock_dial_selector_center_shape(theme: &Theme) -> Corners {
    let key = token_key("clock-dial.selector.center.container.shape");
    shape::corners_or_metric(theme, &key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Corners::all(Px(9999.0)))
}

pub(crate) fn clock_dial_selector_track_width(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("clock-dial.selector.track.container.width"))
        .unwrap_or(Px(2.0))
}

pub(crate) fn clock_dial_selector_track_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("clock-dial.selector.track.container.color"),
        "md.sys.color.primary",
    )
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
    let key = token_key("time-selector.container.shape");
    shape::corners_or_metric(theme, &key)
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
    typography::text_style(
        theme,
        Some(&token_key("time-selector.label-text")),
        "md.sys.typescale.display-large",
        TextIntent::Control,
    )
}

pub(crate) fn time_selector_separator_style(theme: &Theme) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key("time-selector.separator")),
        "md.sys.typescale.display-large",
        TextIntent::Control,
    )
}

pub(crate) fn time_selector_separator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("time-selector.separator.color"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn display_separator_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.sys.fret.material.time-picker.display-separator.width")
        .unwrap_or(Px(24.0))
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
    MaterialTokenResolver::new(theme)
        .number_comp_or_sys(&token_key(suffix), fallback, 0.0)
        .clamp(0.0, 1.0)
}

pub(crate) fn period_selector_container_width(theme: &Theme) -> Px {
    time_period_common::container_width(
        theme,
        COMPONENT_PREFIX,
        "period-selector.vertical.container.width",
    )
}

pub(crate) fn period_selector_container_height(theme: &Theme) -> Px {
    time_period_common::container_height(
        theme,
        COMPONENT_PREFIX,
        "period-selector.vertical.container.height",
        Px(80.0),
    )
}

pub(crate) fn period_selector_shape(theme: &Theme) -> Corners {
    time_period_common::container_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_outline_width(theme: &Theme) -> Px {
    time_period_common::outline_width(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_outline_color(theme: &Theme) -> Color {
    time_period_common::outline_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_selected_container_color(theme: &Theme) -> Color {
    time_period_common::selected_container_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_label_text_style(theme: &Theme) -> TextStyle {
    time_period_common::label_text_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_label_color(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    time_period_common::label_color(theme, COMPONENT_PREFIX, selected, interaction)
}

pub(crate) fn period_selector_state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: PressableInteraction,
) -> Color {
    time_period_common::state_layer_color(theme, COMPONENT_PREFIX, selected, interaction)
}

pub(crate) fn period_selector_state_layer_opacity(
    theme: &Theme,
    interaction: PressableInteraction,
) -> f32 {
    time_period_common::state_layer_opacity(theme, COMPONENT_PREFIX, interaction)
}
