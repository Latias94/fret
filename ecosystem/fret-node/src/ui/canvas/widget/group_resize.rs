use fret_core::{Modifiers, Point, Px, Rect, Size};
use fret_ui::UiHost;

use crate::core::{CanvasPoint, CanvasRect};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn handle_group_resize_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    _zoom: f32,
) -> bool {
    let Some(mut resize) = canvas.interaction.group_resize.clone() else {
        return false;
    };

    let auto_pan_delta = (snapshot.interaction.auto_pan.on_node_drag)
        .then(|| NodeGraphCanvasWith::<M>::auto_pan_delta(snapshot, position, cx.bounds))
        .unwrap_or_default();
    let position = Point::new(
        Px(position.x.0 - auto_pan_delta.x),
        Px(position.y.0 - auto_pan_delta.y),
    );

    let dx = position.x.0 - resize.start_pos.x.0;
    let dy = position.y.0 - resize.start_pos.y.0;

    let mut new_rect = resize.start_rect;
    new_rect.size.width = resize.start_rect.size.width + dx;
    new_rect.size.height = resize.start_rect.size.height + dy;

    let min_w = 80.0;
    let min_h = 60.0;
    if new_rect.size.width.is_finite() {
        new_rect.size.width = new_rect.size.width.max(min_w);
    } else {
        new_rect.size.width = min_w;
    }
    if new_rect.size.height.is_finite() {
        new_rect.size.height = new_rect.size.height.max(min_h);
    } else {
        new_rect.size.height = min_h;
    }

    let geom = canvas.canvas_geometry(&*cx.app, snapshot);
    let (min_w_children, min_h_children) = canvas
        .graph
        .read_ref(cx.app, |g| {
            let mut max_x = new_rect.origin.x;
            let mut max_y = new_rect.origin.y;
            for (node_id, node) in &g.nodes {
                if node.parent != Some(resize.group) {
                    continue;
                }
                let Some(ng) = geom.nodes.get(node_id) else {
                    continue;
                };
                let x1 = ng.rect.origin.x.0 + ng.rect.size.width.0;
                let y1 = ng.rect.origin.y.0 + ng.rect.size.height.0;
                max_x = max_x.max(x1);
                max_y = max_y.max(y1);
            }
            (
                (max_x - new_rect.origin.x).max(0.0),
                (max_y - new_rect.origin.y).max(0.0),
            )
        })
        .ok()
        .unwrap_or_default();

    new_rect.size.width = new_rect.size.width.max(min_w_children);
    new_rect.size.height = new_rect.size.height.max(min_h_children);

    let allow_snap = !modifiers.alt && !modifiers.alt_gr;
    if allow_snap && snapshot.interaction.snap_to_grid {
        let grid = snapshot.interaction.snap_grid;
        let snapped = NodeGraphCanvasWith::<M>::snap_canvas_point(
            CanvasPoint {
                x: new_rect.origin.x + new_rect.size.width,
                y: new_rect.origin.y + new_rect.size.height,
            },
            grid,
        );
        new_rect.size.width = (snapped.x - new_rect.origin.x).max(min_w);
        new_rect.size.height = (snapped.y - new_rect.origin.y).max(min_h);
        new_rect.size.width = new_rect.size.width.max(min_w_children);
        new_rect.size.height = new_rect.size.height.max(min_h_children);
    }

    if resize.current_rect != new_rect {
        resize.current_rect = new_rect;
        resize.preview_rev = resize.preview_rev.wrapping_add(1);
    }
    canvas.interaction.group_resize = Some(resize);

    if auto_pan_delta.x != 0.0 || auto_pan_delta.y != 0.0 {
        canvas.update_view_state(cx.app, |s| {
            s.pan.x += auto_pan_delta.x;
            s.pan.y += auto_pan_delta.y;
        });
    }

    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

pub(super) fn group_rect_to_px(rect: CanvasRect) -> Rect {
    Rect::new(
        Point::new(Px(rect.origin.x), Px(rect.origin.y)),
        Size::new(Px(rect.size.width), Px(rect.size.height)),
    )
}

pub(super) fn group_resize_handle_hit(
    handle: Rect,
    position: Point,
    zoom: f32,
    padding_screen: f32,
) -> bool {
    if !padding_screen.is_finite() || padding_screen <= 0.0 {
        return rect_contains(handle, position);
    }
    let pad = padding_screen / zoom.max(1.0e-6);
    let expanded = Rect::new(
        Point::new(Px(handle.origin.x.0 - pad), Px(handle.origin.y.0 - pad)),
        Size::new(
            Px(handle.size.width.0 + 2.0 * pad),
            Px(handle.size.height.0 + 2.0 * pad),
        ),
    );
    rect_contains(expanded, position)
}

fn rect_contains(rect: Rect, pos: Point) -> bool {
    pos.x.0 >= rect.origin.x.0
        && pos.y.0 >= rect.origin.y.0
        && pos.x.0 <= rect.origin.x.0 + rect.size.width.0
        && pos.y.0 <= rect.origin.y.0 + rect.size.height.0
}
