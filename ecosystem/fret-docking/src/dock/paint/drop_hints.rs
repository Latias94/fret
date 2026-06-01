use std::collections::HashMap;

use fret_core::{
    Color, DockNodeId, DropZone, Edges, Scene, SceneOp,
    geometry::{Point, Px, Rect, Size},
};

use super::super::layout::dock_hint_rects_with_font;
use super::super::types::{DockDropHints, DockDropTarget};

#[allow(clippy::too_many_arguments)]
pub(in crate::dock) fn paint_drop_hints(
    theme: fret_ui::ThemeSnapshot,
    hints: Option<DockDropHints>,
    target: Option<DockDropTarget>,
    hint_font_size_inner: Px,
    hint_font_size_outer: Px,
    _window: fret_core::AppWindowId,
    _bounds: Rect,
    layout: &HashMap<DockNodeId, Rect>,
    scene: &mut Scene,
) {
    let Some(hints) = hints else {
        return;
    };

    let active = match target {
        Some(DockDropTarget::Dock(t)) => Some(t),
        _ => None,
    };
    let active_matches_hints =
        active.is_some_and(|t| t.root == hints.root && t.leaf_tabs == hints.leaf_tabs);
    let active_zone = active_matches_hints
        .then(|| active.unwrap())
        .filter(|t| t.explicit)
        .map(|t| t.zone);
    let active_outer = active_matches_hints && active.unwrap().outer;
    let inner_active_set = active_zone.is_some() && !active_outer;
    let outer_active_set = active_zone.is_some() && active_outer;

    let Some(inner_rect) = layout.get(&hints.leaf_tabs).copied() else {
        return;
    };
    let root_rect = layout.get(&hints.root).copied().unwrap_or(inner_rect);

    let show_outer = hints.root != hints.leaf_tabs;
    let inner_rects = dock_hint_rects_with_font(inner_rect, hint_font_size_inner, false);
    let outer_rects =
        show_outer.then(|| dock_hint_rects_with_font(root_rect, hint_font_size_outer, true));

    let inactive_bg_base = theme.color_token("card");
    let inactive_border_base = theme.color_token("border");
    let active_base = theme.color_token("primary");
    let surface_bg = theme.color_token("background");
    let radius_sm = theme.metric_token("metric.radius.sm");
    let radius_md = theme.metric_token("metric.radius.md");
    let pad_sm = theme.metric_token("metric.padding.sm");

    let inactive_bg = Color {
        a: 0.64,
        ..inactive_bg_base
    };
    let inactive_border = Color {
        a: 0.95,
        ..inactive_border_base
    };
    let active_bg = Color {
        a: 0.92,
        ..active_base
    };
    let active_border = Color {
        a: 1.0,
        ..active_base
    };

    // Keep hint pads above all drop overlays and tab insert markers so the user can always see
    // and target them (ImGui-style).
    let order = fret_core::DrawOrder(10_100);
    let border = Edges::all(Px(2.0));
    let corner_radii = fret_core::Corners::all(Px(radius_sm.0.max(4.0)));

    // Draw a plate behind the inner 5-way pad, closer to common editor docking affordances.
    let pad = Px(pad_sm.0.max(6.0));
    let mut min_x: f32 = f32::INFINITY;
    let mut min_y: f32 = f32::INFINITY;
    let mut max_x: f32 = f32::NEG_INFINITY;
    let mut max_y: f32 = f32::NEG_INFINITY;
    for &(_zone, r) in inner_rects.iter() {
        min_x = min_x.min(r.origin.x.0);
        min_y = min_y.min(r.origin.y.0);
        max_x = max_x.max(r.origin.x.0 + r.size.width.0);
        max_y = max_y.max(r.origin.y.0 + r.size.height.0);
    }
    if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
        let plate = Rect::new(
            Point::new(Px(min_x - pad.0), Px(min_y - pad.0)),
            Size::new(
                Px((max_x - min_x + pad.0 * 2.0).max(0.0)),
                Px((max_y - min_y + pad.0 * 2.0).max(0.0)),
            ),
        );
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(order.0 - 2),
            rect: plate,
            background: fret_core::Paint::Solid(Color {
                a: 0.70,
                ..surface_bg
            })
            .into(),
            border: Edges::all(Px(1.0)),
            border_paint: fret_core::Paint::Solid(Color {
                a: 0.70,
                ..inactive_border_base
            })
            .into(),
            corner_radii: fret_core::Corners::all(Px(radius_md.0.max(6.0))),
        });
    }

    let mut paint_set = |hint_rects: &[(DropZone, Rect); 5],
                         active_set: bool,
                         skip_center: bool,
                         inactive_alpha: f32| {
        for &(zone, hint_rect) in hint_rects.iter() {
            if skip_center && zone == DropZone::Center {
                continue;
            }
            let is_active = active_set && active_zone.is_some_and(|z| z == zone);
            let bg = if is_active {
                active_bg
            } else {
                Color {
                    a: inactive_bg.a * inactive_alpha,
                    ..inactive_bg
                }
            };
            let stroke = if is_active {
                active_border
            } else {
                Color {
                    a: inactive_border.a * inactive_alpha,
                    ..inactive_border
                }
            };

            scene.push(SceneOp::Quad {
                order,
                rect: hint_rect,
                background: fret_core::Paint::Solid(bg).into(),
                border,
                border_paint: fret_core::Paint::Solid(stroke).into(),
                corner_radii,
            });
            paint_drop_hint_icon(
                theme.clone(),
                zone,
                hint_rect,
                is_active,
                scene,
                order.0 + 1,
            );
        }
    };

    // Inner and outer hint sets can coexist; the active set is determined by which family of
    // drop rects was hit-tested.
    paint_set(&inner_rects, inner_active_set, false, 1.0);
    if let Some(outer_rects) = outer_rects.as_ref() {
        paint_set(outer_rects, outer_active_set, true, 0.80);
    }
}

fn paint_drop_hint_icon(
    theme: fret_ui::ThemeSnapshot,
    zone: DropZone,
    hint_rect: Rect,
    is_active: bool,
    scene: &mut Scene,
    order: u32,
) {
    fn inset(rect: Rect, inset: Px) -> Rect {
        let w = (rect.size.width.0 - inset.0 * 2.0).max(0.0);
        let h = (rect.size.height.0 - inset.0 * 2.0).max(0.0);
        Rect::new(
            Point::new(Px(rect.origin.x.0 + inset.0), Px(rect.origin.y.0 + inset.0)),
            Size::new(Px(w), Px(h)),
        )
    }

    let min_dim = hint_rect.size.width.0.min(hint_rect.size.height.0);
    let pad = Px((min_dim * 0.18).clamp(6.0, 10.0));
    let frame = inset(hint_rect, pad);
    let inner = inset(frame, Px((min_dim * 0.08).clamp(2.0, 4.0)));

    let fg = theme.color_token("foreground");
    let stroke = Color {
        a: if is_active { 0.92 } else { 0.80 },
        ..fg
    };
    let base = Color {
        a: if is_active { 0.16 } else { 0.12 },
        ..fg
    };
    let fill = Color {
        a: if is_active { 0.90 } else { 0.72 },
        ..fg
    };

    let frame_radius = Px(theme.metric_token("metric.radius.sm").0.clamp(2.0, 4.0));
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(order),
        rect: frame,
        background: fret_core::Paint::TRANSPARENT.into(),
        border: Edges::all(Px(2.0)),
        border_paint: fret_core::Paint::Solid(stroke).into(),
        corner_radii: fret_core::Corners::all(frame_radius),
    });

    // Base fill so the highlighted region reads as "target placement".
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(order + 1),
        rect: inner,
        background: fret_core::Paint::Solid(base).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    let split_ratio = 0.42_f32;
    let tab_ratio = 0.24_f32;
    let line_thickness = Px((min_dim * 0.04).clamp(1.5, 2.5));

    match zone {
        DropZone::Center => {
            let tab_h = Px((inner.size.height.0 * tab_ratio).max(0.0));
            let tab = Rect::new(inner.origin, Size::new(inner.size.width, tab_h));
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: tab,
                background: fret_core::Paint::Solid(fill).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
        DropZone::Left | DropZone::Right => {
            let w = Px((inner.size.width.0 * split_ratio).max(0.0));
            let (highlight, line_x) = if zone == DropZone::Left {
                (
                    Rect::new(inner.origin, Size::new(w, inner.size.height)),
                    Px(inner.origin.x.0 + w.0),
                )
            } else {
                (
                    Rect::new(
                        Point::new(
                            Px(inner.origin.x.0 + inner.size.width.0 - w.0),
                            inner.origin.y,
                        ),
                        Size::new(w, inner.size.height),
                    ),
                    Px(inner.origin.x.0 + inner.size.width.0 - w.0),
                )
            };
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: highlight,
                background: fret_core::Paint::Solid(fill).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
            let line = Rect::new(
                Point::new(Px(line_x.0 - line_thickness.0 * 0.5), inner.origin.y),
                Size::new(line_thickness, inner.size.height),
            );
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 3),
                rect: line,
                background: fret_core::Paint::Solid(stroke).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
        DropZone::Top | DropZone::Bottom => {
            let h = Px((inner.size.height.0 * split_ratio).max(0.0));
            let (highlight, line_y) = if zone == DropZone::Top {
                (
                    Rect::new(inner.origin, Size::new(inner.size.width, h)),
                    Px(inner.origin.y.0 + h.0),
                )
            } else {
                (
                    Rect::new(
                        Point::new(
                            inner.origin.x,
                            Px(inner.origin.y.0 + inner.size.height.0 - h.0),
                        ),
                        Size::new(inner.size.width, h),
                    ),
                    Px(inner.origin.y.0 + inner.size.height.0 - h.0),
                )
            };
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 2),
                rect: highlight,
                background: fret_core::Paint::Solid(fill).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
            let line = Rect::new(
                Point::new(inner.origin.x, Px(line_y.0 - line_thickness.0 * 0.5)),
                Size::new(inner.size.width, line_thickness),
            );
            scene.push(SceneOp::Quad {
                order: fret_core::DrawOrder(order + 3),
                rect: line,
                background: fret_core::Paint::Solid(stroke).into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }
    }
}
