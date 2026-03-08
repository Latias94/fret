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
                let title = presenter.node_title(graph, node_id);
                let (inputs, outputs) = node_ports(graph, node_id);
                let inputs = inputs
                    .into_iter()
                    .map(|p| presenter.port_label(graph, p))
                    .collect();
                let outputs = outputs
                    .into_iter()
                    .map(|p| presenter.port_label(graph, p))
                    .collect();
                out.push(NodeMeasureInput {
                    node: node_id,
                    title,
                    inputs,
                    outputs,
                });
            }

            out
        })
        .ok()
}
