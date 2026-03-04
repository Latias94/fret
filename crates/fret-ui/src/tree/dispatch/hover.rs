use super::*;
use std::collections::HashMap;

impl<H: UiHost> UiTree<H> {
    fn snapshot_parent_or_retained(
        &self,
        snapshot: Option<&UiDispatchSnapshot>,
        node: NodeId,
    ) -> Option<NodeId> {
        match snapshot {
            Some(snapshot) => {
                if snapshot.pre.get(node).is_none() {
                    debug_assert!(
                        false,
                        "dispatch/hover: node missing from snapshot (node={node:?}, frame_id={:?}, window={:?})",
                        snapshot.frame_id, snapshot.window
                    );
                    return None;
                }
                snapshot.parent.get(node).copied().flatten()
            }
            None => self.nodes.get(node).and_then(|n| n.parent),
        }
    }

    pub(super) fn run_pressable_hover_hook(
        app: &mut H,
        window: AppWindowId,
        element: crate::elements::GlobalElementId,
        is_hovered: bool,
    ) {
        let hook = crate::elements::with_element_state(
            app,
            window,
            element,
            crate::action::PressableHoverActionHooks::default,
            |hooks| hooks.on_hover_change.clone(),
        );

        let Some(hook) = hook else {
            return;
        };

        struct PressableHoverHookHost<'a, H: crate::UiHost> {
            app: &'a mut H,
            window: AppWindowId,
            element: crate::elements::GlobalElementId,
        }

        impl<H: crate::UiHost> crate::action::UiActionHost for PressableHoverHookHost<'_, H> {
            fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
                self.app.models_mut()
            }

            fn push_effect(&mut self, effect: Effect) {
                match effect {
                    Effect::SetTimer {
                        window: Some(window),
                        token,
                        ..
                    } if window == self.window => {
                        crate::elements::record_timer_target(
                            &mut *self.app,
                            window,
                            token,
                            self.element,
                        );
                    }
                    Effect::CancelTimer { token } => {
                        crate::elements::clear_timer_target(&mut *self.app, self.window, token);
                    }
                    _ => {}
                }
                self.app.push_effect(effect);
            }

            fn request_redraw(&mut self, window: AppWindowId) {
                self.app.request_redraw(window);
            }

            fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
                self.app.next_timer_token()
            }

            fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
                self.app.next_clipboard_token()
            }

            fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
                self.app.next_share_sheet_token()
            }
        }

        let mut host = PressableHoverHookHost {
            app,
            window,
            element,
        };
        hook(
            &mut host,
            crate::action::ActionCx {
                window,
                target: element,
            },
            is_hovered,
        );
    }

    pub(super) fn pressable_element_for_hit(
        &self,
        app: &mut H,
        window: AppWindowId,
        hit: Option<NodeId>,
        snapshot: Option<&UiDispatchSnapshot>,
    ) -> Option<crate::elements::GlobalElementId> {
        declarative::with_window_frame(app, window, |window_frame| {
            let window_frame = window_frame?;
            let mut node = hit;
            while let Some(id) = node {
                if let Some(record) = window_frame.instances.get(id)
                    && matches!(record.instance, declarative::ElementInstance::Pressable(_))
                {
                    return Some(record.element);
                }
                node = self.snapshot_parent_or_retained(snapshot, id);
            }
            None
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_hover_state_from_hit(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        barrier_root: Option<NodeId>,
        position: Option<Point>,
        hit_for_hover: Option<NodeId>,
        hit_for_hover_region: Option<NodeId>,
        hit_for_raw_below_barrier: Option<NodeId>,
        snapshot: Option<&UiDispatchSnapshot>,
        invalidation_visited: &mut HashMap<NodeId, u8>,
        needs_redraw: &mut bool,
    ) {
        let hovered_pressable =
            self.pressable_element_for_hit(app, window, hit_for_hover, snapshot);

        let (prev_element, prev_node, next_element, next_node) =
            crate::elements::update_hovered_pressable(app, window, hovered_pressable);
        if prev_node.is_some() || next_node.is_some() {
            *needs_redraw = true;
            self.debug_record_hover_edge_pressable();
            if let Some(node) = prev_node {
                self.mark_view_cache_roots_needs_rerender_from_snapshot(
                    node,
                    snapshot,
                    UiDebugInvalidationSource::Hover,
                    UiDebugInvalidationDetail::PressableHoverEdge,
                );
            }
            if let Some(node) = next_node {
                self.mark_view_cache_roots_needs_rerender_from_snapshot(
                    node,
                    snapshot,
                    UiDebugInvalidationSource::Hover,
                    UiDebugInvalidationDetail::PressableHoverEdge,
                );
            }
            if let Some(node) = prev_node {
                self.mark_invalidation_dedup_with_source(
                    node,
                    Invalidation::Paint,
                    invalidation_visited,
                    UiDebugInvalidationSource::Hover,
                );
            }
            if let Some(node) = next_node {
                self.mark_invalidation_dedup_with_source(
                    node,
                    Invalidation::Paint,
                    invalidation_visited,
                    UiDebugInvalidationSource::Hover,
                );
            }
        }

        if let Some(window) = self.window {
            for (element, node) in [(prev_element, prev_node), (next_element, next_node)] {
                let (Some(element), Some(node)) = (element, node) else {
                    continue;
                };
                let Some(bounds) = self.node_bounds(node) else {
                    continue;
                };
                crate::elements::record_bounds_for_element(app, window, element, bounds);
            }
        }

        if let Some(element) = prev_element
            && prev_node.is_some()
        {
            Self::run_pressable_hover_hook(app, window, element, false);
        }
        if let Some(element) = next_element
            && next_node.is_some()
        {
            Self::run_pressable_hover_hook(app, window, element, true);
        }

        // Raw hover is pressable-targeted hover derived directly from hit testing, regardless of
        // component-local interactivity policy (e.g. disabled scopes).
        let hovered_pressable_raw =
            self.pressable_element_for_hit(app, window, hit_for_hover, snapshot);
        let (prev_raw_element, prev_raw_node, next_raw_element, next_raw_node) =
            crate::elements::update_hovered_pressable_raw(app, window, hovered_pressable_raw);
        if prev_raw_node.is_some() || next_raw_node.is_some() {
            *needs_redraw = true;
            if let Some(node) = prev_raw_node {
                self.mark_invalidation_dedup_with_source(
                    node,
                    Invalidation::Paint,
                    invalidation_visited,
                    UiDebugInvalidationSource::Hover,
                );
            }
            if let Some(node) = next_raw_node {
                self.mark_invalidation_dedup_with_source(
                    node,
                    Invalidation::Paint,
                    invalidation_visited,
                    UiDebugInvalidationSource::Hover,
                );
            }
        }

        if let Some(window) = self.window {
            for (element, node) in [
                (prev_raw_element, prev_raw_node),
                (next_raw_element, next_raw_node),
            ] {
                let (Some(element), Some(node)) = (element, node) else {
                    continue;
                };
                let Some(bounds) = self.node_bounds(node) else {
                    continue;
                };
                crate::elements::record_bounds_for_element(app, window, element, bounds);
            }
        }

        let hovered_pressable_raw_below_barrier = if hit_for_raw_below_barrier.is_some() {
            self.pressable_element_for_hit(app, window, hit_for_raw_below_barrier, snapshot)
        } else if let (Some(barrier_root), Some(position)) = (barrier_root, position) {
            let mut roots: Vec<NodeId> = Vec::new();
            let mut hit_barrier = false;
            for &layer_id in self.layer_order.iter().rev() {
                let Some(layer) = self.layers.get(layer_id) else {
                    continue;
                };
                if !layer.visible {
                    continue;
                }

                if !hit_barrier {
                    if layer.root == barrier_root {
                        hit_barrier = true;
                    }
                    continue;
                }

                if layer.hit_testable {
                    roots.push(layer.root);
                }
            }

            if roots.is_empty() {
                None
            } else {
                let saved = self.hit_test_path_cache.take();
                self.hit_test_path_cache = None;
                let hit = self.hit_test_layers_cached(&roots, position);
                self.hit_test_path_cache = saved;
                if hit.is_none() {
                    None
                } else {
                    let snapshot = self.build_dispatch_snapshot_for_layer_roots(
                        app.frame_id(),
                        roots.as_slice(),
                        Some(barrier_root),
                    );
                    self.pressable_element_for_hit(app, window, hit, Some(&snapshot))
                }
            }
        } else {
            None
        };

        let (prev_below_element, prev_below_node, next_below_element, next_below_node) =
            crate::elements::update_hovered_pressable_raw_below_barrier(
                app,
                window,
                hovered_pressable_raw_below_barrier,
            );
        if prev_below_node.is_some() || next_below_node.is_some() {
            *needs_redraw = true;
            if let Some(node) = prev_below_node {
                self.mark_invalidation_dedup_with_source(
                    node,
                    Invalidation::Paint,
                    invalidation_visited,
                    UiDebugInvalidationSource::Hover,
                );
            }
            if let Some(node) = next_below_node {
                self.mark_invalidation_dedup_with_source(
                    node,
                    Invalidation::Paint,
                    invalidation_visited,
                    UiDebugInvalidationSource::Hover,
                );
            }
        }

        if let Some(window) = self.window {
            for (element, node) in [
                (prev_below_element, prev_below_node),
                (next_below_element, next_below_node),
            ] {
                let (Some(element), Some(node)) = (element, node) else {
                    continue;
                };
                let Some(bounds) = self.node_bounds(node) else {
                    continue;
                };
                crate::elements::record_bounds_for_element(app, window, element, bounds);
            }
        }

        let hovered_hover_region: Option<(crate::elements::GlobalElementId, NodeId)> =
            declarative::with_window_frame(app, window, |window_frame| {
                let window_frame = window_frame?;
                let mut node = hit_for_hover_region;
                while let Some(id) = node {
                    if let Some(record) = window_frame.instances.get(id)
                        && matches!(
                            record.instance,
                            declarative::ElementInstance::HoverRegion(_)
                        )
                    {
                        return Some((record.element, id));
                    }
                    node = self.snapshot_parent_or_retained(snapshot, id);
                }
                None
            });

        let (_prev_element, prev_node, _next_element, next_node) =
            crate::elements::update_hovered_hover_region_with_node(
                app,
                window,
                hovered_hover_region,
            );
        if prev_node.is_some() || next_node.is_some() {
            *needs_redraw = true;
            self.debug_record_hover_edge_hover_region();
            if let Some(node) = prev_node {
                self.mark_view_cache_roots_needs_rerender_from_snapshot(
                    node,
                    snapshot,
                    UiDebugInvalidationSource::Hover,
                    UiDebugInvalidationDetail::HoverRegionEdge,
                );
            }
            if let Some(node) = next_node {
                self.mark_view_cache_roots_needs_rerender_from_snapshot(
                    node,
                    snapshot,
                    UiDebugInvalidationSource::Hover,
                    UiDebugInvalidationDetail::HoverRegionEdge,
                );
            }
            if let Some(node) = prev_node {
                self.mark_invalidation_dedup_with_detail(
                    node,
                    Invalidation::Paint,
                    invalidation_visited,
                    UiDebugInvalidationSource::Hover,
                    UiDebugInvalidationDetail::HoverRegionEdge,
                );
            }
            if let Some(node) = next_node {
                self.mark_invalidation_dedup_with_detail(
                    node,
                    Invalidation::Paint,
                    invalidation_visited,
                    UiDebugInvalidationSource::Hover,
                    UiDebugInvalidationDetail::HoverRegionEdge,
                );
            }
        }
    }
}
