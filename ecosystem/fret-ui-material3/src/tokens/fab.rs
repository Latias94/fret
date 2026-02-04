//! Typed token access for Material 3 floating action buttons (FABs).
//!
//! This module centralizes key mapping and fallback chains so FAB outcomes remain stable and
//! drift-resistant while the component surface evolves.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::fab::{FabSize, FabVariant};
use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) const DISABLED_CONTAINER_OPACITY: f32 = 0.12;
pub(crate) const DISABLED_CONTENT_OPACITY: f32 = 0.38;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FabInteraction {
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn container_size(theme: &Theme, size: FabSize) -> Px {
    let key = format!("{}.container.height", size_prefix(size));
    theme.metric_by_key(&key).unwrap_or_else(|| match size {
        FabSize::Small => Px(40.0),
        FabSize::Medium => Px(56.0),
        FabSize::Large => Px(96.0),
    })
}

pub(crate) fn icon_size(theme: &Theme, size: FabSize) -> Px {
    let key = format!("{}.icon.size", size_prefix(size));
    theme.metric_by_key(&key).unwrap_or_else(|| match size {
        FabSize::Small => Px(24.0),
        FabSize::Medium => Px(24.0),
        FabSize::Large => Px(36.0),
    })
}

pub(crate) fn container_shape(theme: &Theme, size: FabSize) -> Corners {
    let key = format!("{}.container.shape", size_prefix(size));
    let radius = theme
        .metric_by_key(&key)
        .or_else(|| match size {
            FabSize::Small => theme.metric_by_key("md.sys.shape.corner.medium"),
            FabSize::Medium => theme.metric_by_key("md.sys.shape.corner.large"),
            FabSize::Large => theme.metric_by_key("md.sys.shape.corner.extra-large"),
        })
        .unwrap_or_else(|| match size {
            FabSize::Small => Px(12.0),
            FabSize::Medium => Px(16.0),
            FabSize::Large => Px(28.0),
        });

    Corners::all(radius)
}

pub(crate) fn extended_container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.extended-fab.container.height")
        .unwrap_or(Px(56.0))
}

pub(crate) fn extended_icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.extended-fab.icon.size")
        .unwrap_or(Px(24.0))
}

pub(crate) fn extended_container_shape(theme: &Theme) -> Corners {
    let radius = theme
        .metric_by_key("md.comp.extended-fab.container.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.large"))
        .unwrap_or(Px(16.0));

    Corners::all(radius)
}

pub(crate) fn extended_leading_space(theme: &Theme, has_icon: bool) -> Px {
    let leading = theme
        .metric_by_key("md.comp.extended-fab.leading-space")
        .unwrap_or(Px(16.0));
    let trailing = extended_trailing_space(theme);
    if has_icon { leading } else { trailing }
}

pub(crate) fn extended_trailing_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.extended-fab.trailing-space")
        .unwrap_or(Px(20.0))
}

pub(crate) fn extended_icon_label_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.extended-fab.icon-label-space")
        .unwrap_or(Px(12.0))
}

pub(crate) fn container_background(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
    enabled: bool,
    lowered: bool,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let prefix = if extended {
        extended_variant_prefix(variant)
    } else {
        variant_prefix(variant)
    };

    if !enabled {
        let mut c = tokens.color_sys("md.sys.color.on-surface");
        c.a *= DISABLED_CONTAINER_OPACITY;
        return c;
    }

    theme
        .color_by_key(&format!("{prefix}.lowered.container.color"))
        .filter(|_| lowered)
        .or_else(|| theme.color_by_key(&format!("{prefix}.container.color")))
        .or_else(|| theme.color_by_key("md.sys.color.surface-container-high"))
        .or_else(|| theme.color_by_key("md.sys.color.surface-container"))
        .unwrap_or_else(|| tokens.color_sys("md.sys.color.surface-container"))
}

pub(crate) fn container_elevation(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
    enabled: bool,
    lowered: bool,
    interaction: Option<FabInteraction>,
) -> Px {
    if !enabled {
        return Px(0.0);
    }

    let prefix = if extended {
        extended_variant_prefix(variant)
    } else {
        variant_prefix(variant)
    };

    let bases: [String; 2] = if lowered {
        [format!("{prefix}.lowered"), prefix.to_string()]
    } else {
        [prefix.to_string(), String::new()]
    };

    for base in bases {
        if base.is_empty() {
            continue;
        }

        let keys = match interaction {
            Some(FabInteraction::Hovered) => [
                format!("{base}.hovered.container.elevation"),
                format!("{base}.hover.container.elevation"),
            ],
            Some(FabInteraction::Focused) => [
                format!("{base}.focused.container.elevation"),
                format!("{base}.focus.container.elevation"),
            ],
            Some(FabInteraction::Pressed) => {
                [format!("{base}.pressed.container.elevation"), String::new()]
            }
            None => [format!("{base}.container.elevation"), String::new()],
        };

        if let Some(v) = theme.metric_by_key(&keys[0]).or_else(|| {
            (!keys[1].is_empty())
                .then(|| theme.metric_by_key(&keys[1]))
                .flatten()
        }) {
            return v;
        }
    }

    Px(0.0)
}

pub(crate) fn container_shadow_color(theme: &Theme, extended: bool, variant: FabVariant) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let prefix = if extended {
        extended_variant_prefix(variant)
    } else {
        variant_prefix(variant)
    };
    theme
        .color_by_key(&format!("{prefix}.container.shadow-color"))
        .or_else(|| theme.color_by_key("md.sys.color.shadow"))
        .unwrap_or_else(|| tokens.color_sys("md.sys.color.shadow"))
}

pub(crate) fn icon_color(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
    enabled: bool,
    interaction: Option<FabInteraction>,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let prefix = if extended {
        extended_variant_prefix(variant)
    } else {
        variant_prefix(variant)
    };

    let default = theme.color_by_key(&format!("{prefix}.icon.color"));
    let mut color = match interaction {
        Some(FabInteraction::Hovered) => theme
            .color_by_key(&format!("{prefix}.hovered.icon.color"))
            .or_else(|| theme.color_by_key(&format!("{prefix}.hover.icon.color"))),
        Some(FabInteraction::Focused) => theme
            .color_by_key(&format!("{prefix}.focused.icon.color"))
            .or_else(|| theme.color_by_key(&format!("{prefix}.focus.icon.color"))),
        Some(FabInteraction::Pressed) => {
            theme.color_by_key(&format!("{prefix}.pressed.icon.color"))
        }
        None => default,
    }
    .or(default)
    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
    .unwrap_or_else(|| tokens.color_sys("md.sys.color.on-surface"));

    if !enabled {
        color.a *= DISABLED_CONTENT_OPACITY;
    }

    color
}

pub(crate) fn label_color(
    theme: &Theme,
    variant: FabVariant,
    enabled: bool,
    interaction: Option<FabInteraction>,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let prefix = extended_variant_prefix(variant);
    let default = theme.color_by_key(&format!("{prefix}.label-text.color"));

    let mut color = match interaction {
        Some(FabInteraction::Hovered) => theme
            .color_by_key(&format!("{prefix}.hovered.label-text.color"))
            .or_else(|| theme.color_by_key(&format!("{prefix}.hover.label-text.color"))),
        Some(FabInteraction::Focused) => theme
            .color_by_key(&format!("{prefix}.focused.label-text.color"))
            .or_else(|| theme.color_by_key(&format!("{prefix}.focus.label-text.color"))),
        Some(FabInteraction::Pressed) => {
            theme.color_by_key(&format!("{prefix}.pressed.label-text.color"))
        }
        None => default,
    }
    .or(default)
    .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
    .unwrap_or_else(|| tokens.color_sys("md.sys.color.on-surface"));

    if !enabled {
        color.a *= DISABLED_CONTENT_OPACITY;
    }

    color
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
    interaction: FabInteraction,
) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let prefix = if extended {
        extended_variant_prefix(variant)
    } else {
        variant_prefix(variant)
    };

    let mut color = theme
        .color_by_key(&format!("{prefix}.pressed.state-layer.color"))
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| tokens.color_sys("md.sys.color.on-surface"));

    let keys = match interaction {
        FabInteraction::Hovered => [
            format!("{prefix}.hovered.state-layer.color"),
            format!("{prefix}.hover.state-layer.color"),
        ],
        FabInteraction::Focused => [
            format!("{prefix}.focused.state-layer.color"),
            format!("{prefix}.focus.state-layer.color"),
        ],
        FabInteraction::Pressed => [format!("{prefix}.pressed.state-layer.color"), String::new()],
    };

    if let Some(found) = theme.color_by_key(&keys[0]).or_else(|| {
        (!keys[1].is_empty())
            .then(|| theme.color_by_key(&keys[1]))
            .flatten()
    }) {
        color = found;
    }

    color
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
    interaction: FabInteraction,
) -> f32 {
    let prefix = if extended {
        extended_variant_prefix(variant)
    } else {
        variant_prefix(variant)
    };

    let (key_a, key_b, sys_key, fallback) = match interaction {
        FabInteraction::Hovered => (
            format!("{prefix}.hovered.state-layer.opacity"),
            format!("{prefix}.hover.state-layer.opacity"),
            "md.sys.state.hover.state-layer-opacity",
            0.08,
        ),
        FabInteraction::Focused => (
            format!("{prefix}.focused.state-layer.opacity"),
            format!("{prefix}.focus.state-layer.opacity"),
            "md.sys.state.focus.state-layer-opacity",
            0.1,
        ),
        FabInteraction::Pressed => (
            format!("{prefix}.pressed.state-layer.opacity"),
            String::new(),
            "md.sys.state.pressed.state-layer-opacity",
            0.1,
        ),
    };

    theme
        .number_by_key(&key_a)
        .or_else(|| {
            (!key_b.is_empty())
                .then(|| theme.number_by_key(&key_b))
                .flatten()
        })
        .or_else(|| theme.number_by_key(sys_key))
        .unwrap_or(fallback)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    theme
        .number_by_key("md.sys.state.pressed.state-layer-opacity")
        .unwrap_or(0.1)
}

pub(crate) fn pressed_state_layer_opacity_for_variant(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
) -> f32 {
    let prefix = if extended {
        extended_variant_prefix(variant)
    } else {
        variant_prefix(variant)
    };

    theme
        .number_by_key(&format!("{prefix}.pressed.state-layer.opacity"))
        .unwrap_or_else(|| pressed_state_layer_opacity(theme))
}

fn size_prefix(size: FabSize) -> &'static str {
    match size {
        FabSize::Small => "md.comp.fab.small",
        FabSize::Medium => "md.comp.fab",
        FabSize::Large => "md.comp.fab.large",
    }
}

fn variant_prefix(variant: FabVariant) -> &'static str {
    match variant {
        FabVariant::Surface => "md.comp.fab.surface",
        FabVariant::Primary => "md.comp.fab.primary-container",
        FabVariant::Secondary => "md.comp.fab.secondary-container",
        FabVariant::Tertiary => "md.comp.fab.tertiary-container",
    }
}

fn extended_variant_prefix(variant: FabVariant) -> &'static str {
    match variant {
        FabVariant::Surface => "md.comp.extended-fab.surface",
        FabVariant::Primary => "md.comp.extended-fab.primary-container",
        FabVariant::Secondary => "md.comp.extended-fab.secondary-container",
        FabVariant::Tertiary => "md.comp.extended-fab.tertiary-container",
    }
}
