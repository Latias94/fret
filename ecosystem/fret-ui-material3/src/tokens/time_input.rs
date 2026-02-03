//! Typed token access for Material 3 time input primitives.
//!
//! Reference: Material Web v30 `md.comp.time-input.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.time-input";

fn token_key(suffix: &str) -> String {
    format!("{COMPONENT_PREFIX}.{suffix}")
}

pub(crate) fn time_input_field_container_width(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("time-input-field.container.width"))
        .unwrap_or(Px(96.0))
}

pub(crate) fn time_input_field_container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("time-input-field.container.height"))
        .unwrap_or(Px(72.0))
}

pub(crate) fn time_input_field_container_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key(&token_key("time-input-field.container.shape"))
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.small"))
        .unwrap_or(Corners::all(Px(8.0)))
}

pub(crate) fn time_input_field_container_color(theme: &Theme, focused: bool) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(if focused {
            "time-input-field.focus.container.color"
        } else {
            "time-input-field.container.color"
        }),
        if focused {
            "md.sys.color.primary-container"
        } else {
            "md.sys.color.surface-container-highest"
        },
    )
}

pub(crate) fn time_input_field_focus_outline_width(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("time-input-field.focus.outline.width"))
        .unwrap_or(Px(2.0))
}

pub(crate) fn time_input_field_focus_outline_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("time-input-field.focus.outline.color"),
        "md.sys.color.primary",
    )
}

pub(crate) fn time_input_field_label_text_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key(&token_key("time-input-field.label-text"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.display-medium"))
        .unwrap_or_default()
}

pub(crate) fn time_input_field_label_color(theme: &Theme, focused: bool, hovered: bool) -> Color {
    let suffix = match (focused, hovered) {
        (true, _) => "time-input-field.focus.label-text.color",
        (false, true) => "time-input-field.hover.label-text.color",
        _ => "time-input-field.label-text.color",
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(suffix),
        if focused {
            "md.sys.color.on-primary-container"
        } else {
            "md.sys.color.on-surface"
        },
    )
}

pub(crate) fn time_input_field_state_layer_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("time-input-field.hover.state-layer.color"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn time_input_field_state_layer_opacity(theme: &Theme) -> f32 {
    theme
        .number_by_key(&token_key("time-input-field.hover.state-layer.opacity"))
        .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}

pub(crate) fn time_input_field_separator_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key(&token_key("time-input-field.separator"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.display-large"))
        .unwrap_or_default()
}

pub(crate) fn time_input_field_separator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("time-input-field.separator.color"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn time_input_field_supporting_text_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key(&token_key("time-input-field.supporting-text"))
        .or_else(|| theme.text_style_by_key("md.sys.typescale.body-small"))
        .unwrap_or_default()
}

pub(crate) fn time_input_field_supporting_text_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("time-input-field.supporting-text.color"),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn period_selector_container_width(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("period-selector.container.width"))
        .unwrap_or(Px(52.0))
}

pub(crate) fn period_selector_container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key(&token_key("period-selector.container.height"))
        .unwrap_or(Px(72.0))
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
