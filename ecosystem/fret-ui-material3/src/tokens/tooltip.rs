//! Typed token access for Material 3 tooltips.
//!
//! This module centralizes token key mapping and fallback chains so tooltip outcomes remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Corners, Edges, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::typography;

fn tooltip_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

pub(crate) fn plain_container_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.plain-tooltip.container.color",
        "md.sys.color.inverse-surface",
    )
}

pub(crate) fn plain_supporting_text_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.plain-tooltip.supporting-text.color",
        "md.sys.color.inverse-on-surface",
    )
}

pub(crate) fn plain_supporting_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.plain-tooltip.supporting-text"),
        "md.sys.typescale.body-small",
        Some("md.comp.plain-tooltip.supporting-text.weight"),
        TextIntent::Content,
    )
}

pub(crate) fn plain_container_shape(theme: &Theme) -> Corners {
    MaterialTokenResolver::new(theme).corners_chain_or(
        &["md.comp.plain-tooltip.container.shape"],
        Corners::all(Px(4.0)),
    )
}

pub(crate) fn plain_container_padding(theme: &Theme) -> Edges {
    let _ = theme;
    Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(4.0),
        bottom: Px(4.0),
    }
}

pub(crate) fn plain_container_max_width(theme: &Theme) -> Px {
    let _ = theme;
    Px(200.0)
}

pub(crate) fn rich_container_max_width(theme: &Theme) -> Px {
    let _ = theme;
    Px(320.0)
}

pub(crate) fn container_min_width(theme: &Theme) -> Px {
    let _ = theme;
    Px(40.0)
}

pub(crate) fn container_min_height(theme: &Theme) -> Px {
    let _ = theme;
    Px(24.0)
}

pub(crate) fn rich_container_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.rich-tooltip.container.color",
        "md.sys.color.surface-container",
    )
}

pub(crate) fn rich_container_shadow_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.rich-tooltip.container.shadow-color",
        "md.sys.color.shadow",
    )
}

pub(crate) fn rich_container_elevation(theme: &Theme) -> Px {
    tooltip_metric(theme, "md.comp.rich-tooltip.container.elevation", Px(3.0))
}

pub(crate) fn rich_container_shape(theme: &Theme) -> Corners {
    MaterialTokenResolver::new(theme).corners_chain_or(
        &["md.comp.rich-tooltip.container.shape"],
        Corners::all(Px(12.0)),
    )
}

pub(crate) fn rich_container_padding(theme: &Theme, has_title: bool) -> Edges {
    let _ = theme;
    let vertical = if has_title { Px(12.0) } else { Px(4.0) };
    let bottom = if has_title { Px(16.0) } else { vertical };
    Edges {
        left: Px(16.0),
        right: Px(16.0),
        top: vertical,
        bottom,
    }
}

pub(crate) fn rich_text_gap(theme: &Theme) -> Px {
    let _ = theme;
    Px(4.0)
}

pub(crate) fn rich_subhead_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.rich-tooltip.subhead.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn rich_subhead_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.rich-tooltip.subhead"),
        "md.sys.typescale.title-small",
        Some("md.comp.rich-tooltip.subhead.weight"),
        TextIntent::Content,
    )
}

pub(crate) fn rich_supporting_text_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.rich-tooltip.supporting-text.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn rich_supporting_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.rich-tooltip.supporting-text"),
        "md.sys.typescale.body-medium",
        Some("md.comp.rich-tooltip.supporting-text.weight"),
        TextIntent::Content,
    )
}

pub(crate) fn rich_action_label_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.rich-tooltip.action.label-text.color",
        "md.sys.color.primary",
    )
}

pub(crate) fn rich_action_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.rich-tooltip.action.label-text"),
        "md.sys.typescale.label-large",
        Some("md.comp.rich-tooltip.action.label-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn rich_action_min_height(theme: &Theme) -> Px {
    let _ = theme;
    Px(36.0)
}

pub(crate) fn rich_action_bottom_padding(theme: &Theme) -> Px {
    let _ = theme;
    Px(8.0)
}

pub(crate) fn shadow_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_sys("md.sys.color.shadow")
}

pub(crate) fn close_duration_ms(theme: &Theme) -> u32 {
    MaterialTokenResolver::new(theme).duration_ms_sys("md.sys.motion.duration.short1", 50)
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
    fn tooltip_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(plain_container_shape(theme), Corners::all(Px(4.0)));
        assert_eq!(rich_container_elevation(theme), Px(3.0));
        assert_eq!(rich_container_shape(theme), Corners::all(Px(12.0)));
    }

    #[test]
    fn tooltip_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.plain-tooltip.container.shape".to_string(), 6.0);
        patch
            .metrics
            .insert("md.comp.rich-tooltip.container.elevation".to_string(), 4.0);
        patch
            .metrics
            .insert("md.comp.rich-tooltip.container.shape".to_string(), 14.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(plain_container_shape(&theme), Corners::all(Px(6.0)));
        assert_eq!(rich_container_elevation(&theme), Px(4.0));
        assert_eq!(rich_container_shape(&theme), Corners::all(Px(14.0)));
    }

    #[test]
    fn tooltip_shapes_prefer_structured_corners_over_uniform_metric() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.plain-tooltip.container.shape".to_string(), 6.0);
        patch.corners.insert(
            "md.comp.plain-tooltip.container.shape".to_string(),
            Corners {
                top_left: Px(1.0),
                top_right: Px(2.0),
                bottom_right: Px(3.0),
                bottom_left: Px(4.0),
            },
        );
        patch
            .metrics
            .insert("md.comp.rich-tooltip.container.shape".to_string(), 14.0);
        patch.corners.insert(
            "md.comp.rich-tooltip.container.shape".to_string(),
            Corners {
                top_left: Px(5.0),
                top_right: Px(6.0),
                bottom_right: Px(7.0),
                bottom_left: Px(8.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            plain_container_shape(&theme),
            Corners {
                top_left: Px(1.0),
                top_right: Px(2.0),
                bottom_right: Px(3.0),
                bottom_left: Px(4.0),
            }
        );
        assert_eq!(
            rich_container_shape(&theme),
            Corners {
                top_left: Px(5.0),
                top_right: Px(6.0),
                bottom_right: Px(7.0),
                bottom_left: Px(8.0),
            }
        );
    }
}
