use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::json_bundle::SemanticsResolver;

pub(super) fn semantics_diff_detail(
    semantics: &SemanticsResolver<'_>,
    before: &serde_json::Value,
    after: &serde_json::Value,
) -> serde_json::Value {
    let (Some(before_nodes), Some(after_nodes)) = (semantics.nodes(before), semantics.nodes(after))
    else {
        return serde_json::Value::Null;
    };
    semantics_diff_detail_nodes(before_nodes, after_nodes)
}

pub(super) fn semantics_diff_detail_nodes(
    before_nodes: &[serde_json::Value],
    after_nodes: &[serde_json::Value],
) -> serde_json::Value {
    use serde_json::json;

    let mut before_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in before_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        before_by_id.insert(id, node);
    }

    let mut after_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in after_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        after_by_id.insert(id, node);
    }

    let before_ids: HashSet<u64> = before_by_id.keys().copied().collect();
    let after_ids: HashSet<u64> = after_by_id.keys().copied().collect();

    let mut added: Vec<u64> = after_ids.difference(&before_ids).copied().collect();
    let mut removed: Vec<u64> = before_ids.difference(&after_ids).copied().collect();
    added.sort_unstable();
    removed.sort_unstable();

    let mut changed: Vec<(u64, u64)> = Vec::new(); // (score, id)
    for id in before_ids.intersection(&after_ids).copied() {
        let Some(a) = after_by_id.get(&id).copied() else {
            continue;
        };
        let Some(b) = before_by_id.get(&id).copied() else {
            continue;
        };
        let fp_a = semantics_node_fingerprint_json(a);
        let fp_b = semantics_node_fingerprint_json(b);
        if fp_a != fp_b {
            let score = semantics_node_score_json(a);
            changed.push((score, id));
        }
    }
    changed.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    let sample_len = 6usize;

    let added_nodes = added
        .iter()
        .take(sample_len)
        .map(|id| semantics_node_summary_json(*id, after_by_id.get(id).copied()))
        .collect::<Vec<_>>();
    let removed_nodes = removed
        .iter()
        .take(sample_len)
        .map(|id| semantics_node_summary_json(*id, before_by_id.get(id).copied()))
        .collect::<Vec<_>>();
    let changed_nodes = changed
        .iter()
        .take(sample_len)
        .map(|(_score, id)| {
            let before = semantics_node_summary_json(*id, before_by_id.get(id).copied());
            let after = semantics_node_summary_json(*id, after_by_id.get(id).copied());
            json!({ "id": id, "before": before, "after": after })
        })
        .collect::<Vec<_>>();

    json!({
        "counts": {
            "added": added.len(),
            "removed": removed.len(),
            "changed": changed.len(),
        },
        "samples": {
            "added_nodes": added_nodes,
            "removed_nodes": removed_nodes,
            "changed_nodes": changed_nodes,
        }
    })
}

fn semantics_node_summary_json(id: u64, node: Option<&serde_json::Value>) -> serde_json::Value {
    use serde_json::json;
    let Some(node) = node else {
        return json!({ "id": id });
    };

    let role = node.get("role").and_then(|v| v.as_str());
    let parent = node.get("parent").and_then(|v| v.as_u64());
    let test_id = node.get("test_id").and_then(|v| v.as_str());
    let label = node.get("label").and_then(|v| v.as_str());
    let value = node.get("value").and_then(|v| v.as_str());

    let bounds = node.get("bounds").map(|b| {
        json!({
            "x": b.get("x").and_then(|v| v.as_f64()),
            "y": b.get("y").and_then(|v| v.as_f64()),
            "w": b.get("w").and_then(|v| v.as_f64()),
            "h": b.get("h").and_then(|v| v.as_f64()),
        })
    });

    json!({
        "id": id,
        "parent": parent,
        "role": role,
        "test_id": test_id,
        "label": label,
        "value": value,
        "bounds": bounds,
    })
}

pub(super) fn semantics_diff_summary(
    semantics: &SemanticsResolver<'_>,
    before: &serde_json::Value,
    after: &serde_json::Value,
) -> String {
    let (Some(before_nodes), Some(after_nodes)) = (semantics.nodes(before), semantics.nodes(after))
    else {
        return String::new();
    };
    semantics_diff_summary_nodes(before_nodes, after_nodes)
}

pub(super) fn semantics_diff_summary_nodes(
    before_nodes: &[serde_json::Value],
    after_nodes: &[serde_json::Value],
) -> String {

    let mut before_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in before_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        before_by_id.insert(id, node);
    }

    let mut after_by_id: HashMap<u64, &serde_json::Value> = HashMap::new();
    for node in after_nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        after_by_id.insert(id, node);
    }

    let before_ids: HashSet<u64> = before_by_id.keys().copied().collect();
    let after_ids: HashSet<u64> = after_by_id.keys().copied().collect();

    let mut added: Vec<u64> = after_ids.difference(&before_ids).copied().collect();
    let mut removed: Vec<u64> = before_ids.difference(&after_ids).copied().collect();
    added.sort_unstable();
    removed.sort_unstable();

    let mut changed: Vec<(u64, u64, u64)> = Vec::new(); // (score, id, fp_after)
    for id in before_ids.intersection(&after_ids).copied() {
        let Some(a) = after_by_id.get(&id).copied() else {
            continue;
        };
        let Some(b) = before_by_id.get(&id).copied() else {
            continue;
        };
        let fp_a = semantics_node_fingerprint_json(a);
        let fp_b = semantics_node_fingerprint_json(b);
        if fp_a != fp_b {
            // Score heuristic: test_id changes are the most useful to report.
            let score = semantics_node_score_json(a);
            changed.push((score, id, fp_a));
        }
    }

    if added.is_empty() && removed.is_empty() && changed.is_empty() {
        return String::new();
    }

    changed.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

    let mut out = String::new();
    out.push_str("semantics_diff={");
    out.push_str(&format!(
        "added={} removed={} changed={}",
        added.len(),
        removed.len(),
        changed.len()
    ));

    let sample_len = 6usize;
    if !changed.is_empty() {
        out.push_str(" changed_nodes=[");
        for (i, (_score, id, _fp)) in changed.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = after_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if changed.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    if !added.is_empty() {
        out.push_str(" added_nodes=[");
        for (i, id) in added.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = after_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if added.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    if !removed.is_empty() {
        out.push_str(" removed_nodes=[");
        for (i, id) in removed.iter().take(sample_len).enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let node = before_by_id.get(id).copied();
            out.push_str(&semantics_node_label_json(*id, node));
        }
        if removed.len() > sample_len {
            out.push_str(", ...");
        }
        out.push(']');
    }

    out.push('}');
    out
}

fn semantics_node_score_json(node: &serde_json::Value) -> u64 {
    // Higher is "more useful for debugging".
    let mut score: u64 = 0;
    if node.get("test_id").and_then(|v| v.as_str()).is_some() {
        score += 10_000;
    }
    if node.get("label").and_then(|v| v.as_str()).is_some() {
        score += 1_000;
    }
    if node.get("value").and_then(|v| v.as_str()).is_some() {
        score += 500;
    }
    score
}

fn semantics_node_label_json(id: u64, node: Option<&serde_json::Value>) -> String {
    let Some(node) = node else {
        return format!("id={id}");
    };
    let role = node
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let test_id = node
        .get("test_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());
    let label = node
        .get("label")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());
    let value = node
        .get("value")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());

    let mut out = format!("id={id} role={role}");
    if let Some(v) = test_id {
        out.push_str(" test_id=");
        out.push_str(v);
    }
    if let Some(v) = label {
        out.push_str(" label=");
        out.push_str(v);
    }
    if let Some(v) = value {
        out.push_str(" value=");
        out.push_str(v);
    }
    out
}

fn semantics_node_fingerprint_json(node: &serde_json::Value) -> u64 {
    // Use a stable hash for a curated subset of fields.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    node.get("id").and_then(|v| v.as_u64()).hash(&mut hasher);
    node.get("parent")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("role").and_then(|v| v.as_str()).hash(&mut hasher);

    if let Some(bounds) = node.get("bounds") {
        if let Some(v) = bounds.get("x").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("y").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("w").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
        if let Some(v) = bounds.get("h").and_then(|v| v.as_f64()) {
            v.to_bits().hash(&mut hasher);
        }
    }

    if let Some(flags) = node.get("flags") {
        flags
            .get("focused")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("captured")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("disabled")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("selected")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("expanded")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        flags
            .get("checked")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
    }

    node.get("test_id")
        .and_then(|v| v.as_str())
        .hash(&mut hasher);
    node.get("active_descendant")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("pos_in_set")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("set_size")
        .and_then(|v| v.as_u64())
        .hash(&mut hasher);
    node.get("label").and_then(|v| v.as_str()).hash(&mut hasher);
    node.get("value").and_then(|v| v.as_str()).hash(&mut hasher);

    if let Some(actions) = node.get("actions") {
        actions
            .get("focus")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("invoke")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("set_value")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
        actions
            .get("set_text_selection")
            .and_then(|v| v.as_bool())
            .hash(&mut hasher);
    }

    hasher.finish()
}

pub(super) fn semantics_node_y_for_test_id(
    semantics: &SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    test_id: &str,
) -> Option<f64> {
    let node = crate::json_bundle::semantics_node_for_test_id(semantics, snapshot, test_id)?;
    node.get("bounds")
        .and_then(|v| v.get("y"))
        .and_then(|v| v.as_f64())
}

pub(super) fn semantics_node_fields_for_test_id(
    semantics: &SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    test_id: &str,
) -> (Option<f64>, Option<String>, Option<String>) {
    let Some(node) = crate::json_bundle::semantics_node_for_test_id(semantics, snapshot, test_id)
    else {
        return (None, None, None);
    };
    let y = node
        .get("bounds")
        .and_then(|v| v.get("y"))
        .and_then(|v| v.as_f64());
    let label = node
        .get("label")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let value = node
        .get("value")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    (y, label, value)
}

pub(super) fn semantics_node_id_for_test_id(
    semantics: &SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    test_id: &str,
) -> Option<u64> {
    crate::json_bundle::semantics_node_for_test_id(semantics, snapshot, test_id)?
        .get("id")
        .and_then(|v| v.as_u64())
}

pub(super) fn is_descendant(
    mut node: u64,
    ancestor: u64,
    parents: &std::collections::HashMap<u64, u64>,
) -> bool {
    if node == ancestor {
        return true;
    }
    while let Some(parent) = parents.get(&node).copied() {
        if parent == ancestor {
            return true;
        }
        node = parent;
    }
    false
}

pub(super) fn semantics_parent_map(
    semantics: &SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
) -> std::collections::HashMap<u64, u64> {
    let mut parents = std::collections::HashMap::new();
    let Some(nodes) = semantics.nodes(snapshot) else {
        return parents;
    };
    for node in nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        let Some(parent) = node.get("parent").and_then(|v| v.as_u64()) else {
            continue;
        };
        parents.insert(id, parent);
    }
    parents
}
