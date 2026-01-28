use fret_core::{Color, Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::element::{
    ContainerProps, LayoutStyle, Length, MarginEdge, RingPlacement, RingStyle, ShadowLayerStyle,
    ShadowStyle,
};

use crate::style::{
    ChromeRefinement, CornerRadiiRefinement, InsetRefinement, LayoutRefinement, LengthRefinement,
    MarginRefinement, PaddingRefinement, ShadowPreset, SizeRefinement,
};
use crate::{ColorRef, MetricRef, Radius, Space};

pub fn space(theme: &Theme, space: Space) -> Px {
    MetricRef::space(space).resolve(theme)
}

pub fn radius(theme: &Theme, radius: Radius) -> Px {
    MetricRef::radius(radius).resolve(theme)
}

pub fn color(theme: &Theme, color: ColorRef) -> Color {
    color.resolve(theme)
}

fn resolve_length(theme: &Theme, l: &LengthRefinement) -> Length {
    match l {
        LengthRefinement::Auto => Length::Auto,
        LengthRefinement::Fill => Length::Fill,
        LengthRefinement::Px(m) => Length::Px(m.resolve(theme)),
    }
}

fn resolve_padding(theme: &Theme, padding: Option<&PaddingRefinement>) -> Edges {
    let Some(p) = padding else {
        return Edges::all(Px(0.0));
    };
    Edges {
        top: p.top.as_ref().map(|m| m.resolve(theme)).unwrap_or(Px(0.0)),
        right: p
            .right
            .as_ref()
            .map(|m| m.resolve(theme))
            .unwrap_or(Px(0.0)),
        bottom: p
            .bottom
            .as_ref()
            .map(|m| m.resolve(theme))
            .unwrap_or(Px(0.0)),
        left: p.left.as_ref().map(|m| m.resolve(theme)).unwrap_or(Px(0.0)),
    }
}

fn resolve_corner_radii(
    theme: &Theme,
    radii: Option<&CornerRadiiRefinement>,
    fallback_radius: Px,
) -> Corners {
    let Some(r) = radii else {
        return Corners::all(fallback_radius);
    };
    Corners {
        top_left: r
            .top_left
            .as_ref()
            .map(|m| m.resolve(theme))
            .unwrap_or(fallback_radius),
        top_right: r
            .top_right
            .as_ref()
            .map(|m| m.resolve(theme))
            .unwrap_or(fallback_radius),
        bottom_right: r
            .bottom_right
            .as_ref()
            .map(|m| m.resolve(theme))
            .unwrap_or(fallback_radius),
        bottom_left: r
            .bottom_left
            .as_ref()
            .map(|m| m.resolve(theme))
            .unwrap_or(fallback_radius),
    }
}

fn max_corner_radius(corners: Corners) -> Px {
    let mut max = corners.top_left.0;
    max = max.max(corners.top_right.0);
    max = max.max(corners.bottom_right.0);
    max = max.max(corners.bottom_left.0);
    Px(max)
}

pub fn layout_style(theme: &Theme, refinement: LayoutRefinement) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    apply_layout_refinement(theme, refinement, &mut layout);
    layout
}

pub fn apply_layout_refinement(
    theme: &Theme,
    refinement: LayoutRefinement,
    layout: &mut LayoutStyle,
) {
    if let Some(ratio) = refinement.aspect_ratio {
        layout.aspect_ratio = Some(ratio);
    }
    if let Some(position) = refinement.position {
        layout.position = position;
    }
    if let Some(MarginRefinement {
        top,
        right,
        bottom,
        left,
    }) = refinement.margin
    {
        layout.margin.top = top
            .map(|m| m.resolve(theme))
            .unwrap_or(MarginEdge::Px(Px(0.0)));
        layout.margin.right = right
            .map(|m| m.resolve(theme))
            .unwrap_or(MarginEdge::Px(Px(0.0)));
        layout.margin.bottom = bottom
            .map(|m| m.resolve(theme))
            .unwrap_or(MarginEdge::Px(Px(0.0)));
        layout.margin.left = left
            .map(|m| m.resolve(theme))
            .unwrap_or(MarginEdge::Px(Px(0.0)));
    }
    if let Some(InsetRefinement {
        top,
        right,
        bottom,
        left,
    }) = refinement.inset
    {
        layout.inset.top = top.map(|m| m.resolve(theme));
        layout.inset.right = right.map(|m| m.resolve(theme));
        layout.inset.bottom = bottom.map(|m| m.resolve(theme));
        layout.inset.left = left.map(|m| m.resolve(theme));
    }
    if let Some(SizeRefinement {
        width,
        height,
        min_width,
        min_height,
        max_width,
        max_height,
    }) = refinement.size
    {
        if let Some(w) = width.as_ref() {
            layout.size.width = resolve_length(theme, w);
        }
        if let Some(h) = height.as_ref() {
            layout.size.height = resolve_length(theme, h);
        }
        if let Some(m) = min_width.as_ref() {
            layout.size.min_width = Some(m.resolve(theme));
        }
        if let Some(m) = min_height.as_ref() {
            layout.size.min_height = Some(m.resolve(theme));
        }
        if let Some(m) = max_width.as_ref() {
            layout.size.max_width = Some(m.resolve(theme));
        }
        if let Some(m) = max_height.as_ref() {
            layout.size.max_height = Some(m.resolve(theme));
        }
    }

    if let Some(flex) = refinement.flex_item {
        if let Some(grow) = flex.grow {
            layout.flex.grow = grow;
        }
        if let Some(shrink) = flex.shrink {
            layout.flex.shrink = shrink;
        }
        if let Some(basis) = flex.basis.as_ref() {
            layout.flex.basis = resolve_length(theme, basis);
        }
    }

    if let Some(overflow) = refinement.overflow {
        layout.overflow = overflow.to_overflow();
    }
}

pub fn container_props(
    theme: &Theme,
    chrome: ChromeRefinement,
    layout_refinement: LayoutRefinement,
) -> ContainerProps {
    let padding = resolve_padding(theme, chrome.padding.as_ref());

    let background = chrome.background.as_ref().map(|c| c.resolve(theme));

    let border_width = chrome
        .border_width
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));
    let border_color = chrome.border_color.as_ref().map(|c| c.resolve(theme));

    let uniform_radius = chrome
        .radius
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));

    let corner_radii = resolve_corner_radii(theme, chrome.corner_radii.as_ref(), uniform_radius);

    let shadow_radius = max_corner_radius(corner_radii);

    let layout = layout_style(theme, layout_refinement);

    let shadow = match chrome.shadow {
        Some(ShadowPreset::None) => None,
        Some(ShadowPreset::Xs) => Some(shadow_xs(theme, shadow_radius)),
        Some(ShadowPreset::Sm) => Some(shadow_sm(theme, shadow_radius)),
        Some(ShadowPreset::Md) => Some(shadow_md(theme, shadow_radius)),
        Some(ShadowPreset::Lg) => Some(shadow_lg(theme, shadow_radius)),
        Some(ShadowPreset::Xl) => Some(shadow_xl(theme, shadow_radius)),
        None => None,
    };

    ContainerProps {
        layout,
        padding,
        background,
        shadow,
        border: Edges::all(border_width),
        border_color,
        focus_ring: None,
        focus_border_color: None,
        focus_within: false,
        corner_radii,
    }
}

pub fn focus_ring(theme: &Theme, radius: Px) -> RingStyle {
    let width = theme
        .metric_by_key("component.ring.width")
        .unwrap_or(Px(2.0));
    let offset = theme
        .metric_by_key("component.ring.offset")
        .unwrap_or(Px(2.0));
    let color = theme
        .color_by_key("ring/50")
        .or_else(|| theme.color_by_key("ring"))
        .unwrap_or_else(|| theme.color_required("ring"));
    let offset_color = theme
        .color_by_key("ring-offset-background")
        .unwrap_or_else(|| theme.color_required("ring-offset-background"));

    RingStyle {
        placement: RingPlacement::Outset,
        width,
        offset,
        color,
        offset_color: (offset.0 > 0.0).then_some(offset_color),
        corner_radii: Corners::all(radius),
    }
}

fn shadow_color(theme: &Theme, fallback_alpha: f32) -> Color {
    let base = theme.color_by_key("shadow").unwrap_or(Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    Color {
        a: fallback_alpha.clamp(0.0, 1.0),
        ..base
    }
}

fn shadow_metric(theme: &Theme, key: &'static str, fallback: Px) -> Px {
    theme.metric_by_key(key).unwrap_or(fallback)
}

fn shadow_layer_style(
    theme: &Theme,
    offset_x_key: &'static str,
    offset_y_key: &'static str,
    spread_key: &'static str,
    blur_key: &'static str,
    fallback: (Px, Px, Px, Px),
    color: Color,
) -> ShadowLayerStyle {
    let offset_x = shadow_metric(theme, offset_x_key, Px(0.0));
    let offset_y = shadow_metric(theme, offset_y_key, fallback.1);
    let spread = shadow_metric(theme, spread_key, fallback.2);
    // Back-compat: the token name is still `softness` in theme metrics, but maps to CSS blur.
    let blur = shadow_metric(theme, blur_key, fallback.3);

    ShadowLayerStyle {
        color,
        offset_x,
        offset_y,
        blur,
        spread,
    }
}

pub fn shadow(theme: &Theme, radius: Px) -> ShadowStyle {
    // Tailwind default (`shadow`):
    // `0 1px 3px 0 rgba(0,0,0,0.1), 0 1px 2px 0 rgba(0,0,0,0.06)`
    let primary_color = shadow_color(theme, 0.10);
    let primary = shadow_layer_style(
        theme,
        "component.shadow.default.offset_x",
        "component.shadow.default.offset_y",
        "component.shadow.default.spread",
        "component.shadow.default.softness",
        (Px(0.0), Px(1.0), Px(0.0), Px(3.0)),
        primary_color,
    );

    let secondary_color = shadow_color(theme, 0.06);
    let secondary = shadow_layer_style(
        theme,
        "component.shadow.default2.offset_x",
        "component.shadow.default2.offset_y",
        "component.shadow.default2.spread",
        "component.shadow.default2.softness",
        (Px(0.0), Px(1.0), Px(0.0), Px(2.0)),
        secondary_color,
    );

    ShadowStyle {
        primary,
        secondary: Some(secondary),
        corner_radii: Corners::all(radius),
    }
}

pub fn shadow_xs(theme: &Theme, radius: Px) -> ShadowStyle {
    // Tailwind default (`shadow-xs`): `0 1px 2px 0 rgba(0,0,0,0.05)`.
    let color = shadow_color(theme, 0.05);
    let primary = shadow_layer_style(
        theme,
        "component.shadow.xs.offset_x",
        "component.shadow.xs.offset_y",
        "component.shadow.xs.spread",
        "component.shadow.xs.softness",
        (Px(0.0), Px(1.0), Px(0.0), Px(2.0)),
        color,
    );
    ShadowStyle {
        primary,
        secondary: None,
        corner_radii: Corners::all(radius),
    }
}

pub fn shadow_sm(theme: &Theme, radius: Px) -> ShadowStyle {
    let color = shadow_color(theme, 0.14);
    let primary = shadow_layer_style(
        theme,
        "component.shadow.sm.offset_x",
        "component.shadow.sm.offset_y",
        "component.shadow.sm.spread",
        "component.shadow.sm.softness",
        (Px(0.0), Px(2.0), Px(0.0), Px(2.0)),
        color,
    );
    ShadowStyle {
        primary,
        secondary: None,
        corner_radii: Corners::all(radius),
    }
}

pub fn shadow(theme: &Theme, radius: Px) -> ShadowStyle {
    // Tailwind default (`shadow`):
    // `0 1px 3px 0 rgba(0,0,0,0.1), 0 1px 2px -1px rgba(0,0,0,0.1)`
    let color = shadow_color(theme, 0.10);
    let primary = shadow_layer_style(
        theme,
        "component.shadow.default.offset_x",
        "component.shadow.default.offset_y",
        "component.shadow.default.spread",
        "component.shadow.default.softness",
        (Px(0.0), Px(1.0), Px(0.0), Px(3.0)),
        color,
    );
    let secondary = shadow_layer_style(
        theme,
        "component.shadow.default2.offset_x",
        "component.shadow.default2.offset_y",
        "component.shadow.default2.spread",
        "component.shadow.default2.softness",
        (Px(0.0), Px(1.0), Px(-1.0), Px(2.0)),
        color,
    );

    ShadowStyle {
        primary,
        secondary: Some(secondary),
        corner_radii: Corners::all(radius),
    }
}

pub fn shadow_md(theme: &Theme, radius: Px) -> ShadowStyle {
    // Tailwind default (`shadow-md`):
    // `0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1)`
    let color = shadow_color(theme, 0.10);
    let primary = shadow_layer_style(
        theme,
        "component.shadow.md.offset_x",
        "component.shadow.md.offset_y",
        "component.shadow.md.spread",
        "component.shadow.md.softness",
        (Px(0.0), Px(4.0), Px(-1.0), Px(6.0)),
        color,
    );
    let secondary = shadow_layer_style(
        theme,
        "component.shadow.md2.offset_x",
        "component.shadow.md2.offset_y",
        "component.shadow.md2.spread",
        "component.shadow.md2.softness",
        (Px(0.0), Px(2.0), Px(-2.0), Px(4.0)),
        color,
    );

    ShadowStyle {
        primary,
        secondary: Some(secondary),
        corner_radii: Corners::all(radius),
    }
}

pub fn shadow_lg(theme: &Theme, radius: Px) -> ShadowStyle {
    // Tailwind default (`shadow-lg`):
    // `0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1)`
    let color = shadow_color(theme, 0.10);
    let primary = shadow_layer_style(
        theme,
        "component.shadow.lg.offset_x",
        "component.shadow.lg.offset_y",
        "component.shadow.lg.spread",
        "component.shadow.lg.softness",
        (Px(0.0), Px(10.0), Px(-3.0), Px(15.0)),
        color,
    );
    let secondary = shadow_layer_style(
        theme,
        "component.shadow.lg2.offset_x",
        "component.shadow.lg2.offset_y",
        "component.shadow.lg2.spread",
        "component.shadow.lg2.softness",
        (Px(0.0), Px(4.0), Px(-4.0), Px(6.0)),
        color,
    );

    ShadowStyle {
        primary,
        secondary: Some(secondary),
        corner_radii: Corners::all(radius),
    }
}

pub fn shadow_xl(theme: &Theme, radius: Px) -> ShadowStyle {
    // Tailwind default (`shadow-xl`):
    // `0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1)`
    let color = shadow_color(theme, 0.10);
    let primary = shadow_layer_style(
        theme,
        "component.shadow.xl.offset_x",
        "component.shadow.xl.offset_y",
        "component.shadow.xl.spread",
        "component.shadow.xl.softness",
        (Px(0.0), Px(20.0), Px(-5.0), Px(25.0)),
        color,
    );
    let secondary = shadow_layer_style(
        theme,
        "component.shadow.xl2.offset_x",
        "component.shadow.xl2.offset_y",
        "component.shadow.xl2.spread",
        "component.shadow.xl2.softness",
        (Px(0.0), Px(8.0), Px(-6.0), Px(10.0)),
        color,
    );

    ShadowStyle {
        primary,
        secondary: Some(secondary),
        corner_radii: Corners::all(radius),
    }
}
