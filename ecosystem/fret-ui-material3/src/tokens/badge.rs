//! Typed token access for Material 3 badges.
//!
//! Reference: Material Web v30 `md.comp.badge.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::typography;

fn badge_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

fn badge_metric_chain(theme: &Theme, keys: &[&'static str], fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_chain(keys, fallback)
}

pub(crate) fn dot_size(theme: &Theme) -> Px {
    badge_metric(theme, "md.comp.badge.size", Px(6.0))
}

pub(crate) fn large_size(theme: &Theme) -> Px {
    badge_metric(theme, "md.comp.badge.large.size", Px(16.0))
}

pub(crate) fn dot_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys("md.comp.badge.color", "md.sys.color.error")
}

pub(crate) fn large_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys("md.comp.badge.large.color", "md.sys.color.error")
}

pub(crate) fn large_label_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.badge.large.label-text.color",
        "md.sys.color.on-error",
    )
}

pub(crate) fn large_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.badge.large.label-text"),
        "md.sys.typescale.label-small",
        Some("md.comp.badge.large.label-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn shape(theme: &Theme) -> Corners {
    let r = badge_metric_chain(
        theme,
        &["md.comp.badge.shape", "md.sys.shape.corner.full"],
        Px(9999.0),
    );
    Corners::all(r)
}

pub(crate) fn large_shape(theme: &Theme) -> Corners {
    let r = badge_metric_chain(
        theme,
        &["md.comp.badge.large.shape", "md.sys.shape.corner.full"],
        Px(9999.0),
    );
    Corners::all(r)
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
    fn badge_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(dot_size(theme), Px(6.0));
        assert_eq!(large_size(theme), Px(16.0));
        assert_eq!(shape(theme), Corners::all(Px(9999.0)));
        assert_eq!(large_shape(theme), Corners::all(Px(9999.0)));
    }

    #[test]
    fn badge_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert("md.comp.badge.size".to_string(), 8.0);
        patch
            .metrics
            .insert("md.comp.badge.large.size".to_string(), 18.0);
        patch
            .metrics
            .insert("md.sys.shape.corner.full".to_string(), 40.0);
        patch
            .metrics
            .insert("md.comp.badge.large.shape".to_string(), 9.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(dot_size(&theme), Px(8.0));
        assert_eq!(large_size(&theme), Px(18.0));
        assert_eq!(shape(&theme), Corners::all(Px(40.0)));
        assert_eq!(large_shape(&theme), Corners::all(Px(9.0)));
    }
}
