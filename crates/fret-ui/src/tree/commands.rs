use super::*;
use crate::widget::{CommandAvailability, CommandAvailabilityCx};
use fret_runtime::CommandScope;

impl<H: UiHost> UiTree<H> {
    #[stacksafe::stacksafe]
    pub fn is_command_available(&mut self, app: &mut H, command: &CommandId) -> bool {
        self.command_availability(app, command) == CommandAvailability::Available
    }

    /// GPUI naming parity: "is this action available along the dispatch path?"
    ///
    /// Note: Fret models "actions" as `CommandId` today (especially for widget-scoped commands).
    #[stacksafe::stacksafe]
    pub fn is_action_available(&mut self, app: &mut H, command: &CommandId) -> bool {
        self.is_command_available(app, command)
    }

    /// GPUI naming parity for availability queries.
    #[stacksafe::stacksafe]
    pub fn action_availability(&mut self, app: &mut H, command: &CommandId) -> CommandAvailability {
        self.command_availability(app, command)
    }

    #[stacksafe::stacksafe]
    pub fn command_availability(
        &mut self,
        app: &mut H,
        command: &CommandId,
    ) -> CommandAvailability {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return CommandAvailability::NotHandled;
        };

        let (active_layers, barrier_root) = self.active_input_layers();
        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let mut input_ctx: InputContext = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal: barrier_root.is_some(),
            window_arbitration: None,
            focus_is_text_input: self.focus_is_text_input(app),
            text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
            edit_can_undo: true,
            edit_can_redo: true,
            dispatch_phase: InputDispatchPhase::Bubble,
        };
        if let Some(window) = self.window {
            if let Some(mode) = app
                .global::<fret_runtime::WindowTextBoundaryModeService>()
                .and_then(|svc| svc.mode(window))
            {
                input_ctx.text_boundary_mode = mode;
            }
            if let Some(availability) = app
                .global::<fret_runtime::WindowCommandAvailabilityService>()
                .and_then(|svc| svc.snapshot(window))
                .copied()
            {
                input_ctx.edit_can_undo = availability.edit_can_undo;
                input_ctx.edit_can_redo = availability.edit_can_redo;
            }
            input_ctx.window_arbitration = Some(self.window_input_arbitration_snapshot());
        }

        let Some(start) =
            self.command_availability_start_node(base_root, &active_layers, barrier_root)
        else {
            return CommandAvailability::NotHandled;
        };

        let availability = self.command_availability_from_node(app, &input_ctx, start, command);

        if availability == CommandAvailability::NotHandled
            && matches!(command.as_str(), "focus.next" | "focus.previous")
        {
            return self.focus_traversal_command_availability(&active_layers, barrier_root);
        }

        availability
    }

    fn focus_traversal_command_availability(
        &mut self,
        active_layers: &[NodeId],
        barrier_root: Option<NodeId>,
    ) -> CommandAvailability {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return CommandAvailability::NotHandled;
        };

        let scope_root = barrier_root.unwrap_or(base_root);
        let scope_bounds = self
            .nodes
            .get(scope_root)
            .map(|n| n.bounds)
            .unwrap_or_default();

        let mut focusables: Vec<NodeId> = Vec::new();
        for &root in active_layers {
            self.collect_focusables(root, active_layers, scope_bounds, &mut focusables);
        }

        if focusables.is_empty() {
            CommandAvailability::NotHandled
        } else {
            CommandAvailability::Available
        }
    }

    fn command_availability_start_node(
        &mut self,
        base_root: NodeId,
        active_layers: &[NodeId],
        barrier_root: Option<NodeId>,
    ) -> Option<NodeId> {
        if self
            .focus
            .is_some_and(|n| !self.node_in_any_layer(n, active_layers))
        {
            self.focus = None;
        }

        let default_root = barrier_root.unwrap_or(base_root);
        self.focus.or(Some(default_root))
    }

    #[stacksafe::stacksafe]
    fn command_availability_from_node(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
        start: NodeId,
        command: &CommandId,
    ) -> CommandAvailability {
        let mut node_id = start;
        loop {
            let (availability, parent) = self.with_widget_mut(node_id, |widget, tree| {
                let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                let window = tree.window;
                let focus = tree.focus;
                let mut cx = CommandAvailabilityCx {
                    app,
                    tree: &*tree,
                    node: node_id,
                    window,
                    input_ctx: input_ctx.clone(),
                    focus,
                };
                (widget.command_availability(&mut cx, command), parent)
            });

            match availability {
                CommandAvailability::Available | CommandAvailability::Blocked => {
                    return availability;
                }
                CommandAvailability::NotHandled => {}
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }

        CommandAvailability::NotHandled
    }

    /// Publish a per-window action availability snapshot for widget-scoped commands.
    ///
    /// This is a data-only integration seam for runner/platform and UI-kit layers (menus, command
    /// palette, shortcut help). Most apps should prefer publishing a filtered snapshot (e.g. only
    /// menu/palette command sets) at the app-driver layer. This retained-runtime helper exists for
    /// callers that want the "all widget commands" baseline behavior.
    pub fn publish_window_command_action_availability_snapshot(
        &mut self,
        app: &mut H,
        input_ctx: &InputContext,
    ) {
        let Some(window) = self.window else {
            return;
        };

        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return;
        };
        let (active_layers, barrier_root) = self.active_input_layers();
        let Some(start) =
            self.command_availability_start_node(base_root, &active_layers, barrier_root)
        else {
            return;
        };

        let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
        let widget_commands: Vec<CommandId> = app
            .commands()
            .iter()
            .filter_map(|(id, meta)| (meta.scope == CommandScope::Widget).then_some(id.clone()))
            .collect();

        for id in widget_commands {
            if id.as_str() == "focus.menu_bar" {
                let present = app
                    .global::<fret_runtime::WindowMenuBarFocusService>()
                    .is_some_and(|svc| svc.present(window));
                snapshot.insert(id, present);
                continue;
            }

            let mut availability = self.command_availability_from_node(app, input_ctx, start, &id);
            if availability == CommandAvailability::NotHandled
                && matches!(id.as_str(), "focus.next" | "focus.previous")
            {
                availability =
                    self.focus_traversal_command_availability(&active_layers, barrier_root);
            }
            if availability == CommandAvailability::NotHandled && id.as_str() == "focus.menu_bar" {
                let present = app
                    .global::<fret_runtime::WindowMenuBarFocusService>()
                    .is_some_and(|svc| svc.present(window));
                snapshot.insert(id, present);
                continue;
            }
            match availability {
                CommandAvailability::Available => {
                    snapshot.insert(id, true);
                }
                CommandAvailability::Blocked => {
                    snapshot.insert(id, false);
                }
                CommandAvailability::NotHandled => {
                    if matches!(id.as_str(), "focus.next" | "focus.previous") {
                        snapshot.insert(id, false);
                    }
                }
            }
        }

        app.with_global_mut(
            fret_runtime::WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                svc.set_snapshot(window, snapshot);
            },
        );
    }

    #[stacksafe::stacksafe]
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
        let mut input_ctx = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal: barrier_root.is_some(),
            window_arbitration: None,
            focus_is_text_input: self.focus_is_text_input(app),
            text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
            edit_can_undo: true,
            edit_can_redo: true,
            dispatch_phase: InputDispatchPhase::Bubble,
        };
        if let Some(window) = self.window {
            if let Some(mode) = app
                .global::<fret_runtime::WindowTextBoundaryModeService>()
                .and_then(|svc| svc.mode(window))
            {
                input_ctx.text_boundary_mode = mode;
            }
            if let Some(availability) = app
                .global::<fret_runtime::WindowCommandAvailabilityService>()
                .and_then(|svc| svc.snapshot(window))
                .copied()
            {
                input_ctx.edit_can_undo = availability.edit_can_undo;
                input_ctx.edit_can_redo = availability.edit_can_redo;
            }

            let window_arbitration = self.window_input_arbitration_snapshot();
            input_ctx.window_arbitration = Some(window_arbitration);

            let needs_update = app
                .global::<fret_runtime::WindowInputContextService>()
                .and_then(|svc| svc.snapshot(window))
                .is_none_or(|prev| prev != &input_ctx);
            if needs_update {
                app.with_global_mut(
                    fret_runtime::WindowInputContextService::default,
                    |svc, _app| {
                        svc.set_snapshot(window, input_ctx.clone());
                    },
                );
            }
        }
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

        if needs_redraw {
            self.request_redraw_coalesced(app);
        }

        // Publish a post-dispatch snapshot so runner-level integration surfaces (e.g. OS menubars)
        // see the latest focus/modal state without waiting for the next paint pass.
        if let Some(window) = self.window {
            let (_active_layers, barrier_root) = self.active_input_layers();
            let caps = app
                .global::<PlatformCapabilities>()
                .cloned()
                .unwrap_or_default();
            let mut input_ctx = InputContext {
                platform: Platform::current(),
                caps,
                ui_has_modal: barrier_root.is_some(),
                window_arbitration: None,
                focus_is_text_input: self.focus_is_text_input(app),
                text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
                edit_can_undo: true,
                edit_can_redo: true,
                dispatch_phase: InputDispatchPhase::Bubble,
            };
            if let Some(mode) = app
                .global::<fret_runtime::WindowTextBoundaryModeService>()
                .and_then(|svc| svc.mode(window))
            {
                input_ctx.text_boundary_mode = mode;
            }
            if let Some(availability) = app
                .global::<fret_runtime::WindowCommandAvailabilityService>()
                .and_then(|svc| svc.snapshot(window))
                .copied()
            {
                input_ctx.edit_can_undo = availability.edit_can_undo;
                input_ctx.edit_can_redo = availability.edit_can_redo;
            }

            let window_arbitration = self.window_input_arbitration_snapshot();
            input_ctx.window_arbitration = Some(window_arbitration);

            let needs_update = app
                .global::<fret_runtime::WindowInputContextService>()
                .and_then(|svc| svc.snapshot(window))
                .is_none_or(|prev| prev != &input_ctx);
            if needs_update {
                app.with_global_mut(
                    fret_runtime::WindowInputContextService::default,
                    |svc, _app| {
                        svc.set_snapshot(window, input_ctx.clone());
                    },
                );
            }

            self.publish_window_command_action_availability_snapshot(app, &input_ctx);
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
        self.request_redraw_coalesced(app);
        true
    }

    pub fn scroll_node_into_view(&mut self, app: &mut H, target: NodeId) -> bool {
        let Some(target_bounds) = self.nodes.get(target).map(|n| n.bounds) else {
            return false;
        };

        // Only scroll *ancestors* of the target into view.
        //
        // If the target itself is scrollable, attempting to scroll it “into view” via itself can
        // incorrectly mutate its offset (e.g. resetting a virtual list to top when it receives
        // focus).
        let mut node = self.nodes.get(target).and_then(|n| n.parent);
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
                    self.request_redraw_coalesced(app);
                }
                return did_scroll;
            }
        }

        false
    }
}
