use super::command_ui::finish_command_paint;
use super::*;

pub(super) fn cmd_open_insert_node<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let at = canvas
        .interaction
        .last_canvas_pos
        .or_else(|| {
            insert_picker_fallback_canvas_point::<M>(snapshot, canvas.interaction.last_bounds)
        })
        .unwrap_or_default();
    canvas.open_insert_node_picker(cx.app, at);
    finish_command_paint(cx)
}

fn insert_picker_fallback_canvas_point<M: NodeGraphCanvasMiddleware>(
    snapshot: &ViewSnapshot,
    bounds: Option<Rect>,
) -> Option<CanvasPoint> {
    let bounds = bounds?;
    let center = Point::new(
        Px(bounds.origin.x.0 + 0.5 * bounds.size.width.0),
        Px(bounds.origin.y.0 + 0.5 * bounds.size.height.0),
    );
    Some(NodeGraphCanvasWith::<M>::screen_to_canvas(
        bounds,
        center,
        snapshot.pan,
        snapshot.zoom,
    ))
}
