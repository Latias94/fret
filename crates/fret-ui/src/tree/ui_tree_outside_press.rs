use super::*;

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn dispatch_pointer_down_outside(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        params: PointerDownOutsideParams<'_>,
        invalidation_visited: &mut impl InvalidationVisited,
    ) -> PointerDownOutsideOutcome {
        let hit = params.hit;

        let snapshot = self.build_dispatch_snapshot_for_layer_roots(
            app.frame_id(),
            params.active_layer_roots,
            params.barrier_root,
        );

        let hit_root = hit.and_then(|hit| {
            snapshot
                .active_layer_roots
                .iter()
                .copied()
                .find(|&root| snapshot.is_descendant(root, hit))
        });

        let (event_pointer_id, touch_candidate): (
            Option<PointerId>,
            Option<(PointerId, Point, Event)>,
        ) = match params.event {
            Event::Pointer(PointerEvent::Down {
                pointer_id,
                position,
                pointer_type: fret_core::PointerType::Touch,
                ..
            }) => (
                Some(*pointer_id),
                Some((*pointer_id, *position, params.event.clone())),
            ),
            Event::Pointer(PointerEvent::Down { pointer_id, .. }) => (Some(*pointer_id), None),
            _ => (None, None),
        };

        if let Some((pointer_id, _, _)) = touch_candidate {
            self.touch_pointer_down_outside_candidates
                .remove(&pointer_id);
        }

        // Only the topmost "dismissable" non-modal overlay should observe outside presses.
        // This mirrors Radix-style DismissableLayer semantics while staying click-through:
        // the observer pass must not block the underlying hit-tested dispatch.
        let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        for layer_id in layers.into_iter().rev() {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }
            if layer.root == params.base_root {
                continue;
            }
            if layer.blocks_underlay_input {
                continue;
            }
            if !params.active_layer_roots.contains(&layer.root) {
                continue;
            }

            // If another pointer is captured by a different UI layer, treat the active capture as
            // exclusive for outside-press dismissal.
            //
            // This avoids accidental multi-pointer dismissal while editor-style interactions
            // (viewport tools, drags) are in progress (ADR 0049).
            if let Some(event_pointer_id) = event_pointer_id
                && self.captured.iter().any(|(pid, node)| {
                    *pid != event_pointer_id
                        && self
                            .node_layer(*node)
                            .is_some_and(|layer| layer != layer_id)
                })
            {
                return PointerDownOutsideOutcome::default();
            }

            // If the pointer event is inside this layer, it will be handled by the normal hit-test
            // dispatch. Do not dismiss anything under it.
            if hit_root == Some(layer.root) {
                break;
            }

            // Radix-aligned outcome: allow per-layer "branches" that should not trigger outside
            // dismissal even though they live outside the layer subtree (e.g. trigger elements).
            if hit.is_some_and(|hit| {
                layer
                    .pointer_down_outside_branches
                    .iter()
                    .copied()
                    .any(|branch| {
                        if snapshot.pre.get(branch).is_some() && snapshot.pre.get(hit).is_some() {
                            snapshot.is_descendant(branch, hit)
                        } else {
                            self.is_reachable_from_root_via_children(branch, hit)
                        }
                    })
            }) {
                break;
            }

            if !layer.wants_pointer_down_outside_events {
                continue;
            }

            let root = layer.root;
            let consume = layer.consume_pointer_down_outside_events;

            if let Some((pointer_id, position, down_event)) = touch_candidate {
                // Radix-aligned touch behavior: delay outside-press dismissal until pointer-up,
                // and cancel it when the touch turns into a scroll/drag gesture.
                self.touch_pointer_down_outside_candidates.insert(
                    pointer_id,
                    TouchPointerDownOutsideCandidate {
                        layer_id,
                        root,
                        consume,
                        down_event,
                        start_pos: position,
                        moved: false,
                    },
                );
                return PointerDownOutsideOutcome::default();
            }

            #[cfg(debug_assertions)]
            if crate::runtime_config::ui_runtime_config().debug_pointer_down_outside {
                eprintln!(
                    "pointer_down_outside: layer={layer_id:?} root={root:?} consume={consume} focus_before={:?} hit={hit:?} hit_root={hit_root:?}",
                    self.focus(),
                );
            }
            let (window, root_element, tick_id) = if let Some(window) = self.window
                && let Some(root_element) = self.nodes.get(root).and_then(|n| n.element)
            {
                let tick_id = app.tick_id();
                crate::elements::with_element_state(
                    app,
                    window,
                    root_element,
                    crate::action::DismissibleLastDismissRequest::default,
                    |st| {
                        st.tick_id = tick_id;
                        st.reason = None;
                        st.default_prevented = false;
                    },
                );
                (Some(window), Some(root_element), Some(tick_id))
            } else {
                (None, None, None)
            };
            self.dispatch_event_to_node_chain_observer(
                app,
                services,
                params.input_ctx,
                root,
                params.event,
                invalidation_visited,
            );
            // Match Radix/web outcomes: clicking outside a dismissible overlay should clear focus
            // from the overlay subtree. If policy prevents default dismissal, keep focus stable.
            //
            // If the event is click-through, the subsequent hit-tested dispatch can still assign
            // focus to the underlay target.
            let mut clear_focus = true;
            if let (Some(window), Some(root_element), Some(tick_id)) =
                (window, root_element, tick_id)
            {
                let prevented = crate::elements::with_element_state(
                    app,
                    window,
                    root_element,
                    crate::action::DismissibleLastDismissRequest::default,
                    |st| {
                        st.tick_id == tick_id
                            && matches!(
                                st.reason,
                                Some(crate::action::DismissReason::OutsidePress { .. })
                            )
                            && st.default_prevented
                    },
                );
                if prevented {
                    clear_focus = false;
                }
            }

            if clear_focus {
                self.set_focus(None);
            }
            #[cfg(debug_assertions)]
            if crate::runtime_config::ui_runtime_config().debug_pointer_down_outside {
                eprintln!(
                    "pointer_down_outside: focus_after={:?} (suppress_hit_test_dispatch={consume} clear_focus={clear_focus})",
                    self.focus(),
                );
            }
            return PointerDownOutsideOutcome {
                dispatched: true,
                suppress_hit_test_dispatch: consume,
            };
        }

        PointerDownOutsideOutcome::default()
    }

    pub(in crate::tree) fn rects_intersect(a: Rect, b: Rect) -> bool {
        let ax0 = a.origin.x.0;
        let ay0 = a.origin.y.0;
        let ax1 = ax0 + a.size.width.0;
        let ay1 = ay0 + a.size.height.0;

        let bx0 = b.origin.x.0;
        let by0 = b.origin.y.0;
        let bx1 = bx0 + b.size.width.0;
        let by1 = by0 + b.size.height.0;

        ax0 < bx1 && ax1 > bx0 && ay0 < by1 && ay1 > by0
    }

    pub(in crate::tree) fn collect_focusables(
        &self,
        node: NodeId,
        active_layers: &[NodeId],
        scope_bounds: Rect,
        out: &mut Vec<NodeId>,
    ) {
        if !self.node_in_any_layer(node, active_layers) {
            return;
        }

        let Some(n) = self.nodes.get(node) else {
            return;
        };
        if n.bounds.size.width.0 <= 0.0 || n.bounds.size.height.0 <= 0.0 {
            return;
        }
        if !Self::rects_intersect(n.bounds, scope_bounds)
            && !self.node_has_scrollable_ancestor_in_scope(node, active_layers, scope_bounds)
        {
            return;
        }

        let prepaint = (!self.inspection_active && !n.invalidation.hit_test)
            .then_some(n.prepaint_hit_test)
            .flatten();
        let is_focusable = prepaint
            .as_ref()
            .map(|p| p.is_focusable)
            .unwrap_or_else(|| n.widget.as_ref().is_some_and(|w| w.is_focusable()));
        if is_focusable {
            out.push(node);
        }

        let traverse_children = prepaint
            .as_ref()
            .map(|p| p.focus_traversal_children)
            .unwrap_or_else(|| {
                n.widget
                    .as_ref()
                    .map(|w| w.focus_traversal_children())
                    .unwrap_or(true)
            });
        if traverse_children {
            for &child in &n.children {
                self.collect_focusables(child, active_layers, scope_bounds, out);
            }
        }
    }

    fn node_has_scrollable_ancestor_in_scope(
        &self,
        mut node: NodeId,
        active_layers: &[NodeId],
        scope_bounds: Rect,
    ) -> bool {
        loop {
            let Some(parent) = self.nodes.get(node).and_then(|n| n.parent) else {
                return false;
            };
            node = parent;

            if !self.node_in_any_layer(node, active_layers) {
                return false;
            }

            let Some(n) = self.nodes.get(node) else {
                return false;
            };
            if n.bounds.size.width.0 <= 0.0 || n.bounds.size.height.0 <= 0.0 {
                continue;
            }
            if !Self::rects_intersect(n.bounds, scope_bounds) {
                continue;
            }

            let prepaint = (!self.inspection_active && !n.invalidation.hit_test)
                .then_some(n.prepaint_hit_test)
                .flatten();
            let can_scroll_descendant_into_view = prepaint
                .as_ref()
                .map(|p| p.can_scroll_descendant_into_view)
                .unwrap_or_else(|| {
                    n.widget
                        .as_ref()
                        .is_some_and(|w| w.can_scroll_descendant_into_view())
                });
            if can_scroll_descendant_into_view {
                return true;
            }
        }
    }

    pub(in crate::tree) fn focus_is_text_input(&mut self, app: &mut H) -> bool {
        let Some(focus) = self.focus else {
            return false;
        };

        if let Some(window) = self.window
            && let Some(record) = crate::declarative::element_record_for_node(app, window, focus)
        {
            return matches!(
                &record.instance,
                crate::declarative::ElementInstance::TextInput(_)
                    | crate::declarative::ElementInstance::TextArea(_)
                    | crate::declarative::ElementInstance::TextInputRegion(_)
            );
        }

        if self
            .nodes
            .get(focus)
            .and_then(|n| n.widget.as_ref())
            .is_none()
        {
            return false;
        }
        self.with_widget_mut(focus, |widget, _tree| widget.is_text_input())
    }
}
