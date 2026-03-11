mod input;

use super::*;

#[derive(Debug)]
pub(super) struct NodeMeasureInput {
    pub(super) node: GraphNodeId,
    pub(super) title: Arc<str>,
    pub(super) inputs: Vec<Arc<str>>,
    pub(super) outputs: Vec<Arc<str>>,
}

pub(super) fn collect_node_measure_inputs<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) -> Option<Vec<NodeMeasureInput>> {
    let presenter: &dyn NodeGraphPresenter = &*canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut out: Vec<NodeMeasureInput> = Vec::with_capacity(graph.nodes.len());

            for node_id in graph.nodes.keys().copied() {
                out.push(input::collect_node_measure_input(presenter, graph, node_id));
            }

            out
        })
        .ok()
}
