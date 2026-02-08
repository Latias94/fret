use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use fret_app::App;
use fret_bootstrap::ui_diagnostics::{UiDiagnosticsBundleV1, UiSemanticsNodeV1, UiSemanticsRootV1};

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
    for root in index.roots.iter() {
        push_rows_from_node(
            &mut out,
            index,
            root.root,
            0,
            expanded,
            visible.as_ref(),
            has_search,
        );
    }
    out
}

fn push_rows_from_node(
    out: &mut Vec<SemanticsRow>,
    index: &SemanticsIndex,
    id: u64,
    depth: usize,
    expanded: &HashSet<u64>,
    visible: Option<&HashSet<u64>>,
    force_expand_visible: bool,
) {
    if let Some(visible) = visible {
        if !visible.contains(&id) {
            return;
        }
    }

    if index.node(id).is_none() {
        return;
    };

    let children = index.children(id);
    let has_children = !children.is_empty();
    let is_expanded = force_expand_visible || expanded.contains(&id);

    out.push(SemanticsRow {
        id,
        depth,
        has_children,
        is_expanded: has_children && is_expanded,
    });

    if has_children && is_expanded {
        for child in children {
            push_rows_from_node(
                out,
                index,
                *child,
                depth + 1,
                expanded,
                visible,
                force_expand_visible,
            );
        }
    }
}

fn node_matches(node: &UiSemanticsNodeV1, search_lower: &str) -> bool {
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
    false
}

pub(crate) fn node_label(node: &UiSemanticsNodeV1) -> String {
    let role = &node.role;
    let test_id = node.test_id.as_deref().unwrap_or("-");
    let label = node.label.as_deref().unwrap_or("-");
    format!("{role}  test_id={test_id}  label={label}  id={}", node.id)
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
