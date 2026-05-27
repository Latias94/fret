use fret_canvas::view::PanZoom2D;
use fret_core::{Point, Px, Rect};

use crate::ui::canvas::{CanvasGeometry, CanvasSpatialDerived};

use super::{MarqueeDragState, NodeDragState};

pub(super) fn quantize_f32(value: f32, scale: f32) -> i32 {
    if !value.is_finite() || !scale.is_finite() || scale <= 0.0 {
        return 0;
    }
    (value * scale)
        .round()
        .clamp(i32::MIN as f32, i32::MAX as f32) as i32
}

pub(super) fn rect_contains_point(rect: Rect, p: Point) -> bool {
    fret_canvas::view::rect_contains_point(rect, p)
}

pub(super) fn rect_from_points(a: Point, b: Point) -> Rect {
    fret_canvas::view::rect_from_points(a, b)
}

pub(super) fn rects_intersect(a: Rect, b: Rect) -> bool {
    fret_canvas::view::rects_intersect(a, b)
}

pub(super) fn rect_approx_eq(a: Rect, b: Rect, eps: f32) -> bool {
    (a.origin.x.0 - b.origin.x.0).abs() <= eps
        && (a.origin.y.0 - b.origin.y.0).abs() <= eps
        && (a.size.width.0 - b.size.width.0).abs() <= eps
        && (a.size.height.0 - b.size.height.0).abs() <= eps
}

pub(super) fn rect_union(a: Rect, b: Rect) -> Rect {
    fret_canvas::view::rect_union(a, b)
}

pub(super) fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
    fret_canvas::view::rect_contains_rect(outer, inner)
}

pub(super) fn marquee_rect_screen(marquee: &MarqueeDragState) -> Rect {
    rect_from_points(marquee.start_screen, marquee.current_screen)
}

pub(super) fn pointer_crossed_threshold(
    start_screen: Point,
    current_screen: Point,
    threshold: f32,
) -> bool {
    let threshold = threshold.max(0.0);
    let dx = current_screen.x.0 - start_screen.x.0;
    let dy = current_screen.y.0 - start_screen.y.0;
    let dist2 = dx * dx + dy * dy;
    let threshold2 = threshold * threshold;
    dist2 >= threshold2
}

pub(super) fn node_drag_delta_canvas(view: PanZoom2D, drag: &NodeDragState) -> (f32, f32) {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let dx = (drag.current_screen.x.0 - drag.start_screen.x.0) / zoom;
    let dy = (drag.current_screen.y.0 - drag.start_screen.y.0) / zoom;
    (dx, dy)
}

pub(super) fn node_drag_commit_delta(view: PanZoom2D, drag: &NodeDragState) -> Option<(f32, f32)> {
    let (dx, dy) = node_drag_delta_canvas(view, drag);
    if !drag.is_active() || !dx.is_finite() || !dy.is_finite() {
        return None;
    }
    if dx.abs() <= 1.0e-9 && dy.abs() <= 1.0e-9 {
        return None;
    }
    Some((dx, dy))
}

pub(super) fn node_drag_contains(drag: &NodeDragState, id: crate::core::NodeId) -> bool {
    drag.nodes_sorted.binary_search(&id).is_ok()
}

pub(super) fn hit_test_node_at_point(
    view: PanZoom2D,
    bounds: Rect,
    node_click_distance_screen_px: f32,
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
    pos_screen: Point,
    scratch: &mut Vec<crate::core::NodeId>,
) -> Option<crate::core::NodeId> {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let radius_canvas = (node_click_distance_screen_px.max(0.0) / zoom).max(0.0);
    let pos_canvas = view.screen_to_canvas(bounds, pos_screen);

    let query = Rect::new(
        Point::new(
            Px(pos_canvas.x.0 - radius_canvas),
            Px(pos_canvas.y.0 - radius_canvas),
        ),
        fret_core::Size::new(Px(2.0 * radius_canvas), Px(2.0 * radius_canvas)),
    );

    scratch.clear();
    index.query_nodes_in_rect(query, scratch);

    let mut best: Option<(u32, crate::core::NodeId)> = None;
    for id in scratch.iter().copied() {
        let Some(node) = geom.nodes.get(&id) else {
            continue;
        };
        if !rect_contains_point(node.rect, pos_canvas) {
            continue;
        }
        let rank = geom.node_rank.get(&id).copied().unwrap_or(0);
        match best {
            None => best = Some((rank, id)),
            Some((current, _)) if rank >= current => best = Some((rank, id)),
            _ => {}
        }
    }
    best.map(|(_, id)| id)
}
