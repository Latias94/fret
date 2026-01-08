use fret_core::Point;
use fret_ui::UiHost;

use crate::core::EdgeId;

use super::super::state::ViewSnapshot;
use super::NodeGraphCanvas;

pub(super) fn update_hover_edge<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    let new_hover = {
        let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
        let this = &*canvas;
        let index = index.clone();
        this.graph
            .read_ref(cx.app, |graph| {
                let mut scratch: Vec<EdgeId> = Vec::new();
                this.hit_edge(
                    graph,
                    snapshot,
                    geom.as_ref(),
                    index.as_ref(),
                    position,
                    zoom,
                    &mut scratch,
                )
            })
            .ok()
            .flatten()
    };

    if canvas.interaction.hover_edge != new_hover {
        canvas.interaction.hover_edge = new_hover;
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }
}
