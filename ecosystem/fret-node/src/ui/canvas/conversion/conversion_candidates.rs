use super::*;

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
