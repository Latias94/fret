//! Typed token access for Material 3 assist chips.
//!
//! Note: Material Web's v30 sassvars do not currently include the padding/spacing tokens used by
//! the chip recipe (`leading-space`, `trailing-space`, etc.). We keep those as component-level
//! layout constants in `chip.rs` instead of inventing new `md.*` keys.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul};
use crate::tokens::typography;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.assist-chip";

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
        .metric_by_key("md.comp.assist-chip.container.height")
        .unwrap_or(Px(32.0))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    theme
        .metric_by_key("md.comp.assist-chip.container.shape")
        .map(Corners::all)
        .or_else(|| {
            theme
                .metric_by_key("md.sys.shape.corner.small")
                .map(Corners::all)
        })
        .unwrap_or_else(|| Corners::all(Px(8.0)))
}

pub(crate) fn label_color(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.assist-chip.disabled.label-text.color",
            "md.comp.assist-chip.disabled.label-text.opacity",
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
        "md.sys.color.on-surface",
    )
}

pub(crate) fn label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        None,
        "md.sys.typescale.label-large",
        Some("md.comp.assist-chip.label-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn leading_icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.assist-chip.with-icon.icon.size")
        .unwrap_or(Px(18.0))
}

pub(crate) fn leading_icon_color(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.assist-chip.with-icon.disabled.icon.color",
            "md.comp.assist-chip.with-icon.disabled.icon.opacity",
            0.38,
        );
    }

    let key = match interaction {
        Some(PressableInteraction::Pressed) => "with-icon.pressed.icon.color",
        Some(PressableInteraction::Focused) => "with-icon.focus.icon.color",
        Some(PressableInteraction::Hovered) => "with-icon.hover.icon.color",
        None => "with-icon.icon.color",
    };

    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(&format!("{COMPONENT_PREFIX}.{key}"), "md.sys.color.primary")
}

pub(crate) fn state_layer_color(theme: &Theme, interaction: Option<PressableInteraction>) -> Color {
    let key = match interaction {
        Some(PressableInteraction::Pressed) => "pressed.state-layer.color",
        Some(PressableInteraction::Focused) => "focus.state-layer.color",
        Some(PressableInteraction::Hovered) => "hover.state-layer.color",
        None => "hover.state-layer.color",
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{COMPONENT_PREFIX}.{key}"),
        "md.sys.color.on-surface",
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
            "md.comp.assist-chip.pressed.state-layer.opacity",
            "md.sys.state.pressed.state-layer-opacity",
            0.1,
        )
        .clamp(0.0, 1.0)
}

pub(crate) fn elevated_container_background(theme: &Theme, enabled: bool) -> Color {
    if enabled {
        MaterialTokenResolver::new(theme).color_comp_or_sys(
            "md.comp.assist-chip.elevated.container.color",
            "md.sys.color.surface-container-low",
        )
    } else {
        disabled_on_surface_color(
            theme,
            "md.comp.assist-chip.elevated.disabled.container.color",
            "md.comp.assist-chip.elevated.disabled.container.opacity",
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
            .metric_by_key("md.comp.assist-chip.elevated.disabled.container.elevation")
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
        "md.comp.assist-chip.elevated.container.shadow-color",
        "md.sys.color.shadow",
    )
}

pub(crate) fn flat_outline(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Option<ChipOutline> {
    let width = theme
        .metric_by_key("md.comp.assist-chip.flat.outline.width")
        .unwrap_or(Px(1.0));

    if !enabled {
        return Some(ChipOutline {
            width,
            color: disabled_on_surface_color(
                theme,
                "md.comp.assist-chip.flat.disabled.outline.color",
                "md.comp.assist-chip.flat.disabled.outline.opacity",
                0.12,
            ),
        });
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

    Some(ChipOutline { width, color })
}
