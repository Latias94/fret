mod list;
mod menu;
mod reroute;

use super::*;

#[allow(dead_code)]
pub(super) fn reroute_insert_candidate() -> InsertNodeCandidate {
    reroute::reroute_insert_candidate()
}

#[allow(dead_code)]
pub(super) fn prepend_reroute_candidate(
    candidates: Vec<InsertNodeCandidate>,
) -> Vec<InsertNodeCandidate> {
    reroute::prepend_reroute_candidate(candidates)
}

pub(super) fn build_insert_candidate_menu_items(
    candidates: &[InsertNodeCandidate],
) -> Vec<NodeGraphContextMenuItem> {
    menu::build_insert_candidate_menu_items(candidates)
}

pub(in crate::ui::canvas::widget) fn build_insert_candidate_menu_item(
    candidate_ix: usize,
    label: Arc<str>,
    enabled: bool,
) -> NodeGraphContextMenuItem {
    menu::build_insert_candidate_menu_item(candidate_ix, label, enabled)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn list_background_insert_candidates<H: UiHost>(
        &mut self,
        host: &mut H,
    ) -> Vec<InsertNodeCandidate> {
        list::list_background_insert_candidates(self, host)
    }

    pub(super) fn list_connection_insert_candidates<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
    ) -> Vec<InsertNodeCandidate> {
        list::list_connection_insert_candidates(self, host, from)
    }

    pub(super) fn list_edge_insert_candidates<H: UiHost>(
        &mut self,
        host: &mut H,
        edge: EdgeId,
    ) -> Vec<InsertNodeCandidate> {
        list::list_edge_insert_candidates(self, host, edge)
    }
}
