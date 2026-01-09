use super::*;

impl<H: UiHost> UiTree<H> {
    fn update_ime_composing_for_event(&mut self, focus_is_text_input: bool, event: &Event) {
        if !focus_is_text_input {
            self.ime_composing = false;
            return;
        }

        let Event::Ime(ime) = event else {
            return;
        };

        match ime {
            fret_core::ImeEvent::Preedit { text, cursor } => {
                self.ime_composing = crate::text_edit::ime::is_composing(text, *cursor);
            }
            fret_core::ImeEvent::Commit(_) | fret_core::ImeEvent::Disabled => {
                self.ime_composing = false;
            }
            fret_core::ImeEvent::Enabled => {}
        }
    }

    fn active_trapped_focus_scope_root(
        &self,
        app: &mut H,
        window: Option<AppWindowId>,
    ) -> Option<NodeId> {
        let window = window?;
        let mut node = self.focus?;
        loop {
            if let Some(record) = declarative::element_record_for_node(app, window, node)
                && matches!(
                    record.instance,
                    declarative::ElementInstance::FocusScope(p) if p.trap_focus
                )
            {
                return Some(node);
            }

            node = self.nodes.get(node).and_then(|n| n.parent)?;
        }
    }

    pub(super) fn focus_request_is_allowed(
        &self,
        app: &mut H,
        window: Option<AppWindowId>,
        active_roots: &[NodeId],
        requested_focus: NodeId,
    ) -> bool {
        if self.focus == Some(requested_focus) {
            return false;
        }
        if !self.node_in_any_layer(requested_focus, active_roots) {
            return false;
        }

        let Some(trap_root) = self.active_trapped_focus_scope_root(app, window) else {
            return true;
        };
        self.is_descendant(trap_root, requested_focus)
    }

    fn dispatch_event_to_node_chain(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        input_ctx: &InputContext,
        start: NodeId,
        event: &Event,
    ) -> bool {
        let (active_roots, _barrier_root) = self.active_input_layers();
        if event_position(event).is_some() {
            let chain = self.build_mapped_event_chain(start, event);
            for (node_id, event_for_node) in chain {
                let (invalidations, requested_focus, requested_capture, stop_propagation) = self
                    .with_widget_mut(node_id, |widget, tree| {
                        let (children, bounds) = tree
                            .nodes
                            .get(node_id)
                            .map(|n| (n.children.as_slice(), n.bounds))
                            .unwrap_or((&[][..], Rect::default()));
                        let mut cx = EventCx {
                            app,
                            services: &mut *services,
                            node: node_id,
                            window: tree.window,
                            input_ctx: input_ctx.clone(),
                            children,
                            focus: tree.focus,
                            captured: tree.captured,
                            bounds,
                            invalidations: Vec::new(),
                            requested_focus: None,
                            requested_capture: None,
                            requested_cursor: None,
                            stop_propagation: false,
                        };
                        widget.event(&mut cx, &event_for_node);
                        (
                            cx.invalidations,
                            cx.requested_focus,
                            cx.requested_capture,
                            cx.stop_propagation,
                        )
                    });

                for (id, inv) in invalidations {
                    self.mark_invalidation(id, inv);
                }

                if let Some(focus) = requested_focus
                    && self.focus_request_is_allowed(app, self.window, &active_roots, focus)
                {
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                }

                if let Some(capture) = requested_capture
                    && capture.is_none_or(|n| self.node_in_any_layer(n, &active_roots))
                {
                    self.captured = capture;
                }

                if self.captured.is_some() || stop_propagation {
                    return true;
                }
            }
            return false;
        }

        let mut node_id = start;
        loop {
            let (invalidations, requested_focus, requested_capture, stop_propagation, parent) =
                self.with_widget_mut(node_id, |widget, tree| {
                    let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                    let (children, bounds) = tree
                        .nodes
                        .get(node_id)
                        .map(|n| (n.children.as_slice(), n.bounds))
                        .unwrap_or((&[][..], Rect::default()));
                    let mut cx = EventCx {
                        app,
                        services: &mut *services,
                        node: node_id,
                        window: tree.window,
                        input_ctx: input_ctx.clone(),
                        children,
                        focus: tree.focus,
                        captured: tree.captured,
                        bounds,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        requested_capture: None,
                        requested_cursor: None,
                        stop_propagation: false,
                    };
                    widget.event(&mut cx, event);
                    (
                        cx.invalidations,
                        cx.requested_focus,
                        cx.requested_capture,
                        cx.stop_propagation,
                        parent,
                    )
                });

            for (id, inv) in invalidations {
                self.mark_invalidation(id, inv);
            }

            if let Some(focus) = requested_focus
                && self.focus_request_is_allowed(app, self.window, &active_roots, focus)
            {
                if let Some(prev) = self.focus {
                    self.mark_invalidation(prev, Invalidation::Paint);
                }
                self.focus = Some(focus);
                self.mark_invalidation(focus, Invalidation::Paint);
            }

            if let Some(capture) = requested_capture
                && capture.is_none_or(|n| self.node_in_any_layer(n, &active_roots))
            {
                self.captured = capture;
            }

            if self.captured.is_some() || stop_propagation {
                return true;
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }

        false
    }

    pub fn dispatch_event(&mut self, app: &mut H, services: &mut dyn UiServices, event: &Event) {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return;
        };

        let (active_layers, barrier_root) = self.active_input_layers();
        self.enforce_modal_barrier_scope(&active_layers);

        if self
            .captured
            .is_some_and(|n| !self.node_in_any_layer(n, &active_layers))
        {
            self.captured = None;
        }
        if self
            .focus
            .is_some_and(|n| !self.node_in_any_layer(n, &active_layers))
        {
            self.focus = None;
        }

        let focus_is_text_input = self.focus_is_text_input();
        self.update_ime_composing_for_event(focus_is_text_input, event);
        self.set_ime_allowed(app, focus_is_text_input);

        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let input_ctx = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal: barrier_root.is_some(),
            focus_is_text_input,
            dispatch_phase: InputDispatchPhase::Normal,
        };

        // ADR 0012: when a text input is focused, reserve common IME/navigation keys for the
        // text/IME path first, and only fall back to shortcut matching if the widget doesn't
        // consume the event.
        let defer_keydown_shortcuts_until_after_dispatch = !self.replaying_pending_shortcut
            && self.focus.is_some()
            && match event {
                Event::KeyDown { key, modifiers, .. } => {
                    Self::should_defer_keydown_shortcut_matching_to_text_input(
                        *key,
                        *modifiers,
                        focus_is_text_input,
                    )
                }
                _ => false,
            };

        if let Some(window) = self.window {
            let changed = crate::focus_visible::update_for_event(app, window, event);
            if changed {
                if let Some(focus) = self.focus {
                    self.invalidate(focus, Invalidation::Paint);
                } else {
                    self.invalidate(base_root, Invalidation::Paint);
                }
                app.request_redraw(window);
            }

            let changed = crate::input_modality::update_for_event(app, window, event);
            if changed {
                if let Some(focus) = self.focus {
                    self.invalidate(focus, Invalidation::Paint);
                } else {
                    self.invalidate(base_root, Invalidation::Paint);
                }
                app.request_redraw(window);
            }
        }

        if !self.replaying_pending_shortcut
            && !self.pending_shortcut.keystrokes.is_empty()
            && ((self.pending_shortcut.focus.is_some()
                && self.pending_shortcut.focus != self.focus)
                || self.pending_shortcut.barrier_root != barrier_root)
        {
            self.clear_pending_shortcut(app);
        }

        if let Event::Timer { token } = event
            && !self.replaying_pending_shortcut
            && !self.pending_shortcut.keystrokes.is_empty()
            && self.pending_shortcut.timer == Some(*token)
        {
            let pending = std::mem::take(&mut self.pending_shortcut);
            if let Some(command) = pending.fallback {
                app.push_effect(Effect::Command {
                    window: self.window,
                    command,
                });
            } else {
                self.replay_captured_keystrokes(app, services, &input_ctx, pending.keystrokes);
            }
            return;
        }
        if matches!(event, Event::Timer { .. }) {
            if let Event::Timer { token } = event
                && let Some(window) = self.window
                && let Some(node) = crate::elements::timer_target_node(app, window, *token)
            {
                let stopped =
                    self.dispatch_event_to_node_chain(app, services, &input_ctx, node, event);
                if stopped {
                    return;
                }
            }

            let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
            for layer_id in layers.into_iter().rev() {
                let Some(layer) = self.layers.get(layer_id) else {
                    continue;
                };
                if !layer.wants_timer_events || !layer.visible {
                    continue;
                }
                let stopped =
                    self.dispatch_event_to_node_chain(app, services, &input_ctx, layer.root, event);
                if stopped {
                    return;
                }
            }
        }

        if let Event::TextInput(text) = event {
            if !self.replaying_pending_shortcut
                && self.pending_shortcut.capture_next_text_input_key.is_some()
            {
                self.pending_shortcut.capture_next_text_input_key = None;
                if let Some(last) = self.pending_shortcut.keystrokes.last_mut() {
                    last.text = Some(text.clone());
                }
                self.suppress_text_input_until_key_up = None;
                return;
            }

            if self.suppress_text_input_until_key_up.is_some() {
                self.suppress_text_input_until_key_up = None;
                return;
            }
        }

        if let Event::KeyUp { key, .. } = event {
            if self.suppress_text_input_until_key_up == Some(*key) {
                self.suppress_text_input_until_key_up = None;
            }
            if self.pending_shortcut.capture_next_text_input_key == Some(*key) {
                self.pending_shortcut.capture_next_text_input_key = None;
            }
        }

        let mut needs_redraw = false;
        let mut cursor_choice: Option<fret_core::CursorIcon> = None;
        let mut stop_propagation_requested = false;
        let mut pointer_down_outside = PointerDownOutsideOutcome::default();
        let is_wheel = matches!(event, Event::Pointer(PointerEvent::Wheel { .. }));
        let mut wheel_stop_node: Option<NodeId> = None;

        if let Event::KeyDown {
            key,
            modifiers,
            repeat,
        } = event
            && !defer_keydown_shortcuts_until_after_dispatch
            && self.handle_keydown_shortcuts(
                app,
                services,
                KeydownShortcutParams {
                    input_ctx: &input_ctx,
                    barrier_root,
                    focus_is_text_input,
                    key: *key,
                    modifiers: *modifiers,
                    repeat: *repeat,
                },
            )
        {
            return;
        }

        let default_root = barrier_root.unwrap_or(base_root);

        // Pointer capture only affects pointer events. Drag-and-drop style events
        // (external/internal) must continue to follow the cursor for correct cross-window UX.
        let captured = match event {
            Event::Pointer(_) => self.captured,
            _ => None,
        };

        // Internal drag overrides may need to route events to a stable "anchor" node, even if
        // hit-testing fails or the cursor is over an unrelated widget (e.g. docking tear-off).
        let internal_drag_target = (|| {
            if !matches!(event, Event::InternalDrag(_)) {
                return None;
            }
            let window = self.window?;
            let drag = app.drag()?;
            if !drag.cross_window_hover {
                return None;
            }
            let routes = app.global::<crate::drag_route::InternalDragRouteService>()?;
            let target = routes.route(window, drag.kind)?;
            self.node_in_any_layer(target, &active_layers)
                .then_some(target)
        })();

        if let Some(window) = self.window
            && matches!(event, Event::Pointer(_))
            && let Some(pos) = event_position(event)
        {
            let hit = self.hit_test_layers(&active_layers, pos);

            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) && captured.is_none() {
                pointer_down_outside = self.dispatch_pointer_down_outside(
                    app,
                    services,
                    PointerDownOutsideParams {
                        input_ctx: &input_ctx,
                        active_layer_roots: &active_layers,
                        base_root,
                        hit,
                        event,
                    },
                );
                if pointer_down_outside.dispatched {
                    needs_redraw = true;
                }
            }
            let hovered_pressable: Option<crate::elements::GlobalElementId> =
                declarative::with_window_frame(app, window, |window_frame| {
                    let window_frame = window_frame?;
                    let mut node = hit;
                    while let Some(id) = node {
                        if let Some(record) = window_frame.instances.get(&id)
                            && matches!(record.instance, declarative::ElementInstance::Pressable(_))
                        {
                            return Some(record.element);
                        }
                        node = self.nodes.get(id).and_then(|n| n.parent);
                    }
                    None
                });

            let (prev_element, prev_node, next_element, next_node) =
                crate::elements::update_hovered_pressable(app, window, hovered_pressable);
            if prev_node.is_some() || next_node.is_some() {
                needs_redraw = true;
                if let Some(node) = prev_node {
                    self.mark_invalidation(node, Invalidation::Paint);
                }
                if let Some(node) = next_node {
                    self.mark_invalidation(node, Invalidation::Paint);
                }
            }

            if let Some(element) = prev_element
                && prev_node.is_some()
            {
                let hook = crate::elements::with_element_state(
                    app,
                    window,
                    element,
                    crate::action::PressableHoverActionHooks::default,
                    |hooks| hooks.on_hover_change.clone(),
                );

                if let Some(h) = hook {
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
                                    crate::elements::clear_timer_target(
                                        &mut *self.app,
                                        self.window,
                                        token,
                                    );
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
                    }

                    let mut host = PressableHoverHookHost {
                        app,
                        window,
                        element,
                    };
                    h(
                        &mut host,
                        crate::action::ActionCx {
                            window,
                            target: element,
                        },
                        false,
                    );
                }
            }

            if let Some(element) = next_element
                && next_node.is_some()
            {
                let hook = crate::elements::with_element_state(
                    app,
                    window,
                    element,
                    crate::action::PressableHoverActionHooks::default,
                    |hooks| hooks.on_hover_change.clone(),
                );

                if let Some(h) = hook {
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
                                    crate::elements::clear_timer_target(
                                        &mut *self.app,
                                        self.window,
                                        token,
                                    );
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
                    }

                    let mut host = PressableHoverHookHost {
                        app,
                        window,
                        element,
                    };
                    h(
                        &mut host,
                        crate::action::ActionCx {
                            window,
                            target: element,
                        },
                        true,
                    );
                }
            }

            let hovered_hover_region: Option<crate::elements::GlobalElementId> =
                declarative::with_window_frame(app, window, |window_frame| {
                    let window_frame = window_frame?;
                    let mut node = hit;
                    while let Some(id) = node {
                        if let Some(record) = window_frame.instances.get(&id)
                            && matches!(
                                record.instance,
                                declarative::ElementInstance::HoverRegion(_)
                            )
                        {
                            return Some(record.element);
                        }
                        node = self.nodes.get(id).and_then(|n| n.parent);
                    }
                    None
                });

            let (_prev_element, prev_node, _next_element, next_node) =
                crate::elements::update_hovered_hover_region(app, window, hovered_hover_region);
            if prev_node.is_some() || next_node.is_some() {
                needs_redraw = true;
                if let Some(node) = prev_node {
                    self.mark_invalidation(node, Invalidation::Layout);
                    self.mark_invalidation(node, Invalidation::Paint);
                }
                if let Some(node) = next_node {
                    self.mark_invalidation(node, Invalidation::Layout);
                    self.mark_invalidation(node, Invalidation::Paint);
                }
            }
        }

        let target = if let Some(captured) = captured {
            Some(captured)
        } else if let Some(target) = internal_drag_target {
            Some(target)
        } else if let Some(pos) = event_position(event) {
            let hit = self.hit_test_layers(&active_layers, pos);

            if matches!(event, Event::InternalDrag(_)) {
                if let Some(node) = hit {
                    self.last_internal_drag_target = Some(node);
                } else if self
                    .last_internal_drag_target
                    .is_some_and(|n| !self.node_in_any_layer(n, &active_layers))
                {
                    self.last_internal_drag_target = None;
                }
            }

            hit.or_else(|| {
                matches!(event, Event::InternalDrag(_)).then_some(self.last_internal_drag_target)?
            })
            .or(barrier_root)
            .or(Some(default_root))
        } else {
            self.focus.or(Some(default_root))
        };

        let Some(mut node_id) = target else {
            return;
        };

        if matches!(event, Event::Pointer(PointerEvent::Down { .. }))
            && pointer_down_outside.suppress_hit_test_dispatch
        {
            if needs_redraw && let Some(window) = self.window {
                app.request_redraw(window);
            }
            return;
        }

        if event_position(event).is_some() {
            let chain = self.build_mapped_event_chain(node_id, event);
            for (node_id, event_for_node) in chain {
                let (
                    invalidations,
                    requested_focus,
                    requested_capture,
                    requested_cursor,
                    stop_propagation,
                ) = self.with_widget_mut(node_id, |widget, tree| {
                    let (children, bounds) = tree
                        .nodes
                        .get(node_id)
                        .map(|n| (n.children.as_slice(), n.bounds))
                        .unwrap_or((&[][..], Rect::default()));
                    let mut cx = EventCx {
                        app,
                        services: &mut *services,
                        node: node_id,
                        window: tree.window,
                        input_ctx: input_ctx.clone(),
                        children,
                        focus: tree.focus,
                        captured: tree.captured,
                        bounds,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        requested_capture: None,
                        requested_cursor: None,
                        stop_propagation: false,
                    };
                    widget.event(&mut cx, &event_for_node);
                    (
                        cx.invalidations,
                        cx.requested_focus,
                        cx.requested_capture,
                        cx.requested_cursor,
                        cx.stop_propagation,
                    )
                });

                if !invalidations.is_empty()
                    || requested_focus.is_some()
                    || requested_capture.is_some()
                {
                    needs_redraw = true;
                }

                for (id, inv) in invalidations {
                    self.mark_invalidation(id, inv);
                }

                if let Some(focus) = requested_focus
                    && self.focus_request_is_allowed(app, self.window, &active_layers, focus)
                {
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                }

                if let Some(capture) = requested_capture {
                    self.captured = capture;
                }

                if requested_cursor.is_some() && cursor_choice.is_none() {
                    cursor_choice = requested_cursor;
                }

                if stop_propagation {
                    stop_propagation_requested = true;
                    if is_wheel && wheel_stop_node.is_none() {
                        wheel_stop_node = Some(node_id);
                    }
                }

                if self.captured.is_some() || stop_propagation {
                    break;
                }
            }
        } else {
            loop {
                let (
                    invalidations,
                    requested_focus,
                    requested_capture,
                    requested_cursor,
                    stop_propagation,
                    parent,
                ) = self.with_widget_mut(node_id, |widget, tree| {
                    let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                    let (children, bounds) = tree
                        .nodes
                        .get(node_id)
                        .map(|n| (n.children.as_slice(), n.bounds))
                        .unwrap_or((&[][..], Rect::default()));
                    let mut cx = EventCx {
                        app,
                        services: &mut *services,
                        node: node_id,
                        window: tree.window,
                        input_ctx: input_ctx.clone(),
                        children,
                        focus: tree.focus,
                        captured: tree.captured,
                        bounds,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        requested_capture: None,
                        requested_cursor: None,
                        stop_propagation: false,
                    };
                    widget.event(&mut cx, event);
                    (
                        cx.invalidations,
                        cx.requested_focus,
                        cx.requested_capture,
                        cx.requested_cursor,
                        cx.stop_propagation,
                        parent,
                    )
                });

                if !invalidations.is_empty()
                    || requested_focus.is_some()
                    || requested_capture.is_some()
                {
                    needs_redraw = true;
                }

                for (id, inv) in invalidations {
                    self.mark_invalidation(id, inv);
                }

                if let Some(focus) = requested_focus
                    && self.focus_request_is_allowed(app, self.window, &active_layers, focus)
                {
                    if let Some(prev) = self.focus {
                        self.mark_invalidation(prev, Invalidation::Paint);
                    }
                    self.focus = Some(focus);
                    self.mark_invalidation(focus, Invalidation::Paint);
                }

                if let Some(capture) = requested_capture {
                    self.captured = capture;
                };

                if requested_cursor.is_some() && cursor_choice.is_none() {
                    cursor_choice = requested_cursor;
                }

                if stop_propagation {
                    stop_propagation_requested = true;
                    if is_wheel && wheel_stop_node.is_none() {
                        wheel_stop_node = Some(node_id);
                    }
                }

                if self.captured.is_some() || stop_propagation {
                    break;
                }

                node_id = match parent {
                    Some(parent) => parent,
                    None => break,
                };
            }
        }

        if is_wheel
            && let Some(scroll_target) = wheel_stop_node
            && let Some(window) = self.window
        {
            let is_scroll_target = declarative::with_window_frame(app, window, |window_frame| {
                let window_frame = window_frame?;
                let record = window_frame.instances.get(&scroll_target)?;
                Some(matches!(
                    record.instance,
                    declarative::ElementInstance::Scroll(_)
                        | declarative::ElementInstance::VirtualList(_)
                        | declarative::ElementInstance::WheelRegion(_)
                        | declarative::ElementInstance::Scrollbar(_)
                ))
            })
            .unwrap_or(false);

            if is_scroll_target {
                struct ScrollDismissHookHost<'a, H: crate::UiHost> {
                    app: &'a mut H,
                    window: AppWindowId,
                    element: crate::GlobalElementId,
                }

                impl<H: crate::UiHost> crate::action::UiActionHost for ScrollDismissHookHost<'_, H> {
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
                                crate::elements::clear_timer_target(
                                    &mut *self.app,
                                    self.window,
                                    token,
                                );
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
                }

                let mut dismissed_any = false;
                for layer_id in self.visible_layers_in_paint_order() {
                    let Some(layer) = self.layers.get(layer_id) else {
                        continue;
                    };
                    if layer.scroll_dismiss_descendants.is_empty() {
                        continue;
                    }
                    let should_dismiss = layer
                        .scroll_dismiss_descendants
                        .iter()
                        .copied()
                        .any(|node| self.is_descendant(scroll_target, node));
                    if !should_dismiss {
                        continue;
                    }
                    let Some(root_element) = self.nodes.get(layer.root).and_then(|n| n.element)
                    else {
                        continue;
                    };
                    let hook = crate::elements::with_element_state(
                        app,
                        window,
                        root_element,
                        crate::action::DismissibleActionHooks::default,
                        |hooks| hooks.on_dismiss_request.clone(),
                    );
                    let Some(hook) = hook else {
                        continue;
                    };
                    let mut host = ScrollDismissHookHost {
                        app,
                        window,
                        element: root_element,
                    };
                    hook(
                        &mut host,
                        crate::action::ActionCx {
                            window,
                            target: root_element,
                        },
                        crate::action::DismissReason::Scroll,
                    );
                    dismissed_any = true;
                }

                if dismissed_any {
                    needs_redraw = true;
                }
            }
        }

        if defer_keydown_shortcuts_until_after_dispatch
            && !stop_propagation_requested
            && let Event::KeyDown {
                key,
                modifiers,
                repeat,
            } = event
        {
            let focus_is_text_input = self.focus_is_text_input();
            let input_ctx_for_shortcuts = InputContext {
                focus_is_text_input,
                ..input_ctx.clone()
            };

            let ime_reserved = self.ime_composing
                && Self::should_defer_keydown_shortcut_matching_to_text_input(
                    *key,
                    *modifiers,
                    focus_is_text_input,
                );

            if !ime_reserved
                && self.handle_keydown_shortcuts(
                    app,
                    services,
                    KeydownShortcutParams {
                        input_ctx: &input_ctx_for_shortcuts,
                        barrier_root,
                        focus_is_text_input,
                        key: *key,
                        modifiers: *modifiers,
                        repeat: *repeat,
                    },
                )
            {
                if needs_redraw && let Some(window) = self.window {
                    app.request_redraw(window);
                }
                return;
            }
        }

        if input_ctx.caps.ui.cursor_icons
            && let Some(window) = self.window
            && matches!(event, Event::Pointer(_))
        {
            let icon = cursor_choice.unwrap_or(fret_core::CursorIcon::Default);
            app.push_effect(Effect::CursorSetIcon { window, icon });
        }

        if needs_redraw && let Some(window) = self.window {
            app.request_redraw(window);
        }
        if let Event::Pointer(PointerEvent::Move { .. }) = event {
            let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
            for layer_id in layers.into_iter().rev() {
                let Some(layer) = self.layers.get(layer_id) else {
                    continue;
                };
                if !layer.wants_pointer_move_events || !layer.visible {
                    continue;
                }
                let _ =
                    self.dispatch_event_to_node_chain(app, services, &input_ctx, layer.root, event);
            }
        }

        // Keep IME enable/disable tightly coupled to focus changes caused by the event itself.
        let focus_is_text_input = self.focus_is_text_input();
        self.set_ime_allowed(app, focus_is_text_input);
    }

    pub(super) fn dispatch_event_to_node_chain_observer(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        input_ctx: &InputContext,
        start: NodeId,
        event: &Event,
    ) {
        if event_position(event).is_some() {
            let chain = self.build_mapped_event_chain(start, event);
            for (node_id, event_for_node) in chain {
                let (invalidations, _parent) = self.with_widget_mut(node_id, |widget, tree| {
                    let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                    let (children, bounds) = tree
                        .nodes
                        .get(node_id)
                        .map(|n| (n.children.as_slice(), n.bounds))
                        .unwrap_or((&[][..], Rect::default()));
                    let mut observer_ctx = input_ctx.clone();
                    observer_ctx.dispatch_phase = InputDispatchPhase::Observer;
                    let mut cx = EventCx {
                        app,
                        services: &mut *services,
                        node: node_id,
                        window: tree.window,
                        input_ctx: observer_ctx,
                        children,
                        focus: tree.focus,
                        captured: tree.captured,
                        bounds,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        requested_capture: None,
                        requested_cursor: None,
                        stop_propagation: false,
                    };
                    widget.event(&mut cx, &event_for_node);

                    // Observer dispatch must not mutate routing state (capture/focus/propagation). It
                    // exists to allow click-through outside-press policies, not to intercept input.
                    (cx.invalidations, parent)
                });

                for (id, inv) in invalidations {
                    self.mark_invalidation(id, inv);
                }
            }
            return;
        }

        let mut node_id = start;
        loop {
            let (invalidations, parent) = self.with_widget_mut(node_id, |widget, tree| {
                let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                let (children, bounds) = tree
                    .nodes
                    .get(node_id)
                    .map(|n| (n.children.as_slice(), n.bounds))
                    .unwrap_or((&[][..], Rect::default()));
                let mut observer_ctx = input_ctx.clone();
                observer_ctx.dispatch_phase = InputDispatchPhase::Observer;
                let mut cx = EventCx {
                    app,
                    services: &mut *services,
                    node: node_id,
                    window: tree.window,
                    input_ctx: observer_ctx,
                    children,
                    focus: tree.focus,
                    captured: tree.captured,
                    bounds,
                    invalidations: Vec::new(),
                    requested_focus: None,
                    requested_capture: None,
                    requested_cursor: None,
                    stop_propagation: false,
                };
                widget.event(&mut cx, event);

                // Observer dispatch must not mutate routing state (capture/focus/propagation). It
                // exists to allow click-through outside-press policies, not to intercept input.
                (cx.invalidations, parent)
            });

            for (id, inv) in invalidations {
                self.mark_invalidation(id, inv);
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }
    }

    fn apply_vector(t: Transform2D, v: Point) -> Point {
        Point::new(Px(t.a * v.x.0 + t.c * v.y.0), Px(t.b * v.x.0 + t.d * v.y.0))
    }

    fn event_with_mapped_position(event: &Event, position: Point, delta: Option<Point>) -> Event {
        match event {
            Event::Pointer(e) => {
                let e = match e {
                    PointerEvent::Move {
                        buttons,
                        modifiers,
                        pointer_type,
                        ..
                    } => PointerEvent::Move {
                        position,
                        buttons: *buttons,
                        modifiers: *modifiers,
                        pointer_type: *pointer_type,
                    },
                    PointerEvent::Down {
                        button,
                        modifiers,
                        click_count,
                        pointer_type,
                        ..
                    } => PointerEvent::Down {
                        position,
                        button: *button,
                        modifiers: *modifiers,
                        click_count: *click_count,
                        pointer_type: *pointer_type,
                    },
                    PointerEvent::Up {
                        button,
                        modifiers,
                        click_count,
                        pointer_type,
                        ..
                    } => PointerEvent::Up {
                        position,
                        button: *button,
                        modifiers: *modifiers,
                        click_count: *click_count,
                        pointer_type: *pointer_type,
                    },
                    PointerEvent::Wheel {
                        modifiers,
                        pointer_type,
                        ..
                    } => PointerEvent::Wheel {
                        position,
                        delta: delta.unwrap_or(Point::new(Px(0.0), Px(0.0))),
                        modifiers: *modifiers,
                        pointer_type: *pointer_type,
                    },
                };
                Event::Pointer(e)
            }
            Event::ExternalDrag(e) => Event::ExternalDrag(fret_core::ExternalDragEvent {
                position,
                kind: e.kind.clone(),
            }),
            Event::InternalDrag(e) => Event::InternalDrag(fret_core::InternalDragEvent {
                position,
                kind: e.kind.clone(),
                modifiers: e.modifiers,
            }),
            _ => event.clone(),
        }
    }

    fn build_mapped_event_chain(&self, start: NodeId, event: &Event) -> Vec<(NodeId, Event)> {
        let Some(pos) = event_position(event) else {
            return vec![(start, event.clone())];
        };

        let mut chain: Vec<NodeId> = Vec::new();
        let mut cur = Some(start);
        while let Some(id) = cur {
            chain.push(id);
            cur = self.nodes.get(id).and_then(|n| n.parent);
        }

        let mut nodes_root_to_leaf = chain.clone();
        nodes_root_to_leaf.reverse();

        let mut mapped_pos = pos;
        let mut mapped_delta = match event {
            Event::Pointer(PointerEvent::Wheel { delta, .. }) => Some(*delta),
            _ => None,
        };

        let mut out: Vec<(NodeId, Event)> = Vec::with_capacity(chain.len());
        for &node in &nodes_root_to_leaf {
            if let Some(t) = self.node_render_transform(node)
                && let Some(inv) = t.inverse()
            {
                mapped_pos = inv.apply_point(mapped_pos);
                if let Some(d) = mapped_delta {
                    mapped_delta = Some(Self::apply_vector(inv, d));
                }
            }
            out.push((
                node,
                Self::event_with_mapped_position(event, mapped_pos, mapped_delta),
            ));
        }

        out.reverse();
        out
    }
}
