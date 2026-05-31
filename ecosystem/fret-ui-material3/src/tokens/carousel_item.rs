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

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    theme
        .metric_by_key(&format!("{COMPONENT_PREFIX}.container.shape"))
        .map(Corners::all)
        .or_else(|| {
            theme
                .metric_by_key("md.sys.shape.corner.extra-large")
                .map(Corners::all)
        })
        .unwrap_or_else(|| Corners::all(Px(28.0)))
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

    theme.metric_by_key(&key).unwrap_or(Px(0.0))
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

    let width = theme
        .metric_by_key(&format!("{WITH_OUTLINE_PREFIX}.outline.width"))
        .unwrap_or(Px(1.0));

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
