use super::*;

pub(super) fn reroute_insert_candidate() -> InsertNodeCandidate {
    InsertNodeCandidate {
        kind: NodeKindKey::new(REROUTE_KIND),
        label: Arc::<str>::from("Reroute"),
        enabled: true,
        template: None,
        payload: serde_json::Value::Null,
    }
}

pub(super) fn prepend_reroute_candidate(
    candidates: Vec<InsertNodeCandidate>,
) -> Vec<InsertNodeCandidate> {
    let mut out = Vec::with_capacity(candidates.len() + 1);
    out.push(reroute_insert_candidate());
    out.extend(candidates);
    out
}

pub(super) fn build_insert_candidate_menu_items(
    candidates: &[InsertNodeCandidate],
) -> Vec<NodeGraphContextMenuItem> {
    candidates
        .iter()
        .enumerate()
        .map(|(candidate_ix, candidate)| NodeGraphContextMenuItem {
            label: candidate.label.clone(),
            enabled: candidate.enabled,
            action: NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
        })
        .collect()
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn list_background_insert_candidates<H: UiHost>(
        &mut self,
        host: &mut H,
    ) -> Vec<InsertNodeCandidate> {
        let candidates = {
            let presenter = &mut *self.presenter;
            self.graph
                .read_ref(host, |graph| presenter.list_insertable_nodes(graph))
                .ok()
                .unwrap_or_default()
        };
        prepend_reroute_candidate(candidates)
    }

    pub(super) fn list_connection_insert_candidates<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
    ) -> Vec<InsertNodeCandidate> {
        let candidates = {
            let presenter = &mut *self.presenter;
            self.graph
                .read_ref(host, |graph| {
                    presenter.list_insertable_nodes_for_connection(graph, from)
                })
                .ok()
                .unwrap_or_default()
        };
        prepend_reroute_candidate(candidates)
    }

    pub(super) fn list_edge_insert_candidates<H: UiHost>(
        &mut self,
        host: &mut H,
        edge: EdgeId,
    ) -> Vec<InsertNodeCandidate> {
        let candidates = {
            let presenter = &mut *self.presenter;
            self.graph
                .read_ref(host, |graph| {
                    presenter.list_insertable_nodes_for_edge(graph, edge)
                })
                .ok()
                .unwrap_or_default()
        };
        prepend_reroute_candidate(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::{build_insert_candidate_menu_items, prepend_reroute_candidate};
    use crate::core::NodeKindKey;
    use crate::ui::presenter::{InsertNodeCandidate, NodeGraphContextMenuAction};
    use std::sync::Arc;

    #[test]
    fn prepend_reroute_candidate_places_reroute_first() {
        let candidates = vec![
            InsertNodeCandidate {
                kind: NodeKindKey::new("math.add"),
                label: Arc::<str>::from("Add"),
                enabled: true,
                template: None,
                payload: serde_json::Value::Null,
            },
            InsertNodeCandidate {
                kind: NodeKindKey::new("math.mul"),
                label: Arc::<str>::from("Mul"),
                enabled: false,
                template: None,
                payload: serde_json::Value::Null,
            },
        ];

        let prefixed = prepend_reroute_candidate(candidates);

        assert_eq!(prefixed[0].kind.0.as_str(), crate::REROUTE_KIND);
        assert_eq!(prefixed[1].kind.0.as_str(), "math.add");
        assert_eq!(prefixed[2].kind.0.as_str(), "math.mul");
    }

    #[test]
    fn build_insert_candidate_menu_items_preserves_indexes_and_enabled_state() {
        let candidates = prepend_reroute_candidate(vec![InsertNodeCandidate {
            kind: NodeKindKey::new("math.add"),
            label: Arc::<str>::from("Add"),
            enabled: false,
            template: None,
            payload: serde_json::Value::Null,
        }]);

        let items = build_insert_candidate_menu_items(&candidates);

        assert_eq!(items.len(), 2);
        assert!(matches!(
            items[0].action,
            NodeGraphContextMenuAction::InsertNodeCandidate(0)
        ));
        assert!(matches!(
            items[1].action,
            NodeGraphContextMenuAction::InsertNodeCandidate(1)
        ));
        assert!(items[0].enabled);
        assert!(!items[1].enabled);
        assert_eq!(items[0].label.as_ref(), "Reroute");
    }
}
