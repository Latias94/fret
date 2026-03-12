use super::super::*;

pub(super) fn searcher_row_activation_item(
    searcher: &SearcherState,
    row_ix: usize,
) -> Option<NodeGraphContextMenuItem> {
    let row = searcher.rows.get(row_ix)?.clone();
    let SearcherRowKind::Candidate { candidate_ix } = row.kind else {
        return None;
    };
    if !row.enabled {
        return None;
    }

    Some(NodeGraphContextMenuItem {
        label: row.label,
        enabled: true,
        action: NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
    })
}

#[cfg(test)]
mod tests;
