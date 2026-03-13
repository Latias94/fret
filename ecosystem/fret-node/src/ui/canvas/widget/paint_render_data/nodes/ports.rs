use super::super::*;
use crate::ui::PortChromeHint;

#[allow(clippy::too_many_arguments)]
pub(super) fn append_node_port_render_data<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    geom: &CanvasGeometry,
    presenter: &dyn NodeGraphPresenter,
    out: &mut RenderData,
    inputs: &[PortId],
    outputs: &[PortId],
    zoom: f32,
    node_width: Px,
    label_overhead: f32,
) {
    let screen_w = node_width.0 * zoom;
    let screen_max = (screen_w - label_overhead).max(0.0);
    let max_w = Px(screen_max / zoom);

    for port_id in inputs.iter().chain(outputs.iter()).copied() {
        let Some(handle) = geom.ports.get(&port_id) else {
            continue;
        };
        out.port_centers.insert(port_id, handle.center);
        out.port_labels.insert(
            port_id,
            PortLabelRender {
                label: presenter.port_label(graph, port_id),
                dir: handle.dir,
                max_width: max_w,
            },
        );
        let color = presenter.port_color(graph, port_id, &canvas.style);
        let hint = if let Some(skin) = canvas.skin.as_ref() {
            skin.port_chrome_hint(graph, port_id, &canvas.style, color)
        } else {
            PortChromeHint::default()
        };
        let fill = hint.fill.unwrap_or(color);
        out.pins.push((port_id, handle.bounds, fill, hint));
    }
}
