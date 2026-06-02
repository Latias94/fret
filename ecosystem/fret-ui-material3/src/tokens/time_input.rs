//! Typed token access for Material 3 time input primitives.
//!
//! Reference: Material Web v30 `md.comp.time-input.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::{time_period_common, typography};

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.time-input";

fn token_key(suffix: &str) -> String {
    format!("{COMPONENT_PREFIX}.{suffix}")
}

fn time_input_metric(theme: &Theme, suffix: &str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(&token_key(suffix)), fallback)
}

pub(crate) fn time_input_field_container_width(theme: &Theme) -> Px {
    time_input_metric(theme, "time-input-field.container.width", Px(96.0))
}

pub(crate) fn time_input_field_container_height(theme: &Theme) -> Px {
    time_input_metric(theme, "time-input-field.container.height", Px(72.0))
}

pub(crate) fn time_input_field_container_shape(theme: &Theme) -> Corners {
    let key = token_key("time-input-field.container.shape");
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[key.as_str(), "md.sys.shape.corner.small"],
        Corners::all(Px(8.0)),
    )
}

pub(crate) fn time_input_field_container_color(theme: &Theme, focused: bool, error: bool) -> Color {
    if error {
        return MaterialTokenResolver::new(theme).color_comp_or_sys(
            &token_key(if focused {
                "time-input-field.error.focus.container.color"
            } else {
                "time-input-field.error.container.color"
            }),
            "md.sys.color.error-container",
        );
    }

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
    time_input_metric(theme, "time-input-field.focus.outline.width", Px(2.0))
}

pub(crate) fn time_input_field_focus_outline_color(theme: &Theme, error: bool) -> Color {
    if error {
        return MaterialTokenResolver::new(theme).color_comp_or_sys(
            &token_key("time-input-field.error.focus.outline.color"),
            "md.sys.color.error",
        );
    }

    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("time-input-field.focus.outline.color"),
        "md.sys.color.primary",
    )
}

pub(crate) fn time_input_field_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key("time-input-field.label-text")),
        "md.sys.typescale.display-medium",
        TextIntent::Control,
    )
}

pub(crate) fn time_input_field_label_color(
    theme: &Theme,
    focused: bool,
    hovered: bool,
    error: bool,
) -> Color {
    if error {
        let suffix = match (focused, hovered) {
            (true, _) => "time-input-field.error.focus.label-text.color",
            (false, true) => "time-input-field.error.hover.label-text.color",
            _ => "time-input-field.error.label-text.color",
        };
        return MaterialTokenResolver::new(theme)
            .color_comp_or_sys(&token_key(suffix), "md.sys.color.on-error-container");
    }

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
    MaterialTokenResolver::new(theme)
        .number_comp_or_sys(
            &token_key("time-input-field.hover.state-layer.opacity"),
            "md.sys.state.hover.state-layer-opacity",
            0.0,
        )
        .clamp(0.0, 1.0)
}

pub(crate) fn time_input_field_separator_style(theme: &Theme) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key("time-input-field.separator")),
        "md.sys.typescale.display-large",
        TextIntent::Control,
    )
}

pub(crate) fn time_input_field_separator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("time-input-field.separator.color"),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn time_input_field_supporting_text_style(theme: &Theme) -> TextStyle {
    typography::text_style(
        theme,
        Some(&token_key("time-input-field.supporting-text")),
        "md.sys.typescale.body-small",
        TextIntent::Content,
    )
}

pub(crate) fn time_input_field_supporting_text_color(theme: &Theme, error: bool) -> Color {
    if error {
        return MaterialTokenResolver::new(theme).color_comp_or_sys(
            &token_key("time-input-field.error.supporting-text.color"),
            "md.sys.color.error",
        );
    }

    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &token_key("time-input-field.supporting-text.color"),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn period_selector_container_width(theme: &Theme) -> Px {
    time_period_common::container_width(theme, COMPONENT_PREFIX, "period-selector.container.width")
}

pub(crate) fn period_selector_container_height(theme: &Theme) -> Px {
    time_period_common::container_height(
        theme,
        COMPONENT_PREFIX,
        "period-selector.container.height",
        Px(72.0),
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn time_input_metrics_keep_material_defaults() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(time_input_field_container_width(theme), Px(96.0));
        assert_eq!(time_input_field_container_height(theme), Px(72.0));
        assert_eq!(time_input_field_focus_outline_width(theme), Px(2.0));
    }

    #[test]
    fn time_input_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.time-input.time-input-field.container.width".to_string(),
            108.0,
        );
        patch.metrics.insert(
            "md.comp.time-input.time-input-field.container.height".to_string(),
            76.0,
        );
        patch.metrics.insert(
            "md.comp.time-input.time-input-field.focus.outline.width".to_string(),
            3.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(time_input_field_container_width(&theme), Px(108.0));
        assert_eq!(time_input_field_container_height(&theme), Px(76.0));
        assert_eq!(time_input_field_focus_outline_width(&theme), Px(3.0));
    }
}
