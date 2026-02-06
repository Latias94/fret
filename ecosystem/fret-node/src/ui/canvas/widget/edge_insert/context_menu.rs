use super::prelude::*;

pub(in super::super) fn open_edge_insert_context_menu<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    edge: EdgeId,
    invoked_at: Point,
) {
    let candidates: Vec<InsertNodeCandidate> = {
        let presenter = &mut *canvas.presenter;
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                presenter.list_insertable_nodes_for_edge(graph, edge)
            })
            .ok()
            .unwrap_or_default()
    };

    let mut menu_candidates: Vec<InsertNodeCandidate> = Vec::new();
    menu_candidates.push(InsertNodeCandidate {
        kind: NodeKindKey::new(REROUTE_KIND),
        label: Arc::<str>::from("Reroute"),
        enabled: true,
        template: None,
        payload: serde_json::Value::Null,
    });
    menu_candidates.extend(candidates);

    let mut items: Vec<NodeGraphContextMenuItem> = Vec::new();
    for (ix, c) in menu_candidates.iter().enumerate() {
        items.push(NodeGraphContextMenuItem {
            label: c.label.clone(),
            enabled: c.enabled,
            action: NodeGraphContextMenuAction::InsertNodeCandidate(ix),
        });
    }

    let snapshot = canvas.sync_view_state(cx.app);
    let origin = canvas.clamp_context_menu_origin(invoked_at, items.len(), cx.bounds, &snapshot);
    let active_item = items.iter().position(|it| it.enabled).unwrap_or(0);
    canvas.interaction.context_menu = Some(ContextMenuState {
        origin,
        invoked_at,
        target: ContextMenuTarget::EdgeInsertNodePicker(edge),
        items,
        candidates: menu_candidates,
        hovered_item: None,
        active_item,
        typeahead: String::new(),
    });
}
