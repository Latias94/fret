//! Typed conversion helpers for connection workflows.

use crate::core::{CanvasPoint, EdgeId, Graph, PortDirection, PortId};
use crate::rules::{ConnectPlan, InsertNodeTemplate, plan_connect_by_inserting_node};
use crate::ui::presenter::{InsertNodeCandidate, NodeGraphPresenter};
use crate::ui::style::NodeGraphStyle;

pub(crate) fn is_convertible(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    from: PortId,
    to: PortId,
) -> bool {
    !presenter.list_conversions(graph, from, to).is_empty()
}

pub(crate) fn build_picker_candidates(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    from: PortId,
    to: PortId,
    conversions: Vec<InsertNodeTemplate>,
) -> Vec<InsertNodeCandidate> {
    let mut out: Vec<InsertNodeCandidate> = Vec::new();
    for template in conversions {
        let label = presenter.conversion_label(graph, from, to, &template);
        out.push(InsertNodeCandidate {
            kind: template.kind.clone(),
            label,
            enabled: true,
            template: Some(template),
            payload: serde_json::Value::Null,
        });
    }
    out
}

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
        .filter(|p| p.dir == PortDirection::In)
        .count();
    let outputs = template
        .ports
        .iter()
        .filter(|p| p.dir == PortDirection::Out)
        .count();
    let rows = inputs.max(outputs) as f32;
    let base = style.node_header_height + 2.0 * style.node_padding;
    let h = (base + rows * style.pin_row_height) / zoom;
    let w = style.node_width / zoom;
    let default_at = CanvasPoint {
        x: at.x - 0.5 * w,
        y: at.y - 0.5 * h,
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
