//! Typed token access for Material 3 floating action buttons (FABs).
//!
//! This module centralizes key mapping and fallback chains so FAB outcomes remain stable and
//! drift-resistant while the component surface evolves.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::fab::{FabSize, FabVariant};
use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul};
use crate::tokens::typography;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FabInteraction {
    Hovered,
    Focused,
    Pressed,
}

mod defaults {
    use fret_core::Px;

    use crate::fab::FabSize;

    pub(super) fn disabled_container_opacity() -> f32 {
        0.12
    }

    pub(super) fn disabled_content_opacity() -> f32 {
        0.38
    }

    pub(super) fn disabled_container_elevation() -> Px {
        Px(0.0)
    }

    pub(super) fn icon_container_size(size: FabSize) -> Px {
        match size {
            FabSize::Small => Px(40.0),
            FabSize::Regular => Px(56.0),
            FabSize::Medium => Px(80.0),
            FabSize::Large => Px(96.0),
        }
    }

    pub(super) fn icon_size(size: FabSize) -> Px {
        match size {
            FabSize::Small | FabSize::Regular => Px(24.0),
            FabSize::Medium => Px(28.0),
            FabSize::Large => Px(36.0),
        }
    }

    pub(super) fn icon_container_shape_system_key(size: FabSize) -> &'static str {
        match size {
            FabSize::Small => "md.sys.shape.corner.medium",
            FabSize::Regular => "md.sys.shape.corner.large",
            FabSize::Medium => "md.sys.shape.corner.large-increased",
            FabSize::Large => "md.sys.shape.corner.extra-large",
        }
    }

    pub(super) fn icon_container_shape_radius(size: FabSize) -> Px {
        match size {
            FabSize::Small => Px(12.0),
            FabSize::Regular => Px(16.0),
            FabSize::Medium => Px(20.0),
            FabSize::Large => Px(28.0),
        }
    }

    pub(super) fn extended_container_height(size: FabSize) -> Px {
        match size {
            FabSize::Small | FabSize::Regular => Px(56.0),
            FabSize::Medium => Px(80.0),
            FabSize::Large => Px(96.0),
        }
    }

    pub(super) fn extended_min_width(size: FabSize, resolved_height: Px) -> Px {
        match size {
            FabSize::Regular => Px(80.0),
            FabSize::Small | FabSize::Medium | FabSize::Large => resolved_height,
        }
    }

    pub(super) fn extended_icon_size(size: FabSize) -> Px {
        icon_size(size)
    }

    pub(super) fn extended_container_shape_system_key(size: FabSize) -> &'static str {
        match size {
            FabSize::Small | FabSize::Regular => "md.sys.shape.corner.large",
            FabSize::Medium => "md.sys.shape.corner.large-increased",
            FabSize::Large => "md.sys.shape.corner.extra-large",
        }
    }

    pub(super) fn extended_container_shape_radius(size: FabSize) -> Px {
        match size {
            FabSize::Small | FabSize::Regular => Px(16.0),
            FabSize::Medium => Px(20.0),
            FabSize::Large => Px(28.0),
        }
    }

    pub(super) fn extended_leading_space(size: FabSize) -> Px {
        match size {
            FabSize::Small | FabSize::Regular => Px(16.0),
            FabSize::Medium => Px(26.0),
            FabSize::Large => Px(28.0),
        }
    }

    pub(super) fn extended_trailing_space(size: FabSize) -> Px {
        match size {
            FabSize::Regular => Px(20.0),
            FabSize::Small => Px(16.0),
            FabSize::Medium => Px(26.0),
            FabSize::Large => Px(28.0),
        }
    }

    pub(super) fn extended_icon_label_space(size: FabSize) -> Px {
        match size {
            FabSize::Small => Px(8.0),
            FabSize::Regular | FabSize::Medium => Px(12.0),
            FabSize::Large => Px(16.0),
        }
    }

    pub(super) fn extended_label_text_source(size: FabSize) -> &'static str {
        match size {
            FabSize::Small => "md.sys.typescale.title-medium",
            FabSize::Regular => "md.comp.extended-fab.label-text",
            FabSize::Medium => "md.sys.typescale.title-large",
            FabSize::Large => "md.sys.typescale.headline-small",
        }
    }

    pub(super) fn hovered_state_layer_opacity() -> f32 {
        0.08
    }

    pub(super) fn focused_state_layer_opacity() -> f32 {
        0.1
    }

    pub(super) fn pressed_state_layer_opacity() -> f32 {
        0.1
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn icon_fab_size_defaults_match_material_matrix() {
            assert_eq!(icon_container_size(FabSize::Small), Px(40.0));
            assert_eq!(icon_container_size(FabSize::Regular), Px(56.0));
            assert_eq!(icon_container_size(FabSize::Medium), Px(80.0));
            assert_eq!(icon_container_size(FabSize::Large), Px(96.0));

            assert_eq!(icon_size(FabSize::Small), Px(24.0));
            assert_eq!(icon_size(FabSize::Regular), Px(24.0));
            assert_eq!(icon_size(FabSize::Medium), Px(28.0));
            assert_eq!(icon_size(FabSize::Large), Px(36.0));
        }

        #[test]
        fn shape_defaults_keep_size_specific_system_fallbacks() {
            assert_eq!(
                icon_container_shape_system_key(FabSize::Small),
                "md.sys.shape.corner.medium"
            );
            assert_eq!(icon_container_shape_radius(FabSize::Small), Px(12.0));
            assert_eq!(
                extended_container_shape_system_key(FabSize::Medium),
                "md.sys.shape.corner.large-increased"
            );
            assert_eq!(extended_container_shape_radius(FabSize::Large), Px(28.0));
        }

        #[test]
        fn extended_fab_spacing_defaults_match_material_matrix() {
            assert_eq!(extended_container_height(FabSize::Regular), Px(56.0));
            assert_eq!(extended_min_width(FabSize::Regular, Px(72.0)), Px(80.0));
            assert_eq!(extended_min_width(FabSize::Small, Px(56.0)), Px(56.0));
            assert_eq!(extended_leading_space(FabSize::Medium), Px(26.0));
            assert_eq!(extended_trailing_space(FabSize::Regular), Px(20.0));
            assert_eq!(extended_icon_label_space(FabSize::Large), Px(16.0));
        }

        #[test]
        fn opacity_defaults_match_material_state_matrix() {
            assert_eq!(disabled_container_opacity(), 0.12);
            assert_eq!(disabled_content_opacity(), 0.38);
            assert_eq!(hovered_state_layer_opacity(), 0.08);
            assert_eq!(focused_state_layer_opacity(), 0.1);
            assert_eq!(pressed_state_layer_opacity(), 0.1);
        }
    }
}

fn fab_metric(theme: &Theme, key: impl AsRef<str>, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key.as_ref()), fallback)
}

pub(crate) fn container_size(theme: &Theme, size: FabSize) -> Px {
    fab_metric(
        theme,
        format!("{}.container.height", size_prefix(size)),
        defaults::icon_container_size(size),
    )
}

pub(crate) fn icon_size(theme: &Theme, size: FabSize) -> Px {
    fab_metric(
        theme,
        format!("{}.icon.size", size_prefix(size)),
        defaults::icon_size(size),
    )
}

pub(crate) fn container_shape(theme: &Theme, size: FabSize) -> Corners {
    let key = format!("{}.container.shape", size_prefix(size));
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[
            key.as_str(),
            defaults::icon_container_shape_system_key(size),
        ],
        Corners::all(defaults::icon_container_shape_radius(size)),
    )
}

pub(crate) fn extended_container_height(theme: &Theme, size: FabSize) -> Px {
    fab_metric(
        theme,
        format!("{}.container.height", extended_size_prefix(size)),
        defaults::extended_container_height(size),
    )
}

pub(crate) fn extended_min_width(theme: &Theme, size: FabSize) -> Px {
    fab_metric(
        theme,
        format!("{}.container.width", extended_size_prefix(size)),
        defaults::extended_min_width(size, extended_container_height(theme, size)),
    )
}

pub(crate) fn extended_icon_size(theme: &Theme, size: FabSize) -> Px {
    fab_metric(
        theme,
        format!("{}.icon.size", extended_size_prefix(size)),
        defaults::extended_icon_size(size),
    )
}

pub(crate) fn extended_container_shape(theme: &Theme, size: FabSize) -> Corners {
    let key = format!("{}.container.shape", extended_size_prefix(size));
    MaterialTokenResolver::new(theme).corners_chain_or(
        &[
            key.as_str(),
            defaults::extended_container_shape_system_key(size),
        ],
        Corners::all(defaults::extended_container_shape_radius(size)),
    )
}

pub(crate) fn extended_leading_space(theme: &Theme, size: FabSize, has_icon: bool) -> Px {
    let leading = fab_metric(
        theme,
        format!("{}.leading-space", extended_size_prefix(size)),
        defaults::extended_leading_space(size),
    );
    let trailing = extended_trailing_space(theme, size);
    if has_icon { leading } else { trailing }
}

pub(crate) fn extended_trailing_space(theme: &Theme, size: FabSize) -> Px {
    fab_metric(
        theme,
        format!("{}.trailing-space", extended_size_prefix(size)),
        defaults::extended_trailing_space(size),
    )
}

pub(crate) fn extended_icon_label_space(theme: &Theme, size: FabSize) -> Px {
    fab_metric(
        theme,
        format!("{}.icon-label-space", extended_size_prefix(size)),
        defaults::extended_icon_label_space(size),
    )
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
        c.a *= defaults::disabled_container_opacity();
        return c;
    }

    let mut comp_keys = Vec::new();
    if lowered {
        comp_keys.push(format!("{prefix}.lowered.container.color"));
    }
    comp_keys.push(format!("{prefix}.container.color"));
    let comp_refs = comp_keys.iter().map(String::as_str).collect::<Vec<_>>();

    tokens.color_comp_chain_or_sys_chain(
        &comp_refs,
        &[
            "md.sys.color.surface-container-high",
            "md.sys.color.surface-container",
        ],
    )
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
        return defaults::disabled_container_elevation();
    }

    let prefix = if extended {
        extended_variant_prefix(variant)
    } else {
        variant_prefix(variant)
    };

    let mut bases = Vec::new();
    if lowered {
        let lowered_prefix = lowered_variant_prefix(extended, variant);
        bases.push(format!("{lowered_prefix}.lowered"));
        if lowered_prefix != prefix {
            bases.push(format!("{prefix}.lowered"));
        }
    }
    bases.push(prefix.to_string());

    for base in bases {
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

        let tokens = MaterialTokenResolver::new(theme);
        for key in keys.iter().filter(|key| !key.is_empty()) {
            if let Some(value) = tokens.metric_value(key) {
                return value;
            }
        }
    }

    defaults::disabled_container_elevation()
}

pub(crate) fn container_shadow_color(theme: &Theme, extended: bool, variant: FabVariant) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let prefix = if extended {
        extended_variant_prefix(variant)
    } else {
        variant_prefix(variant)
    };
    tokens.color_comp_or_sys(
        &format!("{prefix}.container.shadow-color"),
        "md.sys.color.shadow",
    )
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

    let mut comp_keys = interaction_token_keys(prefix, interaction, "icon.color");
    comp_keys.push(format!("{prefix}.icon.color"));
    let comp_refs = comp_keys.iter().map(String::as_str).collect::<Vec<_>>();
    let mut color = tokens.color_comp_chain_or_sys(&comp_refs, "md.sys.color.on-surface");

    if !enabled {
        color = alpha_mul(color, defaults::disabled_content_opacity());
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
    let mut comp_keys = interaction_token_keys(prefix, interaction, "label-text.color");
    comp_keys.push(format!("{prefix}.label-text.color"));
    let comp_refs = comp_keys.iter().map(String::as_str).collect::<Vec<_>>();
    let mut color = tokens.color_comp_chain_or_sys(&comp_refs, "md.sys.color.on-surface");

    if !enabled {
        color = alpha_mul(color, defaults::disabled_content_opacity());
    }

    color
}

pub(crate) fn extended_label_text_style(
    theme: &Theme,
    size: FabSize,
    variant: FabVariant,
) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some(defaults::extended_label_text_source(size)),
        "md.sys.typescale.label-large",
        Some(extended_label_text_weight_key(variant)),
        TextIntent::Control,
    )
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

    let color = tokens.color_comp_or_sys(
        &format!("{prefix}.pressed.state-layer.color"),
        "md.sys.color.on-surface",
    );

    let comp_keys = interaction_token_keys(prefix, Some(interaction), "state-layer.color");
    let comp_refs = comp_keys.iter().map(String::as_str).collect::<Vec<_>>();
    tokens.color_comp_chain_or_fallback(&comp_refs, color)
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
            defaults::hovered_state_layer_opacity(),
        ),
        FabInteraction::Focused => (
            format!("{prefix}.focused.state-layer.opacity"),
            format!("{prefix}.focus.state-layer.opacity"),
            "md.sys.state.focus.state-layer-opacity",
            defaults::focused_state_layer_opacity(),
        ),
        FabInteraction::Pressed => (
            format!("{prefix}.pressed.state-layer.opacity"),
            String::new(),
            "md.sys.state.pressed.state-layer-opacity",
            defaults::pressed_state_layer_opacity(),
        ),
    };

    let comp_keys = if key_b.is_empty() {
        vec![key_a]
    } else {
        vec![key_a, key_b]
    };
    let comp_refs = comp_keys.iter().map(String::as_str).collect::<Vec<_>>();

    MaterialTokenResolver::new(theme).number_comp_chain_or_sys(&comp_refs, sys_key, fallback)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).number_sys(
        "md.sys.state.pressed.state-layer-opacity",
        defaults::pressed_state_layer_opacity(),
    )
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

    MaterialTokenResolver::new(theme).number_optional(
        Some(&format!("{prefix}.pressed.state-layer.opacity")),
        pressed_state_layer_opacity(theme),
    )
}

fn interaction_token_keys(
    prefix: &str,
    interaction: Option<FabInteraction>,
    suffix: &str,
) -> Vec<String> {
    match interaction {
        Some(FabInteraction::Hovered) => vec![
            format!("{prefix}.hovered.{suffix}"),
            format!("{prefix}.hover.{suffix}"),
        ],
        Some(FabInteraction::Focused) => vec![
            format!("{prefix}.focused.{suffix}"),
            format!("{prefix}.focus.{suffix}"),
        ],
        Some(FabInteraction::Pressed) => vec![format!("{prefix}.pressed.{suffix}")],
        None => Vec::new(),
    }
}

fn size_prefix(size: FabSize) -> &'static str {
    match size {
        FabSize::Small => "md.comp.fab.small",
        FabSize::Regular => "md.comp.fab",
        FabSize::Medium => "md.comp.fab.medium",
        FabSize::Large => "md.comp.fab.large",
    }
}

fn extended_size_prefix(size: FabSize) -> &'static str {
    match size {
        FabSize::Small => "md.comp.extended-fab.small",
        FabSize::Regular => "md.comp.extended-fab",
        FabSize::Medium => "md.comp.extended-fab.medium",
        FabSize::Large => "md.comp.extended-fab.large",
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

fn lowered_variant_prefix(extended: bool, variant: FabVariant) -> &'static str {
    match (extended, variant) {
        (false, FabVariant::Surface) => "md.comp.fab.surface",
        (false, FabVariant::Primary) => "md.comp.fab.primary",
        (false, FabVariant::Secondary) => "md.comp.fab.secondary",
        (false, FabVariant::Tertiary) => "md.comp.fab.tertiary",
        (true, FabVariant::Surface) => "md.comp.extended-fab.surface",
        (true, FabVariant::Primary) => "md.comp.extended-fab.primary",
        (true, FabVariant::Secondary) => "md.comp.extended-fab.secondary",
        (true, FabVariant::Tertiary) => "md.comp.extended-fab.tertiary",
    }
}

fn extended_label_text_weight_key(variant: FabVariant) -> &'static str {
    match variant {
        FabVariant::Surface => "md.comp.extended-fab.surface.label-text.weight",
        FabVariant::Primary => "md.comp.extended-fab.primary.label-text.weight",
        FabVariant::Secondary => "md.comp.extended-fab.secondary.label-text.weight",
        FabVariant::Tertiary => "md.comp.extended-fab.tertiary.label-text.weight",
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
    fn fab_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(container_size(theme, FabSize::Regular), Px(56.0));
        assert_eq!(icon_size(theme, FabSize::Large), Px(36.0));
        assert_eq!(
            container_shape(theme, FabSize::Small),
            Corners::all(Px(12.0))
        );
        assert_eq!(extended_container_height(theme, FabSize::Medium), Px(80.0));
        assert_eq!(extended_min_width(theme, FabSize::Regular), Px(80.0));
        assert_eq!(extended_icon_size(theme, FabSize::Medium), Px(28.0));
        assert_eq!(
            extended_leading_space(theme, FabSize::Regular, true),
            Px(16.0)
        );
        assert_eq!(extended_trailing_space(theme, FabSize::Regular), Px(20.0));
        assert_eq!(extended_icon_label_space(theme, FabSize::Large), Px(16.0));
        assert_eq!(
            container_elevation(theme, false, FabVariant::Primary, true, false, None),
            Px(0.0)
        );
    }

    #[test]
    fn fab_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.fab.container.height".to_string(), 58.0);
        patch
            .metrics
            .insert("md.comp.fab.large.icon.size".to_string(), 38.0);
        patch
            .metrics
            .insert("md.comp.fab.small.container.shape".to_string(), 14.0);
        patch.metrics.insert(
            "md.comp.extended-fab.medium.container.height".to_string(),
            82.0,
        );
        patch
            .metrics
            .insert("md.comp.extended-fab.container.width".to_string(), 84.0);
        patch
            .metrics
            .insert("md.comp.extended-fab.medium.icon.size".to_string(), 30.0);
        patch
            .metrics
            .insert("md.comp.extended-fab.leading-space".to_string(), 18.0);
        patch
            .metrics
            .insert("md.comp.extended-fab.trailing-space".to_string(), 22.0);
        patch.metrics.insert(
            "md.comp.extended-fab.large.icon-label-space".to_string(),
            18.0,
        );
        patch.metrics.insert(
            "md.comp.fab.primary-container.hover.container.elevation".to_string(),
            5.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(container_size(&theme, FabSize::Regular), Px(58.0));
        assert_eq!(icon_size(&theme, FabSize::Large), Px(38.0));
        assert_eq!(
            container_shape(&theme, FabSize::Small),
            Corners::all(Px(14.0))
        );
        assert_eq!(extended_container_height(&theme, FabSize::Medium), Px(82.0));
        assert_eq!(extended_min_width(&theme, FabSize::Regular), Px(84.0));
        assert_eq!(extended_icon_size(&theme, FabSize::Medium), Px(30.0));
        assert_eq!(
            extended_leading_space(&theme, FabSize::Regular, true),
            Px(18.0)
        );
        assert_eq!(extended_trailing_space(&theme, FabSize::Regular), Px(22.0));
        assert_eq!(extended_icon_label_space(&theme, FabSize::Large), Px(18.0));
        assert_eq!(
            container_elevation(
                &theme,
                false,
                FabVariant::Primary,
                true,
                false,
                Some(FabInteraction::Hovered),
            ),
            Px(5.0)
        );
    }

    #[test]
    fn fab_shapes_use_system_fallbacks() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.shape.corner.medium".to_string(), 10.0);
        patch
            .metrics
            .insert("md.sys.shape.corner.large-increased".to_string(), 22.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme, FabSize::Small),
            Corners::all(Px(10.0))
        );
        assert_eq!(
            extended_container_shape(&theme, FabSize::Medium),
            Corners::all(Px(22.0))
        );
    }

    #[test]
    fn fab_shapes_prefer_structured_corners_over_uniform_metric() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.fab.small.container.shape".to_string(), 14.0);
        patch.corners.insert(
            "md.comp.fab.small.container.shape".to_string(),
            Corners {
                top_left: Px(2.0),
                top_right: Px(4.0),
                bottom_right: Px(6.0),
                bottom_left: Px(8.0),
            },
        );
        patch.metrics.insert(
            "md.comp.extended-fab.medium.container.shape".to_string(),
            22.0,
        );
        patch.corners.insert(
            "md.comp.extended-fab.medium.container.shape".to_string(),
            Corners {
                top_left: Px(10.0),
                top_right: Px(12.0),
                bottom_right: Px(14.0),
                bottom_left: Px(16.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme, FabSize::Small),
            Corners {
                top_left: Px(2.0),
                top_right: Px(4.0),
                bottom_right: Px(6.0),
                bottom_left: Px(8.0),
            }
        );
        assert_eq!(
            extended_container_shape(&theme, FabSize::Medium),
            Corners {
                top_left: Px(10.0),
                top_right: Px(12.0),
                bottom_right: Px(14.0),
                bottom_left: Px(16.0),
            }
        );
    }
}
