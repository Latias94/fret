//! Typed token access for Material 3 search bars.
//!
//! Reference: Material Web v30 `md.comp.search-bar.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul};
use crate::tokens::typography;

fn search_bar_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

fn search_bar_metric_chain(theme: &Theme, keys: &[&'static str], fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(keys, fallback)
}

pub(crate) fn container_height(theme: &Theme) -> Px {
    search_bar_metric(theme, "md.comp.search-bar.container.height", Px(56.0))
}

pub(crate) fn container_min_width(theme: &Theme) -> Px {
    search_bar_metric(
        theme,
        "md.sys.fret.material.search-bar.container.min-width",
        Px(360.0),
    )
}

pub(crate) fn container_max_width(theme: &Theme) -> Px {
    search_bar_metric(
        theme,
        "md.sys.fret.material.search-bar.container.max-width",
        Px(720.0),
    )
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    let r = search_bar_metric_chain(
        theme,
        &[
            "md.comp.search-bar.container.shape",
            "md.sys.shape.corner.full",
        ],
        Px(9999.0),
    );
    Corners::all(r)
}

pub(crate) fn container_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.container.color",
        "md.sys.color.surface-container-high",
    )
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    search_bar_metric(theme, "md.comp.search-bar.container.elevation", Px(6.0))
}

pub(crate) fn leading_icon_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.leading-icon.color",
        "md.sys.color.on-surface",
    )
}

pub(crate) fn trailing_icon_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.trailing-icon.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn input_text_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.input-text.color",
        "md.sys.color.on-surface",
    )
}

pub(crate) fn supporting_text_color(theme: &Theme, hovered: bool, pressed: bool) -> Color {
    let key = if pressed {
        "md.comp.search-bar.pressed.supporting-text.color"
    } else if hovered {
        "md.comp.search-bar.hover.supporting-text.color"
    } else {
        "md.comp.search-bar.supporting-text.color"
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.on-surface-variant")
}

pub(crate) fn input_text_style(theme: &Theme) -> TextStyle {
    typography::text_style(
        theme,
        Some("md.comp.search-bar.input-text"),
        "md.sys.typescale.body-large",
        TextIntent::Control,
    )
}

pub(crate) fn hover_state_layer_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.hover.state-layer.color",
        "md.sys.color.on-surface",
    )
}

pub(crate) fn pressed_state_layer_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.pressed.state-layer.color",
        "md.sys.color.on-surface",
    )
}

pub(crate) fn hover_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).number_comp_or_sys(
        "md.comp.search-bar.hover.state-layer.opacity",
        "md.sys.state.hover.state-layer-opacity",
        0.08,
    )
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).number_comp_or_sys(
        "md.comp.search-bar.pressed.state-layer.opacity",
        "md.sys.state.pressed.state-layer-opacity",
        0.1,
    )
}

pub(crate) fn selection_color(theme: &Theme) -> Color {
    let primary = MaterialTokenResolver::new(theme).color_sys("md.sys.color.primary");
    alpha_mul(primary, 0.35)
}

pub(crate) fn caret_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_sys("md.sys.color.primary")
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
    fn search_bar_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(container_height(theme), Px(56.0));
        assert_eq!(container_min_width(theme), Px(360.0));
        assert_eq!(container_max_width(theme), Px(720.0));
        assert_eq!(container_shape(theme), Corners::all(Px(9999.0)));
        assert_eq!(container_elevation(theme), Px(6.0));
    }

    #[test]
    fn search_bar_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.search-bar.container.height".to_string(), 60.0);
        patch.metrics.insert(
            "md.sys.fret.material.search-bar.container.min-width".to_string(),
            320.0,
        );
        patch.metrics.insert(
            "md.sys.fret.material.search-bar.container.max-width".to_string(),
            640.0,
        );
        patch
            .metrics
            .insert("md.comp.search-bar.container.shape".to_string(), 30.0);
        patch
            .metrics
            .insert("md.comp.search-bar.container.elevation".to_string(), 4.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(container_height(&theme), Px(60.0));
        assert_eq!(container_min_width(&theme), Px(320.0));
        assert_eq!(container_max_width(&theme), Px(640.0));
        assert_eq!(container_shape(&theme), Corners::all(Px(30.0)));
        assert_eq!(container_elevation(&theme), Px(4.0));
    }

    #[test]
    fn search_bar_shape_uses_system_fallback() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.sys.shape.corner.full".to_string(), 24.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(container_shape(&theme), Corners::all(Px(24.0)));
    }
}
