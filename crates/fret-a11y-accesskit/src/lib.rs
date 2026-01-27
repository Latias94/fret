use std::collections::{HashMap, HashSet};

use accesskit::{
    Action, ActionRequest, Node, NodeId, Rect, Role, TextPosition, TextSelection, Toggled, Tree,
    TreeUpdate,
};
use fret_core::{SemanticsNode, SemanticsRole, SemanticsSnapshot};
use slotmap::{Key, KeyData};

const ROOT_ID: NodeId = NodeId(0);
const SYNTHETIC_TEXT_RUN_BIT: u64 = 1 << 63;

fn to_accesskit_id(node: fret_core::NodeId) -> NodeId {
    NodeId(node.data().as_ffi().wrapping_add(1))
}

fn from_accesskit_id(node: NodeId) -> Option<fret_core::NodeId> {
    if node.0 == 0 {
        return None;
    }
    Some(fret_core::NodeId::from(KeyData::from_ffi(
        node.0.wrapping_sub(1),
    )))
}

fn text_run_id_for(node: fret_core::NodeId) -> NodeId {
    NodeId(to_accesskit_id(node).0 | SYNTHETIC_TEXT_RUN_BIT)
}

fn parent_from_synthetic_id(node: NodeId) -> Option<fret_core::NodeId> {
    if (node.0 & SYNTHETIC_TEXT_RUN_BIT) == 0 {
        return None;
    }
    from_accesskit_id(NodeId(node.0 & !SYNTHETIC_TEXT_RUN_BIT))
}

// Backend-specific adapter glue lives in crates like `fret-runner-winit` (winit) or future mobile/web backends.

fn map_role(role: SemanticsRole) -> Role {
    match role {
        SemanticsRole::Generic => Role::GenericContainer,
        SemanticsRole::Window => Role::Pane,
        SemanticsRole::Panel => Role::Pane,
        SemanticsRole::Group => Role::Group,
        SemanticsRole::Dialog => Role::Dialog,
        SemanticsRole::AlertDialog => Role::AlertDialog,
        SemanticsRole::Alert => Role::Alert,
        SemanticsRole::Button => Role::Button,
        SemanticsRole::Checkbox => Role::CheckBox,
        SemanticsRole::Switch => Role::Switch,
        SemanticsRole::Slider => Role::Slider,
        SemanticsRole::ComboBox => Role::ComboBox,
        SemanticsRole::RadioGroup => Role::RadioGroup,
        SemanticsRole::RadioButton => Role::RadioButton,
        SemanticsRole::TabList => Role::TabList,
        SemanticsRole::Tab => Role::Tab,
        SemanticsRole::TabPanel => Role::TabPanel,
        SemanticsRole::MenuBar => Role::MenuBar,
        SemanticsRole::Menu => Role::Menu,
        SemanticsRole::MenuItem => Role::MenuItem,
        SemanticsRole::MenuItemCheckbox => Role::MenuItemCheckBox,
        SemanticsRole::MenuItemRadio => Role::MenuItemRadio,
        SemanticsRole::Tooltip => Role::Tooltip,
        SemanticsRole::Text => Role::Label,
        SemanticsRole::TextField => Role::TextInput,
        SemanticsRole::List => Role::List,
        SemanticsRole::ListItem => Role::ListItem,
        SemanticsRole::ListBox => Role::ListBox,
        SemanticsRole::ListBoxOption => Role::ListBoxOption,
        SemanticsRole::TreeItem => Role::TreeItem,
        SemanticsRole::Viewport => Role::ScrollView,
        _ => Role::GenericContainer,
    }
}

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
        if node.flags.selected {
            out.set_selected(true);
        }
        if node.flags.expanded {
            out.set_expanded(true);
        }
        if let Some(checked) = node.flags.checked {
            out.set_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            });
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
        focus,
    }
}

pub fn focus_target_from_action(req: &ActionRequest) -> Option<fret_core::NodeId> {
    if req.action != Action::Focus {
        return None;
    }
    parent_from_synthetic_id(req.target).or_else(|| from_accesskit_id(req.target))
}

pub fn invoke_target_from_action(req: &ActionRequest) -> Option<fret_core::NodeId> {
    if req.action != Action::Click {
        return None;
    }
    parent_from_synthetic_id(req.target).or_else(|| from_accesskit_id(req.target))
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetValueData {
    Text(String),
    Numeric(f64),
}

pub fn set_value_from_action(req: &ActionRequest) -> Option<(fret_core::NodeId, SetValueData)> {
    if req.action != Action::SetValue {
        return None;
    }

    let target = parent_from_synthetic_id(req.target).or_else(|| from_accesskit_id(req.target))?;
    let data = req.data.as_ref()?;
    match data {
        accesskit::ActionData::Value(v) => Some((target, SetValueData::Text(v.to_string()))),
        accesskit::ActionData::NumericValue(v) => Some((target, SetValueData::Numeric(*v))),
        _ => None,
    }
}

pub fn replace_selected_text_from_action(
    req: &ActionRequest,
    snapshot: &SemanticsSnapshot,
) -> Option<(fret_core::NodeId, String)> {
    if req.action != Action::ReplaceSelectedText {
        return None;
    }

    let target = parent_from_synthetic_id(req.target).or_else(|| from_accesskit_id(req.target))?;
    let node = snapshot.nodes.iter().find(|n| n.id == target)?;
    if node.role != SemanticsRole::TextField || node.value.is_none() {
        return None;
    }
    if node.text_composition.is_some() {
        return None;
    }

    let data = req.data.as_ref()?;
    match data {
        accesskit::ActionData::Value(v) => Some((target, v.to_string())),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTextSelectionData {
    pub anchor: u32,
    pub focus: u32,
}

fn character_index_to_byte_offset(value: &str, character_index: usize) -> u32 {
    value
        .chars()
        .take(character_index)
        .fold(0u32, |acc, ch| acc.saturating_add(ch.len_utf8() as u32))
        .min(value.len() as u32)
}

fn text_selection_target_from_position(pos: &TextPosition) -> Option<fret_core::NodeId> {
    parent_from_synthetic_id(pos.node).or_else(|| from_accesskit_id(pos.node))
}

pub fn set_text_selection_from_action(
    req: &ActionRequest,
    snapshot: &SemanticsSnapshot,
) -> Option<(fret_core::NodeId, SetTextSelectionData)> {
    if req.action != Action::SetTextSelection {
        return None;
    }

    let target = parent_from_synthetic_id(req.target).or_else(|| from_accesskit_id(req.target))?;
    let data = req.data.as_ref()?;
    let accesskit::ActionData::SetTextSelection(sel) = data else {
        return None;
    };

    let node = snapshot.nodes.iter().find(|n| n.id == target)?;
    let value = node.value.as_deref()?;
    if node.text_composition.is_some() {
        return None;
    }

    let anchor_target = text_selection_target_from_position(&sel.anchor)?;
    let focus_target = text_selection_target_from_position(&sel.focus)?;
    if anchor_target != target || focus_target != target {
        return None;
    }

    let anchor = character_index_to_byte_offset(value, sel.anchor.character_index);
    let focus = character_index_to_byte_offset(value, sel.focus.character_index);

    Some((target, SetTextSelectionData { anchor, focus }))
}

#[cfg(test)]
mod tests {
    use super::{
        replace_selected_text_from_action, set_text_selection_from_action, text_run_id_for,
        to_accesskit_id, tree_update_from_snapshot,
    };
    use accesskit::Role;
    use fret_core::{
        AppWindowId, Px, Rect, SemanticsActions, SemanticsFlags, SemanticsNode, SemanticsRole,
        SemanticsRoot, SemanticsSnapshot,
    };
    use slotmap::KeyData;

    fn node(id: u64) -> fret_core::NodeId {
        fret_core::NodeId::from(KeyData::from_ffi(id))
    }

    #[test]
    fn maps_extended_semantics_roles_to_accesskit_roles() {
        assert_eq!(
            super::map_role(SemanticsRole::AlertDialog),
            Role::AlertDialog
        );
        assert_eq!(super::map_role(SemanticsRole::RadioGroup), Role::RadioGroup);
        assert_eq!(
            super::map_role(SemanticsRole::RadioButton),
            Role::RadioButton
        );
        assert_eq!(
            super::map_role(SemanticsRole::MenuItemCheckbox),
            Role::MenuItemCheckBox
        );
        assert_eq!(
            super::map_role(SemanticsRole::MenuItemRadio),
            Role::MenuItemRadio
        );
        assert_eq!(super::map_role(SemanticsRole::Tooltip), Role::Tooltip);
    }

    #[test]
    fn active_descendant_is_emitted_for_reachable_descendant() {
        let window = AppWindowId::default();
        let root = node(1);
        let input = node(2);
        let list = node(3);
        let item = node(4);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(10.0), Px(10.0)),
        );

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: Some(input),
            captured: None,
            nodes: vec![
                SemanticsNode {
                    id: root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: input,
                    parent: Some(root),
                    role: SemanticsRole::TextField,
                    bounds,
                    flags: SemanticsFlags {
                        focused: true,
                        ..SemanticsFlags::default()
                    },
                    test_id: None,
                    active_descendant: Some(item),
                    pos_in_set: None,
                    set_size: None,
                    label: Some("Command input".to_string()),
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions {
                        focus: true,
                        set_value: true,
                        ..SemanticsActions::default()
                    },
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: list,
                    parent: Some(root),
                    role: SemanticsRole::List,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: item,
                    parent: Some(list),
                    role: SemanticsRole::ListItem,
                    bounds,
                    flags: SemanticsFlags {
                        selected: true,
                        ..SemanticsFlags::default()
                    },
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: Some("Item 1".to_string()),
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
            ],
        };

        let update = tree_update_from_snapshot(&snapshot, 1.0);
        let input_id = to_accesskit_id(input);
        let item_id = to_accesskit_id(item);

        let input_node = update
            .nodes
            .iter()
            .find_map(|(id, n)| (*id == input_id).then_some(n))
            .expect("input node present");

        assert_eq!(
            input_node.active_descendant(),
            Some(item_id),
            "focused text field should reference the active descendant"
        );
    }

    #[test]
    fn active_descendant_is_not_emitted_when_not_reachable_under_modal_barrier() {
        let window = AppWindowId::default();
        let underlay_root = node(1);
        let underlay_list = node(2);
        let underlay_item = node(3);

        let modal_root = node(10);
        let input = node(11);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(10.0), Px(10.0)),
        );

        // The focused input lives in the modal barrier layer, but it (incorrectly) points its
        // active descendant at an underlay list item. The bridge must not emit that association.
        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![
                SemanticsRoot {
                    root: underlay_root,
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 0,
                },
                SemanticsRoot {
                    root: modal_root,
                    visible: true,
                    blocks_underlay_input: true,
                    hit_testable: true,
                    z_index: 1,
                },
            ],
            barrier_root: Some(modal_root),
            focus_barrier_root: Some(modal_root),
            focus: Some(input),
            captured: None,
            nodes: vec![
                SemanticsNode {
                    id: underlay_root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: underlay_list,
                    parent: Some(underlay_root),
                    role: SemanticsRole::ListBox,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: Some("Underlay list".to_string()),
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: underlay_item,
                    parent: Some(underlay_list),
                    role: SemanticsRole::ListBoxOption,
                    bounds,
                    flags: SemanticsFlags {
                        selected: true,
                        ..SemanticsFlags::default()
                    },
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: Some(1),
                    set_size: Some(1),
                    label: Some("Underlay item".to_string()),
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: modal_root,
                    parent: None,
                    role: SemanticsRole::Dialog,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: Some("Modal".to_string()),
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: input,
                    parent: Some(modal_root),
                    role: SemanticsRole::TextField,
                    bounds,
                    flags: SemanticsFlags {
                        focused: true,
                        ..SemanticsFlags::default()
                    },
                    test_id: None,
                    active_descendant: Some(underlay_item),
                    pos_in_set: None,
                    set_size: None,
                    label: Some("Command input".to_string()),
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions {
                        focus: true,
                        set_value: true,
                        ..SemanticsActions::default()
                    },
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
            ],
        };

        let update = tree_update_from_snapshot(&snapshot, 1.0);
        let input_id = to_accesskit_id(input);

        let input_node = update
            .nodes
            .iter()
            .find_map(|(id, n)| (*id == input_id).then_some(n))
            .expect("input node present");

        assert_eq!(
            input_node.active_descendant(),
            None,
            "active_descendant must be suppressed when it points under the modal barrier"
        );
    }

    #[test]
    fn list_item_pos_in_set_and_set_size_are_emitted() {
        let window = AppWindowId::default();
        let root = node(1);
        let list = node(2);
        let item = node(3);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(10.0), Px(10.0)),
        );

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                SemanticsNode {
                    id: root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: list,
                    parent: Some(root),
                    role: SemanticsRole::List,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: item,
                    parent: Some(list),
                    role: SemanticsRole::ListItem,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: Some(57),
                    set_size: Some(1200),
                    label: Some("Item 57".to_string()),
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
            ],
        };

        let update = tree_update_from_snapshot(&snapshot, 1.0);
        let item_id = to_accesskit_id(item);

        let item_node = update
            .nodes
            .iter()
            .find_map(|(id, n)| (*id == item_id).then_some(n))
            .expect("item node present");

        assert_eq!(item_node.position_in_set(), Some(57));
        assert_eq!(item_node.size_of_set(), Some(1200));
    }

    #[test]
    fn described_by_is_emitted_for_reachable_descendant() {
        let window = AppWindowId::default();
        let root = node(1);
        let dialog = node(2);
        let title = node(3);
        let description = node(4);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(10.0), Px(10.0)),
        );

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                SemanticsNode {
                    id: root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: dialog,
                    parent: Some(root),
                    role: SemanticsRole::Dialog,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: vec![title],
                    described_by: vec![description],
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: title,
                    parent: Some(dialog),
                    role: SemanticsRole::Text,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: Some("Title".to_string()),
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: description,
                    parent: Some(dialog),
                    role: SemanticsRole::Text,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: Some("Description".to_string()),
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
            ],
        };

        let update = tree_update_from_snapshot(&snapshot, 1.0);
        let dialog_id = to_accesskit_id(dialog);

        let dialog_node = update
            .nodes
            .iter()
            .find_map(|(id, n)| (*id == dialog_id).then_some(n))
            .expect("dialog node present");

        let expected_labelled_by = vec![to_accesskit_id(title)];
        assert_eq!(dialog_node.labelled_by(), expected_labelled_by.as_slice());

        let expected_described_by = vec![to_accesskit_id(description)];
        assert_eq!(dialog_node.described_by(), expected_described_by.as_slice());
    }

    #[test]
    fn modal_dialogs_are_marked_modal_under_barrier_root() {
        let window = AppWindowId::default();
        let underlay_root = node(1);
        let modal_root = node(10);
        let dialog = node(11);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(10.0), Px(10.0)),
        );

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![
                SemanticsRoot {
                    root: underlay_root,
                    visible: true,
                    blocks_underlay_input: false,
                    hit_testable: true,
                    z_index: 0,
                },
                SemanticsRoot {
                    root: modal_root,
                    visible: true,
                    blocks_underlay_input: true,
                    hit_testable: true,
                    z_index: 1,
                },
            ],
            barrier_root: Some(modal_root),
            focus_barrier_root: Some(modal_root),
            focus: None,
            captured: None,
            nodes: vec![
                SemanticsNode {
                    id: underlay_root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: modal_root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: dialog,
                    parent: Some(modal_root),
                    role: SemanticsRole::Dialog,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: Some("Modal dialog".to_string()),
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
            ],
        };

        let update = tree_update_from_snapshot(&snapshot, 1.0);
        let dialog_id = to_accesskit_id(dialog);
        let dialog_node = update
            .nodes
            .iter()
            .find_map(|(id, n)| (*id == dialog_id).then_some(n))
            .expect("dialog node present");

        assert!(
            dialog_node.is_modal(),
            "dialogs in the barrier layer must be marked modal"
        );
    }

    #[test]
    fn text_field_emits_synthetic_text_run_and_text_selection() {
        let window = AppWindowId::default();
        let root = node(1);
        let input = node(2);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(10.0), Px(10.0)),
        );

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: Some(input),
            captured: None,
            nodes: vec![
                SemanticsNode {
                    id: root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: input,
                    parent: Some(root),
                    role: SemanticsRole::TextField,
                    bounds,
                    flags: SemanticsFlags {
                        focused: true,
                        ..SemanticsFlags::default()
                    },
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: Some("Search".to_string()),
                    value: Some("hello".to_string()),
                    text_selection: Some((1, 4)),
                    text_composition: None,
                    actions: SemanticsActions {
                        focus: true,
                        set_value: true,
                        ..SemanticsActions::default()
                    },
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
            ],
        };

        let update = tree_update_from_snapshot(&snapshot, 1.0);
        let input_id = to_accesskit_id(input);
        let run_id = text_run_id_for(input);

        let input_node = update
            .nodes
            .iter()
            .find_map(|(id, n)| (*id == input_id).then_some(n))
            .expect("input node present");
        let run_node = update
            .nodes
            .iter()
            .find_map(|(id, n)| (*id == run_id).then_some(n))
            .expect("text run node present");

        assert!(
            input_node.children().contains(&run_id),
            "text field should include synthetic text run child"
        );
        assert_eq!(run_node.value(), Some("hello"));
        assert!(
            !run_node.character_lengths().is_empty(),
            "text run should include character lengths for selection"
        );

        let selection = input_node.text_selection().expect("selection present");
        assert_eq!(selection.anchor.node, run_id);
        assert_eq!(selection.anchor.character_index, 1);
        assert_eq!(selection.focus.node, run_id);
        assert_eq!(selection.focus.character_index, 4);
    }

    #[test]
    fn set_text_selection_action_converts_character_indices_to_utf8_bytes() {
        let window = AppWindowId::default();
        let root = node(1);
        let input = node(2);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(10.0), Px(10.0)),
        );

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: Some(input),
            captured: None,
            nodes: vec![
                SemanticsNode {
                    id: root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: input,
                    parent: Some(root),
                    role: SemanticsRole::TextField,
                    bounds,
                    flags: SemanticsFlags {
                        focused: true,
                        ..SemanticsFlags::default()
                    },
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: Some("a😀b".to_string()),
                    text_selection: Some((0, 0)),
                    text_composition: None,
                    actions: SemanticsActions {
                        focus: true,
                        set_text_selection: true,
                        ..SemanticsActions::default()
                    },
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
            ],
        };

        let req = accesskit::ActionRequest {
            action: accesskit::Action::SetTextSelection,
            target: to_accesskit_id(input),
            data: Some(accesskit::ActionData::SetTextSelection(
                accesskit::TextSelection {
                    anchor: accesskit::TextPosition {
                        node: text_run_id_for(input),
                        character_index: 1,
                    },
                    focus: accesskit::TextPosition {
                        node: text_run_id_for(input),
                        character_index: 2,
                    },
                },
            )),
        };

        let (target, data) =
            set_text_selection_from_action(&req, &snapshot).expect("decoded selection");
        assert_eq!(target, input);
        assert_eq!(data.anchor, 1);
        assert_eq!(data.focus, 5);
    }

    #[test]
    fn replace_selected_text_action_is_decoded() {
        let window = AppWindowId::default();
        let root = node(1);
        let input = node(2);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(10.0), Px(10.0)),
        );

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: Some(input),
            captured: None,
            nodes: vec![
                SemanticsNode {
                    id: root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: input,
                    parent: Some(root),
                    role: SemanticsRole::TextField,
                    bounds,
                    flags: SemanticsFlags {
                        focused: true,
                        ..SemanticsFlags::default()
                    },
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: Some("hello".to_string()),
                    text_selection: Some((0, 5)),
                    text_composition: None,
                    actions: SemanticsActions {
                        focus: true,
                        set_value: true,
                        ..SemanticsActions::default()
                    },
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
            ],
        };

        let update = tree_update_from_snapshot(&snapshot, 1.0);
        let input_id = to_accesskit_id(input);
        let input_node = update
            .nodes
            .iter()
            .find_map(|(id, n)| (*id == input_id).then_some(n))
            .expect("input node present");
        assert!(
            input_node.supports_action(accesskit::Action::ReplaceSelectedText),
            "text field should expose ReplaceSelectedText when editable"
        );

        let req = accesskit::ActionRequest {
            action: accesskit::Action::ReplaceSelectedText,
            target: to_accesskit_id(input),
            data: Some(accesskit::ActionData::Value("x".into())),
        };
        let (target, value) = replace_selected_text_from_action(&req, &snapshot)
            .expect("decoded replace selected text");
        assert_eq!(target, input);
        assert_eq!(value, "x");
    }

    #[test]
    fn replace_selected_text_is_rejected_during_composition() {
        let window = AppWindowId::default();
        let root = node(1);
        let input = node(2);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(10.0), Px(10.0)),
        );

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: Some(input),
            captured: None,
            nodes: vec![
                SemanticsNode {
                    id: root,
                    parent: None,
                    role: SemanticsRole::Window,
                    bounds,
                    flags: SemanticsFlags::default(),
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: None,
                    text_selection: None,
                    text_composition: None,
                    actions: SemanticsActions::default(),
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
                SemanticsNode {
                    id: input,
                    parent: Some(root),
                    role: SemanticsRole::TextField,
                    bounds,
                    flags: SemanticsFlags {
                        focused: true,
                        ..SemanticsFlags::default()
                    },
                    test_id: None,
                    active_descendant: None,
                    pos_in_set: None,
                    set_size: None,
                    label: None,
                    value: Some("he|llo".to_string()),
                    text_selection: Some((2, 2)),
                    text_composition: Some((2, 3)),
                    actions: SemanticsActions {
                        focus: true,
                        set_value: true,
                        ..SemanticsActions::default()
                    },
                    labelled_by: Vec::new(),
                    described_by: Vec::new(),
                    controls: Vec::new(),
                },
            ],
        };

        let req = accesskit::ActionRequest {
            action: accesskit::Action::ReplaceSelectedText,
            target: to_accesskit_id(input),
            data: Some(accesskit::ActionData::Value("x".into())),
        };
        assert!(
            replace_selected_text_from_action(&req, &snapshot).is_none(),
            "should not mutate text while composing"
        );
    }
}
