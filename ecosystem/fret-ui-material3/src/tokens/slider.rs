//! Typed token access for Material 3 sliders.
//!
//! This module centralizes token key mapping and fallback chains so slider visuals remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};
use crate::tokens::typography;

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

    let Some(material_interaction) = material_state_layer_interaction(interaction) else {
        return 0.0;
    };

    MaterialTokenResolver::new(theme)
        .state_layer_opacity(state_layer_opacity_key(interaction), material_interaction)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        state_layer_opacity_key(SliderInteraction::Pressed),
        MaterialStateLayerInteraction::Pressed,
    )
}

pub(crate) fn state_layer_color(theme: &Theme, interaction: SliderInteraction) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(state_layer_color_key(interaction), "md.sys.color.primary")
}

fn material_state_layer_interaction(
    interaction: SliderInteraction,
) -> Option<MaterialStateLayerInteraction> {
    match interaction {
        SliderInteraction::Pressed => Some(MaterialStateLayerInteraction::Pressed),
        SliderInteraction::Focused => Some(MaterialStateLayerInteraction::Focused),
        SliderInteraction::Hovered => Some(MaterialStateLayerInteraction::Hovered),
        SliderInteraction::None => None,
    }
}

fn state_layer_opacity_key(interaction: SliderInteraction) -> &'static str {
    match interaction {
        SliderInteraction::Pressed => "md.comp.slider.pressed.state-layer.opacity",
        SliderInteraction::Focused => "md.comp.slider.focus.state-layer.opacity",
        SliderInteraction::Hovered => "md.comp.slider.hover.state-layer.opacity",
        SliderInteraction::None => "md.comp.slider.hover.state-layer.opacity",
    }
}

fn state_layer_color_key(interaction: SliderInteraction) -> &'static str {
    match interaction {
        SliderInteraction::Hovered => "md.comp.slider.hover.state-layer.color",
        SliderInteraction::Focused => "md.comp.slider.focus.state-layer.color",
        SliderInteraction::Pressed => "md.comp.slider.pressed.state-layer.color",
        SliderInteraction::None => "md.comp.slider.hover.state-layer.color",
    }
}

pub(crate) fn value_indicator_bottom_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.slider.value-indicator.active.bottom-space")
        .unwrap_or(Px(12.0))
}

pub(crate) fn value_indicator_container_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.slider.value-indicator.container.color",
        "md.sys.color.inverse-surface",
    )
}

pub(crate) fn value_indicator_label_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.slider.value-indicator.label.label-text.color",
        "md.sys.color.inverse-on-surface",
    )
}

pub(crate) fn value_indicator_label_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        None,
        "md.sys.typescale.label-large",
        Some("md.comp.slider.value-indicator.label.label-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn tick_mark_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.slider.with-tick-marks.container.size")
        .unwrap_or(Px(2.0))
}

pub(crate) fn tick_mark_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.slider.with-tick-marks.container.shape")
        .or_else(|| {
            theme
                .metric_by_key("md.comp.slider.with-tick-marks.container.shape")
                .map(Corners::all)
        })
        .unwrap_or_else(|| Corners::all(Px(9999.0)))
}

pub(crate) fn tick_mark_color(theme: &Theme, enabled: bool, active: bool) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    if !enabled {
        return tokens.color_comp_or_sys(
            "md.comp.slider.with-tick-marks.disabled.container.color",
            "md.sys.color.on-surface",
        );
    }

    let key = if active {
        "md.comp.slider.with-tick-marks.active.container.color"
    } else {
        "md.comp.slider.with-tick-marks.inactive.container.color"
    };
    tokens.color_comp_or_sys(key, "md.sys.color.on-surface-variant")
}

pub(crate) fn tick_mark_opacity(theme: &Theme, enabled: bool, active: bool) -> f32 {
    let tokens = MaterialTokenResolver::new(theme);
    if !enabled {
        return tokens.number_optional(
            Some("md.comp.slider.with-tick-marks.disabled.container.opacity"),
            0.38,
        );
    }

    let key = if active {
        "md.comp.slider.with-tick-marks.active.container.opacity"
    } else {
        "md.comp.slider.with-tick-marks.inactive.container.opacity"
    };
    tokens.number_optional(Some(key), 0.38)
}

pub(crate) fn stop_indicator_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.slider.stop-indicator.size")
        .unwrap_or(Px(4.0))
}

pub(crate) fn stop_indicator_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.slider.stop-indicator.shape")
        .or_else(|| {
            theme
                .metric_by_key("md.comp.slider.stop-indicator.shape")
                .map(Corners::all)
        })
        .unwrap_or_else(|| Corners::all(Px(9999.0)))
}

pub(crate) fn stop_indicator_trailing_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.slider.stop-indicator.trailing-space")
        .unwrap_or(Px(4.0))
}

pub(crate) fn stop_indicator_color(theme: &Theme, enabled: bool, selected: bool) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let base = if selected {
        tokens.color_comp_or_sys(
            "md.comp.slider.stop-indicator.color-selected",
            "md.sys.color.on-primary",
        )
    } else {
        tokens.color_comp_or_sys(
            "md.comp.slider.stop-indicator.color",
            "md.sys.color.on-secondary-container",
        )
    };

    let opacity = if !enabled {
        tokens.number_optional(
            Some("md.comp.slider.disabled.stop-indicator.container.opacity"),
            0.38,
        )
    } else if selected {
        tokens.number_optional(
            Some("md.comp.slider.active.stop-indicator.container.opacity"),
            1.0,
        )
    } else {
        tokens.number_optional(
            Some("md.comp.slider.inactive.stop-indicator.container.opacity"),
            1.0,
        )
    };

    alpha_mul(base, opacity)
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
    let tokens = MaterialTokenResolver::new(theme);
    if !enabled {
        let base = tokens.color_comp_or_sys(
            "md.comp.slider.disabled.active.track.color",
            "md.sys.color.on-surface",
        );
        let opacity =
            tokens.number_optional(Some("md.comp.slider.disabled.active.track.opacity"), 0.38);
        return alpha_mul(base, opacity);
    }

    let key = match interaction {
        SliderInteraction::Pressed => "md.comp.slider.pressed.active.track.color",
        SliderInteraction::Focused => "md.comp.slider.focus.active.track.color",
        SliderInteraction::Hovered | SliderInteraction::None => "md.comp.slider.active.track.color",
    };

    tokens.color_comp_or_sys(key, "md.sys.color.primary")
}

pub(crate) fn inactive_track_color(
    theme: &Theme,
    enabled: bool,
    interaction: SliderInteraction,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    if !enabled {
        let base = tokens.color_comp_or_sys(
            "md.comp.slider.disabled.inactive.track.color",
            "md.sys.color.on-surface",
        );
        let opacity =
            tokens.number_optional(Some("md.comp.slider.disabled.inactive.track.opacity"), 0.12);
        return alpha_mul(base, opacity);
    }

    let key = match interaction {
        SliderInteraction::Pressed => "md.comp.slider.pressed.inactive.track.color",
        SliderInteraction::Focused => "md.comp.slider.focus.inactive.track.color",
        SliderInteraction::Hovered | SliderInteraction::None => {
            "md.comp.slider.inactive.track.color"
        }
    };

    tokens.color_comp_or_sys(key, "md.sys.color.secondary-container")
}

pub(crate) fn handle_color(theme: &Theme, enabled: bool, interaction: SliderInteraction) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    if !enabled {
        let base = tokens.color_comp_or_sys(
            "md.comp.slider.disabled.handle.color",
            "md.sys.color.on-surface",
        );
        let opacity = tokens.number_optional(Some("md.comp.slider.disabled.handle.opacity"), 0.38);
        return alpha_mul(base, opacity);
    }

    let key = match interaction {
        SliderInteraction::Pressed => "md.comp.slider.pressed.handle.color",
        SliderInteraction::Focused => "md.comp.slider.focus.handle.color",
        SliderInteraction::Hovered => "md.comp.slider.hover.handle.color",
        SliderInteraction::None => "md.comp.slider.handle.color",
    };

    tokens.color_comp_or_sys(key, "md.sys.color.primary")
}

pub(crate) fn track_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.slider.active.track.shape")
        .or_else(|| {
            theme
                .metric_by_key("md.comp.slider.active.track.shape")
                .map(Corners::all)
        })
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
        .or_else(|| {
            theme
                .metric_by_key("md.comp.slider.handle.shape")
                .map(Corners::all)
        })
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or_else(|| Corners::all(Px(9999.0)))
}
