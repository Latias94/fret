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
    let suffix =
        selected_interaction_suffix("period-selector", selected, interaction, "label-text.color");
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, &suffix),
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
    let suffix = selected_interaction_suffix(
        "period-selector",
        selected,
        Some(interaction),
        "state-layer.color",
    );
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key(component_prefix, &suffix),
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
    let suffix = interaction_suffix("period-selector", interaction, "state-layer.opacity");
    MaterialTokenResolver::new(theme)
        .pressable_state_layer_opacity_or(
            &token_key(component_prefix, &suffix),
            interaction,
            DEFAULT_PERIOD_SELECTOR_STATE_LAYER_OPACITY,
        )
        .clamp(0.0, 1.0)
}

fn token_key(component_prefix: &str, suffix: &str) -> String {
    format!("{component_prefix}.{suffix}")
}

pub(crate) fn selected_interaction_suffix(
    selector_prefix: &str,
    selected: bool,
    interaction: Option<PressableInteraction>,
    role_suffix: &str,
) -> String {
    let selected_state = if selected { "selected" } else { "unselected" };
    match interaction {
        Some(interaction) => format!(
            "{selector_prefix}.{selected_state}.{}.{role_suffix}",
            interaction.token_state()
        ),
        None => format!("{selector_prefix}.{selected_state}.{role_suffix}"),
    }
}

pub(crate) fn interaction_suffix(
    selector_prefix: &str,
    interaction: PressableInteraction,
    role_suffix: &str,
) -> String {
    format!(
        "{selector_prefix}.{}.{role_suffix}",
        interaction.token_state()
    )
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
    fn time_family_interaction_suffixes_build_selected_paths() {
        assert_eq!(
            selected_interaction_suffix(
                "period-selector",
                true,
                Some(PressableInteraction::Hovered),
                "label-text.color",
            ),
            "period-selector.selected.hover.label-text.color"
        );
        assert_eq!(
            selected_interaction_suffix(
                "time-selector",
                false,
                Some(PressableInteraction::Pressed),
                "state-layer.color",
            ),
            "time-selector.unselected.pressed.state-layer.color"
        );
        assert_eq!(
            selected_interaction_suffix("time-selector", true, None, "label-text.color"),
            "time-selector.selected.label-text.color"
        );
        assert_eq!(
            interaction_suffix(
                "period-selector",
                PressableInteraction::Focused,
                "state-layer.opacity",
            ),
            "period-selector.focus.state-layer.opacity"
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
