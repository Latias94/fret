#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCacheRootStatsV1 {
    pub root: u64,
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub reused: bool,
    pub contained_layout: bool,
    #[serde(default)]
    pub contained_relayout_in_frame: bool,
    pub paint_replayed_ops: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_child_nodes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtree_nodes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtree_nodes_truncated_at: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_in_semantics: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_old_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_new_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_old_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_new_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_old_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_new_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_frame_id: Option<u64>,
    #[serde(default)]
    pub reuse_reason: Option<String>,
}

impl UiCacheRootStatsV1 {
    fn from_stats(
        window: AppWindowId,
        ui: &UiTree<App>,
        element_runtime: Option<&ElementRuntime>,
        semantics: Option<&UiSemanticsSnapshotV1>,
        contained_relayout_roots: &HashSet<fret_core::NodeId>,
        stats: &fret_ui::tree::UiDebugCacheRootStats,
        max_debug_string_bytes: usize,
    ) -> Self {
        let element_path = stats.element.and_then(|id| {
            element_runtime.and_then(|runtime| runtime.debug_path_for_element(window, id))
        });

        let direct_child_nodes = ui.children(stats.root).len().min(u32::MAX as usize) as u32;

        // Keep bundles bounded: cache roots can cover large subtrees in real apps.
        const MAX_SUBTREE_NODES: usize = 50_000;
        let mut subtree_nodes_truncated_at: Option<u32> = None;
        let mut seen: HashSet<fret_core::NodeId> = HashSet::new();
        let mut stack: Vec<fret_core::NodeId> = vec![stats.root];
        while let Some(node) = stack.pop() {
            if !seen.insert(node) {
                continue;
            }
            if seen.len() > MAX_SUBTREE_NODES {
                subtree_nodes_truncated_at = Some(MAX_SUBTREE_NODES as u32);
                break;
            }
            for child in ui.children(node) {
                stack.push(child);
            }
        }

        let root_in_semantics = semantics.map(|snap| {
            let id = stats.root.data().as_ffi();
            snap.nodes.iter().any(|n| n.id == id)
        });
        let contained_relayout_in_frame = contained_relayout_roots.contains(&stats.root);

        let (
            children_last_set_location,
            children_last_set_old_len,
            children_last_set_new_len,
            children_last_set_old_elements_head,
            children_last_set_new_elements_head,
            children_last_set_old_elements_head_paths,
            children_last_set_new_elements_head_paths,
            children_last_set_frame_id,
        ) = ui
            .debug_set_children_write_for(stats.root)
            .map(|w| {
                let old_elements_head: Vec<_> = w.old_elements_head.iter().flatten().copied().collect();
                let new_elements_head: Vec<_> = w.new_elements_head.iter().flatten().copied().collect();

                let old_paths: Vec<String> = old_elements_head
                    .iter()
                    .filter_map(|id| {
                        element_runtime
                            .and_then(|runtime| runtime.debug_path_for_element(window, *id))
                    })
                    .collect();
                let new_paths: Vec<String> = new_elements_head
                    .iter()
                    .filter_map(|id| {
                        element_runtime
                            .and_then(|runtime| runtime.debug_path_for_element(window, *id))
                    })
                    .collect();

                (
                    Some(format!("{}:{}:{}", w.file, w.line, w.column)),
                    Some(w.old_len),
                    Some(w.new_len),
                    old_elements_head.iter().map(|id| id.0).collect::<Vec<_>>(),
                    new_elements_head.iter().map(|id| id.0).collect::<Vec<_>>(),
                    old_paths,
                    new_paths,
                    Some(w.frame_id.0),
                )
            })
            .unwrap_or((
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                None,
            ));

        let mut out = Self {
            root: stats.root.data().as_ffi(),
            element: stats.element.map(|id| id.0),
            element_path,
            reused: stats.reused,
            contained_layout: stats.contained_layout,
            contained_relayout_in_frame,
            paint_replayed_ops: stats.paint_replayed_ops,
            direct_child_nodes: Some(direct_child_nodes),
            subtree_nodes: Some(seen.len().min(u32::MAX as usize) as u32),
            subtree_nodes_truncated_at,
            root_in_semantics,
            children_last_set_location,
            children_last_set_old_len,
            children_last_set_new_len,
            children_last_set_old_elements_head,
            children_last_set_new_elements_head,
            children_last_set_old_elements_head_paths,
            children_last_set_new_elements_head_paths,
            children_last_set_frame_id,
            reuse_reason: Some(stats.reuse_reason.as_str().to_string()),
        };

        truncate_opt_string_bytes(&mut out.element_path, max_debug_string_bytes);
        truncate_opt_string_bytes(&mut out.children_last_set_location, max_debug_string_bytes);
        truncate_vec_string_bytes(
            &mut out.children_last_set_old_elements_head_paths,
            max_debug_string_bytes,
        );
        truncate_vec_string_bytes(
            &mut out.children_last_set_new_elements_head_paths,
            max_debug_string_bytes,
        );
        truncate_opt_string_bytes(&mut out.reuse_reason, max_debug_string_bytes);

        out
    }
}
