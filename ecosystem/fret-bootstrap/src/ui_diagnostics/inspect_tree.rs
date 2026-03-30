use std::collections::{HashMap, HashSet};

use fret_core::{SemanticsNode, SemanticsSnapshot};
use slotmap::Key as _;

use super::selector::SemanticsIndex;

#[derive(Debug, Default, Clone)]
pub(super) struct InspectTreeModel {
    pub(super) lines: Vec<String>,
    pub(super) flat_node_ids: Vec<u64>,
}

fn push_node_line(
    out_lines: &mut Vec<String>,
    out_ids: &mut Vec<u64>,
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
    out_lines.push(line);
    out_ids.push(node_id);
}

pub(super) fn build_inspect_tree_model(
    snapshot: &SemanticsSnapshot,
    index: &SemanticsIndex<'_>,
    expanded: &HashSet<u64>,
    selected_node_id: Option<u64>,
    redact_text: bool,
) -> InspectTreeModel {
    // Keep this bounded; this is an in-app debug overlay and should remain cheap.
    const MAX_NODES: usize = 5000;
    const MAX_DEPTH: usize = 64;

    let mut out: Vec<String> = Vec::new();
    let mut ids: Vec<u64> = Vec::new();
    out.push(
        "tree: (Ctrl/Cmd+T toggle, Up/Down select, Left collapse, Right expand, Enter lock)"
            .to_string(),
    );

    let mut children: HashMap<u64, Vec<u64>> = HashMap::new();
    for node in &snapshot.nodes {
        let id = node.id.data().as_ffi();
        if !index.is_selectable(id) {
            continue;
        }
        let Some(parent) = node.parent else {
            continue;
        };
        let parent = parent.data().as_ffi();
        if !index.is_selectable(parent) {
            continue;
        }
        children.entry(parent).or_default().push(id);
    }
    for v in children.values_mut() {
        v.sort_unstable();
    }

    let mut roots: Vec<(u32, u64)> = snapshot
        .roots
        .iter()
        .filter(|r| r.visible)
        .map(|r| (r.z_index, r.root.data().as_ffi()))
        .collect();
    roots.sort_by(|(za, a), (zb, b)| zb.cmp(za).then(a.cmp(b)));

    let selected = selected_node_id.and_then(|id| index.by_id.get(&id).copied());
    if let Some(sel) = selected {
        out.push(format!(
            "selected: {} z={} node={}",
            crate::ui_diagnostics::semantics_role_label(sel.role),
            index.root_z_for(sel.id.data().as_ffi()),
            sel.id.data().as_ffi()
        ));
    } else {
        out.push("selected: <none>".to_string());
    }

    for (z, root_id) in roots {
        if ids.len() >= MAX_NODES {
            out.push(format!("... truncated after {MAX_NODES} nodes"));
            break;
        }

        out.push(String::new());
        out.push(format!("root z={z} node={root_id}"));

        let Some(_root) = index.by_id.get(&root_id).copied() else {
            out.push("- <missing root>".to_string());
            continue;
        };

        let mut stack: Vec<(u64, usize, bool)> = vec![(root_id, 0, true)];
        while let Some((id, depth, is_root_line)) = stack.pop() {
            if ids.len() >= MAX_NODES {
                out.push(format!("... truncated after {MAX_NODES} nodes"));
                break;
            }

            let Some(node) = index.by_id.get(&id).copied() else {
                continue;
            };

            let has_children = children.get(&id).is_some_and(|v| !v.is_empty());
            let is_expanded = expanded.contains(&id);
            let exp = if has_children {
                if is_expanded { "▾" } else { "▸" }
            } else {
                " "
            };
            let selected = selected_node_id.is_some_and(|sid| sid == id);
            let marker = if selected { "> " } else { "  " };
            let indent = "  ".repeat(depth.min(MAX_DEPTH));
            let line_prefix = if is_root_line {
                format!("- {marker}{indent}{exp} ")
            } else {
                format!("  {marker}{indent}{exp} ")
            };

            push_node_line(&mut out, &mut ids, &line_prefix, node, index, redact_text);

            if has_children && is_expanded
                && let Some(kids) = children.get(&id) {
                    for child in kids.iter().rev().copied() {
                        stack.push((child, depth.saturating_add(1), false));
                    }
                }
        }
    }

    InspectTreeModel {
        lines: out,
        flat_node_ids: ids,
    }
}
