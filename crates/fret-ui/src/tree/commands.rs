use super::*;

impl<H: UiHost> UiTree<H> {
    pub fn dispatch_command(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        command: &CommandId,
    ) -> bool {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return false;
        };

        let (active_layers, barrier_root) = self.active_input_layers();
        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let input_ctx = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal: barrier_root.is_some(),
            focus_is_text_input: self.focus_is_text_input(),
            dispatch_phase: InputDispatchPhase::Normal,
        };
        let is_focus_traversal_command =
            matches!(command.as_str(), "focus.next" | "focus.previous");

        if self
            .focus
            .is_some_and(|n| !self.node_in_any_layer(n, &active_layers))
        {
            self.focus = None;
        }

        let default_root = barrier_root.unwrap_or(base_root);
        let node_id = self.focus.or(Some(default_root));
        let Some(mut node_id) = node_id else {
            return false;
        };

        let mut handled = false;
        let mut needs_redraw = false;
        let mut stopped = false;

        loop {
            let (did_handle, invalidations, requested_focus, stop_bubbling, parent) = self
                .with_widget_mut(node_id, |widget, tree| {
                    let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                    let window = tree.window;
                    let focus = tree.focus;
                    let mut cx = CommandCx {
                        app,
                        services: &mut *services,
                        tree,
                        node: node_id,
                        window,
                        input_ctx: input_ctx.clone(),
                        focus,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        stop_propagation: false,
                    };
                    let did_handle = widget.command(&mut cx, command);
                    (
                        did_handle,
                        cx.invalidations,
                        cx.requested_focus,
                        cx.stop_propagation,
                        parent,
                    )
                });

            if did_handle {
                handled = true;
            }

            if !invalidations.is_empty() || requested_focus.is_some() {
                needs_redraw = true;
            }

            for (id, inv) in invalidations {
                self.mark_invalidation(id, inv);
            }

            if let Some(focus) = requested_focus {
                let (active_roots, _barrier_root) = self.active_input_layers();
                if self.focus_request_is_allowed(app, self.window, &active_roots, focus) {
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                }
            }

            if did_handle {
                break;
            }
            if stop_bubbling {
                stopped = true;
                break;
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }

        if !handled && !stopped && is_focus_traversal_command {
            handled = self.dispatch_focus_traversal(
                app,
                command,
                &active_layers,
                barrier_root,
                base_root,
            );
            needs_redraw = true;
        }

        if needs_redraw && let Some(window) = self.window {
            app.request_redraw(window);
        }

        handled
    }

    fn dispatch_focus_traversal(
        &mut self,
        app: &mut H,
        command: &CommandId,
        active_layers: &[NodeId],
        barrier_root: Option<NodeId>,
        base_root: NodeId,
    ) -> bool {
        let direction = match command.as_str() {
            "focus.next" => Some(true),
            "focus.previous" => Some(false),
            _ => None,
        };
        let Some(forward) = direction else {
            return false;
        };

        let _ = base_root;
        self.focus_traverse_in_roots(app, active_layers, forward, barrier_root)
    }

    /// Focus traversal mechanism used by both the runtime default and component-owned focus scopes.
    ///
    /// Notes:
    /// - `roots` are treated as candidates; only focusables that are in the current active input layers
    ///   (modal-aware) and intersect the modal scope bounds are included.
    /// - This is intentionally conservative until we formalize a scroll-into-view contract (ADR 0068).
    pub fn focus_traverse_in_roots(
        &mut self,
        app: &mut H,
        roots: &[NodeId],
        forward: bool,
        scope_root: Option<NodeId>,
    ) -> bool {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return true;
        };
        let (active_layers, barrier_root) = self.active_input_layers();

        let scope_root = scope_root.or(barrier_root).unwrap_or(base_root);
        let scope_bounds = self
            .nodes
            .get(scope_root)
            .map(|n| n.bounds)
            .unwrap_or_default();

        let mut focusables: Vec<NodeId> = Vec::new();
        for &root in roots {
            self.collect_focusables(root, &active_layers, scope_bounds, &mut focusables);
        }
        if focusables.is_empty() {
            return true;
        }

        let next = match self
            .focus
            .and_then(|f| focusables.iter().position(|n| *n == f))
        {
            Some(idx) => {
                if forward {
                    focusables[(idx + 1) % focusables.len()]
                } else {
                    focusables[(idx + focusables.len() - 1) % focusables.len()]
                }
            }
            None => {
                if forward {
                    focusables[0]
                } else {
                    focusables[focusables.len() - 1]
                }
            }
        };

        if self.focus != Some(next) {
            if let Some(prev) = self.focus {
                self.mark_invalidation(prev, Invalidation::Paint);
            }
            self.focus = Some(next);
            self.mark_invalidation(next, Invalidation::Paint);
            self.scroll_node_into_view(app, next);
        }
        if let Some(window) = self.window {
            app.request_redraw(window);
        }
        true
    }

    fn scroll_node_into_view(&mut self, app: &mut H, target: NodeId) -> bool {
        let Some(target_bounds) = self.nodes.get(target).map(|n| n.bounds) else {
            return false;
        };

        let mut node = Some(target);
        while let Some(id) = node {
            let parent = self.nodes.get(id).and_then(|n| n.parent);
            node = parent;

            let Some(bounds) = self.nodes.get(id).map(|n| n.bounds) else {
                continue;
            };

            let Some(widget) = self.nodes.get(id).and_then(|n| n.widget.as_ref()) else {
                continue;
            };
            if !widget.can_scroll_descendant_into_view() {
                continue;
            }

            let result = self.with_widget_mut(id, |widget, tree| {
                let mut cx = crate::widget::ScrollIntoViewCx {
                    app,
                    node: id,
                    window: tree.window,
                    bounds,
                };
                widget.scroll_descendant_into_view(&mut cx, target_bounds)
            });

            if let crate::widget::ScrollIntoViewResult::Handled { did_scroll } = result {
                if did_scroll {
                    self.mark_invalidation(id, Invalidation::HitTest);
                    if let Some(window) = self.window {
                        app.request_redraw(window);
                    }
                }
                return did_scroll;
            }
        }

        false
    }
}
