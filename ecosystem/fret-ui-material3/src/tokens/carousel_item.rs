//! Typed token access for Material 3 carousel items.
//!
//! Reference: Material Web v30 `md.comp.carousel-item.*` tokens.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::{
    MaterialStateLayerInteraction, MaterialTokenResolver, alpha_mul,
};

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.carousel-item";
pub(crate) const WITH_OUTLINE_PREFIX: &str = "md.comp.carousel-item.with-outline";

#[derive(Debug, Clone, Copy)]
pub(crate) struct CarouselItemOutline {
    pub width: Px,
    pub color: Color,
}

fn carousel_metric(theme: &Theme, key: impl AsRef<str>, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key.as_ref()), fallback)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    let component_key = format!("{COMPONENT_PREFIX}.container.shape");
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[component_key.as_str(), "md.sys.shape.corner.extra-large"],
        Corners::all(Px(28.0)),
    )
}

pub(crate) fn container_shadow_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{COMPONENT_PREFIX}.container.shadow-color"),
        "md.sys.color.shadow",
    )
}

pub(crate) fn container_background(theme: &Theme, disabled: bool) -> Color {
    let key = if disabled {
        format!("{COMPONENT_PREFIX}.disabled.container.color")
    } else {
        format!("{COMPONENT_PREFIX}.container.color")
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(&key, "md.sys.color.surface")
}

pub(crate) fn disabled_opacity(theme: &Theme) -> f32 {
    let key = format!("{COMPONENT_PREFIX}.disabled.container.opacity");
    MaterialTokenResolver::new(theme)
        .number_optional(Some(key.as_str()), 0.38)
        .clamp(0.0, 1.0)
}

pub(crate) fn container_elevation(
    theme: &Theme,
    disabled: bool,
    interaction: Option<PressableInteraction>,
) -> Px {
    let key = if disabled {
        format!("{COMPONENT_PREFIX}.disabled.container.elevation")
    } else if let Some(interaction) = interaction {
        match interaction {
            PressableInteraction::Hovered => {
                format!("{COMPONENT_PREFIX}.hover.container.elevation")
            }
            PressableInteraction::Focused => {
                format!("{COMPONENT_PREFIX}.focus.container.elevation")
            }
            PressableInteraction::Pressed => {
                format!("{COMPONENT_PREFIX}.pressed.container.elevation")
            }
        }
    } else {
        format!("{COMPONENT_PREFIX}.container.elevation")
    };

    carousel_metric(theme, key, Px(0.0))
}

pub(crate) fn state_layer_color(theme: &Theme, interaction: Option<PressableInteraction>) -> Color {
    let key = match interaction {
        Some(PressableInteraction::Hovered) => {
            format!("{COMPONENT_PREFIX}.hover.state-layer.color")
        }
        Some(PressableInteraction::Focused) => {
            format!("{COMPONENT_PREFIX}.focus.state-layer.color")
        }
        Some(PressableInteraction::Pressed) => {
            format!("{COMPONENT_PREFIX}.pressed.state-layer.color")
        }
        None => return MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"),
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(&key, "md.sys.color.on-surface")
}

pub(crate) fn state_layer_opacity(theme: &Theme, interaction: Option<PressableInteraction>) -> f32 {
    let Some(interaction) = interaction else {
        return 0.0;
    };

    let key = match interaction {
        PressableInteraction::Hovered => format!("{COMPONENT_PREFIX}.hover.state-layer.opacity"),
        PressableInteraction::Focused => format!("{COMPONENT_PREFIX}.focus.state-layer.opacity"),
        PressableInteraction::Pressed => format!("{COMPONENT_PREFIX}.pressed.state-layer.opacity"),
    };

    MaterialTokenResolver::new(theme)
        .state_layer_opacity(&key, material_state_layer_interaction(interaction))
        .clamp(0.0, 1.0)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme)
        .state_layer_opacity(
            &format!("{COMPONENT_PREFIX}.pressed.state-layer.opacity"),
            MaterialStateLayerInteraction::Pressed,
        )
        .clamp(0.0, 1.0)
}

pub(crate) fn outline(
    theme: &Theme,
    with_outline: bool,
    disabled: bool,
    interaction: Option<PressableInteraction>,
) -> Option<CarouselItemOutline> {
    if !with_outline {
        return None;
    }

    let width = carousel_metric(
        theme,
        format!("{WITH_OUTLINE_PREFIX}.outline.width"),
        Px(1.0),
    );

    let (color_key, opacity_key) = if disabled {
        (
            format!("{WITH_OUTLINE_PREFIX}.disabled.outline.color"),
            Some(format!("{WITH_OUTLINE_PREFIX}.disabled.outline.opacity")),
        )
    } else {
        let key = match interaction {
            Some(PressableInteraction::Hovered) => {
                format!("{WITH_OUTLINE_PREFIX}.hover.outline.color")
            }
            Some(PressableInteraction::Focused) => {
                format!("{WITH_OUTLINE_PREFIX}.focus.outline.color")
            }
            Some(PressableInteraction::Pressed) => {
                format!("{WITH_OUTLINE_PREFIX}.pressed.outline.color")
            }
            None => format!("{WITH_OUTLINE_PREFIX}.outline.color"),
        };
        (key, None)
    };

    let tokens = MaterialTokenResolver::new(theme);
    let mut color = tokens.color_comp_or_sys(&color_key, "md.sys.color.outline");

    if let Some(opacity_key) = opacity_key.as_ref() {
        let opacity = tokens
            .number_optional(Some(opacity_key.as_str()), 0.12)
            .clamp(0.0, 1.0);
        color = alpha_mul(color, opacity);
    }

    Some(CarouselItemOutline { width, color })
}

fn material_state_layer_interaction(
    interaction: PressableInteraction,
) -> MaterialStateLayerInteraction {
    match interaction {
        PressableInteraction::Hovered => MaterialStateLayerInteraction::Hovered,
        PressableInteraction::Focused => MaterialStateLayerInteraction::Focused,
        PressableInteraction::Pressed => MaterialStateLayerInteraction::Pressed,
    }
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
    fn carousel_item_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(container_shape(theme), Corners::all(Px(28.0)));
        assert_eq!(container_elevation(theme, false, None), Px(0.0));
        assert_eq!(
            outline(theme, true, false, None)
                .expect("outlined carousel item has outline")
                .width,
            Px(1.0)
        );
    }

    #[test]
    fn carousel_item_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.carousel-item.container.shape".to_string(), 24.0);
        patch.metrics.insert(
            "md.comp.carousel-item.hover.container.elevation".to_string(),
            4.0,
        );
        patch.metrics.insert(
            "md.comp.carousel-item.with-outline.outline.width".to_string(),
            2.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(container_shape(&theme), Corners::all(Px(24.0)));
        assert_eq!(
            container_elevation(&theme, false, Some(PressableInteraction::Hovered)),
            Px(4.0)
        );
        assert_eq!(
            outline(&theme, true, false, None)
                .expect("outlined carousel item has outline")
                .width,
            Px(2.0)
        );
    }

    #[test]
    fn carousel_item_shape_uses_system_fallback() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.shape.corner.extra-large".to_string(), 26.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(container_shape(&theme), Corners::all(Px(26.0)));
    }

    #[test]
    fn carousel_item_shape_prefers_structured_corners_over_uniform_metric() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.carousel-item.container.shape".to_string(), 24.0);
        patch.corners.insert(
            "md.comp.carousel-item.container.shape".to_string(),
            Corners {
                top_left: Px(6.0),
                top_right: Px(8.0),
                bottom_right: Px(10.0),
                bottom_left: Px(12.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme),
            Corners {
                top_left: Px(6.0),
                top_right: Px(8.0),
                bottom_right: Px(10.0),
                bottom_left: Px(12.0),
            }
        );
    }
}
