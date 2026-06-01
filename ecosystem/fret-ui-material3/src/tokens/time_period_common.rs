//! Shared token fallback helpers for Material 3 time-family period selectors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::typography;

const DEFAULT_PERIOD_SELECTOR_CONTAINER_WIDTH: Px = Px(52.0);
const DEFAULT_PERIOD_SELECTOR_CONTAINER_SHAPE: Px = Px(8.0);
const DEFAULT_PERIOD_SELECTOR_OUTLINE_WIDTH: Px = Px(1.0);
const DEFAULT_PERIOD_SELECTOR_STATE_LAYER_OPACITY: f32 = 0.0;

fn period_metric(theme: &Theme, component_prefix: &str, suffix: &str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme)
        .metric_optional(Some(&token_key(component_prefix, suffix)), fallback)
}

pub(crate) fn container_width(theme: &Theme, component_prefix: &str, suffix: &str) -> Px {
    period_metric(
        theme,
        component_prefix,
        suffix,
        DEFAULT_PERIOD_SELECTOR_CONTAINER_WIDTH,
    )
}

pub(crate) fn container_height(
    theme: &Theme,
    component_prefix: &str,
    suffix: &str,
    fallback: Px,
) -> Px {
    period_metric(theme, component_prefix, suffix, fallback)
}

pub(crate) fn container_shape(theme: &Theme, component_prefix: &str) -> Corners {
    let key = token_key(component_prefix, "period-selector.container.shape");
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[key.as_str(), "md.sys.shape.corner.small"],
        Corners::all(DEFAULT_PERIOD_SELECTOR_CONTAINER_SHAPE),
    )
}

pub(crate) fn outline_width(theme: &Theme, component_prefix: &str) -> Px {
    period_metric(
        theme,
        component_prefix,
        "period-selector.outline.width",
        DEFAULT_PERIOD_SELECTOR_OUTLINE_WIDTH,
    )
}

pub(crate) fn outline_color(theme: &Theme, component_prefix: &str) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, "period-selector.outline.color"),
        "md.sys.color.outline",
    )
}

pub(crate) fn selected_container_color(theme: &Theme, component_prefix: &str) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, "period-selector.selected.container.color"),
        "md.sys.color.tertiary-container",
    )
}

pub(crate) fn label_text_style(theme: &Theme, component_prefix: &str) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key(component_prefix, "period-selector.label-text")),
        "md.sys.typescale.title-medium",
        TextIntent::Control,
    )
}

pub(crate) fn label_color(
    theme: &Theme,
    component_prefix: &str,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    let suffix = label_color_suffix(selected, interaction);
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, suffix),
        if selected {
            "md.sys.color.on-tertiary-container"
        } else {
            "md.sys.color.on-surface-variant"
        },
    )
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    component_prefix: &str,
    selected: bool,
    interaction: PressableInteraction,
) -> Color {
    let suffix = state_layer_color_suffix(selected, interaction);
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, suffix),
        if selected {
            "md.sys.color.on-tertiary-container"
        } else {
            "md.sys.color.on-surface-variant"
        },
    )
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    component_prefix: &str,
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
    MaterialTokenResolver::new(theme)
        .number_comp_or_sys(
            &token_key(component_prefix, suffix),
            fallback,
            DEFAULT_PERIOD_SELECTOR_STATE_LAYER_OPACITY,
        )
        .clamp(0.0, 1.0)
}

fn token_key(component_prefix: &str, suffix: &str) -> String {
    format!("{component_prefix}.{suffix}")
}

fn label_color_suffix(selected: bool, interaction: Option<PressableInteraction>) -> &'static str {
    match (selected, interaction) {
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
    }
}

fn state_layer_color_suffix(selected: bool, interaction: PressableInteraction) -> &'static str {
    match (selected, interaction) {
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
    fn period_selector_metrics_keep_prefix_specific_suffixes() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.time-picker.period-selector.vertical.container.width".to_string(),
            64.0,
        );
        patch.metrics.insert(
            "md.comp.time-input.period-selector.container.height".to_string(),
            74.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_width(
                &theme,
                "md.comp.time-picker",
                "period-selector.vertical.container.width",
            ),
            Px(64.0)
        );
        assert_eq!(
            container_height(
                &theme,
                "md.comp.time-input",
                "period-selector.container.height",
                Px(72.0),
            ),
            Px(74.0)
        );
    }

    #[test]
    fn period_selector_shape_falls_back_to_sys_small_shape() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.shape.corner.small".to_string(), 14.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme, "md.comp.test-time"),
            Corners::all(Px(14.0))
        );
    }

    #[test]
    fn period_selector_state_layer_opacity_uses_component_then_system() {
        let mut patch = ThemeConfig::default();
        patch.numbers.insert(
            "md.comp.time-input.period-selector.pressed.state-layer.opacity".to_string(),
            0.22,
        );
        patch
            .numbers
            .insert("md.sys.state.focus.state-layer-opacity".to_string(), 0.13);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            state_layer_opacity(&theme, "md.comp.time-input", PressableInteraction::Pressed,),
            0.22
        );
        assert_eq!(
            state_layer_opacity(&theme, "md.comp.test-time", PressableInteraction::Focused,),
            0.13
        );
    }
}
