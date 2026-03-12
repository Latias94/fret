use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

use super::PreparedEdgePaintFrame;

pub(super) fn prepare_edge_paint_frame<M: NodeGraphCanvasMiddleware, H: UiHost>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    render: &RenderData,
    zoom: f32,
) -> PreparedEdgePaintFrame {
    let interaction_hint = if let Some(skin) = canvas.skin.as_ref() {
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                skin.interaction_chrome_hint(graph, &canvas.style)
            })
            .ok()
            .unwrap_or_default()
    } else {
        crate::ui::InteractionChromeHint::default()
    };

    let custom_paths = canvas.collect_custom_edge_paths(&*cx.app, &render.edges, zoom);
    let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
    let batches = canvas.prepare_edge_paint_batches(snapshot, render, &custom_paths, zoom);

    PreparedEdgePaintFrame {
        interaction_hint,
        custom_paths,
        bezier_steps,
        batches,
    }
}
