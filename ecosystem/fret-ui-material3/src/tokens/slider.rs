//! Typed token access for Material 3 sliders.
//!
//! This module owns Slider token policy directly so size, shape, interaction, and fallback
//! matrices stay local to the component-facing token interface.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};
use crate::tokens::typography;

const DEFAULT_STATE_LAYER_SIZE: Px = Px(40.0);
const DEFAULT_VALUE_INDICATOR_BOTTOM_SPACE: Px = Px(12.0);
const DEFAULT_TICK_MARK_SIZE: Px = Px(2.0);
const DEFAULT_TICK_MARK_OPACITY: f32 = 0.38;
const DEFAULT_STOP_INDICATOR_SIZE: Px = Px(4.0);
const DEFAULT_STOP_INDICATOR_TRAILING_SPACE: Px = Px(4.0);
const DEFAULT_DISABLED_CONTENT_OPACITY: f32 = 0.38;
const DEFAULT_DISABLED_INACTIVE_TRACK_OPACITY: f32 = 0.12;
const DEFAULT_FULL_SHAPE: Corners = Corners::all(Px(9999.0));
const DEFAULT_SELECTED_STOP_INDICATOR_OPACITY: f32 = 1.0;
const DEFAULT_UNSELECTED_STOP_INDICATOR_OPACITY: f32 = 1.0;
const DEFAULT_TRACK_HEIGHT: Px = Px(16.0);
const DEFAULT_HANDLE_HEIGHT: Px = Px(44.0);
const DEFAULT_HANDLE_RESTING_WIDTH: Px = Px(4.0);
const DEFAULT_HANDLE_PRESSED_WIDTH: Px = Px(2.0);
const DEFAULT_HANDLE_FOCUSED_WIDTH: Px = Px(2.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SliderInteraction {
    None,
    Hovered,
    Focused,
    Pressed,
}

fn slider_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

pub(crate) fn state_layer_size(theme: &Theme) -> Px {
    slider_metric(
        theme,
        "md.comp.slider.state-layer.size",
        DEFAULT_STATE_LAYER_SIZE,
    )
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

pub(crate) fn value_indicator_bottom_space(theme: &Theme) -> Px {
    slider_metric(
        theme,
        "md.comp.slider.value-indicator.active.bottom-space",
        DEFAULT_VALUE_INDICATOR_BOTTOM_SPACE,
    )
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
    slider_metric(
        theme,
        "md.comp.slider.with-tick-marks.container.size",
        DEFAULT_TICK_MARK_SIZE,
    )
}

pub(crate) fn tick_mark_shape(theme: &Theme) -> Corners {
    MaterialTokenResolver::new(theme)
        .corners_value("md.comp.slider.with-tick-marks.container.shape")
        .unwrap_or(DEFAULT_FULL_SHAPE)
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
            DEFAULT_TICK_MARK_OPACITY,
        );
    }

    let key = if active {
        "md.comp.slider.with-tick-marks.active.container.opacity"
    } else {
        "md.comp.slider.with-tick-marks.inactive.container.opacity"
    };
    tokens.number_optional(Some(key), DEFAULT_TICK_MARK_OPACITY)
}

pub(crate) fn stop_indicator_size(theme: &Theme) -> Px {
    slider_metric(
        theme,
        "md.comp.slider.stop-indicator.size",
        DEFAULT_STOP_INDICATOR_SIZE,
    )
}

pub(crate) fn stop_indicator_shape(theme: &Theme) -> Corners {
    MaterialTokenResolver::new(theme)
        .corners_value("md.comp.slider.stop-indicator.shape")
        .unwrap_or(DEFAULT_FULL_SHAPE)
}

pub(crate) fn stop_indicator_trailing_space(theme: &Theme) -> Px {
    slider_metric(
        theme,
        "md.comp.slider.stop-indicator.trailing-space",
        DEFAULT_STOP_INDICATOR_TRAILING_SPACE,
    )
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
            DEFAULT_DISABLED_CONTENT_OPACITY,
        )
    } else if selected {
        tokens.number_optional(
            Some("md.comp.slider.active.stop-indicator.container.opacity"),
            DEFAULT_SELECTED_STOP_INDICATOR_OPACITY,
        )
    } else {
        tokens.number_optional(
            Some("md.comp.slider.inactive.stop-indicator.container.opacity"),
            DEFAULT_UNSELECTED_STOP_INDICATOR_OPACITY,
        )
    };

    alpha_mul(base, opacity)
}

pub(crate) fn active_track_height(theme: &Theme) -> Px {
    slider_metric(
        theme,
        "md.comp.slider.active.track.height",
        DEFAULT_TRACK_HEIGHT,
    )
}

pub(crate) fn inactive_track_height(theme: &Theme) -> Px {
    slider_metric(
        theme,
        "md.comp.slider.inactive.track.height",
        DEFAULT_TRACK_HEIGHT,
    )
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
        let opacity = tokens.number_optional(
            Some("md.comp.slider.disabled.active.track.opacity"),
            DEFAULT_DISABLED_CONTENT_OPACITY,
        );
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
        let opacity = tokens.number_optional(
            Some("md.comp.slider.disabled.inactive.track.opacity"),
            DEFAULT_DISABLED_INACTIVE_TRACK_OPACITY,
        );
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
        let opacity = tokens.number_optional(
            Some("md.comp.slider.disabled.handle.opacity"),
            DEFAULT_DISABLED_CONTENT_OPACITY,
        );
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
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[
            "md.comp.slider.active.track.shape",
            "md.sys.shape.corner.full",
        ],
        DEFAULT_FULL_SHAPE,
    )
}

pub(crate) fn handle_height(theme: &Theme) -> Px {
    slider_metric(theme, "md.comp.slider.handle.height", DEFAULT_HANDLE_HEIGHT)
}

pub(crate) fn handle_width(theme: &Theme, enabled: bool, interaction: SliderInteraction) -> Px {
    if !enabled {
        return slider_metric(
            theme,
            "md.comp.slider.disabled.handle.width",
            DEFAULT_HANDLE_RESTING_WIDTH,
        );
    }

    match interaction {
        SliderInteraction::Pressed => slider_metric(
            theme,
            "md.comp.slider.pressed.handle.width",
            DEFAULT_HANDLE_PRESSED_WIDTH,
        ),
        SliderInteraction::Focused => slider_metric(
            theme,
            "md.comp.slider.focus.handle.width",
            DEFAULT_HANDLE_FOCUSED_WIDTH,
        ),
        SliderInteraction::Hovered => MaterialTokenResolver::new(theme)
            .metric_value("md.comp.slider.hover.handle.width")
            .unwrap_or_else(|| handle_width(theme, enabled, SliderInteraction::None)),
        SliderInteraction::None => slider_metric(
            theme,
            "md.comp.slider.handle.width",
            DEFAULT_HANDLE_RESTING_WIDTH,
        ),
    }
}

pub(crate) fn handle_shape(theme: &Theme) -> Corners {
    MaterialTokenResolver::new(theme).corners_chain_or(
        &["md.comp.slider.handle.shape", "md.sys.shape.corner.full"],
        DEFAULT_FULL_SHAPE,
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn empty_theme() -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config_patch(&ThemeConfig::default());
        });
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn slider_size_defaults_match_material_matrix() {
        let (_app, theme) = empty_theme();

        assert_eq!(state_layer_size(&theme), Px(40.0));
        assert_eq!(value_indicator_bottom_space(&theme), Px(12.0));
        assert_eq!(tick_mark_size(&theme), Px(2.0));
        assert_eq!(stop_indicator_size(&theme), Px(4.0));
        assert_eq!(stop_indicator_trailing_space(&theme), Px(4.0));
        assert_eq!(active_track_height(&theme), Px(16.0));
        assert_eq!(inactive_track_height(&theme), Px(16.0));
        assert_eq!(handle_height(&theme), Px(44.0));
        assert_eq!(handle_width(&theme, true, SliderInteraction::None), Px(4.0));
        assert_eq!(
            handle_width(&theme, true, SliderInteraction::Pressed),
            Px(2.0)
        );
        assert_eq!(
            handle_width(&theme, true, SliderInteraction::Focused),
            Px(2.0)
        );
    }

    #[test]
    fn slider_shape_defaults_use_full_shape() {
        let (_app, theme) = empty_theme();

        assert_eq!(tick_mark_shape(&theme), Corners::all(Px(9999.0)));
        assert_eq!(stop_indicator_shape(&theme), Corners::all(Px(9999.0)));
        assert_eq!(track_shape(&theme), Corners::all(Px(9999.0)));
        assert_eq!(handle_shape(&theme), Corners::all(Px(9999.0)));
    }

    #[test]
    fn slider_component_tokens_override_metric_defaults() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.slider.state-layer.size".to_string(), 42.0);
        patch.metrics.insert(
            "md.comp.slider.value-indicator.active.bottom-space".to_string(),
            14.0,
        );
        patch.metrics.insert(
            "md.comp.slider.with-tick-marks.container.size".to_string(),
            3.0,
        );
        patch
            .metrics
            .insert("md.comp.slider.stop-indicator.size".to_string(), 5.0);
        patch.metrics.insert(
            "md.comp.slider.stop-indicator.trailing-space".to_string(),
            6.0,
        );
        patch
            .metrics
            .insert("md.comp.slider.active.track.height".to_string(), 18.0);
        patch
            .metrics
            .insert("md.comp.slider.inactive.track.height".to_string(), 20.0);
        patch
            .metrics
            .insert("md.comp.slider.handle.height".to_string(), 48.0);
        patch
            .metrics
            .insert("md.comp.slider.pressed.handle.width".to_string(), 6.0);
        patch
            .metrics
            .insert("md.comp.slider.hover.handle.width".to_string(), 8.0);
        patch
            .metrics
            .insert("md.comp.slider.disabled.handle.width".to_string(), 10.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(state_layer_size(&theme), Px(42.0));
        assert_eq!(value_indicator_bottom_space(&theme), Px(14.0));
        assert_eq!(tick_mark_size(&theme), Px(3.0));
        assert_eq!(stop_indicator_size(&theme), Px(5.0));
        assert_eq!(stop_indicator_trailing_space(&theme), Px(6.0));
        assert_eq!(active_track_height(&theme), Px(18.0));
        assert_eq!(inactive_track_height(&theme), Px(20.0));
        assert_eq!(handle_height(&theme), Px(48.0));
        assert_eq!(
            handle_width(&theme, true, SliderInteraction::Pressed),
            Px(6.0)
        );
        assert_eq!(
            handle_width(&theme, true, SliderInteraction::Hovered),
            Px(8.0)
        );
        assert_eq!(
            handle_width(&theme, false, SliderInteraction::Pressed),
            Px(10.0)
        );
    }

    #[test]
    fn slider_hovered_handle_width_falls_back_to_resting_width() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.slider.handle.width".to_string(), 7.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            handle_width(&theme, true, SliderInteraction::Hovered),
            Px(7.0)
        );
    }

    #[test]
    fn slider_state_layer_opacity_uses_material_interaction_policy() {
        let mut patch = ThemeConfig::default();
        patch.numbers.insert(
            "md.comp.slider.pressed.state-layer.opacity".to_string(),
            0.31,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            state_layer_target_opacity(&theme, true, SliderInteraction::Pressed),
            0.31
        );
        assert_eq!(
            state_layer_target_opacity(&theme, false, SliderInteraction::Pressed),
            0.0
        );
        assert_eq!(
            state_layer_target_opacity(&theme, true, SliderInteraction::None),
            0.0
        );
    }
}
