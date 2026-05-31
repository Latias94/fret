//! Shared token fallback helpers for Material 3 time picker surfaces.
//!
//! This module owns the stable Material time picker default matrices so `tokens::time_picker`
//! can stay a small component-facing token facade.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::{shape, time_period_common, typography};

const DEFAULT_CONTAINER_ELEVATION: Px = Px(3.0);
const DEFAULT_CONTAINER_SHAPE: Corners = Corners::all(Px(28.0));
const DEFAULT_CLOCK_DIAL_SIZE: Px = Px(256.0);
const DEFAULT_FULL_SHAPE: Corners = Corners::all(Px(9999.0));
const DEFAULT_CLOCK_DIAL_HANDLE_SIZE: Px = Px(48.0);
const DEFAULT_CLOCK_DIAL_SELECTOR_CENTER_SIZE: Px = Px(8.0);
const DEFAULT_CLOCK_DIAL_SELECTOR_TRACK_WIDTH: Px = Px(2.0);
const DEFAULT_TIME_SELECTOR_CONTAINER_WIDTH: Px = Px(96.0);
const DEFAULT_TIME_SELECTOR_CONTAINER_HEIGHT: Px = Px(80.0);
const DEFAULT_TIME_SELECTOR_CONTAINER_SHAPE: Corners = Corners::all(Px(8.0));
const DEFAULT_DISPLAY_SEPARATOR_WIDTH: Px = Px(24.0);
const DEFAULT_TIME_SELECTOR_STATE_LAYER_OPACITY: f32 = 0.0;
const DEFAULT_PERIOD_SELECTOR_CONTAINER_HEIGHT: Px = Px(80.0);

pub(crate) fn container_color(theme: &Theme, component_prefix: &str) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, "container.color"),
        "md.sys.color.surface-container-high",
    )
}

pub(crate) fn container_elevation(theme: &Theme, component_prefix: &str) -> Px {
    theme
        .metric_by_key(&token_key(component_prefix, "container.elevation"))
        .unwrap_or(DEFAULT_CONTAINER_ELEVATION)
}

pub(crate) fn container_shape(theme: &Theme, component_prefix: &str) -> Corners {
    let key = token_key(component_prefix, "container.shape");
    shape::corners_or_metric(theme, &key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-large"))
        .unwrap_or(DEFAULT_CONTAINER_SHAPE)
}

pub(crate) fn headline_style(theme: &Theme, component_prefix: &str) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key(component_prefix, "headline")),
        "md.sys.typescale.label-medium",
        TextIntent::Control,
    )
}

pub(crate) fn headline_color(theme: &Theme, component_prefix: &str) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, "headline.color"),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn clock_dial_size(theme: &Theme, component_prefix: &str) -> Px {
    theme
        .metric_by_key(&token_key(component_prefix, "clock-dial.container.size"))
        .unwrap_or(DEFAULT_CLOCK_DIAL_SIZE)
}

pub(crate) fn clock_dial_background(theme: &Theme, component_prefix: &str) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, "clock-dial.color"),
        "md.sys.color.surface-container-highest",
    )
}

pub(crate) fn clock_dial_shape(theme: &Theme, component_prefix: &str) -> Corners {
    full_shape_or_token(theme, component_prefix, "clock-dial.shape")
}

pub(crate) fn clock_dial_label_text_style(theme: &Theme, component_prefix: &str) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key(component_prefix, "clock-dial.label-text")),
        "md.sys.typescale.body-large",
        TextIntent::Control,
    )
}

pub(crate) fn clock_dial_label_text_color(
    theme: &Theme,
    component_prefix: &str,
    selected: bool,
) -> Color {
    let suffix = if selected {
        "clock-dial.selected.label-text.color"
    } else {
        "clock-dial.unselected.label-text.color"
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, suffix),
        if selected {
            "md.sys.color.on-primary"
        } else {
            "md.sys.color.on-surface"
        },
    )
}

pub(crate) fn clock_dial_handle_size(theme: &Theme, component_prefix: &str) -> Px {
    theme
        .metric_by_key(&token_key(
            component_prefix,
            "clock-dial.selector.handle.container.size",
        ))
        .unwrap_or(DEFAULT_CLOCK_DIAL_HANDLE_SIZE)
}

pub(crate) fn clock_dial_handle_color(theme: &Theme, component_prefix: &str) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(
            component_prefix,
            "clock-dial.selector.handle.container.color",
        ),
        "md.sys.color.primary",
    )
}

pub(crate) fn clock_dial_handle_shape(theme: &Theme, component_prefix: &str) -> Corners {
    full_shape_or_token(
        theme,
        component_prefix,
        "clock-dial.selector.handle.container.shape",
    )
}

pub(crate) fn clock_dial_selector_center_size(theme: &Theme, component_prefix: &str) -> Px {
    theme
        .metric_by_key(&token_key(
            component_prefix,
            "clock-dial.selector.center.container.size",
        ))
        .unwrap_or(DEFAULT_CLOCK_DIAL_SELECTOR_CENTER_SIZE)
}

pub(crate) fn clock_dial_selector_center_color(theme: &Theme, component_prefix: &str) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(
            component_prefix,
            "clock-dial.selector.center.container.color",
        ),
        "md.sys.color.primary",
    )
}

pub(crate) fn clock_dial_selector_center_shape(theme: &Theme, component_prefix: &str) -> Corners {
    full_shape_or_token(
        theme,
        component_prefix,
        "clock-dial.selector.center.container.shape",
    )
}

pub(crate) fn clock_dial_selector_track_width(theme: &Theme, component_prefix: &str) -> Px {
    theme
        .metric_by_key(&token_key(
            component_prefix,
            "clock-dial.selector.track.container.width",
        ))
        .unwrap_or(DEFAULT_CLOCK_DIAL_SELECTOR_TRACK_WIDTH)
}

pub(crate) fn clock_dial_selector_track_color(theme: &Theme, component_prefix: &str) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(
            component_prefix,
            "clock-dial.selector.track.container.color",
        ),
        "md.sys.color.primary",
    )
}

pub(crate) fn time_selector_container_width(theme: &Theme, component_prefix: &str) -> Px {
    theme
        .metric_by_key(&token_key(
            component_prefix,
            "time-selector.container.width",
        ))
        .unwrap_or(DEFAULT_TIME_SELECTOR_CONTAINER_WIDTH)
}

pub(crate) fn time_selector_container_height(theme: &Theme, component_prefix: &str) -> Px {
    theme
        .metric_by_key(&token_key(
            component_prefix,
            "time-selector.container.height",
        ))
        .unwrap_or(DEFAULT_TIME_SELECTOR_CONTAINER_HEIGHT)
}

pub(crate) fn time_selector_shape(theme: &Theme, component_prefix: &str) -> Corners {
    let key = token_key(component_prefix, "time-selector.container.shape");
    shape::corners_or_metric(theme, &key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.small"))
        .unwrap_or(DEFAULT_TIME_SELECTOR_CONTAINER_SHAPE)
}

pub(crate) fn time_selector_container_color(
    theme: &Theme,
    component_prefix: &str,
    selected: bool,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(
            component_prefix,
            if selected {
                "time-selector.selected.container.color"
            } else {
                "time-selector.unselected.container.color"
            },
        ),
        if selected {
            "md.sys.color.primary-container"
        } else {
            "md.sys.color.surface-container-highest"
        },
    )
}

pub(crate) fn time_selector_label_text_style(theme: &Theme, component_prefix: &str) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key(component_prefix, "time-selector.label-text")),
        "md.sys.typescale.display-large",
        TextIntent::Control,
    )
}

pub(crate) fn time_selector_separator_style(theme: &Theme, component_prefix: &str) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key(component_prefix, "time-selector.separator")),
        "md.sys.typescale.display-large",
        TextIntent::Control,
    )
}

pub(crate) fn time_selector_separator_color(theme: &Theme, component_prefix: &str) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, "time-selector.separator.color"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn display_separator_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.sys.fret.material.time-picker.display-separator.width")
        .unwrap_or(DEFAULT_DISPLAY_SEPARATOR_WIDTH)
}

pub(crate) fn time_selector_label_color(
    theme: &Theme,
    component_prefix: &str,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(
            component_prefix,
            time_selector_label_color_suffix(selected, interaction),
        ),
        if selected {
            "md.sys.color.on-primary-container"
        } else {
            "md.sys.color.on-surface"
        },
    )
}

pub(crate) fn time_selector_state_layer_color(
    theme: &Theme,
    component_prefix: &str,
    selected: bool,
    interaction: PressableInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(
            component_prefix,
            time_selector_state_layer_color_suffix(selected, interaction),
        ),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn time_selector_state_layer_opacity(
    theme: &Theme,
    component_prefix: &str,
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
        .number_comp_or_sys(
            &token_key(component_prefix, suffix),
            fallback,
            DEFAULT_TIME_SELECTOR_STATE_LAYER_OPACITY,
        )
        .clamp(0.0, 1.0)
}

pub(crate) fn period_selector_container_width(theme: &Theme, component_prefix: &str) -> Px {
    time_period_common::container_width(
        theme,
        component_prefix,
        "period-selector.vertical.container.width",
    )
}

pub(crate) fn period_selector_container_height(theme: &Theme, component_prefix: &str) -> Px {
    time_period_common::container_height(
        theme,
        component_prefix,
        "period-selector.vertical.container.height",
        DEFAULT_PERIOD_SELECTOR_CONTAINER_HEIGHT,
    )
}

pub(crate) fn period_selector_shape(theme: &Theme, component_prefix: &str) -> Corners {
    time_period_common::container_shape(theme, component_prefix)
}

pub(crate) fn period_selector_outline_width(theme: &Theme, component_prefix: &str) -> Px {
    time_period_common::outline_width(theme, component_prefix)
}

pub(crate) fn period_selector_outline_color(theme: &Theme, component_prefix: &str) -> Color {
    time_period_common::outline_color(theme, component_prefix)
}

pub(crate) fn period_selector_selected_container_color(
    theme: &Theme,
    component_prefix: &str,
) -> Color {
    time_period_common::selected_container_color(theme, component_prefix)
}

pub(crate) fn period_selector_label_text_style(theme: &Theme, component_prefix: &str) -> TextStyle {
    time_period_common::label_text_style(theme, component_prefix)
}

pub(crate) fn period_selector_label_color(
    theme: &Theme,
    component_prefix: &str,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    time_period_common::label_color(theme, component_prefix, selected, interaction)
}

pub(crate) fn period_selector_state_layer_color(
    theme: &Theme,
    component_prefix: &str,
    selected: bool,
    interaction: PressableInteraction,
) -> Color {
    time_period_common::state_layer_color(theme, component_prefix, selected, interaction)
}

pub(crate) fn period_selector_state_layer_opacity(
    theme: &Theme,
    component_prefix: &str,
    interaction: PressableInteraction,
) -> f32 {
    time_period_common::state_layer_opacity(theme, component_prefix, interaction)
}

fn full_shape_or_token(theme: &Theme, component_prefix: &str, suffix: &str) -> Corners {
    let key = token_key(component_prefix, suffix);
    shape::corners_or_metric(theme, &key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.full"))
        .unwrap_or(DEFAULT_FULL_SHAPE)
}

fn token_key(component_prefix: &str, suffix: &str) -> String {
    format!("{component_prefix}.{suffix}")
}

fn time_selector_label_color_suffix(
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> &'static str {
    match (selected, interaction) {
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
    }
}

fn time_selector_state_layer_color_suffix(
    selected: bool,
    interaction: PressableInteraction,
) -> &'static str {
    match (selected, interaction) {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::v30::{TypographyOptions, theme_config};
    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        let base = theme_config(TypographyOptions::default());
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&base));
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn time_picker_metrics_prefer_component_tokens_and_keep_material_defaults() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.test-time-picker.clock-dial.container.size".to_string(),
            300.0,
        );
        patch.metrics.insert(
            "md.sys.fret.material.time-picker.display-separator.width".to_string(),
            32.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            clock_dial_size(&theme, "md.comp.test-time-picker"),
            Px(300.0)
        );
        assert_eq!(
            clock_dial_handle_size(&theme, "md.comp.test-time-picker"),
            Px(48.0)
        );
        assert_eq!(
            clock_dial_selector_center_size(&theme, "md.comp.test-time-picker"),
            Px(8.0)
        );
        assert_eq!(
            clock_dial_selector_track_width(&theme, "md.comp.test-time-picker"),
            Px(2.0)
        );
        assert_eq!(
            time_selector_container_width(&theme, "md.comp.test-time-picker"),
            Px(96.0)
        );
        assert_eq!(
            time_selector_container_height(&theme, "md.comp.test-time-picker"),
            Px(80.0)
        );
        assert_eq!(display_separator_width(&theme), Px(32.0));
    }

    #[test]
    fn time_picker_shapes_fall_back_to_system_shape_tokens() {
        let mut patch = ThemeConfig::default();
        patch.corners.insert(
            "md.sys.shape.corner.extra-large".to_string(),
            Corners::all(Px(32.0)),
        );
        patch.corners.insert(
            "md.sys.shape.corner.full".to_string(),
            Corners::all(Px(80.0)),
        );
        patch.corners.insert(
            "md.sys.shape.corner.small".to_string(),
            Corners::all(Px(10.0)),
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme, "md.comp.test-time-picker"),
            Corners::all(Px(32.0))
        );
        assert_eq!(
            clock_dial_shape(&theme, "md.comp.test-time-picker"),
            Corners::all(Px(80.0))
        );
        assert_eq!(
            clock_dial_handle_shape(&theme, "md.comp.test-time-picker"),
            Corners::all(Px(80.0))
        );
        assert_eq!(
            time_selector_shape(&theme, "md.comp.test-time-picker"),
            Corners::all(Px(10.0))
        );
    }

    #[test]
    fn time_selector_state_layer_opacity_uses_component_then_system() {
        let mut patch = ThemeConfig::default();
        patch.numbers.insert(
            "md.comp.test-time-picker.time-selector.pressed.state-layer.opacity".to_string(),
            0.22,
        );
        patch
            .numbers
            .insert("md.sys.state.focus.state-layer-opacity".to_string(), 0.13);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            time_selector_state_layer_opacity(
                &theme,
                "md.comp.test-time-picker",
                PressableInteraction::Pressed,
            ),
            0.22
        );
        assert_eq!(
            time_selector_state_layer_opacity(
                &theme,
                "md.comp.other-time-picker",
                PressableInteraction::Focused,
            ),
            0.13
        );
    }
}
