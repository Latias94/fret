use super::super::*;
use crate::ui::NodeChromeHint;

#[allow(clippy::too_many_arguments)]
pub(super) fn append_node_render_data<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    geom: &CanvasGeometry,
    presenter: &dyn NodeGraphPresenter,
    out: &mut RenderData,
    node: GraphNodeId,
    is_selected: bool,
    zoom: f32,
    label_overhead: f32,
) {
    let Some(node_geom) = geom.nodes.get(&node) else {
        return;
    };

    let hint = if let Some(skin) = canvas.skin.as_ref() {
        skin.node_chrome_hint(graph, node, &canvas.style, is_selected)
    } else {
        NodeChromeHint::default()
    };
    let title = presenter.node_title(graph, node);
    let (inputs, outputs) = node_ports(graph, node);
    let pin_rows = inputs.len().max(outputs.len());
    let body = presenter.node_body_label(graph, node);
    let resize_handles = presenter.node_resize_handles(graph, node, &canvas.style);
    out.nodes.push((
        node,
        node_geom.rect,
        is_selected,
        title,
        body,
        pin_rows,
        resize_handles,
        hint,
    ));
    out.metrics.node_visible = out.metrics.node_visible.saturating_add(1);

    super::ports::append_node_port_render_data(
        canvas,
        graph,
        geom,
        presenter,
        out,
        &inputs,
        &outputs,
        zoom,
        node_geom.rect.size.width,
        label_overhead,
    );
}
