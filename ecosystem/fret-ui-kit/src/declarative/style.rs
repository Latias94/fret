use fret_core::{Color, Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::element::{
    ContainerProps, LayoutStyle, Length, MarginEdge, RingPlacement, RingStyle, ShadowStyle,
};

use crate::style::{
    ChromeRefinement, InsetRefinement, LayoutRefinement, LengthRefinement, MarginRefinement,
    PaddingRefinement, SizeRefinement,
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

    let radius = chrome
        .radius
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));

    let layout = layout_style(theme, layout_refinement);

    ContainerProps {
        layout,
        padding,
        background,
        shadow: None,
        border: Edges::all(border_width),
        border_color,
        corner_radii: Corners::all(radius),
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

fn shadow_style(
    theme: &Theme,
    offset_x_key: &'static str,
    offset_y_key: &'static str,
    spread_key: &'static str,
    softness_key: &'static str,
    radius: Px,
    fallback_alpha: f32,
) -> ShadowStyle {
    let offset_x = shadow_metric(theme, offset_x_key, Px(0.0));
    let offset_y = shadow_metric(theme, offset_y_key, Px(2.0));
    let spread = shadow_metric(theme, spread_key, Px(0.0));
    let softness_px = shadow_metric(theme, softness_key, Px(2.0));
    let softness = softness_px.0.round().clamp(0.0, 8.0) as u8;

    ShadowStyle {
        color: shadow_color(theme, fallback_alpha),
        offset_x,
        offset_y,
        spread,
        softness,
        corner_radii: Corners::all(radius),
    }
}

pub fn shadow_xs(theme: &Theme, radius: Px) -> ShadowStyle {
    let offset_x = shadow_metric(theme, "component.shadow.xs.offset_x", Px(0.0));
    let offset_y = shadow_metric(theme, "component.shadow.xs.offset_y", Px(1.0));
    let spread = shadow_metric(theme, "component.shadow.xs.spread", Px(0.0));
    let softness_px = shadow_metric(theme, "component.shadow.xs.softness", Px(1.0));
    let softness = softness_px.0.round().clamp(0.0, 8.0) as u8;

    ShadowStyle {
        color: shadow_color(theme, 0.12),
        offset_x,
        offset_y,
        spread,
        softness,
        corner_radii: Corners::all(radius),
    }
}

pub fn shadow_sm(theme: &Theme, radius: Px) -> ShadowStyle {
    shadow_style(
        theme,
        "component.shadow.sm.offset_x",
        "component.shadow.sm.offset_y",
        "component.shadow.sm.spread",
        "component.shadow.sm.softness",
        radius,
        0.14,
    )
}

pub fn shadow_md(theme: &Theme, radius: Px) -> ShadowStyle {
    shadow_style(
        theme,
        "component.shadow.md.offset_x",
        "component.shadow.md.offset_y",
        "component.shadow.md.spread",
        "component.shadow.md.softness",
        radius,
        0.18,
    )
}

pub fn shadow_lg(theme: &Theme, radius: Px) -> ShadowStyle {
    shadow_style(
        theme,
        "component.shadow.lg.offset_x",
        "component.shadow.lg.offset_y",
        "component.shadow.lg.spread",
        "component.shadow.lg.softness",
        radius,
        0.24,
    )
}
