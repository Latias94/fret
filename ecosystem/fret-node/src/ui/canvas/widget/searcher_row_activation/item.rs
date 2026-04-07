use super::super::*;

pub(super) fn searcher_row_activation_item(
    searcher: &SearcherState,
    row_ix: usize,
) -> Option<NodeGraphContextMenuItem> {
    let row = searcher.rows.get(row_ix)?.clone();
    if !super::super::searcher_is_selectable_row(&row) {
        return None;
    }

    let SearcherRowKind::Candidate { candidate_ix } = row.kind else {
        return None;
    };
    let item = super::super::build_insert_candidate_menu_item(candidate_ix, row.label, row.enabled);
    item.enabled.then_some(item)
}

#[cfg(test)]
mod tests;
