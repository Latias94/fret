//! Typed token access for Material 3 search views.
//!
//! Reference: Material Web v30 `md.comp.search-view.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::typography;

fn search_view_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

fn search_view_metric_chain(theme: &Theme, keys: &[&'static str], fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(keys, fallback)
}

pub(crate) fn container_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-view.container.color",
        "md.sys.color.surface-container-high",
    )
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    search_view_metric(theme, "md.comp.search-view.container.elevation", Px(6.0))
}

pub(crate) fn full_screen_header_container_height(theme: &Theme) -> Px {
    search_view_metric(
        theme,
        "md.comp.search-view.full-screen.header.container.height",
        Px(72.0),
    )
}

pub(crate) fn divider_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys("md.comp.search-view.divider.color", "md.sys.color.outline")
}

pub(crate) fn docked_container_shape(theme: &Theme) -> Corners {
    let r = search_view_metric_chain(
        theme,
        &[
            "md.comp.search-view.docked.container.shape",
            "md.sys.shape.corner.extra-large",
        ],
        Px(28.0),
    );
    Corners::all(r)
}

pub(crate) fn header_leading_icon_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-view.header.leading-icon.color",
        "md.sys.color.on-surface",
    )
}

pub(crate) fn header_trailing_icon_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-view.header.trailing-icon.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn header_input_text_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-view.header.input-text.color",
        "md.sys.color.on-surface",
    )
}

pub(crate) fn header_supporting_text_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-view.header.supporting-text.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn header_input_text_style(theme: &Theme) -> TextStyle {
    typography::text_style(
        theme,
        Some("md.comp.search-view.header.input-text"),
        "md.sys.typescale.body-large",
        TextIntent::Control,
    )
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
    fn search_view_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(container_elevation(theme), Px(6.0));
        assert_eq!(full_screen_header_container_height(theme), Px(72.0));
        assert_eq!(docked_container_shape(theme), Corners::all(Px(28.0)));
    }

    #[test]
    fn search_view_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.search-view.container.elevation".to_string(), 4.0);
        patch.metrics.insert(
            "md.comp.search-view.full-screen.header.container.height".to_string(),
            80.0,
        );
        patch.metrics.insert(
            "md.comp.search-view.docked.container.shape".to_string(),
            18.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(container_elevation(&theme), Px(4.0));
        assert_eq!(full_screen_header_container_height(&theme), Px(80.0));
        assert_eq!(docked_container_shape(&theme), Corners::all(Px(18.0)));
    }

    #[test]
    fn search_view_shape_uses_system_fallback() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.shape.corner.extra-large".to_string(), 26.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(docked_container_shape(&theme), Corners::all(Px(26.0)));
    }
}
