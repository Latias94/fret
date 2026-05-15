use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use fret_app::App;
use fret_bootstrap::ui_diagnostics::{
    UiDiagnosticsBundleV1, UiSemanticsNodeV1, UiSemanticsRootV1,
};

use crate::State;

#[derive(Debug, Clone)]
pub(crate) struct SemanticsIndex {
    pub window: u64,
    pub roots: Vec<UiSemanticsRootV1>,
    pub nodes_by_id: HashMap<u64, UiSemanticsNodeV1>,
    pub children_by_parent: HashMap<u64, Vec<u64>>,
}

impl SemanticsIndex {
    fn from_roots_and_nodes(
        window: u64,
        roots: Vec<UiSemanticsRootV1>,
        nodes: Vec<UiSemanticsNodeV1>,
    ) -> Self {
        let mut nodes_by_id: HashMap<u64, UiSemanticsNodeV1> = HashMap::new();
        nodes_by_id.reserve(nodes.len());
        for n in nodes {
            nodes_by_id.insert(n.id, n);
        }

        let mut children_by_parent: HashMap<u64, Vec<u64>> = HashMap::new();
        for (id, node) in nodes_by_id.iter() {
            if let Some(parent) = node.parent {
                children_by_parent.entry(parent).or_default().push(*id);
            }
        }

        for children in children_by_parent.values_mut() {
            children.sort_unstable();
        }

        Self {
            window,
            roots,
            nodes_by_id,
            children_by_parent,
        }
    }

    pub(crate) fn node(&self, id: u64) -> Option<&UiSemanticsNodeV1> {
        self.nodes_by_id.get(&id)
    }

    pub(crate) fn children(&self, id: u64) -> &[u64] {
        self.children_by_parent
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SemanticsRow {
    pub id: u64,
    pub depth: usize,
    pub has_children: bool,
    pub is_expanded: bool,
}

pub(crate) fn refresh_semantics_cache_if_needed(app: &mut App, st: &State) {
    let bundle_text = app
        .models()
        .read(&st.last_bundle_dump_bundle_json, |v| v.clone())
        .ok()
        .flatten();

    let Some(bundle_text) = bundle_text else {
        let _ = app.models_mut().update(&st.semantics_cache, |v| *v = None);
        let _ = app.models_mut().update(&st.semantics_error, |v| *v = None);
        let _ = app
            .models_mut()
            .update(&st.semantics_source_hash, |v| *v = None);
        return;
    };

    let new_hash = hash_str(bundle_text.as_ref());
    let old_hash = app
        .models()
        .read(&st.semantics_source_hash, |v| *v)
        .ok()
        .flatten();
    if old_hash == Some(new_hash) {
        return;
    }

    let parsed = parse_latest_semantics_index(bundle_text.as_ref());
    match parsed {
        Ok(index) => {
            let selected_id = app
                .models()
                .read(&st.semantics_selected_id, |v| *v)
                .ok()
                .flatten();
            let text = selected_node_json(&index, selected_id);
            let _ = app
                .models_mut()
                .update(&st.semantics_selected_node_json, |v| *v = text);
            let _ = app
                .models_mut()
                .update(&st.semantics_cache, |v| *v = Some(Arc::new(index)));
            let _ = app.models_mut().update(&st.semantics_error, |v| *v = None);
        }
        Err(err) => {
            let _ = app.models_mut().update(&st.semantics_cache, |v| *v = None);
            let _ = app.models_mut().update(&st.semantics_error, |v| {
                *v = Some(Arc::<str>::from(err));
            });
        }
    }

    let _ = app
        .models_mut()
        .update(&st.semantics_source_hash, |v| *v = Some(new_hash));
}

fn parse_latest_semantics_index(bundle_text: &str) -> Result<SemanticsIndex, String> {
    let bundle: UiDiagnosticsBundleV1 =
        serde_json::from_str(bundle_text).map_err(|e| format!("bundle parse failed: {e}"))?;

    let window = bundle
        .windows
        .first()
        .ok_or_else(|| "bundle contains no windows".to_string())?;

    let snapshot = window
        .snapshots
        .iter()
        .rev()
        .find(|s| s.debug.semantics.is_some())
        .or_else(|| window.snapshots.last())
        .ok_or_else(|| "window contains no snapshots".to_string())?;

    let semantics = snapshot
        .debug
        .semantics
        .clone()
        .ok_or_else(|| "snapshot contains no semantics".to_string())?;

    Ok(SemanticsIndex::from_roots_and_nodes(
        semantics.window,
        semantics.roots,
        semantics.nodes,
    ))
}

pub(crate) fn compute_rows(
    index: &SemanticsIndex,
    expanded: &HashSet<u64>,
    search: &str,
) -> Vec<SemanticsRow> {
    let search = search.trim().to_lowercase();
    let has_search = !search.is_empty();

    let visible: Option<HashSet<u64>> = has_search.then(|| {
        let mut vis: HashSet<u64> = HashSet::new();
        for node in index.nodes_by_id.values() {
            if node_matches(node, &search) {
                let mut cur = Some(node.id);
                while let Some(id) = cur {
                    if !vis.insert(id) {
                        break;
                    }
                    cur = index.node(id).and_then(|n| n.parent);
                }
            }
        }
        vis
    });

    let mut out = Vec::new();
    let mut stack = Vec::with_capacity(index.roots.len().max(1));
    for root in index.roots.iter().rev() {
        stack.push((root.root, 0usize));
    }

    while let Some((id, depth)) = stack.pop() {
        if let Some(visible) = visible.as_ref() {
            if !visible.contains(&id) {
                continue;
            }
        }

        if index.node(id).is_none() {
            continue;
        };

        let children = index.children(id);
        let has_children = !children.is_empty();
        let is_expanded = has_search || expanded.contains(&id);

        out.push(SemanticsRow {
            id,
            depth,
            has_children,
            is_expanded: has_children && is_expanded,
        });

        if has_children && is_expanded {
            for child in children.iter().rev() {
                stack.push((*child, depth + 1));
            }
        }
    }

    out
}

fn node_matches(node: &UiSemanticsNodeV1, search_lower: &str) -> bool {
    if node.id.to_string().contains(search_lower) {
        return true;
    }
    if let Some(parent) = node.parent {
        let parent_text = format!("parent={parent}");
        if parent.to_string().contains(search_lower) || parent_text.contains(search_lower) {
            return true;
        }
    }
    if node.role.to_lowercase().contains(search_lower) {
        return true;
    }
    if let Some(s) = node.test_id.as_deref() {
        if s.to_lowercase().contains(search_lower) {
            return true;
        }
    }
    if let Some(s) = node.label.as_deref() {
        if s.to_lowercase().contains(search_lower) {
            return true;
        }
    }
    if let Some(s) = node.value.as_deref() {
        if s.to_lowercase().contains(search_lower) {
            return true;
        }
    }
    let bounds = &node.bounds;
    let bounds_text = format!(
        "{:.1},{:.1},{:.1},{:.1}",
        bounds.x, bounds.y, bounds.w, bounds.h
    )
    .to_lowercase();
    if bounds_text.contains(search_lower) {
        return true;
    }
    false
}

pub(crate) fn node_label(node: &UiSemanticsNodeV1) -> String {
    let role = &node.role;
    let test_id = node.test_id.as_deref().unwrap_or("-");
    let label = node.label.as_deref().unwrap_or("-");
    format!("{role}  test_id={test_id}  label={label}  id={}", node.id)
}

pub(crate) fn layout_node_label(node: &UiSemanticsNodeV1) -> String {
    let bounds = &node.bounds;
    let test_id = node.test_id.as_deref().unwrap_or("-");
    let parent = node
        .parent
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    format!(
        "id={} parent={} bounds=({:.1},{:.1} {:.1}x{:.1}) role={} test_id={}",
        node.id, parent, bounds.x, bounds.y, bounds.w, bounds.h, node.role, test_id
    )
}

pub(crate) fn element_node_label(node: &UiSemanticsNodeV1) -> String {
    let test_id = node.test_id.as_deref().unwrap_or("-");
    let parent = node
        .parent
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    format!(
        "sem_node={} parent={} role={} test_id={} labelled_by={} described_by={} controls={}",
        node.id,
        parent,
        node.role,
        test_id,
        format_refs(&node.labelled_by),
        format_refs(&node.described_by),
        format_refs(&node.controls)
    )
}

fn format_refs(values: &[u64]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

pub(crate) fn selected_node_json(index: &SemanticsIndex, selected_id: Option<u64>) -> String {
    selected_id
        .and_then(|id| index.node(id))
        .and_then(|n| serde_json::to_string_pretty(n).ok())
        .unwrap_or_default()
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_bootstrap::ui_diagnostics::{
        RectV1, UiSemanticsActionsV1, UiSemanticsFlagsV1,
    };

    fn root() -> UiSemanticsRootV1 {
        UiSemanticsRootV1 {
            root: 1,
            visible: true,
            blocks_underlay_input: false,
            hit_testable: true,
            z_index: 0,
        }
    }

    fn node(id: u64, parent: Option<u64>) -> UiSemanticsNodeV1 {
        UiSemanticsNodeV1 {
            id,
            parent,
            role: if id == 1 { "root" } else { "button" }.to_string(),
            bounds: RectV1 {
                x: 0.0,
                y: id as f32,
                w: 10.0,
                h: 10.0,
            },
            flags: UiSemanticsFlagsV1::default(),
            test_id: Some(format!("node-{id}")),
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            level: None,
            label: Some(format!("Node {id}")),
            value: None,
            text_selection: None,
            text_composition: None,
            actions: UiSemanticsActionsV1::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
            scroll: Default::default(),
        }
    }

    #[test]
    fn compute_rows_search_matches_id_parent_and_bounds() {
        let index = SemanticsIndex::from_roots_and_nodes(
            7,
            vec![root()],
            vec![node(1, None), node(42, Some(1))],
        );

        let by_id = compute_rows(&index, &HashSet::new(), "42");
        let by_parent = compute_rows(&index, &HashSet::new(), "parent=1");
        let by_bounds = compute_rows(&index, &HashSet::new(), "42.0");

        assert_eq!(by_id.iter().map(|r| r.id).collect::<Vec<_>>(), vec![1, 42]);
        assert_eq!(
            by_parent.iter().map(|r| r.id).collect::<Vec<_>>(),
            vec![1, 42]
        );
        assert_eq!(
            by_bounds.iter().map(|r| r.id).collect::<Vec<_>>(),
            vec![1, 42]
        );
    }

    #[test]
    fn secondary_tree_labels_surface_layout_and_identity_fields() {
        let mut n = node(42, Some(1));
        n.labelled_by = vec![7];
        n.described_by = vec![8, 9];
        n.controls = vec![10];

        let layout = layout_node_label(&n);
        let element = element_node_label(&n);

        assert!(layout.contains("bounds=(0.0,42.0 10.0x10.0)"));
        assert!(layout.contains("parent=1"));
        assert!(element.contains("sem_node=42"));
        assert!(element.contains("labelled_by=7"));
        assert!(element.contains("described_by=8,9"));
        assert!(element.contains("controls=10"));
    }

    #[test]
    fn compute_rows_handles_50k_flat_semantics_nodes() {
        let node_count = 50_000u64;
        let mut nodes = Vec::with_capacity(node_count as usize);
        nodes.push(node(1, None));
        for id in 2..=node_count {
            nodes.push(node(id, Some(1)));
        }
        let index = SemanticsIndex::from_roots_and_nodes(7, vec![root()], nodes);
        let expanded = HashSet::from([1]);

        let rows = compute_rows(&index, &expanded, "");

        assert_eq!(rows.len(), node_count as usize);
        assert_eq!(rows.first().map(|r| r.id), Some(1));
        assert_eq!(rows.last().map(|r| r.id), Some(node_count));
        assert_eq!(rows[0].depth, 0);
        assert_eq!(rows[1].depth, 1);
    }

    #[test]
    fn compute_rows_handles_50k_deep_semantics_tree_without_recursion() {
        let node_count = 50_000u64;
        let mut nodes = Vec::with_capacity(node_count as usize);
        nodes.push(node(1, None));
        let mut expanded = HashSet::with_capacity(node_count as usize);
        for id in 1..node_count {
            expanded.insert(id);
            nodes.push(node(id + 1, Some(id)));
        }
        let index = SemanticsIndex::from_roots_and_nodes(7, vec![root()], nodes);

        let rows = compute_rows(&index, &expanded, "");

        assert_eq!(rows.len(), node_count as usize);
        assert_eq!(rows.first().map(|r| r.id), Some(1));
        assert_eq!(rows.last().map(|r| r.id), Some(node_count));
        assert_eq!(rows.last().map(|r| r.depth), Some(node_count as usize - 1));
    }

    #[test]
    fn compute_rows_search_forces_visible_ancestor_path_on_large_tree() {
        let node_count = 50_000u64;
        let mut nodes = Vec::with_capacity(node_count as usize);
        nodes.push(node(1, None));
        for id in 1..node_count {
            nodes.push(node(id + 1, Some(id)));
        }
        let index = SemanticsIndex::from_roots_and_nodes(7, vec![root()], nodes);

        let rows = compute_rows(&index, &HashSet::new(), "node-50000");

        assert_eq!(rows.len(), node_count as usize);
        assert_eq!(rows.first().map(|r| r.id), Some(1));
        assert_eq!(rows.last().map(|r| r.id), Some(node_count));
        assert!(rows.iter().all(|r| r.is_expanded || r.id == node_count));
    }
}
