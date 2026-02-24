#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiRemovedSubtreeV1 {
    pub root: u64,
    #[serde(default)]
    pub root_element: Option<u64>,
    #[serde(default)]
    pub root_parent_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_element_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_element_path: Option<String>,
    #[serde(default)]
    pub root_parent: Option<u64>,
    #[serde(default)]
    pub root_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_parent_is_view_cache_reuse_root: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_root_parent_sever_frame_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_old_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_old_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_new_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_root_parent_sever_parent_children_last_set_new_elements_head_paths: Vec<String>,
    #[serde(default)]
    pub root_layer: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_layer_visible: Option<bool>,
    #[serde(default)]
    pub reachable_from_layer_roots: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reachable_from_view_cache_roots: Option<bool>,
    #[serde(default)]
    pub unreachable_from_liveness_roots: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub liveness_layer_roots_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_cache_reuse_roots_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_cache_reuse_root_nodes_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_in_view_cache_keep_alive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_listed_under_reuse_root: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_element_root_path: Option<String>,
    #[serde(default)]
    pub root_children_len: u32,
    #[serde(default)]
    pub root_parent_children_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_contains_root: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_frame_children_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_frame_children_contains_root: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_frame_instance_present: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_frame_children_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_path: Vec<u64>,
    #[serde(default)]
    pub root_path_truncated: bool,
    /// For each `root_path` edge (`child -> parent`), whether `UiTree` currently has the
    /// corresponding `parent.children` edge:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing node entry)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_path_edge_ui_contains_child: Vec<u8>,
    /// For each `root_path` edge (`child -> parent`), whether `WindowFrame.children[parent]`
    /// contains the child node:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing frame edge capture)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_path_edge_frame_contains_child: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_old_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_new_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_parent_children_last_set_frame_id: Option<u64>,
    #[serde(default)]
    pub removed_nodes: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed_tail: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl UiRemovedSubtreeV1 {
    fn from_record(
        window: AppWindowId,
        ui: &UiTree<App>,
        element_runtime_state: Option<&ElementRuntime>,
        r: &fret_ui::tree::UiDebugRemoveSubtreeRecord,
        max_debug_string_bytes: usize,
    ) -> Self {
        let outcome = match r.outcome {
            fret_ui::tree::UiDebugRemoveSubtreeOutcome::SkippedLayerRoot => "skipped_layer_root",
            fret_ui::tree::UiDebugRemoveSubtreeOutcome::RootMissing => "root_missing",
            fret_ui::tree::UiDebugRemoveSubtreeOutcome::Removed => "removed",
        };

        let mut root_element_path = r.root_element.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });
        truncate_opt_string_bytes(&mut root_element_path, max_debug_string_bytes);

        let mut root_parent_element_path = r.root_parent_element.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });
        truncate_opt_string_bytes(&mut root_parent_element_path, max_debug_string_bytes);

        let mut trigger_element_path = r.trigger_element.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });
        truncate_opt_string_bytes(&mut trigger_element_path, max_debug_string_bytes);

        let mut trigger_element_root_path = r.trigger_element_root.and_then(|element| {
            element_runtime_state
                .and_then(|runtime| runtime.debug_path_for_element(window, element))
        });
        truncate_opt_string_bytes(&mut trigger_element_root_path, max_debug_string_bytes);

        let root_path = r.root_path[..(r.root_path_len as usize).min(r.root_path.len())].to_vec();
        let root_path_edge_len = (r.root_path_edge_len as usize)
            .min(r.root_path_edge_ui_contains_child.len())
            .min(r.root_path_edge_frame_contains_child.len());
        let root_path_edge_ui_contains_child =
            r.root_path_edge_ui_contains_child[..root_path_edge_len].to_vec();
        let root_path_edge_frame_contains_child =
            r.root_path_edge_frame_contains_child[..root_path_edge_len].to_vec();

        let (
            root_parent_children_last_set_location,
            root_parent_children_last_set_old_len,
            root_parent_children_last_set_new_len,
            root_parent_children_last_set_frame_id,
        ) = r
            .root_parent
            .and_then(|parent| ui.debug_set_children_write_for(parent))
            .map(|w| {
                let mut location = Some(format!("{}:{}:{}", w.file, w.line, w.column));
                truncate_opt_string_bytes(&mut location, max_debug_string_bytes);
                (
                    location,
                    Some(w.old_len),
                    Some(w.new_len),
                    Some(w.frame_id.0),
                )
            })
            .unwrap_or((None, None, None, None));

        let (
            root_root_parent_sever_parent,
            root_root_parent_sever_parent_element,
            root_root_parent_sever_parent_path,
            root_root_parent_sever_parent_is_view_cache_reuse_root,
            root_root_parent_sever_location,
            root_root_parent_sever_frame_id,
            root_root_parent_sever_parent_children_last_set_old_elements_head,
            root_root_parent_sever_parent_children_last_set_old_elements_head_paths,
            root_root_parent_sever_parent_children_last_set_new_elements_head,
            root_root_parent_sever_parent_children_last_set_new_elements_head_paths,
        ) = r
            .root_root
            .and_then(|root| ui.debug_parent_sever_write_for(root))
            .map(|w| {
                let parent_element = element_runtime_state
                    .and_then(|runtime| runtime.element_for_node(window, w.parent));
                let mut parent_path = parent_element.and_then(|element| {
                    element_runtime_state
                        .and_then(|runtime| runtime.debug_path_for_element(window, element))
                });
                truncate_opt_string_bytes(&mut parent_path, max_debug_string_bytes);
                let parent_is_view_cache_reuse_root = parent_element.and_then(|element| {
                    element_runtime_state.and_then(|runtime| {
                        runtime
                            .diagnostics_snapshot(window)
                            .map(|s| s.view_cache_reuse_roots.contains(&element))
                    })
                });

                let mut old_elements_head: Vec<u64> = Vec::new();
                let mut old_elements_head_paths: Vec<String> = Vec::new();
                let mut new_elements_head: Vec<u64> = Vec::new();
                let mut new_elements_head_paths: Vec<String> = Vec::new();

                if let Some(write) = ui.debug_set_children_write_for(w.parent) {
                    for element in write.old_elements_head.into_iter().flatten() {
                        old_elements_head.push(element.0);
                        if let Some(path) = element_runtime_state
                            .and_then(|runtime| runtime.debug_path_for_element(window, element))
                        {
                            old_elements_head_paths.push(path);
                        }
                    }
                    for element in write.new_elements_head.into_iter().flatten() {
                        new_elements_head.push(element.0);
                        if let Some(path) = element_runtime_state
                            .and_then(|runtime| runtime.debug_path_for_element(window, element))
                        {
                            new_elements_head_paths.push(path);
                        }
                    }
                }

                truncate_vec_string_bytes(&mut old_elements_head_paths, max_debug_string_bytes);
                truncate_vec_string_bytes(&mut new_elements_head_paths, max_debug_string_bytes);

                let mut location = Some(format!("{}:{}:{}", w.file, w.line, w.column));
                truncate_opt_string_bytes(&mut location, max_debug_string_bytes);

                (
                    Some(key_to_u64(w.parent)),
                    parent_element.map(|e| e.0),
                    parent_path,
                    parent_is_view_cache_reuse_root,
                    location,
                    Some(w.frame_id.0),
                    old_elements_head,
                    old_elements_head_paths,
                    new_elements_head,
                    new_elements_head_paths,
                )
            })
            .unwrap_or((
                None,
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ));

        Self {
            root: key_to_u64(r.root),
            root_element: r.root_element.map(|e| e.0),
            root_parent_element: r.root_parent_element.map(|e| e.0),
            root_parent_element_path,
            root_element_path,
            root_parent: r.root_parent.map(key_to_u64),
            root_root: r.root_root.map(key_to_u64),
            root_root_parent_sever_parent,
            root_root_parent_sever_parent_element,
            root_root_parent_sever_parent_path,
            root_root_parent_sever_parent_is_view_cache_reuse_root,
            root_root_parent_sever_location,
            root_root_parent_sever_frame_id,
            root_root_parent_sever_parent_children_last_set_old_elements_head,
            root_root_parent_sever_parent_children_last_set_old_elements_head_paths,
            root_root_parent_sever_parent_children_last_set_new_elements_head,
            root_root_parent_sever_parent_children_last_set_new_elements_head_paths,
            root_layer: r.root_layer.map(|id| id.data().as_ffi()),
            root_layer_visible: r.root_layer_visible,
            reachable_from_layer_roots: r.reachable_from_layer_roots,
            reachable_from_view_cache_roots: r.reachable_from_view_cache_roots,
            unreachable_from_liveness_roots: r.unreachable_from_liveness_roots,
            liveness_layer_roots_len: r.liveness_layer_roots_len,
            view_cache_reuse_roots_len: r.view_cache_reuse_roots_len,
            view_cache_reuse_root_nodes_len: r.view_cache_reuse_root_nodes_len,
            trigger_element: r.trigger_element.map(|e| e.0),
            trigger_element_root: r.trigger_element_root.map(|e| e.0),
            trigger_element_in_view_cache_keep_alive: r.trigger_element_in_view_cache_keep_alive,
            trigger_element_listed_under_reuse_root: r
                .trigger_element_listed_under_reuse_root
                .map(|id| id.0),
            trigger_element_path,
            trigger_element_root_path,
            root_children_len: r.root_children_len,
            root_parent_children_len: r.root_parent_children_len,
            root_parent_children_contains_root: r.root_parent_children_contains_root,
            root_parent_frame_children_len: r.root_parent_frame_children_len,
            root_parent_frame_children_contains_root: r.root_parent_frame_children_contains_root,
            root_frame_instance_present: r.root_frame_instance_present,
            root_frame_children_len: r.root_frame_children_len,
            root_path,
            root_path_truncated: r.root_path_truncated,
            root_path_edge_ui_contains_child,
            root_path_edge_frame_contains_child,
            root_parent_children_last_set_location,
            root_parent_children_last_set_old_len,
            root_parent_children_last_set_new_len,
            root_parent_children_last_set_frame_id,
            removed_nodes: r.removed_nodes,
            removed_head: r.removed_head[..(r.removed_head_len as usize).min(r.removed_head.len())]
                .to_vec(),
            removed_tail: r.removed_tail[..(r.removed_tail_len as usize).min(r.removed_tail.len())]
                .to_vec(),
            outcome: Some(outcome.to_string()),
            frame_id: Some(r.frame_id.0),
            location: {
                let mut location = Some(format!("{}:{}:{}", r.file, r.line, r.column));
                truncate_opt_string_bytes(&mut location, max_debug_string_bytes);
                location
            },
        }
    }
}
