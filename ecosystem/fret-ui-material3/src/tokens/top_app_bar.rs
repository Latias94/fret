//! Typed token access for Material 3 top app bars.
//!
//! Reference: Material Web v30 `md.comp.top-app-bar.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::top_app_bar::TopAppBarVariant;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.top-app-bar";

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
    theme
        .metric_by_key(container_height_key(variant))
        .unwrap_or(fallback)
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

pub(crate) fn container_elevation(theme: &Theme, variant: TopAppBarVariant, scrolled: bool) -> Px {
    if scrolled {
        if let Some(key) = on_scroll_container_elevation_key(variant) {
            return theme.metric_by_key(key).unwrap_or(Px(3.0));
        }

        // Medium/Large v1 behavior: treat `scrolled` as level2 until we model a full scroll
        // behavior surface (Compose).
        return Px(3.0);
    }

    theme
        .metric_by_key(container_elevation_key(variant))
        .unwrap_or(Px(0.0))
}

pub(crate) fn container_shape(theme: &Theme, variant: TopAppBarVariant) -> Corners {
    let r = theme
        .metric_by_key(container_shape_key(variant))
        .unwrap_or(Px(0.0));
    Corners::all(r)
}

pub(crate) fn headline_color(theme: &Theme, variant: TopAppBarVariant) -> Color {
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(headline_color_key(variant), "md.sys.color.on-surface")
}

pub(crate) fn headline_text_style(theme: &Theme, variant: TopAppBarVariant) -> TextStyle {
    if let Some(style) = theme.text_style_by_key(headline_text_style_key(variant)) {
        return style;
    }

    let fallback_key = match variant {
        TopAppBarVariant::Small | TopAppBarVariant::SmallCentered => "md.sys.typescale.title-large",
        TopAppBarVariant::Medium => "md.sys.typescale.headline-small",
        TopAppBarVariant::Large => "md.sys.typescale.headline-medium",
    };
    theme
        .text_style_by_key(fallback_key)
        .unwrap_or_else(TextStyle::default)
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
