use super::*;

impl<H: UiHost> UiTree<H> {
    pub fn request_semantics_snapshot(&mut self) {
        self.semantics_requested = true;
    }

    pub fn request_semantics_snapshot_if_dirty(&mut self) -> bool {
        let semantics_input_state_dirty = self.semantics.as_deref().is_some_and(|snapshot| {
            snapshot.focus != self.focus || snapshot.captured != self.captured_for(PointerId(0))
        });

        if self.semantics.is_none() || self.semantics_dirty || semantics_input_state_dirty {
            self.request_semantics_snapshot();
            true
        } else {
            false
        }
    }

    pub fn semantics_snapshot(&self) -> Option<&SemanticsSnapshot> {
        self.semantics.as_deref()
    }

    pub fn semantics_snapshot_arc(&self) -> Option<Arc<SemanticsSnapshot>> {
        self.semantics.clone()
    }

    pub(in crate::tree) fn mark_semantics_dirty(&mut self) {
        self.semantics_dirty = true;
    }

    pub(in crate::tree) fn invalidation_may_affect_semantics(
        source: UiDebugInvalidationSource,
        inv: Invalidation,
        detail: UiDebugInvalidationDetail,
    ) -> bool {
        if matches!(
            detail,
            UiDebugInvalidationDetail::AnimationFrameRequest
                | UiDebugInvalidationDetail::PressableHoverEdge
                | UiDebugInvalidationDetail::HoverRegionEdge
                | UiDebugInvalidationDetail::FocusVisiblePolicy
                | UiDebugInvalidationDetail::InputModalityPolicy
        ) || source == UiDebugInvalidationSource::Hover
        {
            return false;
        }

        if matches!(
            detail,
            UiDebugInvalidationDetail::DeclarativeTextContentChanged
        ) {
            return true;
        }

        match inv {
            Invalidation::Layout | Invalidation::HitTest | Invalidation::HitTestOnly => true,
            Invalidation::Paint => matches!(
                source,
                UiDebugInvalidationSource::Notify
                    | UiDebugInvalidationSource::ModelChange
                    | UiDebugInvalidationSource::GlobalChange
                    | UiDebugInvalidationSource::Focus
            ),
        }
    }

    pub(in crate::tree) fn refresh_semantics_snapshot(&mut self, app: &mut H) {
        let Some(window) = self.window else {
            self.semantics = None;
            self.semantics_dirty = true;
            return;
        };

        let profile_semantics = crate::runtime_config::ui_runtime_config().semantics_profile;
        let trace_semantics = tracing::enabled!(tracing::Level::TRACE);
        let profile_started = profile_semantics.then(Instant::now);
        let frame_id = app.frame_id();

        let base_root = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root));

        let visible_layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        if visible_layers.is_empty() {
            self.semantics = Some(Arc::new(SemanticsSnapshot {
                window,
                ..SemanticsSnapshot::default()
            }));
            self.semantics_dirty = false;
            return;
        }

        let (element_id_map, element_id_elapsed) = fret_perf::measure_span(
            profile_semantics,
            trace_semantics,
            || {
                tracing::trace_span!(
                    "fret.ui.semantics.element_id_map",
                    window = ?window,
                    frame_id = frame_id.0,
                )
            },
            || crate::declarative::frame::element_id_map_for_window(app, window),
        );
        let t_element_id_map = element_id_elapsed;

        // View-cache reuse can legitimately skip re-setting `UiTree` child edges for cached
        // subtrees. `WindowFrame` retains the authoritative element-tree edges, so semantics
        // traversal should treat the union as the effective child list (mirrors GC reachability
        // bookkeeping). Only pay the cost when view-cache reuse can occur.
        let view_cache_active = self.view_cache_active();
        let (window_frame_children, window_frame_children_elapsed): (
            slotmap::SecondaryMap<NodeId, Arc<[NodeId]>>,
            _,
        ) = fret_perf::measure_span(
            profile_semantics,
            trace_semantics,
            || {
                tracing::trace_span!(
                    "fret.ui.semantics.window_frame_children",
                    window = ?window,
                    frame_id = frame_id.0,
                    view_cache_active,
                )
            },
            || {
                if view_cache_active {
                    crate::declarative::with_window_frame(app, window, |window_frame| {
                        window_frame.map(|w| w.children.clone()).unwrap_or_default()
                    })
                } else {
                    slotmap::SecondaryMap::new()
                }
            },
        );
        let t_window_frame_children = window_frame_children_elapsed;

        let mut barrier_index: Option<usize> = None;
        for (idx, layer) in visible_layers.iter().enumerate() {
            if self.layers[*layer].blocks_underlay_input {
                barrier_index = Some(idx);
            }
        }
        let barrier_root = barrier_index.map(|idx| self.layers[visible_layers[idx]].root);

        let mut focus_barrier_index: Option<usize> = None;
        for (idx, layer) in visible_layers.iter().enumerate() {
            if self.layers[*layer].blocks_underlay_focus {
                focus_barrier_index = Some(idx);
            }
        }
        let focus_barrier_root =
            focus_barrier_index.map(|idx| self.layers[visible_layers[idx]].root);

        let mut roots: Vec<SemanticsRoot> = Vec::with_capacity(visible_layers.len());
        for (z, layer_id) in visible_layers.iter().enumerate() {
            let layer = &self.layers[*layer_id];
            roots.push(SemanticsRoot {
                root: layer.root,
                visible: layer.visible,
                blocks_underlay_input: layer.blocks_underlay_input,
                hit_testable: layer.hit_testable,
                z_index: z as u32,
            });
        }

        let focus = self.focus;
        let captured = self.captured_for(PointerId(0));

        let mut nodes: Vec<SemanticsNode> = Vec::with_capacity(self.nodes.len());

        let retained_node_count = self.nodes.len();
        let root_count = roots.len();
        let (_, traversal_elapsed) = fret_perf::measure_span(
            profile_semantics,
            trace_semantics,
            || {
                tracing::trace_span!(
                    "fret.ui.semantics.traversal",
                    window = ?window,
                    frame_id = frame_id.0,
                    roots = root_count,
                    retained_nodes = retained_node_count,
                    view_cache_active,
                )
            },
            || {
                for root in roots.iter().map(|r| r.root) {
                    let mut visited = self.take_scratch_semantics_visited();
                    visited.clear();
                    // Stack entries carry the transform that maps this node's local bounds into
                    // screen-space (excluding this node's own `render_transform`).
                    let mut stack = self.take_scratch_semantics_stack();
                    stack.clear();
                    stack.push((root, Transform2D::IDENTITY));
                    while let Some((id, before)) = stack.pop() {
                        if !visited.insert(id) {
                            if crate::strict_runtime::strict_runtime_enabled() {
                                panic!(
                                    "cycle detected while building semantics snapshot: node={id:?}"
                                );
                            }
                            tracing::error!(
                                ?id,
                                "cycle detected while building semantics snapshot"
                            );
                            continue;
                        }
                        let (
                            parent,
                            bounds,
                            children,
                            is_text_input,
                            is_focusable,
                            traverse_children,
                            before_child,
                        ) = {
                            let Some(node) = self.nodes.get(id) else {
                                continue;
                            };

                            // Declarative `InteractivityGate(present=false)` subtrees behave like
                            // `display: none`: they should not be exposed to the semantics snapshot even if
                            // the underlying nodes remain mounted (e.g. during close animations / force-mount).
                            //
                            // We cannot rely solely on the widget-level `semantics_present()` cache here
                            // because the layout engine may skip visiting display-none nodes in a frame,
                            // leaving stale derived flags until the next layout pass.
                            if node.element.is_some()
                        && crate::declarative::frame::element_record_for_node(app, window, id)
                            .is_some_and(|record| {
                                matches!(
                                    record.instance,
                                    crate::declarative::frame::ElementInstance::InteractivityGate(p)
                                        if !p.present
                                )
                            })
                    {
                        continue;
                    }
                            let widget = node.widget.as_ref();
                            if widget.is_some_and(|w| !w.semantics_present()) {
                                continue;
                            }

                            // Prefer prepaint-derived transforms when they are known to be valid, but
                            // fall back to live widget transforms while hit-test invalidations are
                            // pending.
                            //
                            // Hit-testing intentionally avoids `prepaint_hit_test` when `hit_test` is
                            // invalidated (see `hit_test.rs`) to prevent stale transforms from affecting
                            // pointer routing. Semantics should follow the same rule so scripted
                            // diagnostics (which pick click points from semantics bounds) remain aligned
                            // with the actual hit-test coordinate space.
                            let prepaint = (!self.inspection_active && !node.invalidation.hit_test)
                                .then_some(node.prepaint_hit_test)
                                .flatten();

                            let node_transform = prepaint
                                .as_ref()
                                .and_then(|p| p.render_transform_inv)
                                .and_then(|inv| inv.inverse())
                                .or_else(|| {
                                    widget
                                        .and_then(|w| w.render_transform(node.bounds))
                                        .filter(|t| t.inverse().is_some())
                                })
                                .unwrap_or(Transform2D::IDENTITY);
                            let at_node = before.compose(node_transform);
                            let bounds = rect_aabb_transformed(node.bounds, at_node);
                            let ui_children = node.children.clone();
                            let children = match window_frame_children.get(id) {
                                None => ui_children,
                                Some(frame_children) if ui_children.is_empty() => {
                                    frame_children.as_ref().to_vec()
                                }
                                Some(frame_children) => {
                                    let mut out = ui_children;
                                    for &child in frame_children.iter() {
                                        if !out.contains(&child) {
                                            out.push(child);
                                        }
                                    }
                                    out
                                }
                            };
                            let is_text_input = widget.is_some_and(|w| w.is_text_input());
                            let is_focusable = widget.is_some_and(|w| w.is_focusable());
                            let traverse_children =
                                widget.map(|w| w.semantics_children()).unwrap_or(true);
                            let child_transform = prepaint
                                .as_ref()
                                .and_then(|p| p.children_render_transform_inv)
                                .and_then(|inv| inv.inverse())
                                .or_else(|| {
                                    widget
                                        .and_then(|w| w.children_render_transform(node.bounds))
                                        .filter(|t| t.inverse().is_some())
                                })
                                .unwrap_or(Transform2D::IDENTITY);
                            let before_child = at_node.compose(child_transform);

                            (
                                node.parent,
                                bounds,
                                children,
                                is_text_input,
                                is_focusable,
                                traverse_children,
                                before_child,
                            )
                        };

                        let mut role = if Some(id) == base_root {
                            SemanticsRole::Window
                        } else {
                            SemanticsRole::Generic
                        };
                        // Heuristic baseline: text-input widgets should surface as text fields even if
                        // they don't implement an explicit semantics hook yet.
                        if is_text_input {
                            role = SemanticsRole::TextField;
                        }

                        let mut flags = fret_core::SemanticsFlags {
                            focused: focus == Some(id),
                            captured: captured == Some(id),
                            ..fret_core::SemanticsFlags::default()
                        };

                        let mut active_descendant: Option<NodeId> = None;
                        let mut pos_in_set: Option<u32> = None;
                        let mut set_size: Option<u32> = None;
                        let mut label: Option<String> = None;
                        let mut value: Option<String> = None;
                        let mut extra = fret_core::SemanticsNodeExtra::default();
                        let mut test_id: Option<String> = None;
                        let mut text_selection: Option<(u32, u32)> = None;
                        let mut text_composition: Option<(u32, u32)> = None;
                        let mut labelled_by: Vec<NodeId> = Vec::new();
                        let mut described_by: Vec<NodeId> = Vec::new();
                        let mut controls: Vec<NodeId> = Vec::new();
                        let mut inline_spans: Vec<fret_core::SemanticsInlineSpan> = Vec::new();
                        let mut actions = fret_core::SemanticsActions {
                            focus: is_focusable || is_text_input,
                            invoke: false,
                            set_value: is_text_input,
                            decrement: false,
                            increment: false,
                            scroll_by: false,
                            set_text_selection: is_text_input,
                        };

                        // Allow widgets to override semantics metadata.
                        if let Some(widget) =
                            self.nodes.get_mut(id).and_then(|node| node.widget.as_mut())
                        {
                            let mut cx = SemanticsCx {
                                app,
                                node: id,
                                window: Some(window),
                                element_id_map: Some(element_id_map.as_ref()),
                                bounds,
                                children: children.as_slice(),
                                focus,
                                captured,
                                role: &mut role,
                                flags: &mut flags,
                                label: &mut label,
                                value: &mut value,
                                test_id: &mut test_id,
                                extra: &mut extra,
                                text_selection: &mut text_selection,
                                text_composition: &mut text_composition,
                                actions: &mut actions,
                                active_descendant: &mut active_descendant,
                                pos_in_set: &mut pos_in_set,
                                set_size: &mut set_size,
                                labelled_by: &mut labelled_by,
                                described_by: &mut described_by,
                                controls: &mut controls,
                                inline_spans: &mut inline_spans,
                            };
                            widget.semantics(&mut cx);
                        }

                        // Derive a conservative slider `SetValue` surface.
                        //
                        // Rationale: many assistive technology stacks issue `SetValue(NumericValue)` for
                        // sliders. However, this should only be exposed when we have enough structured
                        // numeric metadata to act on it deterministically.
                        if (role == SemanticsRole::Slider
                            || role == SemanticsRole::SpinButton
                            || role == SemanticsRole::Splitter)
                            && (actions.increment || actions.decrement)
                        {
                            let numeric = extra.numeric;
                            let has_range = numeric.min.is_some() && numeric.max.is_some();
                            let has_value = numeric.value.is_some();
                            let has_step = numeric.step.is_some_and(|v| v.is_finite() && v > 0.0);
                            actions.set_value = has_range && has_value && has_step;
                        } else if role == SemanticsRole::Slider
                            || role == SemanticsRole::SpinButton
                            || role == SemanticsRole::Splitter
                        {
                            actions.set_value = false;
                        }

                        if pos_in_set.is_some_and(|p| p == 0) {
                            pos_in_set = None;
                        }
                        if set_size.is_some_and(|s| s == 0) {
                            set_size = None;
                        }
                        if let (Some(pos), Some(size)) = (pos_in_set, set_size)
                            && pos > size
                        {
                            pos_in_set = None;
                            set_size = None;
                        }

                        nodes.push(SemanticsNode {
                            id,
                            parent,
                            role,
                            bounds,
                            flags,
                            test_id,
                            active_descendant,
                            pos_in_set,
                            set_size,
                            label,
                            value,
                            extra,
                            text_selection,
                            text_composition,
                            actions,
                            labelled_by,
                            described_by,
                            controls,
                            inline_spans,
                        });

                        if traverse_children {
                            // Preserve a stable-ish order: visit children in declared order.
                            for &child in children.iter().rev() {
                                stack.push((child, before_child));
                            }
                        }
                    }

                    visited.clear();
                    stack.clear();
                    self.restore_scratch_semantics_visited(visited);
                    self.restore_scratch_semantics_stack(stack);
                }
            },
        );
        let t_traversal = traversal_elapsed;

        // Normalize relation edges: for some composite widgets, authoring only sets `labelled_by`
        // (e.g. TabPanel -> Tab) but the platform-facing semantics want the controller to also
        // advertise `controls` (e.g. Tab -> TabPanel). We derive that edge for the subset of
        // role pairs where this bidirectional link is expected.
        let nodes_before_relations = nodes.len();
        let (_, relations_elapsed) = fret_perf::measure_span(
            profile_semantics,
            trace_semantics,
            || {
                tracing::trace_span!(
                    "fret.ui.semantics.relations",
                    window = ?window,
                    frame_id = frame_id.0,
                    nodes = nodes_before_relations,
                )
            },
            || {
                let mut index_by_id: HashMap<NodeId, usize> = HashMap::with_capacity(nodes.len());
                for (idx, node) in nodes.iter().enumerate() {
                    index_by_id.insert(node.id, idx);
                }
                for node in nodes.iter_mut() {
                    if node
                        .active_descendant
                        .is_some_and(|target| !index_by_id.contains_key(&target))
                    {
                        node.active_descendant = None;
                    }
                    node.labelled_by
                        .retain(|target| index_by_id.contains_key(target));
                    node.described_by
                        .retain(|target| index_by_id.contains_key(target));
                    node.controls
                        .retain(|target| index_by_id.contains_key(target));
                }
                for idx in 0..nodes.len() {
                    let controlled = nodes[idx].id;
                    let controlled_role = nodes[idx].role;
                    let controllers = nodes[idx].labelled_by.clone();
                    for controller in controllers {
                        if let Some(&controller_idx) = index_by_id.get(&controller) {
                            let controller_role = nodes[controller_idx].role;
                            let derive = matches!(
                                controlled_role,
                                SemanticsRole::TabPanel | SemanticsRole::ListBox
                            ) && matches!(
                                controller_role,
                                SemanticsRole::Tab
                                    | SemanticsRole::TextField
                                    | SemanticsRole::ComboBox
                                    | SemanticsRole::Button
                            );
                            if !derive {
                                continue;
                            }
                            if !nodes[controller_idx].controls.contains(&controlled) {
                                nodes[controller_idx].controls.push(controlled);
                            }
                        }
                    }
                }
            },
        );
        let t_relations = relations_elapsed;

        let nodes_len = nodes.len();
        self.semantics = Some(Arc::new(SemanticsSnapshot {
            window,
            roots,
            barrier_root,
            focus_barrier_root,
            focus,
            captured,
            nodes,
        }));
        self.semantics_dirty = false;

        if let Some(snapshot) = self.semantics.as_deref() {
            semantics::validate_semantics_if_enabled(snapshot);
        }

        if let Some(started) = profile_started {
            let total = started.elapsed();
            tracing::info!(
                window = ?window,
                view_cache_active = self.view_cache_active(),
                nodes = nodes_len,
                total_ms = total.as_millis(),
                element_id_map_ms = t_element_id_map.map(|d| d.as_millis()),
                window_frame_children_ms = t_window_frame_children.map(|d| d.as_millis()),
                traversal_ms = t_traversal.map(|d| d.as_millis()),
                relations_ms = t_relations.map(|d| d.as_millis()),
                "semantics snapshot built"
            );
        }
    }

    pub(in crate::tree) fn node_root(&self, mut node: NodeId) -> Option<NodeId> {
        while let Some(parent) = self.nodes.get(node).and_then(|n| n.parent) {
            node = parent;
        }
        self.nodes.contains_key(node).then_some(node)
    }

    pub fn is_descendant(&self, root: NodeId, mut node: NodeId) -> bool {
        if root == node {
            return true;
        }
        while let Some(parent) = self.nodes.get(node).and_then(|n| n.parent) {
            if parent == root {
                return true;
            }
            node = parent;
        }
        false
    }

    /// Returns `true` when `node` is reachable from `root` by following authoritative child
    /// edges for the current frame.
    ///
    /// Unlike [`UiTree::is_descendant`], this does not rely on retained parent pointers, which
    /// can be stale under view-cache / overlay reuse while the child graph is already correct.
    pub fn is_descendant_via_children(&self, root: NodeId, node: NodeId) -> bool {
        self.is_reachable_from_root_via_children(root, node)
    }
}
