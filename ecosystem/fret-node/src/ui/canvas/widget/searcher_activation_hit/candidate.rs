use super::super::*;
use crate::ui::canvas::searcher::SearcherRowKind;

pub(in super::super) fn searcher_candidate_for_row(
    searcher: &SearcherState,
    row_ix: usize,
) -> Option<InsertNodeCandidate> {
    let row = searcher.rows.get(row_ix)?;
    if !row.enabled {
        return None;
    }
    let SearcherRowKind::Candidate { candidate_ix } = row.kind else {
        return None;
    };
    searcher.candidates.get(candidate_ix).cloned()
}

#[cfg(test)]
mod tests;
