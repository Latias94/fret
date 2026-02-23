use std::collections::{HashMap, HashSet};

use accesskit::{
    Action, Invalid, Node, NodeId, Rect, Role, TextPosition, TextSelection, Toggled, Tree, TreeId,
    TreeUpdate,
};
use fret_core::{SemanticsNode, SemanticsOrientation, SemanticsRole, SemanticsSnapshot};

use crate::ids::{ROOT_ID, text_run_id_for, to_accesskit_id};
use crate::roles::map_role;

fn px_rect_to_accesskit(bounds: fret_core::Rect, scale_factor: f64) -> Rect {
    let x0 = bounds.origin.x.0 as f64 * scale_factor;
    let y0 = bounds.origin.y.0 as f64 * scale_factor;
    let x1 = (bounds.origin.x.0 + bounds.size.width.0) as f64 * scale_factor;
    let y1 = (bounds.origin.y.0 + bounds.size.height.0) as f64 * scale_factor;
    Rect { x0, y0, x1, y1 }
}

fn choose_visible_roots(snapshot: &SemanticsSnapshot) -> Vec<fret_core::NodeId> {
    let mut roots: Vec<_> = snapshot.roots.iter().filter(|r| r.visible).collect();
    roots.sort_by_key(|r| r.z_index);

    if let Some(barrier_root) = snapshot.barrier_root
        && let Some(barrier) = roots.iter().find(|r| r.root == barrier_root)
    {
        let barrier_z = barrier.z_index;
        return roots
            .into_iter()
            .filter(|r| r.z_index >= barrier_z)
            .map(|r| r.root)
            .collect();
    }

    roots.into_iter().map(|r| r.root).collect()
}

fn build_children_index(
    nodes: &[SemanticsNode],
) -> HashMap<fret_core::NodeId, Vec<fret_core::NodeId>> {
    let mut by_parent: HashMap<fret_core::NodeId, Vec<fret_core::NodeId>> = HashMap::new();
    for node in nodes {
        if let Some(parent) = node.parent {
            by_parent.entry(parent).or_default().push(node.id);
        }
    }
    by_parent
}

fn collect_reachable(
    roots: &[fret_core::NodeId],
    children: &HashMap<fret_core::NodeId, Vec<fret_core::NodeId>>,
) -> HashSet<fret_core::NodeId> {
    let mut seen: HashSet<fret_core::NodeId> = HashSet::new();
    let mut stack: Vec<fret_core::NodeId> = roots.to_vec();
    while let Some(node) = stack.pop() {
        if !seen.insert(node) {
            continue;
        }
        if let Some(kids) = children.get(&node) {
            for &child in kids.iter().rev() {
                stack.push(child);
            }
        }
    }
    seen
}

fn utf8_character_lengths(value: &str) -> Vec<u8> {
    value.chars().map(|c| c.len_utf8() as u8).collect()
}

fn root_for_node(
    mut node: fret_core::NodeId,
    parents: &HashMap<fret_core::NodeId, Option<fret_core::NodeId>>,
) -> fret_core::NodeId {
    while let Some(parent) = parents.get(&node).copied().flatten() {
        node = parent;
    }
    node
}

fn byte_to_character_index(value: &str, byte_offset: u32) -> usize {
    let target = byte_offset.min(value.len() as u32);
    let mut offset: u32 = 0;
    let mut index: usize = 0;
    for c in value.chars() {
        let len = c.len_utf8() as u32;
        if offset + len > target {
            break;
        }
        offset += len;
        index += 1;
        if offset == target {
            break;
        }
    }
    index
}

/// Builds an AccessKit [`TreeUpdate`] from a Fret [`SemanticsSnapshot`].
///
/// `scale_factor` should match the target's pixel scale (e.g. window scale factor) so bounds are
/// converted from logical pixels to physical pixels as expected by AccessKit.
pub fn tree_update_from_snapshot(snapshot: &SemanticsSnapshot, scale_factor: f64) -> TreeUpdate {
    let visible_roots = choose_visible_roots(snapshot);
    let children = build_children_index(&snapshot.nodes);
    let reachable = collect_reachable(&visible_roots, &children);
    let parents: HashMap<fret_core::NodeId, Option<fret_core::NodeId>> =
        snapshot.nodes.iter().map(|n| (n.id, n.parent)).collect();

    let mut nodes_out: Vec<(NodeId, Node)> = Vec::new();

    let mut root = Node::new(Role::Window);
    root.set_children(
        visible_roots
            .iter()
            .copied()
            .map(to_accesskit_id)
            .collect::<Vec<_>>(),
    );
    nodes_out.push((ROOT_ID, root));

    for node in &snapshot.nodes {
        if !reachable.contains(&node.id) {
            continue;
        }

        let mut out = Node::new(map_role(node.role));
        out.set_bounds(px_rect_to_accesskit(node.bounds, scale_factor));

        let mut out_children: Vec<NodeId> = children
            .get(&node.id)
            .map(|c| c.iter().copied().map(to_accesskit_id).collect())
            .unwrap_or_default();

        let mut synthetic_text_run: Option<(NodeId, Node)> = None;
        if node.role == SemanticsRole::TextField
            && let Some(value) = node.value.as_ref()
        {
            let run_id = text_run_id_for(node.id);
            out_children.push(run_id);

            let mut run = Node::new(Role::TextRun);
            run.set_bounds(px_rect_to_accesskit(node.bounds, scale_factor));
            run.set_value(value.clone());
            run.set_character_lengths(utf8_character_lengths(value));

            if let Some((anchor, focus)) = node.text_selection {
                out.set_text_selection(TextSelection {
                    anchor: TextPosition {
                        node: run_id,
                        character_index: byte_to_character_index(value, anchor),
                    },
                    focus: TextPosition {
                        node: run_id,
                        character_index: byte_to_character_index(value, focus),
                    },
                });
            }

            synthetic_text_run = Some((run_id, run));
        }

        if !out_children.is_empty() {
            out.set_children(out_children);
        }

        if node.flags.disabled {
            out.set_disabled();
        }
        if node.flags.read_only {
            out.set_read_only();
        }
        if node.flags.required {
            out.set_required();
        }
        if node.flags.busy {
            out.set_busy();
        }
        if let Some(invalid) = node.flags.invalid {
            let invalid = match invalid {
                fret_core::SemanticsInvalid::True => Some(Invalid::True),
                fret_core::SemanticsInvalid::Grammar => Some(Invalid::Grammar),
                fret_core::SemanticsInvalid::Spelling => Some(Invalid::Spelling),
                _ => None,
            };
            if let Some(invalid) = invalid {
                out.set_invalid(invalid);
            }
        }
        if node.flags.selected {
            out.set_selected(true);
        }
        if node.flags.expanded {
            out.set_expanded(true);
        }
        if let Some(checked) = node.flags.checked_state {
            let checked = match checked {
                fret_core::SemanticsCheckedState::False => Some(Toggled::False),
                fret_core::SemanticsCheckedState::True => Some(Toggled::True),
                fret_core::SemanticsCheckedState::Mixed => Some(Toggled::Mixed),
                _ => None,
            };
            if let Some(checked) = checked {
                out.set_toggled(checked);
            }
        } else if let Some(checked) = node.flags.checked {
            out.set_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            });
        } else if let Some(pressed) = node.flags.pressed_state {
            let pressed = match pressed {
                fret_core::SemanticsPressedState::False => Some(Toggled::False),
                fret_core::SemanticsPressedState::True => Some(Toggled::True),
                fret_core::SemanticsPressedState::Mixed => Some(Toggled::Mixed),
                _ => None,
            };
            if let Some(pressed) = pressed {
                out.set_toggled(pressed);
            }
        }

        if snapshot.barrier_root.is_some()
            && matches!(
                node.role,
                SemanticsRole::Dialog | SemanticsRole::AlertDialog
            )
        {
            let root = root_for_node(node.id, &parents);
            if snapshot.barrier_root == Some(root) {
                out.set_modal();
            }
        }

        if node.actions.focus {
            out.add_action(Action::Focus);
        }

        if node.actions.invoke {
            out.add_action(Action::Click);
        }
        if node.actions.set_value {
            out.add_action(Action::SetValue);
        }
        if node.actions.decrement {
            out.add_action(Action::Decrement);
        }
        if node.actions.increment {
            out.add_action(Action::Increment);
        }
        if node.actions.scroll_by {
            out.add_action(Action::SetScrollOffset);
            if node.extra.scroll.x_max.is_some() {
                out.add_action(Action::ScrollLeft);
                out.add_action(Action::ScrollRight);
            }
            if node.extra.scroll.y_max.is_some() {
                out.add_action(Action::ScrollUp);
                out.add_action(Action::ScrollDown);
            }
        }
        if node.actions.set_text_selection && node.value.is_some() {
            out.add_action(Action::SetTextSelection);
        }
        if node.actions.set_value && node.role == SemanticsRole::TextField && node.value.is_some() {
            out.add_action(Action::ReplaceSelectedText);
        }

        if let Some(label) = node.label.as_ref() {
            match node.role {
                SemanticsRole::Text => out.set_value(label.clone()),
                _ => out.set_label(label.clone()),
            }
        }
        if let Some(value) = node.value.as_ref() {
            out.set_value(value.clone());
        }
        if let Some(placeholder) = node.extra.placeholder.as_ref() {
            out.set_placeholder(placeholder.clone());
        }
        if let Some(url) = node.extra.url.as_ref() {
            out.set_url(url.clone());
        }
        if let Some(level) = node
            .extra
            .level
            .and_then(|level| usize::try_from(level).ok())
        {
            out.set_level(level);
        }

        if let Some(orientation) = node.extra.orientation {
            let orientation = match orientation {
                SemanticsOrientation::Horizontal => Some(accesskit::Orientation::Horizontal),
                SemanticsOrientation::Vertical => Some(accesskit::Orientation::Vertical),
                _ => None,
            };
            if let Some(orientation) = orientation {
                out.set_orientation(orientation);
            }
        }

        let numeric = node.extra.numeric;
        if let Some(value) = numeric.value.filter(|v| v.is_finite()) {
            out.set_numeric_value(value);
        }
        if let Some(value) = numeric.min.filter(|v| v.is_finite()) {
            out.set_min_numeric_value(value);
        }
        if let Some(value) = numeric.max.filter(|v| v.is_finite()) {
            out.set_max_numeric_value(value);
        }
        if let Some(value) = numeric.step.filter(|v| v.is_finite()) {
            out.set_numeric_value_step(value);
        }
        if let Some(value) = numeric.jump.filter(|v| v.is_finite()) {
            out.set_numeric_value_jump(value);
        }

        let scroll = node.extra.scroll;
        if let Some(value) = scroll.x.filter(|v| v.is_finite()) {
            out.set_scroll_x(value);
        }
        if let Some(value) = scroll.x_min.filter(|v| v.is_finite()) {
            out.set_scroll_x_min(value);
        }
        if let Some(value) = scroll.x_max.filter(|v| v.is_finite()) {
            out.set_scroll_x_max(value);
        }
        if let Some(value) = scroll.y.filter(|v| v.is_finite()) {
            out.set_scroll_y(value);
        }
        if let Some(value) = scroll.y_min.filter(|v| v.is_finite()) {
            out.set_scroll_y_min(value);
        }
        if let Some(value) = scroll.y_max.filter(|v| v.is_finite()) {
            out.set_scroll_y_max(value);
        }

        if let Some(active) = node.active_descendant
            && reachable.contains(&active)
        {
            out.set_active_descendant(to_accesskit_id(active));
        }

        if !node.labelled_by.is_empty() {
            let labelled_by: Vec<NodeId> = node
                .labelled_by
                .iter()
                .copied()
                .filter(|id| reachable.contains(id))
                .map(to_accesskit_id)
                .collect();
            if !labelled_by.is_empty() {
                out.set_labelled_by(labelled_by);
            }
        }

        if !node.controls.is_empty() {
            let controls: Vec<NodeId> = node
                .controls
                .iter()
                .copied()
                .filter(|id| reachable.contains(id))
                .map(to_accesskit_id)
                .collect();
            if !controls.is_empty() {
                out.set_controls(controls);
            }
        }

        if !node.described_by.is_empty() {
            let described_by: Vec<NodeId> = node
                .described_by
                .iter()
                .copied()
                .filter(|id| reachable.contains(id))
                .map(to_accesskit_id)
                .collect();
            if !described_by.is_empty() {
                out.set_described_by(described_by);
            }
        }

        if let Some(pos_in_set) = node.pos_in_set.and_then(|p| usize::try_from(p).ok()) {
            out.set_position_in_set(pos_in_set);
        }
        if let Some(set_size) = node.set_size.and_then(|s| usize::try_from(s).ok()) {
            out.set_size_of_set(set_size);
        }

        nodes_out.push((to_accesskit_id(node.id), out));
        if let Some((id, run)) = synthetic_text_run {
            nodes_out.push((id, run));
        }
    }

    let focus = snapshot
        .focus
        .filter(|id| reachable.contains(id))
        .map(to_accesskit_id)
        .unwrap_or(ROOT_ID);

    TreeUpdate {
        nodes: nodes_out,
        tree: Some(Tree {
            root: ROOT_ID,
            toolkit_name: Some("fret".to_string()),
            toolkit_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
        tree_id: TreeId::ROOT,
        focus,
    }
}
