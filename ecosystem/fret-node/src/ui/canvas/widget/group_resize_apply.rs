use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::{CanvasPoint, CanvasRect};
use crate::ui::canvas::state::GroupResize;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

const MIN_GROUP_WIDTH: f32 = 80.0;
const MIN_GROUP_HEIGHT: f32 = 60.0;

pub(super) fn next_group_resize_rect<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    resize: &GroupResize,
    position: Point,
    modifiers: Modifiers,
) -> CanvasRect {
    let mut new_rect = group_resize_rect_from_pointer(resize, position);
    clamp_group_resize_size(&mut new_rect.size.width, MIN_GROUP_WIDTH);
    clamp_group_resize_size(&mut new_rect.size.height, MIN_GROUP_HEIGHT);

    let (min_w_children, min_h_children) =
        min_group_resize_children_size(canvas, host, snapshot, resize, &new_rect);
    new_rect.size.width = new_rect.size.width.max(min_w_children);
    new_rect.size.height = new_rect.size.height.max(min_h_children);

    if allow_group_resize_snap(snapshot, modifiers) {
        let (snapped_w, snapped_h) =
            snapped_group_resize_size::<M>(&new_rect, snapshot, min_w_children, min_h_children);
        new_rect.size.width = snapped_w;
        new_rect.size.height = snapped_h;
    }

    new_rect
}

fn group_resize_rect_from_pointer(resize: &GroupResize, position: Point) -> CanvasRect {
    let dx = position.x.0 - resize.start_pos.x.0;
    let dy = position.y.0 - resize.start_pos.y.0;

    let mut new_rect = resize.start_rect;
    new_rect.size.width = resize.start_rect.size.width + dx;
    new_rect.size.height = resize.start_rect.size.height + dy;
    new_rect
}

fn clamp_group_resize_size(size: &mut f32, min_size: f32) {
    if size.is_finite() {
        *size = size.max(min_size);
    } else {
        *size = min_size;
    }
}

fn min_group_resize_children_size<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    resize: &GroupResize,
    new_rect: &CanvasRect,
) -> (f32, f32) {
    let geom = canvas.canvas_geometry(&*host, snapshot);
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut max_x = new_rect.origin.x;
            let mut max_y = new_rect.origin.y;
            for (node_id, node) in &graph.nodes {
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
        .unwrap_or_default()
}

fn allow_group_resize_snap(snapshot: &ViewSnapshot, modifiers: Modifiers) -> bool {
    !modifiers.alt && !modifiers.alt_gr && snapshot.interaction.snap_to_grid
}

fn snapped_group_resize_size<M: NodeGraphCanvasMiddleware>(
    new_rect: &CanvasRect,
    snapshot: &ViewSnapshot,
    min_w_children: f32,
    min_h_children: f32,
) -> (f32, f32) {
    let snapped = NodeGraphCanvasWith::<M>::snap_canvas_point(
        CanvasPoint {
            x: new_rect.origin.x + new_rect.size.width,
            y: new_rect.origin.y + new_rect.size.height,
        },
        snapshot.interaction.snap_grid,
    );
    (
        (snapped.x - new_rect.origin.x)
            .max(MIN_GROUP_WIDTH)
            .max(min_w_children),
        (snapped.y - new_rect.origin.y)
            .max(MIN_GROUP_HEIGHT)
            .max(min_h_children),
    )
}
