use super::super::*;

pub(super) fn collect_node_measure_input(
    presenter: &dyn NodeGraphPresenter,
    graph: &Graph,
    node_id: GraphNodeId,
) -> super::NodeMeasureInput {
    let title = presenter.node_title(graph, node_id);
    let (inputs, outputs) = node_ports(graph, node_id);
    let inputs = inputs
        .into_iter()
        .map(|port| presenter.port_label(graph, port))
        .collect();
    let outputs = outputs
        .into_iter()
        .map(|port| presenter.port_label(graph, port))
        .collect();
    super::NodeMeasureInput {
        node: node_id,
        title,
        inputs,
        outputs,
    }
}
