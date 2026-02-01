//! Typed token access for Material 3 input chips.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.input-chip";

#[derive(Debug, Clone, Copy)]
pub(crate) struct ChipOutline {
    pub width: Px,
    pub color: Color,
}

pub(crate) fn container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.input-chip.container.height")
        .unwrap_or(Px(32.0))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    theme
        .metric_by_key("md.comp.input-chip.container.shape")
        .map(Corners::all)
        .or_else(|| {
            theme
                .metric_by_key("md.sys.shape.corner.small")
                .map(Corners::all)
        })
        .unwrap_or_else(|| Corners::all(Px(8.0)))
}

pub(crate) fn leading_icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.input-chip.with-leading-icon.leading-icon.size")
        .unwrap_or(Px(18.0))
}

pub(crate) fn trailing_icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.input-chip.with-trailing-icon.trailing-icon.size")
        .unwrap_or(Px(18.0))
}

pub(crate) fn selected_container_background(theme: &Theme, enabled: bool) -> Color {
    if enabled {
        theme
            .color_by_key("md.comp.input-chip.selected.container.color")
            .or_else(|| theme.color_by_key("md.sys.color.secondary-container"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.secondary-container"))
    } else {
        let base = theme
            .color_by_key("md.comp.input-chip.disabled.selected.container.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.input-chip.disabled.selected.container.opacity")
            .unwrap_or(0.12);
        let mut c = base;
        c.a *= opacity.clamp(0.0, 1.0);
        c
    }
}

pub(crate) fn unselected_outline(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> ChipOutline {
    let width = theme
        .metric_by_key("md.comp.input-chip.unselected.outline.width")
        .unwrap_or(Px(1.0));

    if !enabled {
        let base = theme
            .color_by_key("md.comp.input-chip.disabled.unselected.outline.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.input-chip.disabled.unselected.outline.opacity")
            .unwrap_or(0.12);
        let mut c = base;
        c.a *= opacity.clamp(0.0, 1.0);
        return ChipOutline { width, color: c };
    }

    let key = match interaction {
        Some(PressableInteraction::Focused) => "unselected.focus.outline.color",
        None | Some(_) => "unselected.outline.color",
    };

    let mut color = theme
        .color_by_key(&format!("{COMPONENT_PREFIX}.{key}"))
        .or_else(|| theme.color_by_key("md.sys.color.outline-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.outline-variant"));
    color.a = 1.0;

    ChipOutline { width, color }
}

pub(crate) fn label_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        let base = theme
            .color_by_key("md.comp.input-chip.disabled.label-text.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.input-chip.disabled.label-text.opacity")
            .unwrap_or(0.38);
        let mut c = base;
        c.a *= opacity.clamp(0.0, 1.0);
        return c;
    }

    let state = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Some(PressableInteraction::Pressed) => format!("{state}.pressed.label-text.color"),
        Some(PressableInteraction::Focused) => format!("{state}.focus.label-text.color"),
        Some(PressableInteraction::Hovered) => format!("{state}.hover.label-text.color"),
        None => format!("{state}.label-text.color"),
    };

    theme
        .color_by_key(&format!("{COMPONENT_PREFIX}.{key}"))
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"))
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    let state = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Some(PressableInteraction::Pressed) => format!("{state}.pressed.state-layer.color"),
        Some(PressableInteraction::Focused) => format!("{state}.focus.state-layer.color"),
        Some(PressableInteraction::Hovered) => format!("{state}.hover.state-layer.color"),
        None => return Color::TRANSPARENT,
    };

    theme
        .color_by_key(&format!("{COMPONENT_PREFIX}.{key}"))
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"))
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> f32 {
    let state = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Some(PressableInteraction::Pressed) => format!("{state}.pressed.state-layer.opacity"),
        Some(PressableInteraction::Focused) => format!("{state}.focus.state-layer.opacity"),
        Some(PressableInteraction::Hovered) => format!("{state}.hover.state-layer.opacity"),
        None => return 0.0,
    };

    theme
        .number_by_key(&format!("{COMPONENT_PREFIX}.{key}"))
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, selected: bool) -> f32 {
    let state = if selected { "selected" } else { "unselected" };
    theme
        .number_by_key(&format!(
            "{COMPONENT_PREFIX}.{state}.pressed.state-layer.opacity"
        ))
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
        .clamp(0.0, 1.0)
}

pub(crate) fn leading_icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        let base = theme
            .color_by_key("md.comp.input-chip.with-leading-icon.disabled.leading-icon.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.input-chip.with-leading-icon.disabled.leading-icon.opacity")
            .unwrap_or(0.38);
        let mut c = base;
        c.a *= opacity.clamp(0.0, 1.0);
        return c;
    }

    let state = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Some(PressableInteraction::Pressed) => {
            format!("with-leading-icon.{state}.pressed.leading-icon.color")
        }
        Some(PressableInteraction::Focused) => {
            format!("with-leading-icon.{state}.focus.leading-icon.color")
        }
        Some(PressableInteraction::Hovered) => {
            format!("with-leading-icon.{state}.hover.leading-icon.color")
        }
        None => format!("with-leading-icon.{state}.leading-icon.color"),
    };

    theme
        .color_by_key(&format!("{COMPONENT_PREFIX}.{key}"))
        .or_else(|| theme.color_by_key("md.sys.color.primary"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
}

pub(crate) fn trailing_icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        let base = theme
            .color_by_key("md.comp.input-chip.with-trailing-icon.disabled.trailing-icon.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.input-chip.with-trailing-icon.disabled.trailing-icon.opacity")
            .unwrap_or(0.38);
        let mut c = base;
        c.a *= opacity.clamp(0.0, 1.0);
        return c;
    }

    let state = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Some(PressableInteraction::Pressed) => {
            format!("with-trailing-icon.{state}.pressed.trailing-icon.color")
        }
        Some(PressableInteraction::Focused) => {
            format!("with-trailing-icon.{state}.focus.trailing-icon.color")
        }
        Some(PressableInteraction::Hovered) => {
            format!("with-trailing-icon.{state}.hover.trailing-icon.color")
        }
        None => format!("with-trailing-icon.{state}.trailing-icon.color"),
    };

    theme
        .color_by_key(&format!("{COMPONENT_PREFIX}.{key}"))
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"))
}
