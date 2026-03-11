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
mod tests {
    use super::*;

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
}
