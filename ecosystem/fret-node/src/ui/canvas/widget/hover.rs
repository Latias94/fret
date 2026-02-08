use fret_core::Point;
use fret_ui::UiHost;

use crate::core::EdgeId;
use crate::rules::EdgeEndpoint;

use super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn update_hover_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    let mut new_hover_anchor: Option<(EdgeId, EdgeEndpoint)> = None;
    if canvas.interaction.wire_drag.is_none()
        && canvas.interaction.insert_node_drag_preview.is_none()
        && canvas.interaction.pending_edge_insert_drag.is_none()
        && canvas.interaction.edge_insert_drag.is_none()
        && canvas.interaction.edge_drag.is_none()
        && canvas.interaction.node_drag.is_none()
        && canvas.interaction.node_resize.is_none()
        && canvas.interaction.group_drag.is_none()
        && canvas.interaction.group_resize.is_none()
        && canvas.interaction.marquee.is_none()
        && canvas.interaction.context_menu.is_none()
        && canvas.interaction.searcher.is_none()
    {
        let target_edge = canvas
            .interaction
            .focused_edge
            .or_else(|| (snapshot.selected_edges.len() == 1).then(|| snapshot.selected_edges[0]));
        if let Some(edge_id) = target_edge {
            let hit = {
                let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
                let this = &*canvas;
                let index = index.clone();
                this.graph
                    .read_ref(cx.app, |graph| {
                        let mut scratch = HitTestScratch::default();
                        let mut ctx =
                            HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                        this.hit_edge_focus_anchor(graph, snapshot, &mut ctx, position)
                            .filter(|(id, ..)| *id == edge_id)
                            .map(|(id, endpoint, _fixed)| (id, endpoint))
                    })
                    .ok()
                    .flatten()
            };
            new_hover_anchor = hit;
        }
    }

    let new_hover = if new_hover_anchor.is_some() {
        None
    } else {
        let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
        let this = &*canvas;
        let index = index.clone();
        this.graph
            .read_ref(cx.app, |graph| {
                let mut scratch = HitTestScratch::default();
                let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                this.hit_edge(graph, snapshot, &mut ctx, position)
            })
            .ok()
            .flatten()
    };

    if canvas.interaction.hover_edge_anchor != new_hover_anchor {
        canvas.interaction.hover_edge_anchor = new_hover_anchor;
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }

    if canvas.interaction.hover_edge != new_hover {
        canvas.interaction.hover_edge = new_hover;
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }
}
