use super::super::*;
use super::dispatch::DirectCommandRoute;

pub(super) fn handle_direct_insert_command<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
    route: DirectCommandRoute,
) -> bool {
    match route {
        DirectCommandRoute::OpenInsertNode => canvas.cmd_open_insert_node(cx, snapshot),
        DirectCommandRoute::OpenSplitEdgeInsertNode => {
            canvas.cmd_open_split_edge_insert_node(cx, snapshot)
        }
        DirectCommandRoute::InsertReroute => canvas.cmd_insert_reroute(cx, snapshot),
        DirectCommandRoute::OpenConversionPicker => canvas.cmd_open_conversion_picker(cx, snapshot),
        _ => false,
    }
}
