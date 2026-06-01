//! Typed token access for Material 3 top app bars.
//!
//! Reference: Material Web v30 `md.comp.top-app-bar.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::{Theme, theme::CubicBezier};
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::motion::cubic_bezier_ease;
use crate::tokens::typography;
use crate::top_app_bar::TopAppBarVariant;

fn top_app_bar_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    MaterialTokenResolver::new(theme).metric_optional(Some(key), fallback)
}

fn container_height_key(variant: TopAppBarVariant) -> &'static str {
    match variant {
        TopAppBarVariant::Small => "md.comp.top-app-bar.small.container.height",
        TopAppBarVariant::SmallCentered => "md.comp.top-app-bar.small.centered.container.height",
        TopAppBarVariant::Medium => "md.comp.top-app-bar.medium.container.height",
        TopAppBarVariant::Large => "md.comp.top-app-bar.large.container.height",
    }
}

fn container_color_key(variant: TopAppBarVariant, scrolled: bool) -> &'static str {
    match (variant, scrolled) {
        (TopAppBarVariant::Small, false) => "md.comp.top-app-bar.small.container.color",
        (TopAppBarVariant::Small, true) => "md.comp.top-app-bar.small.on-scroll.container.color",
        (TopAppBarVariant::SmallCentered, false) => {
            "md.comp.top-app-bar.small.centered.container.color"
        }
        (TopAppBarVariant::SmallCentered, true) => {
            "md.comp.top-app-bar.small.centered.on-scroll.container.color"
        }
        (TopAppBarVariant::Medium, _) => "md.comp.top-app-bar.medium.container.color",
        (TopAppBarVariant::Large, _) => "md.comp.top-app-bar.large.container.color",
    }
}

fn container_elevation_key(variant: TopAppBarVariant) -> &'static str {
    match variant {
        TopAppBarVariant::Small => "md.comp.top-app-bar.small.container.elevation",
        TopAppBarVariant::SmallCentered => "md.comp.top-app-bar.small.centered.container.elevation",
        TopAppBarVariant::Medium => "md.comp.top-app-bar.medium.container.elevation",
        TopAppBarVariant::Large => "md.comp.top-app-bar.large.container.elevation",
    }
}

fn on_scroll_container_elevation_key(variant: TopAppBarVariant) -> Option<&'static str> {
    match variant {
        TopAppBarVariant::Small => Some("md.comp.top-app-bar.small.on-scroll.container.elevation"),
        TopAppBarVariant::SmallCentered => {
            Some("md.comp.top-app-bar.small.centered.on-scroll.container.elevation")
        }
        TopAppBarVariant::Medium | TopAppBarVariant::Large => None,
    }
}

fn container_shape_key(variant: TopAppBarVariant) -> &'static str {
    match variant {
        TopAppBarVariant::Small => "md.comp.top-app-bar.small.container.shape",
        TopAppBarVariant::SmallCentered => "md.comp.top-app-bar.small.centered.container.shape",
        TopAppBarVariant::Medium => "md.comp.top-app-bar.medium.container.shape",
        TopAppBarVariant::Large => "md.comp.top-app-bar.large.container.shape",
    }
}

fn headline_color_key(variant: TopAppBarVariant) -> &'static str {
    match variant {
        TopAppBarVariant::Small => "md.comp.top-app-bar.small.headline.color",
        TopAppBarVariant::SmallCentered => "md.comp.top-app-bar.small.centered.headline.color",
        TopAppBarVariant::Medium => "md.comp.top-app-bar.medium.headline.color",
        TopAppBarVariant::Large => "md.comp.top-app-bar.large.headline.color",
    }
}

fn headline_text_style_key(variant: TopAppBarVariant) -> &'static str {
    match variant {
        TopAppBarVariant::Small => "md.comp.top-app-bar.small.headline",
        TopAppBarVariant::SmallCentered => "md.comp.top-app-bar.small.centered.headline",
        TopAppBarVariant::Medium => "md.comp.top-app-bar.medium.headline",
        TopAppBarVariant::Large => "md.comp.top-app-bar.large.headline",
    }
}

fn leading_icon_color_key(variant: TopAppBarVariant) -> &'static str {
    match variant {
        TopAppBarVariant::Small => "md.comp.top-app-bar.small.leading-icon.color",
        TopAppBarVariant::SmallCentered => "md.comp.top-app-bar.small.centered.leading-icon.color",
        TopAppBarVariant::Medium => "md.comp.top-app-bar.medium.leading-icon.color",
        TopAppBarVariant::Large => "md.comp.top-app-bar.large.leading-icon.color",
    }
}

fn trailing_icon_color_key(variant: TopAppBarVariant) -> &'static str {
    match variant {
        TopAppBarVariant::Small => "md.comp.top-app-bar.small.trailing-icon.color",
        TopAppBarVariant::SmallCentered => "md.comp.top-app-bar.small.centered.trailing-icon.color",
        TopAppBarVariant::Medium => "md.comp.top-app-bar.medium.trailing-icon.color",
        TopAppBarVariant::Large => "md.comp.top-app-bar.large.trailing-icon.color",
    }
}

pub(crate) fn container_height(theme: &Theme, variant: TopAppBarVariant) -> Px {
    let fallback = match variant {
        TopAppBarVariant::Small | TopAppBarVariant::SmallCentered => Px(64.0),
        TopAppBarVariant::Medium => Px(112.0),
        TopAppBarVariant::Large => Px(152.0),
    };
    top_app_bar_metric(theme, container_height_key(variant), fallback)
}

pub(crate) fn container_background(
    theme: &Theme,
    variant: TopAppBarVariant,
    scrolled: bool,
) -> Color {
    if scrolled && matches!(variant, TopAppBarVariant::Medium | TopAppBarVariant::Large) {
        return MaterialTokenResolver::new(theme).color_sys("md.sys.color.surface-container");
    }
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        container_color_key(variant, scrolled),
        if scrolled {
            "md.sys.color.surface-container"
        } else {
            "md.sys.color.surface"
        },
    )
}

pub(crate) fn container_background_for_fraction(
    theme: &Theme,
    variant: TopAppBarVariant,
    transition_fraction: f32,
) -> Color {
    let fraction = transition_fraction.clamp(0.0, 1.0);
    if fraction <= 0.0 {
        return container_background(theme, variant, false);
    }
    if fraction >= 1.0 {
        return container_background(theme, variant, true);
    }

    let easing = CubicBezier {
        x1: 0.4,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    };
    lerp_color(
        container_background(theme, variant, false),
        container_background(theme, variant, true),
        cubic_bezier_ease(easing, fraction),
    )
}

fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: from.a + (to.a - from.a) * t,
    }
}

pub(crate) fn container_elevation(theme: &Theme, variant: TopAppBarVariant, scrolled: bool) -> Px {
    if scrolled {
        if let Some(key) = on_scroll_container_elevation_key(variant) {
            return top_app_bar_metric(theme, key, Px(3.0));
        }

        // Medium/Large v1 behavior: treat `scrolled` as level2 until we model a full scroll
        // behavior surface (Compose).
        return Px(3.0);
    }

    top_app_bar_metric(theme, container_elevation_key(variant), Px(0.0))
}

pub(crate) fn container_shape(theme: &Theme, variant: TopAppBarVariant) -> Corners {
    let r = top_app_bar_metric(theme, container_shape_key(variant), Px(0.0));
    Corners::all(r)
}

pub(crate) fn headline_color(theme: &Theme, variant: TopAppBarVariant) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(headline_color_key(variant), "md.sys.color.on-surface")
}

pub(crate) fn headline_text_style(theme: &Theme, variant: TopAppBarVariant) -> TextStyle {
    let fallback_key = match variant {
        TopAppBarVariant::Small | TopAppBarVariant::SmallCentered => "md.sys.typescale.title-large",
        TopAppBarVariant::Medium => "md.sys.typescale.headline-small",
        TopAppBarVariant::Large => "md.sys.typescale.headline-medium",
    };
    typography::text_style(
        theme,
        Some(headline_text_style_key(variant)),
        fallback_key,
        TextIntent::Control,
    )
}

pub(crate) fn leading_icon_color(theme: &Theme, variant: TopAppBarVariant) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(leading_icon_color_key(variant), "md.sys.color.on-surface")
}

pub(crate) fn trailing_icon_color(theme: &Theme, variant: TopAppBarVariant) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        trailing_icon_color_key(variant),
        "md.sys.color.on-surface-variant",
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
    fn top_app_bar_metrics_default_to_material_matrix() {
        let app = App::new();
        let theme = Theme::global(&app);

        assert_eq!(container_height(theme, TopAppBarVariant::Small), Px(64.0));
        assert_eq!(container_height(theme, TopAppBarVariant::Medium), Px(112.0));
        assert_eq!(container_height(theme, TopAppBarVariant::Large), Px(152.0));
        assert_eq!(
            container_elevation(theme, TopAppBarVariant::Small, false),
            Px(0.0)
        );
        assert_eq!(
            container_elevation(theme, TopAppBarVariant::Small, true),
            Px(3.0)
        );
        assert_eq!(
            container_shape(theme, TopAppBarVariant::Small),
            Corners::all(Px(0.0))
        );
    }

    #[test]
    fn top_app_bar_metrics_prefer_material_tokens() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.top-app-bar.small.container.height".to_string(),
            68.0,
        );
        patch.metrics.insert(
            "md.comp.top-app-bar.small.container.elevation".to_string(),
            1.0,
        );
        patch.metrics.insert(
            "md.comp.top-app-bar.small.on-scroll.container.elevation".to_string(),
            4.0,
        );
        patch
            .metrics
            .insert("md.comp.top-app-bar.small.container.shape".to_string(), 8.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(container_height(&theme, TopAppBarVariant::Small), Px(68.0));
        assert_eq!(
            container_elevation(&theme, TopAppBarVariant::Small, false),
            Px(1.0)
        );
        assert_eq!(
            container_elevation(&theme, TopAppBarVariant::Small, true),
            Px(4.0)
        );
        assert_eq!(
            container_shape(&theme, TopAppBarVariant::Small),
            Corners::all(Px(8.0))
        );
    }
}
