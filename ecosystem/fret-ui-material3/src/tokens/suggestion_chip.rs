//! Typed token access for Material 3 suggestion chips.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul};
use crate::tokens::typography;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.suggestion-chip";

#[derive(Debug, Clone, Copy)]
pub(crate) struct ChipOutline {
    pub width: Px,
    pub color: Color,
}

fn disabled_on_surface_color(
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

pub(crate) fn container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.suggestion-chip.container.height")
        .unwrap_or(Px(32.0))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    theme
        .metric_by_key("md.comp.suggestion-chip.container.shape")
        .map(Corners::all)
        .or_else(|| {
            theme
                .metric_by_key("md.sys.shape.corner.small")
                .map(Corners::all)
        })
        .unwrap_or_else(|| Corners::all(Px(8.0)))
}

pub(crate) fn leading_icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.suggestion-chip.with-leading-icon.leading-icon.size")
        .unwrap_or(Px(18.0))
}

pub(crate) fn elevated_container_background(theme: &Theme, enabled: bool) -> Color {
    if enabled {
        MaterialTokenResolver::new(theme).color_comp_or_sys(
            "md.comp.suggestion-chip.elevated.container.color",
            "md.sys.color.surface-container-low",
        )
    } else {
        disabled_on_surface_color(
            theme,
            "md.comp.suggestion-chip.elevated.disabled.container.color",
            "md.comp.suggestion-chip.elevated.disabled.container.opacity",
            0.12,
        )
    }
}

pub(crate) fn elevated_container_elevation(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Px {
    if !enabled {
        return theme
            .metric_by_key("md.comp.suggestion-chip.elevated.disabled.container.elevation")
            .unwrap_or(Px(0.0));
    }

    let key = match interaction {
        Some(PressableInteraction::Pressed) => "elevated.pressed.container.elevation",
        Some(PressableInteraction::Focused) => "elevated.focus.container.elevation",
        Some(PressableInteraction::Hovered) => "elevated.hover.container.elevation",
        None => "elevated.container.elevation",
    };

    theme
        .metric_by_key(&format!("{COMPONENT_PREFIX}.{key}"))
        .unwrap_or(Px(0.0))
}

pub(crate) fn elevated_container_shadow_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.suggestion-chip.elevated.container.shadow-color",
        "md.sys.color.shadow",
    )
}

pub(crate) fn label_color(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.suggestion-chip.disabled.label-text.color",
            "md.comp.suggestion-chip.disabled.label-text.opacity",
            0.38,
        );
    }

    let key = match interaction {
        Some(PressableInteraction::Pressed) => "pressed.label-text.color",
        Some(PressableInteraction::Focused) => "focus.label-text.color",
        Some(PressableInteraction::Hovered) => "hover.label-text.color",
        None => "label-text.color",
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{COMPONENT_PREFIX}.{key}"),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        None,
        "md.sys.typescale.label-large",
        Some("md.comp.suggestion-chip.label-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn state_layer_color(theme: &Theme, interaction: Option<PressableInteraction>) -> Color {
    let key = match interaction {
        Some(PressableInteraction::Pressed) => "pressed.state-layer.color",
        Some(PressableInteraction::Focused) => "focus.state-layer.color",
        Some(PressableInteraction::Hovered) => "hover.state-layer.color",
        None => return Color::TRANSPARENT,
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{COMPONENT_PREFIX}.{key}"),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn state_layer_opacity(theme: &Theme, interaction: Option<PressableInteraction>) -> f32 {
    let key = match interaction {
        Some(PressableInteraction::Pressed) => "pressed.state-layer.opacity",
        Some(PressableInteraction::Focused) => "focus.state-layer.opacity",
        Some(PressableInteraction::Hovered) => "hover.state-layer.opacity",
        None => return 0.0,
    };

    MaterialTokenResolver::new(theme)
        .number_optional(Some(&format!("{COMPONENT_PREFIX}.{key}")), 0.0)
        .clamp(0.0, 1.0)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme)
        .number_comp_or_sys(
            "md.comp.suggestion-chip.pressed.state-layer.opacity",
            "md.sys.state.pressed.state-layer-opacity",
            0.1,
        )
        .clamp(0.0, 1.0)
}

pub(crate) fn leading_icon_color(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.suggestion-chip.with-leading-icon.disabled.leading-icon.color",
            "md.comp.suggestion-chip.with-leading-icon.disabled.leading-icon.opacity",
            0.38,
        );
    }

    let key = match interaction {
        Some(PressableInteraction::Pressed) => "with-leading-icon.pressed.leading-icon.color",
        Some(PressableInteraction::Focused) => "with-leading-icon.focus.leading-icon.color",
        Some(PressableInteraction::Hovered) => "with-leading-icon.hover.leading-icon.color",
        None => "with-leading-icon.leading-icon.color",
    };

    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(&format!("{COMPONENT_PREFIX}.{key}"), "md.sys.color.primary")
}

pub(crate) fn flat_outline(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> ChipOutline {
    let width = theme
        .metric_by_key("md.comp.suggestion-chip.flat.outline.width")
        .unwrap_or(Px(1.0));

    if !enabled {
        return ChipOutline {
            width,
            color: disabled_on_surface_color(
                theme,
                "md.comp.suggestion-chip.flat.disabled.outline.color",
                "md.comp.suggestion-chip.flat.disabled.outline.opacity",
                0.12,
            ),
        };
    }

    let key = match interaction {
        Some(PressableInteraction::Focused) => "flat.focus.outline.color",
        None | Some(_) => "flat.outline.color",
    };

    let mut color = MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{COMPONENT_PREFIX}.{key}"),
        "md.sys.color.outline-variant",
    );
    color.a = 1.0;

    ChipOutline { width, color }
}
