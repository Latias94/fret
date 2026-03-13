use super::super::*;

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

#[cfg(test)]
mod tests;
