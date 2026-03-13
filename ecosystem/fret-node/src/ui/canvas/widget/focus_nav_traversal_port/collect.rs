use crate::ui::canvas::widget::*;

pub(super) fn candidate_ports<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    focused_node: GraphNodeId,
    wire_dir: Option<PortDirection>,
) -> Vec<PortId> {
    canvas
        .graph
        .read_ref(host, |g| {
            let (inputs, outputs) = node_ports(g, focused_node);
            let mut ports = Vec::with_capacity(inputs.len() + outputs.len());
            ports.extend(inputs);
            ports.extend(outputs);

            if let Some(wire_dir) = wire_dir {
                let want = match wire_dir {
                    PortDirection::In => PortDirection::Out,
                    PortDirection::Out => PortDirection::In,
                };
                ports.retain(|id| g.ports.get(id).is_some_and(|p| p.dir == want));
            }

            ports
        })
        .ok()
        .unwrap_or_default()
}
