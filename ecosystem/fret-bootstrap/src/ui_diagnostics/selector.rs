use std::collections::{HashMap, HashSet};

use fret_core::{AppWindowId, NodeId, SemanticsRole};
use fret_ui::elements::ElementRuntime;
use slotmap::Key as _;

use super::{UiDiagnosticsConfig, UiRoleAndNameV1, UiSelectorV1, UiSemanticsNodeV1};

mod validate;
pub(super) use validate::{best_selector_for_node_validated, inspect_selector_candidates_report};

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
        SemanticsRole::Status => "status",
        SemanticsRole::Log => "log",
        SemanticsRole::Button => "button",
        SemanticsRole::Link => "link",
        SemanticsRole::Image => "image",
        SemanticsRole::Checkbox => "checkbox",
        SemanticsRole::Switch => "switch",
        SemanticsRole::Slider => "slider",
        SemanticsRole::ProgressBar => "progress_bar",
        SemanticsRole::ScrollBar => "scroll_bar",
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
        "status" => SemanticsRole::Status,
        "log" => SemanticsRole::Log,
        "button" => SemanticsRole::Button,
        "link" => SemanticsRole::Link,
        "image" => SemanticsRole::Image,
        "checkbox" => SemanticsRole::Checkbox,
        "switch" => SemanticsRole::Switch,
        "slider" => SemanticsRole::Slider,
        "progress_bar" => SemanticsRole::ProgressBar,
        "scroll_bar" | "scrollbar" => SemanticsRole::ScrollBar,
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

pub(super) fn extend_test_id_chrome_fallback<'a, FScope, FRoot>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    index: &SemanticsIndex<'a>,
    id: &str,
    in_scope: &FScope,
    matches_root_z: &FRoot,
    matches: &mut Vec<&'a fret_core::SemanticsNode>,
) -> bool
where
    FScope: Fn(u64) -> bool,
    FRoot: Fn(u64) -> bool,
{
    if !matches.is_empty() {
        return false;
    }

    // Many interactive surfaces expose the actionable paint/hit-test node as `*.chrome`.
    // When the logical base test_id is absent from the selectable semantics tree, allow a unique
    // chrome node to satisfy the selector so diag scripts can keep targeting the caller-owned id.
    let chrome_id = format!("{id}.chrome");
    let chrome_id = chrome_id.as_str();
    let mut chrome_matches: Vec<&'a fret_core::SemanticsNode> = snapshot
        .nodes
        .iter()
        .filter(|n| {
            let node_id = n.id.data().as_ffi();
            index.is_selectable(node_id)
                && in_scope(node_id)
                && matches_root_z(node_id)
                && n.test_id.as_deref() == Some(chrome_id)
        })
        .collect();

    if chrome_matches.len() != 1 {
        return false;
    }

    matches.append(&mut chrome_matches);
    true
}

#[cfg(test)]
#[allow(dead_code)]
pub(super) fn select_semantics_node<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    selector: &UiSelectorV1,
) -> Option<&'a fret_core::SemanticsNode> {
    select_semantics_node_scoped(snapshot, window, element_runtime, selector, None)
}
pub(super) fn select_semantics_node_scoped<'a>(
    snapshot: &'a fret_core::SemanticsSnapshot,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    selector: &UiSelectorV1,
    scope_root: Option<u64>,
) -> Option<&'a fret_core::SemanticsNode> {
    let index = SemanticsIndex::new(snapshot);
    let want_root_z_index = match selector {
        UiSelectorV1::RoleAndName { root_z_index, .. } => *root_z_index,
        UiSelectorV1::RoleAndPath { root_z_index, .. } => *root_z_index,
        UiSelectorV1::TestId { root_z_index, .. } => *root_z_index,
        UiSelectorV1::GlobalElementId { root_z_index, .. } => *root_z_index,
        UiSelectorV1::NodeId { root_z_index, .. } => *root_z_index,
    };

    let in_scope = |id: u64| -> bool {
        scope_root
            .map(|root| index.is_descendant_of_or_self(id, root))
            .unwrap_or(true)
    };
    let matches_root_z = |id: u64| -> bool {
        want_root_z_index
            .map(|z| index.root_z_for(id) == z)
            .unwrap_or(true)
    };

    match selector {
        UiSelectorV1::NodeId { node, .. } => index.by_id.get(node).copied().filter(|n| {
            let id = n.id.data().as_ffi();
            index.is_selectable(id) && in_scope(id) && matches_root_z(id)
        }),
        UiSelectorV1::RoleAndName { role, name, .. } => {
            let role = parse_semantics_role(role)?;
            super::pick::pick_best_match(
                snapshot.nodes.iter().filter(|n| {
                    let id = n.id.data().as_ffi();
                    index.is_selectable(id)
                        && in_scope(id)
                        && matches_root_z(id)
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
            ..
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
                        && in_scope(id)
                        && matches_root_z(id)
                        && n.role == role
                        && n.label.as_deref().is_some_and(|label| label == name)
                        && index.ancestors_match_subsequence(n.parent, &parsed_ancestors)
                }),
                &index,
            )
        }
        UiSelectorV1::TestId { id, .. } => super::pick::pick_best_match(
            snapshot.nodes.iter().filter(|n| {
                let node_id = n.id.data().as_ffi();
                index.is_selectable(node_id)
                    && in_scope(node_id)
                    && matches_root_z(node_id)
                    && n.test_id.as_deref().is_some_and(|v| v == id)
            }),
            &index,
        )
        .or_else(|| {
            // Fallback for debugging: allow selecting hidden nodes if no visible match exists.
            super::pick::pick_best_match(
                snapshot.nodes.iter().filter(|n| {
                    let node_id = n.id.data().as_ffi();
                    in_scope(node_id)
                        && matches_root_z(node_id)
                        && n.test_id.as_deref().is_some_and(|v| v == id)
                }),
                &index,
            )
        }),
        UiSelectorV1::GlobalElementId { element, .. } => {
            let node = element_runtime.and_then(|runtime| {
                runtime.node_for_element(window, fret_ui::elements::GlobalElementId(*element))
            })?;
            let node_id = node.data().as_ffi();
            index.by_id.get(&node_id).copied().filter(|n| {
                let id = n.id.data().as_ffi();
                index.is_selectable(id) && in_scope(id) && matches_root_z(id)
            })
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

pub(super) fn is_redacted_string(s: &str) -> bool {
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
        out.push(UiSelectorV1::TestId {
            id: id.to_string(),
            root_z_index: None,
        });
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
                    root_z_index: None,
                });
            }
            out.push(UiSelectorV1::RoleAndName {
                role: role.clone(),
                name: name.to_string(),
                root_z_index: None,
            });
        }
    }

    if let Some(element) = element {
        out.push(UiSelectorV1::GlobalElementId {
            element,
            root_z_index: None,
        });
    }

    out.push(UiSelectorV1::NodeId {
        node: raw_node.id.data().as_ffi(),
        root_z_index: None,
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

#[cfg(any())]
mod legacy_inline_validate {
    #[derive(Debug, Clone, Copy)]
    struct SelectorEvalSummary {
        match_count: u32,
        chosen_node_id: Option<u64>,
        note: Option<&'static str>,
    }

    fn eval_selector_scoped(
        snapshot: &fret_core::SemanticsSnapshot,
        window: AppWindowId,
        element_runtime: Option<&ElementRuntime>,
        selector: &UiSelectorV1,
        scope_root: Option<u64>,
    ) -> SelectorEvalSummary {
        let index = SemanticsIndex::new(snapshot);
        let mut matches: Vec<&fret_core::SemanticsNode> = Vec::new();
        let mut note: Option<&'static str> = None;
        let want_root_z_index = match selector {
            UiSelectorV1::RoleAndName { root_z_index, .. } => *root_z_index,
            UiSelectorV1::RoleAndPath { root_z_index, .. } => *root_z_index,
            UiSelectorV1::TestId { root_z_index, .. } => *root_z_index,
            UiSelectorV1::GlobalElementId { root_z_index, .. } => *root_z_index,
            UiSelectorV1::NodeId { root_z_index, .. } => *root_z_index,
        };

        let in_scope = |id: u64| -> bool {
            scope_root
                .map(|root| index.is_descendant_of_or_self(id, root))
                .unwrap_or(true)
        };
        let matches_root_z = |id: u64| -> bool {
            want_root_z_index
                .map(|z| index.root_z_for(id) == z)
                .unwrap_or(true)
        };

        match selector {
            UiSelectorV1::NodeId { node, .. } => {
                if let Some(n) = index.by_id.get(node).copied().filter(|n| {
                    let id = n.id.data().as_ffi();
                    index.is_selectable(id) && in_scope(id) && matches_root_z(id)
                }) {
                    matches.push(n);
                }
            }
            UiSelectorV1::RoleAndName { role, name, .. } => {
                let Some(role) = parse_semantics_role(role) else {
                    return SelectorEvalSummary {
                        match_count: 0,
                        chosen_node_id: None,
                        note: Some("invalid_role"),
                    };
                };

                matches.extend(snapshot.nodes.iter().filter(|n| {
                    let id = n.id.data().as_ffi();
                    index.is_selectable(id)
                        && in_scope(id)
                        && matches_root_z(id)
                        && n.role == role
                        && n.label.as_deref() == Some(name)
                }));
            }
            UiSelectorV1::RoleAndPath {
                role,
                name,
                ancestors,
                ..
            } => {
                let Some(role) = parse_semantics_role(role) else {
                    return SelectorEvalSummary {
                        match_count: 0,
                        chosen_node_id: None,
                        note: Some("invalid_role"),
                    };
                };

                let mut parsed_ancestors: Vec<(SemanticsRole, &str)> =
                    Vec::with_capacity(ancestors.len());
                for a in ancestors {
                    let Some(r) = parse_semantics_role(&a.role) else {
                        return SelectorEvalSummary {
                            match_count: 0,
                            chosen_node_id: None,
                            note: Some("invalid_ancestor_role"),
                        };
                    };
                    parsed_ancestors.push((r, a.name.as_str()));
                }

                matches.extend(snapshot.nodes.iter().filter(|n| {
                    let id = n.id.data().as_ffi();
                    index.is_selectable(id)
                        && in_scope(id)
                        && matches_root_z(id)
                        && n.role == role
                        && n.label.as_deref() == Some(name)
                        && index.ancestors_match_subsequence(n.parent, &parsed_ancestors)
                }));
            }
            UiSelectorV1::TestId { id, .. } => {
                matches.extend(snapshot.nodes.iter().filter(|n| {
                    let node_id = n.id.data().as_ffi();
                    index.is_selectable(node_id)
                        && in_scope(node_id)
                        && matches_root_z(node_id)
                        && n.test_id.as_deref() == Some(id)
                }));
                if matches.is_empty() {
                    if extend_test_id_chrome_fallback(
                        snapshot,
                        &index,
                        id,
                        &in_scope,
                        &matches_root_z,
                        &mut matches,
                    ) {
                        note = Some("fallback_chrome_suffix");
                    }
                }
                if matches.is_empty() {
                    // Fallback for debugging: allow selecting hidden nodes if no visible match exists.
                    note = Some("fallback_hidden_nodes");
                    matches.extend(snapshot.nodes.iter().filter(|n| {
                        let node_id = n.id.data().as_ffi();
                        in_scope(node_id)
                            && matches_root_z(node_id)
                            && n.test_id.as_deref() == Some(id)
                    }));
                }
            }
            UiSelectorV1::GlobalElementId { element, .. } => {
                let Some(node) = element_runtime.and_then(|runtime| {
                    runtime.node_for_element(window, fret_ui::elements::GlobalElementId(*element))
                }) else {
                    return SelectorEvalSummary {
                        match_count: 0,
                        chosen_node_id: None,
                        note: Some("element_runtime_missing"),
                    };
                };
                let node_id = node.data().as_ffi();
                if let Some(n) = index.by_id.get(&node_id).copied().filter(|n| {
                    let id = n.id.data().as_ffi();
                    index.is_selectable(id) && in_scope(id) && matches_root_z(id)
                }) {
                    matches.push(n);
                }
            }
        }

        let match_count = matches.len().min(u32::MAX as usize) as u32;
        let chosen = super::pick::pick_best_match(matches.iter().copied(), &index);
        let chosen_node_id = chosen.map(|n| n.id.data().as_ffi());

        SelectorEvalSummary {
            match_count,
            chosen_node_id,
            note,
        }
    }

    pub(super) fn best_selector_for_node_validated(
        snapshot: &fret_core::SemanticsSnapshot,
        window: AppWindowId,
        element_runtime: Option<&ElementRuntime>,
        raw_node: &fret_core::SemanticsNode,
        element: Option<u64>,
        cfg: &UiDiagnosticsConfig,
    ) -> Option<UiSelectorV1> {
        let exported =
            UiSemanticsNodeV1::from_node(raw_node, cfg.redact_text, cfg.max_debug_string_bytes);
        let candidates = candidate_selectors_for_node(snapshot, raw_node, &exported, element, cfg);
        let target_node_id = raw_node.id.data().as_ffi();
        let index = SemanticsIndex::new(snapshot);
        let target_root_z_index = index.root_z_for(target_node_id);

        for selector in candidates {
            // `NodeId` is always unique but is intentionally an in-run-only reference.
            // Keep it out of the "validated" fast path so callers can fall back to more stable selectors.
            if matches!(selector, UiSelectorV1::NodeId { .. }) {
                continue;
            }
            let eval = eval_selector_scoped(snapshot, window, element_runtime, &selector, None);
            if eval.match_count == 1 && eval.chosen_node_id == Some(target_node_id) {
                return Some(selector);
            }
        }

        // If nothing is unique across multiple roots, try again with root-z gating.
        let candidates = candidate_selectors_for_node(snapshot, raw_node, &exported, element, cfg);
        for selector in candidates {
            if matches!(selector, UiSelectorV1::NodeId { .. }) {
                continue;
            }

            let gated = match selector {
                UiSelectorV1::RoleAndName {
                    role,
                    name,
                    root_z_index: None,
                } => UiSelectorV1::RoleAndName {
                    role,
                    name,
                    root_z_index: Some(target_root_z_index),
                },
                UiSelectorV1::RoleAndPath {
                    role,
                    name,
                    ancestors,
                    root_z_index: None,
                } => UiSelectorV1::RoleAndPath {
                    role,
                    name,
                    ancestors,
                    root_z_index: Some(target_root_z_index),
                },
                UiSelectorV1::TestId {
                    id,
                    root_z_index: None,
                } => UiSelectorV1::TestId {
                    id,
                    root_z_index: Some(target_root_z_index),
                },
                UiSelectorV1::GlobalElementId {
                    element,
                    root_z_index: None,
                } => UiSelectorV1::GlobalElementId {
                    element,
                    root_z_index: Some(target_root_z_index),
                },
                other => other,
            };

            let eval = eval_selector_scoped(snapshot, window, element_runtime, &gated, None);
            if eval.match_count == 1 && eval.chosen_node_id == Some(target_node_id) {
                return Some(gated);
            }
        }

        None
    }

    pub(super) fn inspect_selector_candidates_report(
        snapshot: &fret_core::SemanticsSnapshot,
        window: AppWindowId,
        element_runtime: Option<&ElementRuntime>,
        raw_node: &fret_core::SemanticsNode,
        element: Option<u64>,
        cfg: &UiDiagnosticsConfig,
    ) -> String {
        let exported =
            UiSemanticsNodeV1::from_node(raw_node, cfg.redact_text, cfg.max_debug_string_bytes);
        let mut candidates =
            candidate_selectors_for_node(snapshot, raw_node, &exported, element, cfg);

        // Dedupe by stable JSON encoding (selectors can repeat for short ancestor paths).
        let mut seen: HashSet<String> = HashSet::new();
        candidates.retain(|sel| {
            let Ok(json) = serde_json::to_string(sel) else {
                return false;
            };
            seen.insert(json)
        });

        let target_node_id = raw_node.id.data().as_ffi();

        let mut scored: Vec<(i32, String)> = Vec::new();
        for selector in candidates {
            let eval = eval_selector_scoped(snapshot, window, element_runtime, &selector, None);
            let chosen_is_target = eval.chosen_node_id == Some(target_node_id);

            let (kind, base): (&'static str, i32) = match &selector {
                UiSelectorV1::TestId { .. } => ("test_id", 900),
                UiSelectorV1::RoleAndPath { .. } => ("role_path", 850),
                UiSelectorV1::RoleAndName { .. } => ("role_name", 700),
                UiSelectorV1::GlobalElementId { .. } => ("global_element_id", 650),
                UiSelectorV1::NodeId { .. } => ("node_id", 100),
            };

            let mut score = base;
            if eval.match_count == 0 {
                score -= 500;
            } else if eval.match_count == 1 {
                score += 300;
            } else {
                score -= (eval.match_count.saturating_sub(1) as i32) * 50;
            }

            if chosen_is_target {
                score += 500;
            } else {
                score -= 1000;
            }

            if let UiSelectorV1::RoleAndPath { ancestors, .. } = &selector {
                score -= (ancestors.len() as i32) * 5;
            }

            let selector_json =
                serde_json::to_string(&selector).unwrap_or_else(|_| "<unserializable>".to_string());
            let note = eval.note.unwrap_or("-");
            let chosen = eval
                .chosen_node_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_string());

            let line = format!(
                "- score={score} ok={chosen_is_target} matches={} chosen={chosen} kind={kind} note={note} selector={selector_json}",
                eval.match_count
            );
            scored.push((score, line));
        }

        scored.sort_by(|(a, _), (b, _)| b.cmp(a));
        scored
            .into_iter()
            .take(12)
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use fret_core::{
            Point, Px, Rect, SemanticsActions, SemanticsFlags, SemanticsNode, SemanticsRoot,
            SemanticsSnapshot, Size,
        };
        use slotmap::KeyData;

        fn node_id(id: u64) -> NodeId {
            NodeId::from(KeyData::from_ffi(id))
        }

        fn window_id(id: u64) -> AppWindowId {
            AppWindowId::from(KeyData::from_ffi(id))
        }

        fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
            Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
        }

        fn semantics_node(
            id: u64,
            parent: Option<u64>,
            role: SemanticsRole,
            bounds: Rect,
            label: &str,
            test_id: Option<&str>,
        ) -> SemanticsNode {
            SemanticsNode {
                id: node_id(id),
                parent: parent.map(node_id),
                role,
                bounds,
                flags: SemanticsFlags::default(),
                test_id: test_id.map(|s| s.to_string()),
                active_descendant: None,
                pos_in_set: None,
                set_size: None,
                label: Some(label.to_string()),
                value: None,
                extra: Default::default(),
                text_selection: None,
                text_composition: None,
                actions: SemanticsActions::default(),
                labelled_by: Vec::new(),
                described_by: Vec::new(),
                controls: Vec::new(),
                inline_spans: Vec::new(),
            }
        }

        #[test]
        fn best_selector_validated_prefers_unique_role_path_when_test_id_ambiguous() {
            let window = window_id(1);

            let snapshot = SemanticsSnapshot {
                window,
                roots: vec![SemanticsRoot {
                    root: node_id(1),
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
                    semantics_node(
                        1,
                        None,
                        SemanticsRole::Window,
                        rect(0.0, 0.0, 100.0, 100.0),
                        "root",
                        None,
                    ),
                    semantics_node(
                        10,
                        Some(1),
                        SemanticsRole::Group,
                        rect(0.0, 0.0, 100.0, 10.0),
                        "toolbar",
                        None,
                    ),
                    semantics_node(
                        11,
                        Some(1),
                        SemanticsRole::Group,
                        rect(0.0, 90.0, 100.0, 10.0),
                        "footer",
                        None,
                    ),
                    semantics_node(
                        2,
                        Some(10),
                        SemanticsRole::Button,
                        rect(0.0, 0.0, 10.0, 10.0),
                        "OK",
                        Some("dup"),
                    ),
                    semantics_node(
                        3,
                        Some(11),
                        SemanticsRole::Button,
                        rect(0.0, 90.0, 10.0, 10.0),
                        "OK",
                        Some("dup"),
                    ),
                ],
            };

            let mut cfg = UiDiagnosticsConfig::default();
            cfg.redact_text = false;
            let node = &snapshot.nodes[3];

            let sel = best_selector_for_node_validated(&snapshot, window, None, node, None, &cfg)
                .expect("expected a validated selector");

            match sel {
                UiSelectorV1::RoleAndPath { ancestors, .. } => {
                    assert!(
                        ancestors.iter().any(|a| a.name == "toolbar"),
                        "expected the selector path to include the unique ancestor"
                    );
                }
                other => panic!("expected role+path selector, got {other:?}"),
            }
        }

        #[test]
        fn selector_root_z_index_gates_test_id_resolution() {
            let window = window_id(1);

            let snapshot = SemanticsSnapshot {
                window,
                roots: vec![
                    SemanticsRoot {
                        root: node_id(1),
                        visible: true,
                        blocks_underlay_input: false,
                        hit_testable: true,
                        z_index: 0,
                    },
                    SemanticsRoot {
                        root: node_id(100),
                        visible: true,
                        blocks_underlay_input: false,
                        hit_testable: true,
                        z_index: 10,
                    },
                ],
                barrier_root: None,
                focus_barrier_root: None,
                focus: None,
                captured: None,
                nodes: vec![
                    semantics_node(
                        1,
                        None,
                        SemanticsRole::Window,
                        rect(0.0, 0.0, 100.0, 100.0),
                        "root-a",
                        None,
                    ),
                    semantics_node(
                        100,
                        None,
                        SemanticsRole::Window,
                        rect(0.0, 0.0, 100.0, 100.0),
                        "root-b",
                        None,
                    ),
                    semantics_node(
                        2,
                        Some(1),
                        SemanticsRole::Button,
                        rect(0.0, 0.0, 10.0, 10.0),
                        "OK",
                        Some("dup"),
                    ),
                    semantics_node(
                        3,
                        Some(100),
                        SemanticsRole::Button,
                        rect(0.0, 0.0, 10.0, 10.0),
                        "OK",
                        Some("dup"),
                    ),
                ],
            };

            let selector = UiSelectorV1::TestId {
                id: "dup".to_string(),
                root_z_index: Some(0),
            };
            let picked =
                select_semantics_node_scoped(&snapshot, window, None, &selector, None).unwrap();
            assert_eq!(picked.id, node_id(2));

            let selector = UiSelectorV1::TestId {
                id: "dup".to_string(),
                root_z_index: Some(10),
            };
            let picked =
                select_semantics_node_scoped(&snapshot, window, None, &selector, None).unwrap();
            assert_eq!(picked.id, node_id(3));
        }

        #[test]
        fn selector_test_id_falls_back_to_unique_chrome_suffix() {
            let window = window_id(1);

            let mut chrome_actions = SemanticsActions::default();
            chrome_actions.invoke = true;

            let snapshot = SemanticsSnapshot {
                window,
                roots: vec![SemanticsRoot {
                    root: node_id(1),
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
                    semantics_node(
                        1,
                        None,
                        SemanticsRole::Window,
                        rect(0.0, 0.0, 100.0, 100.0),
                        "root",
                        None,
                    ),
                    semantics_node(
                        2,
                        Some(1),
                        SemanticsRole::Group,
                        rect(0.0, 0.0, 40.0, 20.0),
                        "trigger-wrapper",
                        None,
                    ),
                    SemanticsNode {
                        actions: chrome_actions,
                        ..semantics_node(
                            3,
                            Some(2),
                            SemanticsRole::Button,
                            rect(0.0, 0.0, 40.0, 20.0),
                            "Open",
                            Some("sheet-trigger.chrome"),
                        )
                    },
                ],
            };

            let selector = UiSelectorV1::TestId {
                id: "sheet-trigger".to_string(),
                root_z_index: None,
            };
            let resolved = select_semantics_node_scoped(&snapshot, window, None, &selector, None)
                .expect("expected chrome fallback to resolve");

            assert_eq!(resolved.id.data().as_ffi(), 3);
            assert_eq!(resolved.test_id.as_deref(), Some("sheet-trigger.chrome"));
        }
    }
}
