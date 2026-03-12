use super::*;
use crate::core::NodeKindKey;
use serde_json::Value;

pub(super) fn regular_candidate() -> InsertNodeCandidate {
    InsertNodeCandidate {
        kind: NodeKindKey::new("regular"),
        label: Arc::<str>::from("Regular"),
        enabled: true,
        template: None,
        payload: Value::Null,
    }
}
