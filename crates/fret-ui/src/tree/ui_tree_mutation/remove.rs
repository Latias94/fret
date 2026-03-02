use super::super::*;

impl<H: UiHost> UiTree<H> {
    #[track_caller]
    pub fn remove_subtree(&mut self, services: &mut dyn UiServices, root: NodeId) -> Vec<NodeId> {
        #[cfg(feature = "diagnostics")]
        let remove_record = if self.debug_enabled {
            let location = std::panic::Location::caller();
            let pre_exists = self.nodes.contains_key(root);
            let root_element = self.nodes.get(root).and_then(|n| n.element);
            let root_parent = self.nodes.get(root).and_then(|n| n.parent);
            let root_parent_element =
                root_parent.and_then(|p| self.nodes.get(p).and_then(|n| n.element));
            let root_root = self.node_root(root);
            let root_layer = self.node_layer(root);
            let root_layer_visible =
                root_layer.and_then(|layer| self.layers.get(layer).map(|l| l.visible));
            let root_children_len = self
                .nodes
                .get(root)
                .map(|n| n.children.len().min(u32::MAX as usize) as u32)
                .unwrap_or(0);
            let root_parent_children_len = root_parent.and_then(|p| {
                self.nodes
                    .get(p)
                    .map(|n| n.children.len().min(u32::MAX as usize) as u32)
            });
            let root_parent_children_contains_root =
                root_parent.and_then(|p| self.nodes.get(p).map(|n| n.children.contains(&root)));
            let frame_context = self.debug_remove_subtree_frame_context.remove(&root);
            let reachable_from_layer_roots = pre_exists
                && frame_context
                    .as_ref()
                    .map(|ctx| ctx.root_reachable_from_layer_roots)
                    .unwrap_or_else(|| self.debug_is_reachable_from_layer_roots(root));
            let mut root_path: [u64; 16] = [0u64; 16];
            let mut root_path_nodes: [Option<NodeId>; 16] = [None; 16];
            let mut root_path_len: u8 = 0;
            let mut root_path_truncated = false;
            let mut current = Some(root);
            while let Some(id) = current {
                if (root_path_len as usize) >= root_path.len() {
                    root_path_truncated = true;
                    break;
                }
                root_path_nodes[root_path_len as usize] = Some(id);
                root_path[root_path_len as usize] = id.data().as_ffi();
                root_path_len = root_path_len.saturating_add(1);
                current = self.nodes.get(id).and_then(|n| n.parent);
            }
            let root_path_edge_len = root_path_len.saturating_sub(1);
            let mut root_path_edge_ui_contains_child: [u8; 16] = [2u8; 16];
            for idx in 0..(root_path_edge_len as usize).min(root_path_edge_ui_contains_child.len())
            {
                let Some(child) = root_path_nodes[idx] else {
                    continue;
                };
                let Some(parent) = root_path_nodes[idx + 1] else {
                    continue;
                };
                let contains = self.nodes.get(parent).map(|n| n.children.contains(&child));
                root_path_edge_ui_contains_child[idx] = match contains {
                    Some(true) => 1,
                    Some(false) => 0,
                    None => 2,
                };
            }
            Some((
                location.file(),
                location.line(),
                location.column(),
                pre_exists,
                root_element,
                root_parent,
                root_parent_element,
                root_root,
                root_layer,
                root_layer_visible,
                reachable_from_layer_roots,
                root_children_len,
                root_parent_children_len,
                root_parent_children_contains_root,
                frame_context,
                root_path_len,
                root_path,
                root_path_truncated,
                root_path_edge_len,
                root_path_edge_ui_contains_child,
            ))
        } else {
            None
        };

        if self.root_to_layer.contains_key(&root) {
            #[cfg(feature = "diagnostics")]
            if let Some((
                file,
                line,
                column,
                _pre_exists,
                root_element,
                root_parent,
                root_parent_element,
                root_root,
                root_layer,
                root_layer_visible,
                reachable_from_layer_roots,
                root_children_len,
                root_parent_children_len,
                root_parent_children_contains_root,
                frame_context,
                root_path_len,
                root_path,
                root_path_truncated,
                root_path_edge_len,
                root_path_edge_ui_contains_child,
            )) = remove_record
            {
                let root_path_edge_frame_contains_child = frame_context
                    .map(|ctx| ctx.path_edge_frame_contains_child)
                    .unwrap_or([2u8; 16]);
                let reachable_from_view_cache_roots =
                    frame_context.and_then(|ctx| ctx.root_reachable_from_view_cache_roots);
                let unreachable_from_liveness_roots = !reachable_from_layer_roots
                    && !matches!(reachable_from_view_cache_roots, Some(true));
                let trigger_element = frame_context.and_then(|ctx| ctx.trigger_element);
                let trigger_element_root = frame_context.and_then(|ctx| ctx.trigger_element_root);
                let trigger_element_in_view_cache_keep_alive =
                    frame_context.and_then(|ctx| ctx.trigger_element_in_view_cache_keep_alive);
                let trigger_element_listed_under_reuse_root =
                    frame_context.and_then(|ctx| ctx.trigger_element_listed_under_reuse_root);
                let liveness_layer_roots_len =
                    frame_context.map(|ctx| ctx.liveness_layer_roots_len);
                let view_cache_reuse_roots_len =
                    frame_context.map(|ctx| ctx.view_cache_reuse_roots_len);
                let view_cache_reuse_root_nodes_len =
                    frame_context.map(|ctx| ctx.view_cache_reuse_root_nodes_len);
                let (root_parent_frame_children_len, root_parent_frame_children_contains_root) =
                    frame_context
                        .map(|ctx| {
                            (
                                ctx.parent_frame_children_len,
                                ctx.parent_frame_children_contains_root,
                            )
                        })
                        .unwrap_or((None, None));
                let (root_frame_instance_present, root_frame_children_len) = frame_context
                    .map(|ctx| {
                        (
                            Some(ctx.root_frame_instance_present),
                            ctx.root_frame_children_len,
                        )
                    })
                    .unwrap_or((None, None));
                self.debug_removed_subtrees
                    .push(UiDebugRemoveSubtreeRecord {
                        outcome: UiDebugRemoveSubtreeOutcome::SkippedLayerRoot,
                        frame_id: self.debug_stats.frame_id,
                        root,
                        root_element,
                        root_parent,
                        root_parent_element,
                        root_root,
                        root_layer,
                        root_layer_visible,
                        reachable_from_layer_roots,
                        reachable_from_view_cache_roots,
                        unreachable_from_liveness_roots,
                        liveness_layer_roots_len,
                        view_cache_reuse_roots_len,
                        view_cache_reuse_root_nodes_len,
                        trigger_element,
                        trigger_element_root,
                        trigger_element_in_view_cache_keep_alive,
                        trigger_element_listed_under_reuse_root,
                        root_children_len,
                        root_parent_children_len,
                        root_parent_children_contains_root,
                        root_parent_frame_children_len,
                        root_parent_frame_children_contains_root,
                        root_frame_instance_present,
                        root_frame_children_len,
                        root_path_len,
                        root_path,
                        root_path_truncated,
                        root_path_edge_len,
                        root_path_edge_ui_contains_child,
                        root_path_edge_frame_contains_child,
                        removed_nodes: 0,
                        removed_head_len: 0,
                        removed_head: [0u64; 16],
                        removed_tail_len: 0,
                        removed_tail: [0u64; 16],
                        file,
                        line,
                        column,
                    });
            }
            return Vec::new();
        }
        let mut removed: Vec<NodeId> = Vec::new();
        self.remove_subtree_inner(services, root, &mut removed);

        #[cfg(feature = "diagnostics")]
        if let Some((
            file,
            line,
            column,
            pre_exists,
            root_element,
            root_parent,
            root_parent_element,
            root_root,
            root_layer,
            root_layer_visible,
            reachable_from_layer_roots,
            root_children_len,
            root_parent_children_len,
            root_parent_children_contains_root,
            frame_context,
            root_path_len,
            root_path,
            root_path_truncated,
            root_path_edge_len,
            root_path_edge_ui_contains_child,
        )) = remove_record
        {
            let root_path_edge_frame_contains_child = frame_context
                .map(|ctx| ctx.path_edge_frame_contains_child)
                .unwrap_or([2u8; 16]);
            let reachable_from_view_cache_roots =
                frame_context.and_then(|ctx| ctx.root_reachable_from_view_cache_roots);
            let unreachable_from_liveness_roots = !reachable_from_layer_roots
                && !matches!(reachable_from_view_cache_roots, Some(true));
            let trigger_element = frame_context.and_then(|ctx| ctx.trigger_element);
            let trigger_element_root = frame_context.and_then(|ctx| ctx.trigger_element_root);
            let trigger_element_in_view_cache_keep_alive =
                frame_context.and_then(|ctx| ctx.trigger_element_in_view_cache_keep_alive);
            let trigger_element_listed_under_reuse_root =
                frame_context.and_then(|ctx| ctx.trigger_element_listed_under_reuse_root);
            let liveness_layer_roots_len = frame_context.map(|ctx| ctx.liveness_layer_roots_len);
            let view_cache_reuse_roots_len =
                frame_context.map(|ctx| ctx.view_cache_reuse_roots_len);
            let view_cache_reuse_root_nodes_len =
                frame_context.map(|ctx| ctx.view_cache_reuse_root_nodes_len);
            let (root_parent_frame_children_len, root_parent_frame_children_contains_root) =
                frame_context
                    .map(|ctx| {
                        (
                            ctx.parent_frame_children_len,
                            ctx.parent_frame_children_contains_root,
                        )
                    })
                    .unwrap_or((None, None));
            let (root_frame_instance_present, root_frame_children_len) = frame_context
                .map(|ctx| {
                    (
                        Some(ctx.root_frame_instance_present),
                        ctx.root_frame_children_len,
                    )
                })
                .unwrap_or((None, None));
            let outcome = if pre_exists {
                UiDebugRemoveSubtreeOutcome::Removed
            } else {
                UiDebugRemoveSubtreeOutcome::RootMissing
            };

            let mut removed_head: [u64; 16] = [0u64; 16];
            let mut removed_head_len: u8 = 0;
            for (idx, node) in removed.iter().take(16).enumerate() {
                removed_head[idx] = node.data().as_ffi();
                removed_head_len = removed_head_len.saturating_add(1);
            }

            let mut removed_tail: [u64; 16] = [0u64; 16];
            let mut removed_tail_len: u8 = 0;
            for (idx, node) in removed.iter().rev().take(16).enumerate() {
                removed_tail[idx] = node.data().as_ffi();
                removed_tail_len = removed_tail_len.saturating_add(1);
            }

            self.debug_removed_subtrees
                .push(UiDebugRemoveSubtreeRecord {
                    outcome,
                    frame_id: self.debug_stats.frame_id,
                    root,
                    root_element,
                    root_parent,
                    root_parent_element,
                    root_root,
                    root_layer,
                    root_layer_visible,
                    reachable_from_layer_roots,
                    reachable_from_view_cache_roots,
                    unreachable_from_liveness_roots,
                    liveness_layer_roots_len,
                    view_cache_reuse_roots_len,
                    view_cache_reuse_root_nodes_len,
                    trigger_element,
                    trigger_element_root,
                    trigger_element_in_view_cache_keep_alive,
                    trigger_element_listed_under_reuse_root,
                    root_children_len,
                    root_parent_children_len,
                    root_parent_children_contains_root,
                    root_parent_frame_children_len,
                    root_parent_frame_children_contains_root,
                    root_frame_instance_present,
                    root_frame_children_len,
                    root_path_len,
                    root_path,
                    root_path_truncated,
                    root_path_edge_len,
                    root_path_edge_ui_contains_child,
                    root_path_edge_frame_contains_child,
                    removed_nodes: removed.len().min(u32::MAX as usize) as u32,
                    removed_head_len,
                    removed_head,
                    removed_tail_len,
                    removed_tail,
                    file,
                    line,
                    column,
                });
        }

        removed
    }

    pub(in crate::tree) fn remove_subtree_inner(
        &mut self,
        services: &mut dyn UiServices,
        root: NodeId,
        removed: &mut Vec<NodeId>,
    ) {
        // Avoid recursion: removing or cleaning up deep trees can overflow the stack.
        //
        // We remove nodes in a post-order traversal so children are removed before their parent.
        let mut stack: Vec<(NodeId, bool)> = Vec::new();
        stack.push((root, false));

        while let Some((node, children_pushed)) = stack.pop() {
            if self.root_to_layer.contains_key(&node) {
                continue;
            }
            let Some(n) = self.nodes.get(node) else {
                continue;
            };
            let layout_invalidated = n.invalidation.layout;
            let subtree_layout_dirty_count = n.subtree_layout_dirty_count;

            if !children_pushed {
                let children = n.children.clone();
                stack.push((node, true));
                for child in children {
                    stack.push((child, false));
                }
                continue;
            }

            let parent = self.nodes.get(node).and_then(|n| n.parent);
            if let Some(parent) = parent
                && let Some(p) = self.nodes.get_mut(parent)
            {
                p.children.retain(|&c| c != node);
                if self.subtree_layout_dirty_aggregation_enabled() && subtree_layout_dirty_count > 0
                {
                    let delta = -(subtree_layout_dirty_count.min(i32::MAX as u32) as i32);
                    self.apply_subtree_layout_dirty_delta_to_node_and_ancestors(parent, delta);
                }
            }

            if self.focus == Some(node) {
                self.set_focus_unchecked(None, "mutation/remove: removed focused node");
            }
            self.captured.retain(|_, n| *n != node);

            self.cleanup_node_resources(services, node);
            if let Some(n) = self.nodes.get(node) {
                self.update_invalidation_counters(n.invalidation, InvalidationFlags::default());
            }
            if layout_invalidated {
                record_layout_invalidation_transition(
                    &mut self.layout_invalidations_count,
                    true,
                    false,
                );
            }
            self.nodes.remove(node);
            self.observed_in_layout.remove_node(node);
            self.observed_in_paint.remove_node(node);
            self.observed_globals_in_layout.remove_node(node);
            self.observed_globals_in_paint.remove_node(node);
            removed.push(node);
        }
    }
}
