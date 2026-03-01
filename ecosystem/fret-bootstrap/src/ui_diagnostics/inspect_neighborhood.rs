use fret_core::{SemanticsNode, SemanticsSnapshot};
use slotmap::Key as _;

use super::selector::SemanticsIndex;

fn push_node_brief(
    out: &mut Vec<String>,
    prefix: &str,
    node: &SemanticsNode,
    index: &SemanticsIndex<'_>,
    redact_text: bool,
) {
    let role = crate::ui_diagnostics::semantics_role_label(node.role);
    let node_id = node.id.data().as_ffi();
    let mut line = format!(
        "{prefix}{role} z={} node={node_id}",
        index.root_z_for(node_id)
    );
    if let Some(test_id) = node.test_id.as_deref() {
        line.push_str(&format!(
            " test_id={}",
            crate::ui_diagnostics::truncate_debug_value(test_id, 48)
        ));
    }
    if !redact_text && let Some(label) = node.label.as_deref() {
        line.push_str(&format!(
            " label={}",
            crate::ui_diagnostics::truncate_debug_value(label, 48)
        ));
    }
    out.push(line);
}

pub(super) fn build_inspect_neighborhood_lines(
    snapshot: &SemanticsSnapshot,
    index: &SemanticsIndex<'_>,
    focus_node_id: Option<u64>,
    query: Option<&str>,
    redact_text: bool,
) -> Vec<String> {
    const MAX_ITEMS: usize = 8;
    const MAX_MATCHES: usize = 10;

    let mut out: Vec<String> = Vec::new();
    out.push("neighborhood:".to_string());

    let query = query.map(|s| s.trim()).filter(|s| !s.is_empty());

    if let Some(q) = query {
        let surface = if redact_text {
            "test_id"
        } else {
            "test_id + label"
        };
        out.push(format!("search: {q} ({surface})"));
    } else {
        out.push("search: <type to filter> (Backspace delete, Enter clear)".to_string());
    }

    let Some(focus_id) = focus_node_id else {
        out.push("focus: <none>".to_string());
        return out;
    };
    let Some(focus) = snapshot
        .nodes
        .iter()
        .find(|n| n.id.data().as_ffi() == focus_id)
    else {
        out.push("focus: <missing>".to_string());
        return out;
    };

    let parent_id = focus.parent.map(|p| p.data().as_ffi());

    out.push("parent:".to_string());
    if let Some(pid) = parent_id {
        if let Some(parent) = snapshot.nodes.iter().find(|n| n.id.data().as_ffi() == pid) {
            push_node_brief(&mut out, "- ", parent, index, redact_text);
        } else {
            out.push(format!("- <missing node={pid}>"));
        }
    } else {
        out.push("- <none>".to_string());
    }

    let mut siblings_total: u32 = 0;
    let mut children_total: u32 = 0;
    let mut matches_total: u32 = 0;
    let mut siblings: Vec<&SemanticsNode> = Vec::new();
    let mut children: Vec<&SemanticsNode> = Vec::new();
    let mut matches: Vec<&SemanticsNode> = Vec::new();

    for node in &snapshot.nodes {
        let id = node.id.data().as_ffi();
        if !index.is_selectable(id) {
            continue;
        }

        let node_parent = node.parent.map(|p| p.data().as_ffi());
        if parent_id.is_some_and(|pid| node_parent == Some(pid)) {
            siblings_total = siblings_total.saturating_add(1);
            if siblings.len() < MAX_ITEMS {
                siblings.push(node);
            }
        }
        if node_parent == Some(focus_id) {
            children_total = children_total.saturating_add(1);
            if children.len() < MAX_ITEMS {
                children.push(node);
            }
        }

        if let Some(q) = query {
            let mut ok = node
                .test_id
                .as_deref()
                .is_some_and(|test_id| test_id.contains(q));
            if !ok && !redact_text {
                ok = node.label.as_deref().is_some_and(|label| label.contains(q));
            }
            if ok {
                matches_total = matches_total.saturating_add(1);
                if matches.len() < MAX_MATCHES {
                    matches.push(node);
                }
            }
        }
    }

    out.push(format!(
        "siblings: {siblings_total} (showing {})",
        siblings.len()
    ));
    for s in siblings {
        let star = if s.id.data().as_ffi() == focus_id {
            "* "
        } else {
            "  "
        };
        push_node_brief(&mut out, &format!("- {star}"), s, index, redact_text);
    }

    out.push(format!(
        "children: {children_total} (showing {})",
        children.len()
    ));
    for c in children {
        push_node_brief(&mut out, "- ", c, index, redact_text);
    }

    if query.is_some() {
        out.push(format!(
            "matches: {matches_total} (showing {})",
            matches.len()
        ));
        for m in matches {
            push_node_brief(&mut out, "- ", m, index, redact_text);
        }
    }

    out
}
