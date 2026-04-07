use super::super::*;

pub(in crate::ui::canvas::widget) fn build_insert_candidate_menu_item(
    candidate_ix: usize,
    label: Arc<str>,
    enabled: bool,
) -> NodeGraphContextMenuItem {
    NodeGraphContextMenuItem {
        label,
        enabled,
        action: NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
    }
}

pub(super) fn build_insert_candidate_menu_items(
    candidates: &[InsertNodeCandidate],
) -> Vec<NodeGraphContextMenuItem> {
    candidates
        .iter()
        .enumerate()
        .map(|(candidate_ix, candidate)| {
            build_insert_candidate_menu_item(
                candidate_ix,
                candidate.label.clone(),
                candidate.enabled,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests;
