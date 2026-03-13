use super::*;

pub(crate) fn plan_insert_conversion(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    style: &NodeGraphStyle,
    zoom: f32,
    from: PortId,
    to: PortId,
    at: CanvasPoint,
    template: &InsertNodeTemplate,
) -> ConnectPlan {
    let zoom = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    let inputs = template
        .ports
        .iter()
        .filter(|port| port.dir == PortDirection::In)
        .count();
    let outputs = template
        .ports
        .iter()
        .filter(|port| port.dir == PortDirection::Out)
        .count();
    let (width_px, height_px) = node_size_default_px(inputs, outputs, style);
    let height = height_px / zoom;
    let width = width_px / zoom;
    let default_at = CanvasPoint {
        x: at.x - 0.5 * width,
        y: at.y - 0.5 * height,
    };

    let at = presenter.conversion_insert_position(graph, from, to, default_at, template);
    let spec = match template.instantiate(at) {
        Ok(spec) => spec,
        Err(err) => return ConnectPlan::reject(err),
    };

    plan_connect_by_inserting_node(graph, from, to, EdgeId::new(), EdgeId::new(), spec)
}

pub(crate) fn try_auto_insert_conversion(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    style: &NodeGraphStyle,
    zoom: f32,
    from: PortId,
    to: PortId,
    at: CanvasPoint,
    conversions: &[InsertNodeTemplate],
) -> Option<ConnectPlan> {
    if conversions.len() != 1 {
        return None;
    }
    let template = &conversions[0];
    Some(plan_insert_conversion(
        presenter, graph, style, zoom, from, to, at, template,
    ))
}
