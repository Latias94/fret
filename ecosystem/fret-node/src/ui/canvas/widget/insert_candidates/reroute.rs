use super::super::*;

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

#[cfg(test)]
mod tests;
