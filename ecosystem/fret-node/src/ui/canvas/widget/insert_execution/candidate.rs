use super::super::*;

pub(in super::super) fn is_reroute_insert_candidate(candidate: &InsertNodeCandidate) -> bool {
    insert_execution_point::is_reroute_insert_candidate(candidate)
}
