//! Typed token access for Material 3 sliders.
//!
//! This module centralizes token key mapping and fallback chains so slider visuals remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Corners, FontWeight, Px, TextStyle};
use fret_ui::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SliderInteraction {
    None,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn state_layer_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.slider.state-layer.size")
        .unwrap_or(Px(40.0))
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: SliderInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    match interaction {
        SliderInteraction::None => 0.0,
        SliderInteraction::Pressed => theme
            .number_by_key("md.comp.slider.pressed.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        SliderInteraction::Focused => theme
            .number_by_key("md.comp.slider.focus.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        SliderInteraction::Hovered => theme
            .number_by_key("md.comp.slider.hover.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
    }
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    theme
        .number_by_key("md.comp.slider.pressed.state-layer.opacity")
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
}

pub(crate) fn state_layer_color(theme: &Theme, interaction: SliderInteraction) -> Color {
    let key = match interaction {
        SliderInteraction::Hovered => "md.comp.slider.hover.state-layer.color",
        SliderInteraction::Focused => "md.comp.slider.focus.state-layer.color",
        SliderInteraction::Pressed => "md.comp.slider.pressed.state-layer.color",
        SliderInteraction::None => "md.comp.slider.hover.state-layer.color",
    };

    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.primary"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
}

pub(crate) fn value_indicator_bottom_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.slider.value-indicator.active.bottom-space")
        .unwrap_or(Px(12.0))
}

pub(crate) fn value_indicator_container_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.slider.value-indicator.container.color")
        .or_else(|| theme.color_by_key("md.sys.color.inverse-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.inverse-surface"))
}

pub(crate) fn value_indicator_label_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.slider.value-indicator.label.label-text.color")
        .or_else(|| theme.color_by_key("md.sys.color.inverse-on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.inverse-on-surface"))
}

pub(crate) fn value_indicator_label_style(theme: &Theme) -> TextStyle {
    let mut style = theme
        .text_style_by_key("md.sys.typescale.label-large")
        .unwrap_or_default();

    if let Some(weight) =
        theme.number_by_key("md.comp.slider.value-indicator.label.label-text.weight")
    {
        style.weight = FontWeight(weight.round().clamp(1.0, 1000.0) as u16);
    }

    style
}

pub(crate) fn tick_mark_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.slider.with-tick-marks.container.size")
        .unwrap_or(Px(2.0))
}

pub(crate) fn tick_mark_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.slider.with-tick-marks.container.shape")
        .unwrap_or_else(|| Corners::all(Px(9999.0)))
}

pub(crate) fn tick_mark_color(theme: &Theme, enabled: bool, active: bool) -> Color {
    if !enabled {
        return theme
            .color_by_key("md.comp.slider.with-tick-marks.disabled.container.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
    }

    let key = if active {
        "md.comp.slider.with-tick-marks.active.container.color"
    } else {
        "md.comp.slider.with-tick-marks.inactive.container.color"
    };
    theme
        .color_by_key(key)
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"))
}

pub(crate) fn tick_mark_opacity(theme: &Theme, enabled: bool, active: bool) -> f32 {
    if !enabled {
        return theme
            .number_by_key("md.comp.slider.with-tick-marks.disabled.container.opacity")
            .unwrap_or(0.38);
    }

    let key = if active {
        "md.comp.slider.with-tick-marks.active.container.opacity"
    } else {
        "md.comp.slider.with-tick-marks.inactive.container.opacity"
    };
    theme.number_by_key(key).unwrap_or(0.38)
}

pub(crate) fn active_track_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.slider.active.track.height")
        .unwrap_or(Px(16.0))
}

pub(crate) fn inactive_track_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.slider.inactive.track.height")
        .unwrap_or(Px(16.0))
}

pub(crate) fn active_track_color(
    theme: &Theme,
    enabled: bool,
    interaction: SliderInteraction,
) -> Color {
    if !enabled {
        let base = theme
            .color_by_key("md.comp.slider.disabled.active.track.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.slider.disabled.active.track.opacity")
            .unwrap_or(0.38);
        return alpha_mul(base, opacity);
    }

    let key = match interaction {
        SliderInteraction::Pressed => "md.comp.slider.pressed.active.track.color",
        SliderInteraction::Focused => "md.comp.slider.focus.active.track.color",
        SliderInteraction::Hovered | SliderInteraction::None => "md.comp.slider.active.track.color",
    };

    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.primary"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
}

pub(crate) fn inactive_track_color(
    theme: &Theme,
    enabled: bool,
    interaction: SliderInteraction,
) -> Color {
    if !enabled {
        let base = theme
            .color_by_key("md.comp.slider.disabled.inactive.track.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.slider.disabled.inactive.track.opacity")
            .unwrap_or(0.12);
        return alpha_mul(base, opacity);
    }

    let key = match interaction {
        SliderInteraction::Pressed => "md.comp.slider.pressed.inactive.track.color",
        SliderInteraction::Focused => "md.comp.slider.focus.inactive.track.color",
        SliderInteraction::Hovered | SliderInteraction::None => {
            "md.comp.slider.inactive.track.color"
        }
    };

    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.secondary-container"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.secondary-container"))
}

pub(crate) fn handle_color(theme: &Theme, enabled: bool, interaction: SliderInteraction) -> Color {
    if !enabled {
        let base = theme
            .color_by_key("md.comp.slider.disabled.handle.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.slider.disabled.handle.opacity")
            .unwrap_or(0.38);
        return alpha_mul(base, opacity);
    }

    let key = match interaction {
        SliderInteraction::Pressed => "md.comp.slider.pressed.handle.color",
        SliderInteraction::Focused => "md.comp.slider.focus.handle.color",
        SliderInteraction::Hovered => "md.comp.slider.hover.handle.color",
        SliderInteraction::None => "md.comp.slider.handle.color",
    };

    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.primary"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
}

pub(crate) fn track_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.slider.active.track.shape")
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or_else(|| Corners::all(Px(9999.0)))
}

pub(crate) fn handle_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.slider.handle.height")
        .unwrap_or(Px(44.0))
}

pub(crate) fn handle_width(theme: &Theme, enabled: bool, interaction: SliderInteraction) -> Px {
    if !enabled {
        return theme
            .metric_by_key("md.comp.slider.disabled.handle.width")
            .unwrap_or(Px(4.0));
    }

    match interaction {
        SliderInteraction::Pressed => theme
            .metric_by_key("md.comp.slider.pressed.handle.width")
            .unwrap_or(Px(2.0)),
        SliderInteraction::Focused => theme
            .metric_by_key("md.comp.slider.focus.handle.width")
            .unwrap_or(Px(2.0)),
        SliderInteraction::Hovered => theme
            .metric_by_key("md.comp.slider.hover.handle.width")
            .unwrap_or_else(|| handle_width(theme, enabled, SliderInteraction::None)),
        SliderInteraction::None => theme
            .metric_by_key("md.comp.slider.handle.width")
            .unwrap_or(Px(4.0)),
    }
}

pub(crate) fn handle_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.slider.handle.shape")
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or_else(|| Corners::all(Px(9999.0)))
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}
