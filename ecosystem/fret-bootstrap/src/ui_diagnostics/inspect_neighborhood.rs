use fret_core::{SemanticsNode, SemanticsSnapshot};
use slotmap::Key as _;

use super::selector::SemanticsIndex;

#[derive(Debug, Default, Clone)]
pub(super) struct InspectNeighborhoodModel {
    pub(super) lines: Vec<String>,
    pub(super) match_node_ids: Vec<u64>,
}

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

pub(super) fn build_inspect_neighborhood_model(
    snapshot: &SemanticsSnapshot,
    index: &SemanticsIndex<'_>,
    focus_node_id: Option<u64>,
    query: Option<&str>,
    redact_text: bool,
    selected_match_index: Option<usize>,
) -> InspectNeighborhoodModel {
    const MAX_ITEMS: usize = 8;
    const MAX_MATCHES: usize = 10;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct MatchKey<'a> {
        surface_rank: u8,
        match_rank: u8,
        value: &'a str,
        node_id: u64,
    }

    fn match_rank(haystack: &str, needle: &str) -> Option<u8> {
        if haystack == needle {
            Some(0)
        } else if haystack.starts_with(needle) {
            Some(1)
        } else if haystack.contains(needle) {
            Some(2)
        } else {
            None
        }
    }

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

    let (focus_id, parent_id, focus_valid) = if let Some(focus_id) = focus_node_id {
        let focus = snapshot
            .nodes
            .iter()
            .find(|n| n.id.data().as_ffi() == focus_id);
        let parent_id = focus.and_then(|f| f.parent.map(|p| p.data().as_ffi()));
        (Some(focus_id), parent_id, focus.is_some())
    } else {
        out.push("focus: <none>".to_string());
        (None, None, false)
    };

    if focus_id.is_some() && !focus_valid {
        out.push("focus: <missing>".to_string());
    }

    let mut siblings_total: u32 = 0;
    let mut children_total: u32 = 0;
    let mut matches_total: u32 = 0;
    let mut siblings: Vec<&SemanticsNode> = Vec::new();
    let mut children: Vec<&SemanticsNode> = Vec::new();
    let mut matches: Vec<(MatchKey<'_>, &SemanticsNode)> = Vec::new();

    for node in &snapshot.nodes {
        let id = node.id.data().as_ffi();
        if !index.is_selectable(id) {
            continue;
        }

        let node_parent = node.parent.map(|p| p.data().as_ffi());
        if focus_valid {
            if parent_id.is_some_and(|pid| node_parent == Some(pid)) {
                siblings_total = siblings_total.saturating_add(1);
                if siblings.len() < MAX_ITEMS {
                    siblings.push(node);
                }
            }
            if node_parent == focus_id {
                children_total = children_total.saturating_add(1);
                if children.len() < MAX_ITEMS {
                    children.push(node);
                }
            }
        }

        if let Some(q) = query {
            let mut best_key: Option<MatchKey<'_>> = node.test_id.as_deref().and_then(|test_id| {
                match_rank(test_id, q).map(|match_rank| MatchKey {
                    surface_rank: 0,
                    match_rank,
                    value: test_id,
                    node_id: id,
                })
            });

            if !redact_text
                && let Some(label) = node.label.as_deref()
                && let Some(match_rank) = match_rank(label, q)
            {
                let label_key = MatchKey {
                    surface_rank: 1,
                    match_rank,
                    value: label,
                    node_id: id,
                };
                if best_key.is_none_or(|k| label_key < k) {
                    best_key = Some(label_key);
                }
            }

            if let Some(key) = best_key {
                matches_total = matches_total.saturating_add(1);
                if matches.len() < MAX_MATCHES {
                    matches.push((key, node));
                    matches.sort_by(|(a, _), (b, _)| a.cmp(b));
                } else {
                    let worst = matches.last().map(|(k, _)| *k);
                    if worst.is_some_and(|worst| key < worst) {
                        matches.push((key, node));
                        matches.sort_by(|(a, _), (b, _)| a.cmp(b));
                        matches.truncate(MAX_MATCHES);
                    }
                }
            }
        }
    }

    if focus_valid {
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

        out.push(format!(
            "siblings: {siblings_total} (showing {})",
            siblings.len()
        ));
        for s in siblings {
            let star = if focus_id.is_some_and(|id| s.id.data().as_ffi() == id) {
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
    }

    if query.is_some() {
        let shown = matches.len();
        let selected = (shown > 0).then(|| {
            selected_match_index
                .unwrap_or(0)
                .min(shown.saturating_sub(1))
        });
        let selected_id = selected
            .and_then(|i| matches.get(i))
            .map(|(_, n)| n.id.data().as_ffi());
        let selected_1 = selected.map(|i| i.saturating_add(1)).unwrap_or(0);

        out.push(format!(
            "matches: {matches_total} (showing {shown}) selected={selected_1}/{shown} (Up/Down select, Enter lock, Ctrl/Cmd+Enter lock+copy)",
        ));
        for (_, m) in matches.iter().copied() {
            let is_selected = selected_id.is_some_and(|id| id == m.id.data().as_ffi());
            let prefix = if is_selected { "- > " } else { "-   " };
            push_node_brief(&mut out, prefix, m, index, redact_text);
        }
    }

    let match_node_ids = if query.is_some() {
        matches.iter().map(|(_, n)| n.id.data().as_ffi()).collect()
    } else {
        Default::default()
    };

    InspectNeighborhoodModel {
        lines: out,
        match_node_ids,
    }
}
