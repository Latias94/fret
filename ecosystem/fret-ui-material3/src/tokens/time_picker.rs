//! Typed token access for Material 3 time picker primitives.
//!
//! Reference: Material Web v30 `md.comp.time-picker.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.time-picker";

pub(crate) fn container_color(theme: &Theme) -> Color {
    implementation::container_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    implementation::container_elevation(theme, COMPONENT_PREFIX)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    implementation::container_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn headline_style(theme: &Theme) -> TextStyle {
    implementation::headline_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn headline_color(theme: &Theme) -> Color {
    implementation::headline_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_size(theme: &Theme) -> Px {
    implementation::clock_dial_size(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_background(theme: &Theme) -> Color {
    implementation::clock_dial_background(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_shape(theme: &Theme) -> Corners {
    implementation::clock_dial_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_label_text_style(theme: &Theme) -> TextStyle {
    implementation::clock_dial_label_text_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_label_text_color(theme: &Theme, selected: bool) -> Color {
    implementation::clock_dial_label_text_color(theme, COMPONENT_PREFIX, selected)
}

pub(crate) fn clock_dial_handle_size(theme: &Theme) -> Px {
    implementation::clock_dial_handle_size(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_handle_color(theme: &Theme) -> Color {
    implementation::clock_dial_handle_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_handle_shape(theme: &Theme) -> Corners {
    implementation::clock_dial_handle_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_selector_center_size(theme: &Theme) -> Px {
    implementation::clock_dial_selector_center_size(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_selector_center_color(theme: &Theme) -> Color {
    implementation::clock_dial_selector_center_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_selector_center_shape(theme: &Theme) -> Corners {
    implementation::clock_dial_selector_center_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_selector_track_width(theme: &Theme) -> Px {
    implementation::clock_dial_selector_track_width(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_selector_track_color(theme: &Theme) -> Color {
    implementation::clock_dial_selector_track_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_container_width(theme: &Theme) -> Px {
    implementation::time_selector_container_width(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_container_height(theme: &Theme) -> Px {
    implementation::time_selector_container_height(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_shape(theme: &Theme) -> Corners {
    implementation::time_selector_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_container_color(theme: &Theme, selected: bool) -> Color {
    implementation::time_selector_container_color(theme, COMPONENT_PREFIX, selected)
}

pub(crate) fn time_selector_label_text_style(theme: &Theme) -> TextStyle {
    implementation::time_selector_label_text_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_separator_style(theme: &Theme) -> TextStyle {
    implementation::time_selector_separator_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_separator_color(theme: &Theme) -> Color {
    implementation::time_selector_separator_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn display_separator_width(theme: &Theme) -> Px {
    implementation::display_separator_width(theme)
}

pub(crate) fn time_selector_label_color(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    implementation::time_selector_label_color(theme, COMPONENT_PREFIX, selected, interaction)
}

pub(crate) fn time_selector_state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: PressableInteraction,
) -> Color {
    implementation::time_selector_state_layer_color(theme, COMPONENT_PREFIX, selected, interaction)
}

pub(crate) fn time_selector_state_layer_opacity(
    theme: &Theme,
    interaction: PressableInteraction,
) -> f32 {
    implementation::time_selector_state_layer_opacity(theme, COMPONENT_PREFIX, interaction)
}

pub(crate) fn period_selector_container_width(theme: &Theme) -> Px {
    implementation::period_selector_container_width(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_container_height(theme: &Theme) -> Px {
    implementation::period_selector_container_height(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_shape(theme: &Theme) -> Corners {
    implementation::period_selector_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_outline_width(theme: &Theme) -> Px {
    implementation::period_selector_outline_width(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_outline_color(theme: &Theme) -> Color {
    implementation::period_selector_outline_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_selected_container_color(theme: &Theme) -> Color {
    implementation::period_selector_selected_container_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_label_text_style(theme: &Theme) -> TextStyle {
    implementation::period_selector_label_text_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_label_color(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    implementation::period_selector_label_color(theme, COMPONENT_PREFIX, selected, interaction)
}

pub(crate) fn period_selector_state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: PressableInteraction,
) -> Color {
    implementation::period_selector_state_layer_color(
        theme,
        COMPONENT_PREFIX,
        selected,
        interaction,
    )
}

pub(crate) fn period_selector_state_layer_opacity(
    theme: &Theme,
    interaction: PressableInteraction,
) -> f32 {
    implementation::period_selector_state_layer_opacity(theme, COMPONENT_PREFIX, interaction)
}

mod implementation {
    //! Local token fallback helpers for Material 3 time picker surfaces.
    //!
    //! This module owns the stable Material time picker default matrices behind the
    //! component-facing `tokens::time_picker` interface.

    use fret_core::{Color, Corners, Px, TextStyle};
    use fret_ui::Theme;
    use fret_ui_kit::typography::TextIntent;

    use crate::foundation::interaction::PressableInteraction;
    use crate::foundation::token_resolver::MaterialTokenResolver;
    use crate::tokens::{time_period_common, typography};

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

    fn time_picker_metric(theme: &Theme, component_prefix: &str, suffix: &str, fallback: Px) -> Px {
        MaterialTokenResolver::new(theme)
            .metric_optional(Some(&token_key(component_prefix, suffix)), fallback)
    }

    fn material_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
        MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
    }

    pub(crate) fn container_elevation(theme: &Theme, component_prefix: &str) -> Px {
        time_picker_metric(
            theme,
            component_prefix,
            "container.elevation",
            DEFAULT_CONTAINER_ELEVATION,
        )
    }

    pub(crate) fn container_shape(theme: &Theme, component_prefix: &str) -> Corners {
        let key = token_key(component_prefix, "container.shape");
        MaterialTokenResolver::new(theme).corners_chain_or(
            &[key.as_str(), "md.sys.shape.corner.extra-large"],
            DEFAULT_CONTAINER_SHAPE,
        )
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
        time_picker_metric(
            theme,
            component_prefix,
            "clock-dial.container.size",
            DEFAULT_CLOCK_DIAL_SIZE,
        )
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
        time_picker_metric(
            theme,
            component_prefix,
            "clock-dial.selector.handle.container.size",
            DEFAULT_CLOCK_DIAL_HANDLE_SIZE,
        )
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
        time_picker_metric(
            theme,
            component_prefix,
            "clock-dial.selector.center.container.size",
            DEFAULT_CLOCK_DIAL_SELECTOR_CENTER_SIZE,
        )
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

    pub(crate) fn clock_dial_selector_center_shape(
        theme: &Theme,
        component_prefix: &str,
    ) -> Corners {
        full_shape_or_token(
            theme,
            component_prefix,
            "clock-dial.selector.center.container.shape",
        )
    }

    pub(crate) fn clock_dial_selector_track_width(theme: &Theme, component_prefix: &str) -> Px {
        time_picker_metric(
            theme,
            component_prefix,
            "clock-dial.selector.track.container.width",
            DEFAULT_CLOCK_DIAL_SELECTOR_TRACK_WIDTH,
        )
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
        time_picker_metric(
            theme,
            component_prefix,
            "time-selector.container.width",
            DEFAULT_TIME_SELECTOR_CONTAINER_WIDTH,
        )
    }

    pub(crate) fn time_selector_container_height(theme: &Theme, component_prefix: &str) -> Px {
        time_picker_metric(
            theme,
            component_prefix,
            "time-selector.container.height",
            DEFAULT_TIME_SELECTOR_CONTAINER_HEIGHT,
        )
    }

    pub(crate) fn time_selector_shape(theme: &Theme, component_prefix: &str) -> Corners {
        let key = token_key(component_prefix, "time-selector.container.shape");
        MaterialTokenResolver::new(theme).corners_chain_or(
            &[key.as_str(), "md.sys.shape.corner.small"],
            DEFAULT_TIME_SELECTOR_CONTAINER_SHAPE,
        )
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

    pub(crate) fn time_selector_label_text_style(
        theme: &Theme,
        component_prefix: &str,
    ) -> TextStyle {
        typography::text_style(
            theme,
            Some(&token_key(component_prefix, "time-selector.label-text")),
            "md.sys.typescale.display-large",
            TextIntent::Control,
        )
    }

    pub(crate) fn time_selector_separator_style(
        theme: &Theme,
        component_prefix: &str,
    ) -> TextStyle {
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
        material_metric(
            theme,
            "md.sys.fret.material.time-picker.display-separator.width",
            DEFAULT_DISPLAY_SEPARATOR_WIDTH,
        )
    }

    pub(crate) fn time_selector_label_color(
        theme: &Theme,
        component_prefix: &str,
        selected: bool,
        interaction: Option<PressableInteraction>,
    ) -> Color {
        let suffix = time_period_common::selected_interaction_suffix(
            "time-selector",
            selected,
            interaction,
            "label-text.color",
        );
        MaterialTokenResolver::new(theme).color_comp_or_sys(
            &token_key(component_prefix, &suffix),
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
        let suffix = time_period_common::selected_interaction_suffix(
            "time-selector",
            selected,
            Some(interaction),
            "state-layer.color",
        );
        MaterialTokenResolver::new(theme).color_comp_or_sys(
            &token_key(component_prefix, &suffix),
            "md.sys.color.on-surface",
        )
    }

    pub(crate) fn time_selector_state_layer_opacity(
        theme: &Theme,
        component_prefix: &str,
        interaction: PressableInteraction,
    ) -> f32 {
        let suffix = time_period_common::interaction_suffix(
            "time-selector",
            interaction,
            "state-layer.opacity",
        );
        MaterialTokenResolver::new(theme)
            .pressable_state_layer_opacity_or(
                &token_key(component_prefix, &suffix),
                interaction,
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

    pub(crate) fn period_selector_label_text_style(
        theme: &Theme,
        component_prefix: &str,
    ) -> TextStyle {
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
        MaterialTokenResolver::new(theme).corners_chain_or(
            &[key.as_str(), "md.sys.shape.corner.full"],
            DEFAULT_FULL_SHAPE,
        )
    }

    fn token_key(component_prefix: &str, suffix: &str) -> String {
        format!("{component_prefix}.{suffix}")
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
                "md.comp.test-time-picker.container.elevation".to_string(),
                5.0,
            );
            patch.metrics.insert(
                "md.comp.test-time-picker.time-selector.container.width".to_string(),
                112.0,
            );
            patch.metrics.insert(
                "md.comp.test-time-picker.time-selector.container.height".to_string(),
                84.0,
            );
            patch.metrics.insert(
                "md.sys.fret.material.time-picker.display-separator.width".to_string(),
                32.0,
            );
            let (_app, theme) = theme_with_patch(patch);

            assert_eq!(
                container_elevation(&theme, "md.comp.test-time-picker"),
                Px(5.0)
            );
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
                Px(112.0)
            );
            assert_eq!(
                time_selector_container_height(&theme, "md.comp.test-time-picker"),
                Px(84.0)
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
}
