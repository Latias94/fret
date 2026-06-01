//! Shared token fallback helpers for Material 3 chip families.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul};
use crate::tokens::typography;

const DEFAULT_CONTAINER_HEIGHT: Px = Px(32.0);
const DEFAULT_CONTAINER_SHAPE: Px = Px(8.0);
const DEFAULT_ICON_SIZE: Px = Px(18.0);
const DEFAULT_ELEVATION: Px = Px(0.0);
const DEFAULT_OUTLINE_WIDTH: Px = Px(1.0);
const DEFAULT_DISABLED_OUTLINE_OPACITY: f32 = 0.12;
const DEFAULT_PRESSED_STATE_LAYER_OPACITY: f32 = 0.1;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ChipOutline {
    pub width: Px,
    pub color: Color,
}

fn chip_metric(theme: &Theme, key: impl AsRef<str>, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key.as_ref()), fallback)
}

pub(crate) fn disabled_on_surface_color(
    theme: &Theme,
    color_key: &str,
    opacity_key: &str,
    fallback_opacity: f32,
) -> Color {
    let (base, opacity) = MaterialTokenResolver::new(theme).color_comp_or_sys_with_opacity(
        color_key,
        "md.sys.color.on-surface",
        Some(opacity_key),
        fallback_opacity,
    );
    alpha_mul(base, opacity.clamp(0.0, 1.0))
}

pub(crate) fn container_height(theme: &Theme, component_prefix: &str) -> Px {
    chip_metric(
        theme,
        format!("{component_prefix}.container.height"),
        DEFAULT_CONTAINER_HEIGHT,
    )
}

pub(crate) fn container_shape(theme: &Theme, component_prefix: &str) -> Corners {
    let component_key = format!("{component_prefix}.container.shape");
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[component_key.as_str(), "md.sys.shape.corner.small"],
        Corners::all(DEFAULT_CONTAINER_SHAPE),
    )
}

pub(crate) fn icon_size(theme: &Theme, key: &str) -> Px {
    chip_metric(theme, key, DEFAULT_ICON_SIZE)
}

pub(crate) fn label_text_style(theme: &Theme, component_prefix: &str) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        None,
        "md.sys.typescale.label-large",
        Some(&format!("{component_prefix}.label-text.weight")),
        TextIntent::Control,
    )
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    component_prefix: &str,
    state_prefix: Option<&str>,
    interaction: Option<PressableInteraction>,
    sys_key: &str,
) -> Color {
    let Some(interaction_suffix) = interaction_suffix(interaction) else {
        return Color::TRANSPARENT;
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &state_key(
            component_prefix,
            state_prefix,
            interaction_suffix,
            "state-layer.color",
        ),
        sys_key,
    )
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    component_prefix: &str,
    state_prefix: Option<&str>,
    interaction: Option<PressableInteraction>,
) -> f32 {
    let Some(interaction_suffix) = interaction_suffix(interaction) else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme)
        .number_optional(
            Some(&state_key(
                component_prefix,
                state_prefix,
                interaction_suffix,
                "state-layer.opacity",
            )),
            0.0,
        )
        .clamp(0.0, 1.0)
}

pub(crate) fn pressed_state_layer_opacity(
    theme: &Theme,
    component_prefix: &str,
    state_prefix: Option<&str>,
) -> f32 {
    MaterialTokenResolver::new(theme)
        .number_comp_or_sys(
            &state_key(
                component_prefix,
                state_prefix,
                "pressed",
                "state-layer.opacity",
            ),
            "md.sys.state.pressed.state-layer-opacity",
            DEFAULT_PRESSED_STATE_LAYER_OPACITY,
        )
        .clamp(0.0, 1.0)
}

pub(crate) fn elevated_container_elevation(
    theme: &Theme,
    component_prefix: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Px {
    if !enabled {
        return chip_metric(
            theme,
            format!("{component_prefix}.elevated.disabled.container.elevation"),
            DEFAULT_ELEVATION,
        );
    }

    let key = match interaction {
        Some(PressableInteraction::Pressed) => "elevated.pressed.container.elevation",
        Some(PressableInteraction::Focused) => "elevated.focus.container.elevation",
        Some(PressableInteraction::Hovered) => "elevated.hover.container.elevation",
        None => "elevated.container.elevation",
    };

    chip_metric(
        theme,
        format!("{component_prefix}.{key}"),
        DEFAULT_ELEVATION,
    )
}

pub(crate) fn elevated_container_shadow_color(theme: &Theme, component_prefix: &str) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{component_prefix}.elevated.container.shadow-color"),
        "md.sys.color.shadow",
    )
}

pub(crate) fn outline(
    theme: &Theme,
    component_prefix: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    keys: ChipOutlineKeys,
) -> ChipOutline {
    let width = chip_metric(
        theme,
        format!("{component_prefix}.{}", keys.width),
        DEFAULT_OUTLINE_WIDTH,
    );

    if !enabled {
        return ChipOutline {
            width,
            color: disabled_on_surface_color(
                theme,
                &format!("{component_prefix}.{}", keys.disabled_color),
                &format!("{component_prefix}.{}", keys.disabled_opacity),
                DEFAULT_DISABLED_OUTLINE_OPACITY,
            ),
        };
    }

    let color_suffix = match interaction {
        Some(PressableInteraction::Focused) => keys.focus_color,
        None | Some(_) => keys.color,
    };

    let mut color = MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{component_prefix}.{color_suffix}"),
        "md.sys.color.outline-variant",
    );
    color.a = 1.0;

    ChipOutline { width, color }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ChipOutlineKeys {
    pub width: &'static str,
    pub disabled_color: &'static str,
    pub disabled_opacity: &'static str,
    pub focus_color: &'static str,
    pub color: &'static str,
}

fn interaction_suffix(interaction: Option<PressableInteraction>) -> Option<&'static str> {
    match interaction {
        Some(PressableInteraction::Pressed) => Some("pressed"),
        Some(PressableInteraction::Focused) => Some("focus"),
        Some(PressableInteraction::Hovered) => Some("hover"),
        None => None,
    }
}

fn state_key(
    component_prefix: &str,
    state_prefix: Option<&str>,
    interaction_suffix: &str,
    role_suffix: &str,
) -> String {
    match state_prefix {
        Some(state_prefix) => {
            format!("{component_prefix}.{state_prefix}.{interaction_suffix}.{role_suffix}")
        }
        None => format!("{component_prefix}.{interaction_suffix}.{role_suffix}"),
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
    fn shared_chip_metrics_prefer_component_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.assist-chip.container.height".to_string(), 40.0);
        patch
            .metrics
            .insert("md.comp.assist-chip.with-icon.icon.size".to_string(), 20.0);
        patch.metrics.insert(
            "md.comp.assist-chip.elevated.hover.container.elevation".to_string(),
            6.0,
        );
        patch
            .metrics
            .insert("md.comp.assist-chip.outline.width".to_string(), 2.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(container_height(&theme, "md.comp.assist-chip"), Px(40.0));
        assert_eq!(
            icon_size(&theme, "md.comp.assist-chip.with-icon.icon.size"),
            Px(20.0)
        );
        assert_eq!(
            elevated_container_elevation(
                &theme,
                "md.comp.assist-chip",
                true,
                Some(PressableInteraction::Hovered),
            ),
            Px(6.0)
        );
        assert_eq!(
            outline(
                &theme,
                "md.comp.assist-chip",
                true,
                None,
                ChipOutlineKeys {
                    width: "outline.width",
                    disabled_color: "disabled.outline.color",
                    disabled_opacity: "disabled.outline.opacity",
                    focus_color: "focus.outline.color",
                    color: "outline.color",
                },
            )
            .width,
            Px(2.0)
        );
    }

    #[test]
    fn shared_chip_shape_falls_back_to_sys_small_shape() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.shape.corner.small".to_string(), 12.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme, "md.comp.test-chip"),
            Corners::all(Px(12.0))
        );
    }

    #[test]
    fn shared_chip_shape_prefers_structured_corners_over_uniform_metric() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.test-chip.container.shape".to_string(), 8.0);
        patch.corners.insert(
            "md.comp.test-chip.container.shape".to_string(),
            Corners {
                top_left: Px(2.0),
                top_right: Px(3.0),
                bottom_right: Px(4.0),
                bottom_left: Px(5.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme, "md.comp.test-chip"),
            Corners {
                top_left: Px(2.0),
                top_right: Px(3.0),
                bottom_right: Px(4.0),
                bottom_left: Px(5.0),
            }
        );
    }

    #[test]
    fn shared_chip_state_layer_opacity_preserves_optional_state_fallback() {
        let mut patch = ThemeConfig::default();
        patch.numbers.insert(
            "md.comp.filter-chip.selected.hover.state-layer.opacity".to_string(),
            0.27,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            state_layer_opacity(
                &theme,
                "md.comp.filter-chip",
                Some("selected"),
                Some(PressableInteraction::Hovered),
            ),
            0.27
        );
        assert_eq!(
            state_layer_opacity(&theme, "md.comp.filter-chip", Some("selected"), None,),
            0.0
        );
    }
}
