use fret_core::{Color, Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, RingPlacement, RingStyle, ShadowStyle};

use crate::style::{InsetRefinement, MarginRefinement};
use crate::{ColorRef, MetricRef, Radius, Space, StyleRefinement};

pub fn space(theme: &Theme, space: Space) -> Px {
    MetricRef::space(space).resolve(theme)
}

pub fn radius(theme: &Theme, radius: Radius) -> Px {
    MetricRef::radius(radius).resolve(theme)
}

pub fn color(theme: &Theme, color: ColorRef) -> Color {
    color.resolve(theme)
}

pub fn container_props(theme: &Theme, refinement: StyleRefinement) -> ContainerProps {
    let padding_x = refinement
        .padding_x
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));
    let padding_y = refinement
        .padding_y
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));

    let background = refinement.background.as_ref().map(|c| c.resolve(theme));

    let border_width = refinement
        .border_width
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));
    let border_color = refinement.border_color.as_ref().map(|c| c.resolve(theme));

    let radius = refinement
        .radius
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));

    let mut layout = fret_ui::element::LayoutStyle::default();
    if let Some(min_h) = refinement.min_height.as_ref().map(|m| m.resolve(theme)) {
        layout.size.min_height = Some(min_h);
    }
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
        layout.margin.top = top.map(|m| m.resolve(theme)).unwrap_or(Px(0.0));
        layout.margin.right = right.map(|m| m.resolve(theme)).unwrap_or(Px(0.0));
        layout.margin.bottom = bottom.map(|m| m.resolve(theme)).unwrap_or(Px(0.0));
        layout.margin.left = left.map(|m| m.resolve(theme)).unwrap_or(Px(0.0));
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

    ContainerProps {
        layout,
        padding_x,
        padding_y,
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
        .color_by_key("ring")
        .unwrap_or(theme.colors.focus_ring);
    let offset_color = theme
        .color_by_key("ring-offset-background")
        .unwrap_or(theme.colors.surface_background);

    RingStyle {
        placement: RingPlacement::Outset,
        width,
        offset,
        color,
        offset_color: Some(offset_color),
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
