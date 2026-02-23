use std::collections::{HashMap, HashSet};

use fret_core::{AppWindowId, NodeId, SemanticsRole};
use fret_ui::elements::ElementRuntime;
use slotmap::Key as _;

use super::{UiDiagnosticsConfig, UiRoleAndNameV1, UiSelectorV1, UiSemanticsNodeV1};

pub(crate) fn semantics_role_label(role: SemanticsRole) -> &'static str {
    match role {
        SemanticsRole::Generic => "generic",
        SemanticsRole::Window => "window",
        SemanticsRole::Panel => "panel",
        SemanticsRole::Group => "group",
        SemanticsRole::Toolbar => "toolbar",
        SemanticsRole::Heading => "heading",
        SemanticsRole::Dialog => "dialog",
        SemanticsRole::AlertDialog => "alert_dialog",
        SemanticsRole::Alert => "alert",
        SemanticsRole::Button => "button",
        SemanticsRole::Link => "link",
        SemanticsRole::Image => "image",
        SemanticsRole::Checkbox => "checkbox",
        SemanticsRole::Switch => "switch",
        SemanticsRole::Slider => "slider",
        SemanticsRole::ProgressBar => "progress_bar",
        SemanticsRole::ComboBox => "combo_box",
        SemanticsRole::RadioGroup => "radio_group",
        SemanticsRole::RadioButton => "radio_button",
        SemanticsRole::TabList => "tab_list",
        SemanticsRole::Tab => "tab",
        SemanticsRole::TabPanel => "tab_panel",
        SemanticsRole::MenuBar => "menu_bar",
        SemanticsRole::Menu => "menu",
        SemanticsRole::MenuItem => "menu_item",
        SemanticsRole::MenuItemCheckbox => "menu_item_checkbox",
        SemanticsRole::MenuItemRadio => "menu_item_radio",
        SemanticsRole::Tooltip => "tooltip",
        SemanticsRole::Text => "text",
        SemanticsRole::TextField => "text_field",
        SemanticsRole::List => "list",
        SemanticsRole::ListItem => "list_item",
        SemanticsRole::Separator => "separator",
        SemanticsRole::ListBox => "list_box",
        SemanticsRole::ListBoxOption => "list_box_option",
        SemanticsRole::TreeItem => "tree_item",
        SemanticsRole::Viewport => "viewport",
        _ => "unknown",
    }
}

pub(super) fn parse_semantics_role(s: &str) -> Option<SemanticsRole> {
    let s = s.trim().to_ascii_lowercase();
    Some(match s.as_str() {
        "generic" => SemanticsRole::Generic,
        "window" => SemanticsRole::Window,
        "panel" => SemanticsRole::Panel,
        "group" => SemanticsRole::Group,
        "toolbar" => SemanticsRole::Toolbar,
        "heading" => SemanticsRole::Heading,
        "dialog" => SemanticsRole::Dialog,
        "alert_dialog" => SemanticsRole::AlertDialog,
        "alert" => SemanticsRole::Alert,
        "button" => SemanticsRole::Button,
        "link" => SemanticsRole::Link,
        "image" => SemanticsRole::Image,
        "checkbox" => SemanticsRole::Checkbox,
        "switch" => SemanticsRole::Switch,
        "slider" => SemanticsRole::Slider,
        "progress_bar" => SemanticsRole::ProgressBar,
        "combo_box" => SemanticsRole::ComboBox,
        "radio_group" => SemanticsRole::RadioGroup,
        "radio_button" => SemanticsRole::RadioButton,
        "tab_list" => SemanticsRole::TabList,
        "tab" => SemanticsRole::Tab,
        "tab_panel" => SemanticsRole::TabPanel,
        "menu_bar" => SemanticsRole::MenuBar,
        "menu" => SemanticsRole::Menu,
        "menu_item" => SemanticsRole::MenuItem,
        "menu_item_checkbox" => SemanticsRole::MenuItemCheckbox,
        "menu_item_radio" => SemanticsRole::MenuItemRadio,
        "tooltip" => SemanticsRole::Tooltip,
        "text" => SemanticsRole::Text,
        "text_field" => SemanticsRole::TextField,
        "list" => SemanticsRole::List,
        "list_item" => SemanticsRole::ListItem,
        "separator" => SemanticsRole::Separator,
        "list_box" => SemanticsRole::ListBox,
        "list_box_option" => SemanticsRole::ListBoxOption,
        "tree_item" => SemanticsRole::TreeItem,
        "viewport" => SemanticsRole::Viewport,
        _ => return None,
    })
}

pub(super) fn select_semantics_node<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    selector: &UiSelectorV1,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = SemanticsIndex::new(snapshot);

    match selector {
        UiSelectorV1::NodeId { node } => index
            .by_id
            .get(node)
            .copied()
            .filter(|n| index.is_selectable(n.id.data().as_ffi())),
        UiSelectorV1::RoleAndName { role, name } => {
            let role = parse_semantics_role(role)?;
            super::pick::pick_best_match(
                snapshot.nodes.iter().filter(|n| {
                    let id = n.id.data().as_ffi();
                    index.is_selectable(id)
                        && n.role == role
                        && n.label.as_deref().is_some_and(|label| label == name)
                }),
                &index,
            )
        }
        UiSelectorV1::RoleAndPath {
            role,
            name,
            ancestors,
        } => {
            let role = parse_semantics_role(role)?;

            let mut parsed_ancestors: Vec<(SemanticsRole, &str)> =
                Vec::with_capacity(ancestors.len());
            for a in ancestors {
                parsed_ancestors.push((parse_semantics_role(&a.role)?, a.name.as_str()));
            }

            super::pick::pick_best_match(
                snapshot.nodes.iter().filter(|n| {
                    let id = n.id.data().as_ffi();
                    index.is_selectable(id)
                        && n.role == role
                        && n.label.as_deref().is_some_and(|label| label == name)
                        && index.ancestors_match_subsequence(n.parent, &parsed_ancestors)
                }),
                &index,
            )
        }
        UiSelectorV1::TestId { id } => super::pick::pick_best_match(
            snapshot.nodes.iter().filter(|n| {
                let node_id = n.id.data().as_ffi();
                index.is_selectable(node_id) && n.test_id.as_deref().is_some_and(|v| v == id)
            }),
            &index,
        )
        .or_else(|| {
            // Fallback for debugging: allow selecting hidden nodes if no visible match exists.
            super::pick::pick_best_match(
                snapshot
                    .nodes
                    .iter()
                    .filter(|n| n.test_id.as_deref().is_some_and(|v| v == id)),
                &index,
            )
        }),
        UiSelectorV1::GlobalElementId { element } => {
            let node = element_runtime.and_then(|runtime| {
                runtime.node_for_element(window, fret_ui::elements::GlobalElementId(*element))
            })?;
            let node_id = node.data().as_ffi();
            index
                .by_id
                .get(&node_id)
                .copied()
                .filter(|n| index.is_selectable(n.id.data().as_ffi()))
        }
    }
}

pub(super) struct SemanticsIndex<'a> {
    pub(super) by_id: HashMap<u64, &'a fret_core::SemanticsNode>,
    visible_ids: HashSet<u64>,
    barrier_root: Option<u64>,
    root_z_index: HashMap<u64, u32>,
}

impl<'a> SemanticsIndex<'a> {
    pub(super) fn new(snapshot: &'a fret_core::SemanticsSnapshot) -> Self {
        let mut by_id: HashMap<u64, &'a fret_core::SemanticsNode> = HashMap::new();
        let mut children: HashMap<u64, Vec<u64>> = HashMap::new();

        for n in &snapshot.nodes {
            let id = n.id.data().as_ffi();
            by_id.insert(id, n);
            if let Some(parent) = n.parent {
                children.entry(parent.data().as_ffi()).or_default().push(id);
            }
        }

        let mut root_z_index: HashMap<u64, u32> = HashMap::new();
        for r in &snapshot.roots {
            root_z_index.insert(r.root.data().as_ffi(), r.z_index);
        }

        let barrier_root = snapshot.barrier_root.map(|n| n.data().as_ffi());

        let mut visible_ids: HashSet<u64> = HashSet::new();
        for root in snapshot.roots.iter().filter(|r| r.visible) {
            collect_subtree_ids(root.root.data().as_ffi(), &children, &mut visible_ids);
        }

        Self {
            by_id,
            visible_ids,
            barrier_root,
            root_z_index,
        }
    }

    pub(super) fn is_selectable(&self, id: u64) -> bool {
        if !self.visible_ids.contains(&id) {
            return false;
        }
        if let Some(barrier) = self.barrier_root {
            return self.is_descendant_of_or_self(id, barrier);
        }
        true
    }

    pub(super) fn is_descendant_of_or_self(&self, mut id: u64, ancestor: u64) -> bool {
        if id == ancestor {
            return true;
        }
        while let Some(node) = self.by_id.get(&id).copied() {
            let Some(parent) = node.parent else {
                return false;
            };
            id = parent.data().as_ffi();
            if id == ancestor {
                return true;
            }
        }
        false
    }

    /// Match `ancestors` (outermost -> innermost) as an ordered subsequence along the parent chain.
    pub(super) fn ancestors_match_subsequence(
        &self,
        start_parent: Option<NodeId>,
        ancestors: &[(SemanticsRole, &str)],
    ) -> bool {
        let mut cur = start_parent.and_then(|p| self.by_id.get(&p.data().as_ffi()).copied());

        for (want_role, want_name) in ancestors.iter().rev() {
            let mut found = false;
            while let Some(node) = cur {
                if node.role == *want_role && node.label.as_deref() == Some(*want_name) {
                    found = true;
                    cur = node
                        .parent
                        .and_then(|p| self.by_id.get(&p.data().as_ffi()).copied());
                    break;
                }
                cur = node
                    .parent
                    .and_then(|p| self.by_id.get(&p.data().as_ffi()).copied());
            }
            if !found {
                return false;
            }
        }

        true
    }

    pub(super) fn root_z_for(&self, id: u64) -> u32 {
        let mut cur = Some(id);
        while let Some(node_id) = cur {
            if let Some(z) = self.root_z_index.get(&node_id).copied() {
                return z;
            }
            cur = self
                .by_id
                .get(&node_id)
                .and_then(|n| n.parent.map(|p| p.data().as_ffi()));
        }
        0
    }

    pub(super) fn depth_for(&self, id: u64) -> u32 {
        let mut depth = 0u32;
        let mut cur = Some(id);
        while let Some(node_id) = cur {
            let Some(node) = self.by_id.get(&node_id).copied() else {
                break;
            };
            let Some(parent) = node.parent else {
                break;
            };
            depth = depth.saturating_add(1);
            cur = Some(parent.data().as_ffi());
        }
        depth
    }
}

fn collect_subtree_ids(root: u64, children: &HashMap<u64, Vec<u64>>, out: &mut HashSet<u64>) {
    let mut stack: Vec<u64> = vec![root];
    while let Some(id) = stack.pop() {
        if !out.insert(id) {
            continue;
        }
        if let Some(kids) = children.get(&id) {
            stack.extend(kids.iter().copied());
        }
    }
}

pub(super) fn parent_node_id(snapshot: &fret_core::SemanticsSnapshot, node: u64) -> Option<u64> {
    let n = snapshot
        .nodes
        .iter()
        .find(|n| n.id.data().as_ffi() == node)?;
    n.parent.map(|p| p.data().as_ffi())
}

pub(super) fn truncate_debug_value(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut out = s[..max_bytes.min(s.len())].to_string();
    out.push('…');
    out
}

pub(super) fn format_inspect_path(
    snapshot: &fret_core::SemanticsSnapshot,
    focus_node_id: u64,
    redact_text: bool,
    max_parts: usize,
) -> Option<String> {
    if max_parts == 0 {
        return None;
    }
    let mut parts: Vec<String> = Vec::new();
    let mut cur: Option<u64> = Some(focus_node_id);
    while let Some(id) = cur {
        let Some(node) = snapshot.nodes.iter().find(|n| n.id.data().as_ffi() == id) else {
            break;
        };

        let role = semantics_role_label(node.role);
        let mut part = role.to_string();
        if let Some(test_id) = node.test_id.as_deref() {
            part.push('[');
            part.push_str(&truncate_debug_value(test_id, 32));
            part.push(']');
        } else if !redact_text && let Some(label) = node.label.as_deref() {
            part.push('(');
            part.push_str(&truncate_debug_value(label, 32));
            part.push(')');
        }
        parts.push(part);

        cur = node.parent.map(|p| p.data().as_ffi());
    }
    if parts.is_empty() {
        return None;
    }
    parts.reverse();

    if parts.len() > max_parts {
        parts = parts.split_off(parts.len() - max_parts);
        parts.insert(0, "…".to_string());
    }

    Some(format!("path: {}", parts.join(" > ")))
}

pub(super) fn selector_ancestors_for(
    snapshot: &fret_core::SemanticsSnapshot,
    node: &fret_core::SemanticsNode,
) -> Vec<UiRoleAndNameV1> {
    let index = SemanticsIndex::new(snapshot);
    let mut rev: Vec<UiRoleAndNameV1> = Vec::new();

    let mut cur = node
        .parent
        .and_then(|p| index.by_id.get(&p.data().as_ffi()).copied());
    while let Some(n) = cur {
        if let Some(label) = n.label.as_deref() {
            rev.push(UiRoleAndNameV1 {
                role: semantics_role_label(n.role).to_string(),
                name: label.to_string(),
            });
        }
        cur = n
            .parent
            .and_then(|p| index.by_id.get(&p.data().as_ffi()).copied());
    }

    rev.reverse();
    rev
}

fn is_redacted_string(s: &str) -> bool {
    s.trim_start().starts_with("<redacted")
}

pub(super) fn suggest_selectors(
    snapshot: &fret_core::SemanticsSnapshot,
    raw_node: &fret_core::SemanticsNode,
    exported_node: &UiSemanticsNodeV1,
    element: Option<u64>,
    cfg: &UiDiagnosticsConfig,
) -> Vec<UiSelectorV1> {
    let mut out = Vec::new();

    if let Some(id) = raw_node.test_id.as_deref() {
        out.push(UiSelectorV1::TestId { id: id.to_string() });
    }

    let role = semantics_role_label(raw_node.role).to_string();
    if let Some(name) = exported_node.label.as_deref() {
        if !(cfg.redact_text && is_redacted_string(name)) {
            let ancestors = selector_ancestors_for(snapshot, raw_node);
            if !ancestors.is_empty() {
                out.push(UiSelectorV1::RoleAndPath {
                    role: role.clone(),
                    name: name.to_string(),
                    ancestors,
                });
            }
            out.push(UiSelectorV1::RoleAndName {
                role: role.clone(),
                name: name.to_string(),
            });
        }
    }

    if let Some(element) = element {
        out.push(UiSelectorV1::GlobalElementId { element });
    }

    out.push(UiSelectorV1::NodeId {
        node: raw_node.id.data().as_ffi(),
    });
    out
}

pub(super) fn best_selector_for_node(
    snapshot: &fret_core::SemanticsSnapshot,
    raw_node: &fret_core::SemanticsNode,
    element: Option<u64>,
    cfg: &UiDiagnosticsConfig,
) -> Option<UiSelectorV1> {
    let exported =
        UiSemanticsNodeV1::from_node(raw_node, cfg.redact_text, cfg.max_debug_string_bytes);
    suggest_selectors(snapshot, raw_node, &exported, element, cfg)
        .into_iter()
        .next()
}
