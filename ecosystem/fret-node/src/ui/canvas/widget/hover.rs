mod hit;
mod state;
mod target;

use fret_core::Point;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn update_hover_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    let mut new_hover_anchor = None;
    if super::interaction_gate::allow_edge_hover_anchor(&canvas.interaction) {
        let target_edge = self::target::hover_anchor_target_edge(&canvas.interaction, snapshot);
        if let Some(edge_id) = target_edge {
            new_hover_anchor =
                self::hit::hit_hover_edge_anchor(canvas, cx.app, snapshot, position, zoom, edge_id);
        }
    }

    let new_hover = if new_hover_anchor.is_some() {
        None
    } else {
        self::hit::hit_hover_edge(canvas, cx.app, snapshot, position, zoom)
    };

    let (anchor_changed, edge_changed) =
        self::state::sync_hover_edge_state(&mut canvas.interaction, new_hover_anchor, new_hover);
    if anchor_changed {
        super::paint_invalidation::invalidate_paint(cx);
    }
    if edge_changed {
        super::paint_invalidation::invalidate_paint(cx);
    }
}
