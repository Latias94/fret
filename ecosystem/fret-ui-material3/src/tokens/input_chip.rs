//! Typed token access for Material 3 input chips.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul};
use crate::tokens::typography;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.input-chip";

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
        .metric_by_key("md.comp.input-chip.container.height")
        .unwrap_or(Px(32.0))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    theme
        .metric_by_key("md.comp.input-chip.container.shape")
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
        .metric_by_key("md.comp.input-chip.with-leading-icon.leading-icon.size")
        .unwrap_or(Px(18.0))
}

pub(crate) fn trailing_icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.input-chip.with-trailing-icon.trailing-icon.size")
        .unwrap_or(Px(18.0))
}

pub(crate) fn selected_container_background(theme: &Theme, enabled: bool) -> Color {
    if enabled {
        MaterialTokenResolver::new(theme).color_comp_or_sys(
            "md.comp.input-chip.selected.container.color",
            "md.sys.color.secondary-container",
        )
    } else {
        disabled_on_surface_color(
            theme,
            "md.comp.input-chip.disabled.selected.container.color",
            "md.comp.input-chip.disabled.selected.container.opacity",
            0.12,
        )
    }
}

pub(crate) fn unselected_outline(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> ChipOutline {
    let width = theme
        .metric_by_key("md.comp.input-chip.unselected.outline.width")
        .unwrap_or(Px(1.0));

    if !enabled {
        return ChipOutline {
            width,
            color: disabled_on_surface_color(
                theme,
                "md.comp.input-chip.disabled.unselected.outline.color",
                "md.comp.input-chip.disabled.unselected.outline.opacity",
                0.12,
            ),
        };
    }

    let key = match interaction {
        Some(PressableInteraction::Focused) => "unselected.focus.outline.color",
        None | Some(_) => "unselected.outline.color",
    };

    let mut color = MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{COMPONENT_PREFIX}.{key}"),
        "md.sys.color.outline-variant",
    );
    color.a = 1.0;

    ChipOutline { width, color }
}

pub(crate) fn label_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.input-chip.disabled.label-text.color",
            "md.comp.input-chip.disabled.label-text.opacity",
            0.38,
        );
    }

    let state = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Some(PressableInteraction::Pressed) => format!("{state}.pressed.label-text.color"),
        Some(PressableInteraction::Focused) => format!("{state}.focus.label-text.color"),
        Some(PressableInteraction::Hovered) => format!("{state}.hover.label-text.color"),
        None => format!("{state}.label-text.color"),
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
        Some("md.comp.input-chip.label-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    let state = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Some(PressableInteraction::Pressed) => format!("{state}.pressed.state-layer.color"),
        Some(PressableInteraction::Focused) => format!("{state}.focus.state-layer.color"),
        Some(PressableInteraction::Hovered) => format!("{state}.hover.state-layer.color"),
        None => return Color::TRANSPARENT,
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{COMPONENT_PREFIX}.{key}"),
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> f32 {
    let state = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Some(PressableInteraction::Pressed) => format!("{state}.pressed.state-layer.opacity"),
        Some(PressableInteraction::Focused) => format!("{state}.focus.state-layer.opacity"),
        Some(PressableInteraction::Hovered) => format!("{state}.hover.state-layer.opacity"),
        None => return 0.0,
    };

    MaterialTokenResolver::new(theme)
        .number_optional(Some(&format!("{COMPONENT_PREFIX}.{key}")), 0.0)
        .clamp(0.0, 1.0)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, selected: bool) -> f32 {
    let state = if selected { "selected" } else { "unselected" };
    MaterialTokenResolver::new(theme)
        .number_comp_or_sys(
            &format!("{COMPONENT_PREFIX}.{state}.pressed.state-layer.opacity"),
            "md.sys.state.pressed.state-layer-opacity",
            0.1,
        )
        .clamp(0.0, 1.0)
}

pub(crate) fn leading_icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.input-chip.with-leading-icon.disabled.leading-icon.color",
            "md.comp.input-chip.with-leading-icon.disabled.leading-icon.opacity",
            0.38,
        );
    }

    let state = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Some(PressableInteraction::Pressed) => {
            format!("with-leading-icon.{state}.pressed.leading-icon.color")
        }
        Some(PressableInteraction::Focused) => {
            format!("with-leading-icon.{state}.focus.leading-icon.color")
        }
        Some(PressableInteraction::Hovered) => {
            format!("with-leading-icon.{state}.hover.leading-icon.color")
        }
        None => format!("with-leading-icon.{state}.leading-icon.color"),
    };

    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(&format!("{COMPONENT_PREFIX}.{key}"), "md.sys.color.primary")
}

pub(crate) fn trailing_icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    if !enabled {
        return disabled_on_surface_color(
            theme,
            "md.comp.input-chip.with-trailing-icon.disabled.trailing-icon.color",
            "md.comp.input-chip.with-trailing-icon.disabled.trailing-icon.opacity",
            0.38,
        );
    }

    let state = if selected { "selected" } else { "unselected" };
    let key = match interaction {
        Some(PressableInteraction::Pressed) => {
            format!("with-trailing-icon.{state}.pressed.trailing-icon.color")
        }
        Some(PressableInteraction::Focused) => {
            format!("with-trailing-icon.{state}.focus.trailing-icon.color")
        }
        Some(PressableInteraction::Hovered) => {
            format!("with-trailing-icon.{state}.hover.trailing-icon.color")
        }
        None => format!("with-trailing-icon.{state}.trailing-icon.color"),
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{COMPONENT_PREFIX}.{key}"),
        "md.sys.color.on-surface-variant",
    )
}
