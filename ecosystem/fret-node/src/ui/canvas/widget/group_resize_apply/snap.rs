use fret_core::Modifiers;

use crate::core::{CanvasPoint, CanvasRect};

use super::{
    MIN_GROUP_HEIGHT, MIN_GROUP_WIDTH, NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot,
};

pub(super) fn allow_group_resize_snap(snapshot: &ViewSnapshot, modifiers: Modifiers) -> bool {
    !modifiers.alt && !modifiers.alt_gr && snapshot.interaction.snap_to_grid
}

pub(super) fn snapped_group_resize_size<M: NodeGraphCanvasMiddleware>(
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
