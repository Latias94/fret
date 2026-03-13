use crate::core::CanvasPoint;
use crate::ui::canvas::widget::*;

pub(super) fn apply_node_focus<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    next: GraphNodeId,
) {
    super::super::focus_session::focus_node(&mut canvas.interaction, next);
    canvas.update_view_state(host, |state| {
        super::super::focus_session::select_only_node(state, next, true);
    });

    let snapshot = canvas.sync_view_state(host);
    if snapshot.interaction.auto_pan.on_node_focus {
        canvas.stop_viewport_animation_timer(host);
        let (geom, _index) = canvas.canvas_derived(&*host, &snapshot);
        if let Some(node_geom) = geom.nodes.get(&next) {
            let rect = node_geom.rect;
            let center = CanvasPoint {
                x: rect.origin.x.0 + 0.5 * rect.size.width.0,
                y: rect.origin.y.0 + 0.5 * rect.size.height.0,
            };
            canvas.ensure_canvas_point_visible(host, &snapshot, center);
        }
    }
}
