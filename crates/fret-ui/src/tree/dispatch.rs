use super::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
struct PendingInvalidation {
    inv: Invalidation,
    source: UiDebugInvalidationSource,
    detail: UiDebugInvalidationDetail,
}

impl<H: UiHost> UiTree<H> {
    fn event_is_scroll_like(event: &Event) -> bool {
        // Wheel-only for now (trackpad pan / inertial scrolling can be added later as explicit
        // inputs without changing the meaning of "Wheel" today).
        matches!(event, Event::Pointer(PointerEvent::Wheel { .. }))
    }

    fn run_pressable_hover_hook(
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

    fn update_hover_state_from_hit(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        hit_for_hover: Option<NodeId>,
        hit_for_hover_region: Option<NodeId>,
        invalidation_visited: &mut HashMap<NodeId, u8>,
        needs_redraw: &mut bool,
    ) {
        let hovered_pressable: Option<crate::elements::GlobalElementId> =
            declarative::with_window_frame(app, window, |window_frame| {
                let window_frame = window_frame?;
                let mut node = hit_for_hover;
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
            *needs_redraw = true;
            self.debug_record_hover_edge_pressable();
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

        let hovered_hover_region: Option<crate::elements::GlobalElementId> =
            declarative::with_window_frame(app, window, |window_frame| {
                let window_frame = window_frame?;
                let mut node = hit_for_hover_region;
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
            *needs_redraw = true;
            self.debug_record_hover_edge_hover_region();
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
    }

    fn invalidation_rank(inv: Invalidation) -> u8 {
        match inv {
            Invalidation::Paint => 1,
            Invalidation::HitTestOnly => 2,
            Invalidation::Layout => 3,
            Invalidation::HitTest => 4,
        }
    }

    fn stronger_invalidation(a: Invalidation, b: Invalidation) -> Invalidation {
        if Self::invalidation_rank(a) >= Self::invalidation_rank(b) {
            a
        } else {
            b
        }
    }

    fn invalidation_source_rank(source: UiDebugInvalidationSource) -> u8 {
        match source {
            UiDebugInvalidationSource::ModelChange => 6,
            UiDebugInvalidationSource::GlobalChange => 5,
            UiDebugInvalidationSource::Hover => 4,
            UiDebugInvalidationSource::Focus => 3,
            UiDebugInvalidationSource::Notify => 2,
            UiDebugInvalidationSource::Other => 1,
        }
    }

    fn stronger_invalidation_source(
        a: UiDebugInvalidationSource,
        b: UiDebugInvalidationSource,
    ) -> UiDebugInvalidationSource {
        if Self::invalidation_source_rank(a) >= Self::invalidation_source_rank(b) {
            a
        } else {
            b
        }
    }

    fn pending_invalidation_merge(
        pending: &mut HashMap<NodeId, PendingInvalidation>,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        pending
            .entry(node)
            .and_modify(|cur| {
                cur.inv = Self::stronger_invalidation(cur.inv, inv);
                cur.source = Self::stronger_invalidation_source(cur.source, source);
                cur.detail = if cur.source == UiDebugInvalidationSource::Other {
                    match (cur.detail, detail) {
                        (UiDebugInvalidationDetail::Unknown, d) => d,
                        (d, UiDebugInvalidationDetail::Unknown) => d,
                        (d, _) => d,
                    }
                } else {
                    UiDebugInvalidationDetail::from_source(cur.source)
                };
            })
            .or_insert(PendingInvalidation {
                inv,
                source,
                detail,
            });
    }

    fn dispatch_pointer_move_layer_observers(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        input_ctx: &InputContext,
        barrier_root: Option<NodeId>,
        event: &Event,
        needs_redraw: &mut bool,
        invalidation_visited: &mut HashMap<NodeId, u8>,
    ) {
        let Event::Pointer(PointerEvent::Move {
            pointer_id,
            pointer_type,
            ..
        }) = event
        else {
            return;
        };

        if !pointer_type_supports_hover(*pointer_type) {
            return;
        }

        let captured_layer_for_pointer_move = self
            .captured
            .get(pointer_id)
            .copied()
            .and_then(|n| self.node_layer(n));
        let pointer_move_occlusion_layer = captured_layer_for_pointer_move
            .is_none()
            .then(|| self.topmost_pointer_occlusion_layer(barrier_root))
            .flatten()
            .filter(|(_, occlusion)| *occlusion != PointerOcclusion::None)
            .map(|(layer, _)| layer);
        let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        let mut hit_barrier = false;
        for layer_id in layers.into_iter().rev() {
            let Some((layer_root, visible, wants_pointer_move_events)) = self
                .layers
                .get(layer_id)
                .map(|layer| (layer.root, layer.visible, layer.wants_pointer_move_events))
            else {
                continue;
            };
            if !visible {
                continue;
            }
            if barrier_root.is_some() && hit_barrier {
                break;
            }
            if !wants_pointer_move_events {
                if barrier_root == Some(layer_root) {
                    hit_barrier = true;
                }
                if pointer_move_occlusion_layer == Some(layer_id) {
                    break;
                }
                continue;
            }
            if captured_layer_for_pointer_move.is_some_and(|layer| layer != layer_id) {
                // Pointer-move observer hooks are used by overlay policies (e.g. Radix menu safe
                // corridor). When a pointer is captured by a different layer (viewport tools,
                // docking drags, etc.), do not let unrelated overlay layers observe that move
                // stream. This keeps captured interactions stable and avoids cross-layer
                // arbitration fights during drags.
                if barrier_root == Some(layer_root) {
                    hit_barrier = true;
                }
                continue;
            }
            if self.dispatch_event_to_node_chain_observer(
                app,
                services,
                input_ctx,
                layer_root,
                event,
                invalidation_visited,
            ) {
                *needs_redraw = true;
            }
            if barrier_root == Some(layer_root) {
                hit_barrier = true;
            }
            if pointer_move_occlusion_layer == Some(layer_id) {
                break;
            }
        }
    }

    fn node_depth_for_invalidation_order(&self, node: NodeId) -> u32 {
        let mut depth: u32 = 0;
        let mut current: Option<NodeId> = Some(node);
        while let Some(id) = current {
            let Some(n) = self.nodes.get(id) else {
                break;
            };
            depth = depth.saturating_add(1);
            current = n.parent;
        }
        depth
    }

    fn apply_pending_invalidations(
        &mut self,
        pending: HashMap<NodeId, PendingInvalidation>,
        visited: &mut HashMap<NodeId, u8>,
    ) {
        if pending.is_empty() {
            return;
        }

        let mut entries: Vec<(NodeId, PendingInvalidation)> = pending.into_iter().collect();
        entries.sort_by_key(|(node, _)| {
            std::cmp::Reverse(self.node_depth_for_invalidation_order(*node))
        });
        for (node, pending) in entries {
            self.mark_invalidation_dedup_with_detail(
                node,
                pending.inv,
                visited,
                pending.source,
                pending.detail,
            );
        }
    }

    fn dismiss_topmost_overlay_on_escape(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        base_root: NodeId,
        barrier_root: Option<NodeId>,
    ) -> bool {
        struct EscapeDismissHookHost<'a, H: crate::UiHost> {
            app: &'a mut H,
            window: AppWindowId,
            element: crate::GlobalElementId,
        }

        impl<H: crate::UiHost> crate::action::UiActionHost for EscapeDismissHookHost<'_, H> {
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
        }

        let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        for layer_id in layers.into_iter().rev() {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if layer.root == base_root {
                continue;
            }

            let Some(root_element) = self.nodes.get(layer.root).and_then(|n| n.element) else {
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
                if barrier_root == Some(layer.root) {
                    break;
                }
                continue;
            };

            let mut host = EscapeDismissHookHost {
                app,
                window,
                element: root_element,
            };
            let mut req =
                crate::action::DismissRequestCx::new(crate::action::DismissReason::Escape);
            hook(
                &mut host,
                crate::action::ActionCx {
                    window,
                    target: root_element,
                },
                &mut req,
            );
            return true;
        }

        false
    }

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
            fret_core::ImeEvent::DeleteSurrounding { .. } => {}
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
        // Focus gating should be resilient to temporarily-broken parent pointers under retained /
        // view-cache-reused subtrees. Use reachability from active layer roots via child edges as
        // the authoritative layer membership check.
        if !self.is_reachable_from_any_root_via_children(requested_focus, active_roots) {
            return false;
        }

        let Some(trap_root) = self.active_trapped_focus_scope_root(app, window) else {
            return true;
        };
        self.is_descendant(trap_root, requested_focus)
    }

    fn is_reachable_from_any_root_via_children(&self, target: NodeId, roots: &[NodeId]) -> bool {
        if roots.is_empty() {
            return false;
        }
        if roots.iter().any(|&root| root == target) {
            return true;
        }

        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<NodeId> = Vec::new();
        for &root in roots {
            if visited.insert(root) {
                stack.push(root);
            }
        }

        while let Some(node) = stack.pop() {
            let Some(entry) = self.nodes.get(node) else {
                continue;
            };
            for &child in &entry.children {
                if child == target {
                    return true;
                }
                if visited.insert(child) {
                    stack.push(child);
                }
            }
        }

        false
    }

    fn dispatch_event_to_node_chain(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        input_ctx: &InputContext,
        start: NodeId,
        event: &Event,
        needs_redraw: &mut bool,
        invalidation_visited: &mut HashMap<NodeId, u8>,
    ) -> bool {
        let pointer_id_for_capture: Option<fret_core::PointerId> = match event {
            Event::Pointer(PointerEvent::Move { pointer_id, .. })
            | Event::Pointer(PointerEvent::Down { pointer_id, .. })
            | Event::Pointer(PointerEvent::Up { pointer_id, .. })
            | Event::Pointer(PointerEvent::Wheel { pointer_id, .. })
            | Event::Pointer(PointerEvent::PinchGesture { pointer_id, .. }) => Some(*pointer_id),
            Event::PointerCancel(e) => Some(e.pointer_id),
            _ => None,
        };

        let mut pending_invalidations = HashMap::<NodeId, PendingInvalidation>::new();
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();

        let (active_roots, _barrier_root) = self.active_input_layers();
        if event_position(event).is_some() {
            let chain = self.build_mapped_event_chain(start, event);
            for (node_id, event_for_node) in chain {
                let (
                    invalidations,
                    requested_focus,
                    requested_capture,
                    notify_requested,
                    notify_requested_location,
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
                        layer_root: tree.node_root(node_id),
                        window: tree.window,
                        pointer_id: pointer_id_for_capture,
                        input_ctx: input_ctx.clone(),
                        prevented_default_actions: &mut prevented_default_actions,
                        children,
                        focus: tree.focus,
                        captured: pointer_id_for_capture
                            .and_then(|p| tree.captured.get(&p).copied()),
                        bounds,
                        invalidations: Vec::new(),
                        requested_focus: None,
                        requested_capture: None,
                        requested_cursor: None,
                        notify_requested: false,
                        notify_requested_location: None,
                        stop_propagation: false,
                    };
                    widget.event(&mut cx, &event_for_node);
                    (
                        cx.invalidations,
                        cx.requested_focus,
                        cx.requested_capture,
                        cx.notify_requested,
                        cx.notify_requested_location,
                        cx.stop_propagation,
                    )
                });

                if !invalidations.is_empty()
                    || requested_focus.is_some()
                    || requested_capture.is_some()
                    || notify_requested
                {
                    *needs_redraw = true;
                }

                for (id, inv) in invalidations {
                    Self::pending_invalidation_merge(
                        &mut pending_invalidations,
                        id,
                        inv,
                        UiDebugInvalidationSource::Other,
                        UiDebugInvalidationDetail::Unknown,
                    );
                }

                if notify_requested {
                    self.debug_record_notify_request(
                        app.frame_id(),
                        node_id,
                        notify_requested_location,
                    );
                    Self::pending_invalidation_merge(
                        &mut pending_invalidations,
                        node_id,
                        Invalidation::Paint,
                        UiDebugInvalidationSource::Notify,
                        UiDebugInvalidationDetail::from_source(UiDebugInvalidationSource::Notify),
                    );
                    *needs_redraw = true;
                }

                if let Some(focus) = requested_focus
                    && self.focus_request_is_allowed(app, self.window, &active_roots, focus)
                {
                    if let Some(prev) = self.focus {
                        Self::pending_invalidation_merge(
                            &mut pending_invalidations,
                            prev,
                            Invalidation::Paint,
                            UiDebugInvalidationSource::Focus,
                            UiDebugInvalidationDetail::from_source(
                                UiDebugInvalidationSource::Focus,
                            ),
                        );
                    }
                    self.focus = Some(focus);
                    Self::pending_invalidation_merge(
                        &mut pending_invalidations,
                        focus,
                        Invalidation::Paint,
                        UiDebugInvalidationSource::Focus,
                        UiDebugInvalidationDetail::from_source(UiDebugInvalidationSource::Focus),
                    );
                    self.scroll_node_into_view(app, focus);
                }

                if let Some(capture) = requested_capture
                    && capture.is_none_or(|n| self.node_in_any_layer(n, &active_roots))
                {
                    if let Some(pointer_id) = pointer_id_for_capture {
                        match capture {
                            Some(node) => {
                                self.captured.insert(pointer_id, node);
                            }
                            None => {
                                self.captured.remove(&pointer_id);
                            }
                        }
                    }
                }

                let captured_now =
                    pointer_id_for_capture.and_then(|p| self.captured.get(&p).copied());
                if captured_now.is_some() || stop_propagation {
                    self.apply_pending_invalidations(
                        std::mem::take(&mut pending_invalidations),
                        invalidation_visited,
                    );
                    return true;
                }
            }
            self.apply_pending_invalidations(
                std::mem::take(&mut pending_invalidations),
                invalidation_visited,
            );
            return false;
        }

        let mut node_id = start;
        loop {
            let (
                invalidations,
                requested_focus,
                requested_capture,
                notify_requested,
                notify_requested_location,
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
                    layer_root: tree.node_root(node_id),
                    window: tree.window,
                    pointer_id: pointer_id_for_capture,
                    input_ctx: input_ctx.clone(),
                    prevented_default_actions: &mut prevented_default_actions,
                    children,
                    focus: tree.focus,
                    captured: pointer_id_for_capture.and_then(|p| tree.captured.get(&p).copied()),
                    bounds,
                    invalidations: Vec::new(),
                    requested_focus: None,
                    requested_capture: None,
                    requested_cursor: None,
                    notify_requested: false,
                    notify_requested_location: None,
                    stop_propagation: false,
                };
                widget.event(&mut cx, event);
                (
                    cx.invalidations,
                    cx.requested_focus,
                    cx.requested_capture,
                    cx.notify_requested,
                    cx.notify_requested_location,
                    cx.stop_propagation,
                    parent,
                )
            });

            if !invalidations.is_empty()
                || requested_focus.is_some()
                || requested_capture.is_some()
                || notify_requested
            {
                *needs_redraw = true;
            }

            for (id, inv) in invalidations {
                Self::pending_invalidation_merge(
                    &mut pending_invalidations,
                    id,
                    inv,
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::Unknown,
                );
            }

            if notify_requested {
                self.debug_record_notify_request(
                    app.frame_id(),
                    node_id,
                    notify_requested_location,
                );
                Self::pending_invalidation_merge(
                    &mut pending_invalidations,
                    node_id,
                    Invalidation::Paint,
                    UiDebugInvalidationSource::Notify,
                    UiDebugInvalidationDetail::from_source(UiDebugInvalidationSource::Notify),
                );
                *needs_redraw = true;
            }

            if let Some(focus) = requested_focus
                && self.focus_request_is_allowed(app, self.window, &active_roots, focus)
            {
                if let Some(prev) = self.focus {
                    Self::pending_invalidation_merge(
                        &mut pending_invalidations,
                        prev,
                        Invalidation::Paint,
                        UiDebugInvalidationSource::Focus,
                        UiDebugInvalidationDetail::from_source(UiDebugInvalidationSource::Focus),
                    );
                }
                self.focus = Some(focus);
                Self::pending_invalidation_merge(
                    &mut pending_invalidations,
                    focus,
                    Invalidation::Paint,
                    UiDebugInvalidationSource::Focus,
                    UiDebugInvalidationDetail::from_source(UiDebugInvalidationSource::Focus),
                );
                self.scroll_node_into_view(app, focus);
            }

            if let Some(capture) = requested_capture
                && capture.is_none_or(|n| self.node_in_any_layer(n, &active_roots))
            {
                if let Some(pointer_id) = pointer_id_for_capture {
                    match capture {
                        Some(node) => {
                            self.captured.insert(pointer_id, node);
                        }
                        None => {
                            self.captured.remove(&pointer_id);
                        }
                    }
                }
            }

            let captured_now = pointer_id_for_capture.and_then(|p| self.captured.get(&p).copied());
            if captured_now.is_some() || stop_propagation {
                self.apply_pending_invalidations(
                    std::mem::take(&mut pending_invalidations),
                    invalidation_visited,
                );
                return true;
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }

        self.apply_pending_invalidations(
            std::mem::take(&mut pending_invalidations),
            invalidation_visited,
        );
        false
    }

    #[stacksafe::stacksafe]
    pub fn dispatch_event(&mut self, app: &mut H, services: &mut dyn UiServices, event: &Event) {
        let Some(base_root) = self
            .base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
        else {
            return;
        };

        self.begin_debug_frame_if_needed(app.frame_id());

        // Keep wheel routing and hover detection in sync with out-of-band scroll handle mutations
        // (e.g. forwarded wheel handlers) by applying scroll-handle-driven invalidations before
        // hit-testing.
        if matches!(event, Event::Pointer(_)) {
            self.invalidate_scroll_handle_bindings_for_changed_handles(
                app,
                crate::layout_pass::LayoutPassKind::Final,
                false,
                false,
            );
        }

        let is_wheel = matches!(event, Event::Pointer(PointerEvent::Wheel { .. }));

        let (active_layers, barrier_root) = self.active_input_layers();
        self.enforce_modal_barrier_scope(&active_layers);

        // If the topmost barrier is a hit-test-inert pointer occlusion layer (e.g. Radix
        // `disableOutsidePointerEvents`), allow wheel events to route to the underlay scroll target.
        //
        // Modal barriers must continue to block wheel events while present.
        let wheel_hit_test_layers: Option<Vec<NodeId>> = (is_wheel
            && barrier_root.is_some_and(|barrier_root| {
                self.root_to_layer
                    .get(&barrier_root)
                    .copied()
                    .and_then(|layer| self.layers.get(layer))
                    .is_some_and(|layer| !layer.hit_testable)
            }))
        .then(|| {
            let visible: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
            let mut roots: Vec<NodeId> = Vec::new();
            for layer_id in visible.into_iter().rev() {
                let layer = &self.layers[layer_id];
                if layer.hit_testable {
                    roots.push(layer.root);
                }
            }
            roots
        });
        let hit_test_layer_roots: &[NodeId] = wheel_hit_test_layers
            .as_deref()
            .unwrap_or(active_layers.as_slice());

        let to_remove: Vec<fret_core::PointerId> = self
            .captured
            .iter()
            .filter_map(|(p, n)| (!self.node_in_any_layer(*n, &active_layers)).then_some(*p))
            .collect();
        for p in to_remove {
            self.captured.remove(&p);
        }
        if self
            .focus
            .is_some_and(|n| !self.node_in_any_layer(n, &active_layers))
        {
            self.focus = None;
        }

        let focus_is_text_input = self.focus_is_text_input(app);
        self.update_ime_composing_for_event(focus_is_text_input, event);
        self.set_ime_allowed(app, focus_is_text_input);

        let caps = app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let mut input_ctx = InputContext {
            platform: Platform::current(),
            caps,
            ui_has_modal: barrier_root.is_some(),
            window_arbitration: None,
            focus_is_text_input,
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
            if let Some(mode) = self.focus_text_boundary_mode_override() {
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

            app.with_global_mut(
                fret_runtime::WindowInputContextService::default,
                |svc, _app| {
                    svc.set_snapshot(window, input_ctx.clone());
                },
            );
        }

        let mut invalidation_visited = HashMap::<NodeId, u8>::new();
        let mut needs_redraw = false;

        // ADR 0012: when a text input is focused, reserve common IME/navigation keys for the
        // text/IME path first, and only fall back to shortcut matching if the widget doesn't
        // consume the event.
        let defer_keydown_shortcuts_until_after_dispatch =
            self.pending_shortcut.keystrokes.is_empty()
                && !self.replaying_pending_shortcut
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
                    self.mark_invalidation_dedup_with_detail(
                        focus,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Other,
                        UiDebugInvalidationDetail::FocusVisiblePolicy,
                    );
                } else {
                    self.mark_invalidation_dedup_with_detail(
                        base_root,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Other,
                        UiDebugInvalidationDetail::FocusVisiblePolicy,
                    );
                }
                self.request_redraw_coalesced(app);
            }

            let changed = crate::input_modality::update_for_event(app, window, event);
            if changed {
                if let Some(focus) = self.focus {
                    self.mark_invalidation_dedup_with_detail(
                        focus,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Other,
                        UiDebugInvalidationDetail::InputModalityPolicy,
                    );
                } else {
                    self.mark_invalidation_dedup_with_detail(
                        base_root,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Other,
                        UiDebugInvalidationDetail::InputModalityPolicy,
                    );
                }
                self.request_redraw_coalesced(app);
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
            self.sync_pending_shortcut_overlay_state(app, None);
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
                let stopped = self.dispatch_event_to_node_chain(
                    app,
                    services,
                    &input_ctx,
                    node,
                    event,
                    &mut needs_redraw,
                    &mut invalidation_visited,
                );
                if stopped {
                    if needs_redraw {
                        self.request_redraw_coalesced(app);
                    }
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
                let stopped = self.dispatch_event_to_node_chain(
                    app,
                    services,
                    &input_ctx,
                    layer.root,
                    event,
                    &mut needs_redraw,
                    &mut invalidation_visited,
                );
                if stopped {
                    if needs_redraw {
                        self.request_redraw_coalesced(app);
                    }
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

        let mut cursor_choice: Option<fret_core::CursorIcon> = None;
        let mut cursor_choice_from_query = false;
        let mut cursor_query_choice: Option<fret_core::CursorIcon> = None;
        let mut stop_propagation_requested = false;
        let mut pointer_down_outside = PointerDownOutsideOutcome::default();
        let mut suppress_touch_up_outside_dispatch = false;
        let mut suppress_pointer_dispatch = false;
        let is_scroll_like = Self::event_is_scroll_like(event);
        let mut wheel_stop_node: Option<NodeId> = None;
        let mut synth_pointer_move_prev_target: Option<NodeId> = None;
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut focus_requested = false;

        if let Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            repeat: false,
            ..
        } = event
            && let Some(window) = self.window
            && {
                let dock_drag_affects_window = app.any_drag_session(|d| {
                    d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                        && (d.source_window == window || d.current_window == window)
                });
                if dock_drag_affects_window {
                    // ADR 0072: Escape cancels the active dock drag session, and must not be
                    // routed to overlays while the drag is in progress.
                    let canceled = app.cancel_drag_sessions(|d| {
                        d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                            && (d.source_window == window || d.current_window == window)
                    });
                    for pointer_id in canceled {
                        self.captured.remove(&pointer_id);
                    }
                    true
                } else {
                    self.dismiss_topmost_overlay_on_escape(app, window, base_root, barrier_root)
                }
            }
        {
            self.request_redraw_coalesced(app);
            return;
        }

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
        let event_pointer_id_for_capture: Option<fret_core::PointerId> = match event {
            Event::Pointer(PointerEvent::Move { pointer_id, .. })
            | Event::Pointer(PointerEvent::Down { pointer_id, .. })
            | Event::Pointer(PointerEvent::Up { pointer_id, .. })
            | Event::Pointer(PointerEvent::Wheel { pointer_id, .. })
            | Event::Pointer(PointerEvent::PinchGesture { pointer_id, .. }) => Some(*pointer_id),
            Event::PointerCancel(e) => Some(e.pointer_id),
            _ => None,
        };

        let captured = event_pointer_id_for_capture.and_then(|p| self.captured.get(&p).copied());
        if let Event::Pointer(PointerEvent::Move {
            pointer_id,
            position,
            pointer_type: fret_core::PointerType::Touch,
            ..
        }) = event
        {
            self.update_touch_pointer_down_outside_move(*pointer_id, *position);
        }
        let (dock_drag_affects_window, dock_drag_capture_anchor) = self
            .window
            .map(|window| {
                let affects = app.any_drag_session(|d| {
                    d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                        && (d.source_window == window || d.current_window == window)
                });
                let anchor =
                    crate::internal_drag::route(&*app, window, fret_runtime::DRAG_KIND_DOCK_PANEL);
                (affects, anchor)
            })
            .unwrap_or((false, None));

        // Internal drag overrides may need to route events to a stable "anchor" node, even if
        // hit-testing fails or the cursor is over an unrelated widget (e.g. docking tear-off).
        let internal_drag_target = (|| {
            let Event::InternalDrag(e) = event else {
                return None;
            };
            let window = self.window?;
            let drag = app.drag(e.pointer_id)?;
            if !drag.cross_window_hover {
                return None;
            }
            let target = crate::internal_drag::route(app, window, drag.kind)?;
            self.node_in_any_layer(target, &active_layers)
                .then_some(target)
        })();

        if let Some(window) = self.window
            && matches!(event, Event::Pointer(_))
            && let Some(pos) = event_position(event)
        {
            // Hit-testing is performance-sensitive (especially for pointer move), but must remain
            // correct across discrete interactions like clicks where the pointer position can jump
            // substantially between events.
            //
            // For now, allow cached hit-test reuse only for high-frequency, cursor-driven event
            // streams. Discrete interactions (e.g. clicks) still rebuild the cache from a full
            // hit-test pass.
            let hit = if event_allows_hit_test_path_cache_reuse(event) {
                self.hit_test_layers_cached(hit_test_layer_roots, pos)
            } else {
                self.hit_test_path_cache = None;
                self.hit_test_layers_cached(hit_test_layer_roots, pos)
            };

            if let Event::Pointer(PointerEvent::Up {
                pointer_id,
                pointer_type: fret_core::PointerType::Touch,
                ..
            }) = event
                && captured.is_none()
            {
                if dock_drag_affects_window {
                    self.touch_pointer_down_outside_candidates
                        .remove(pointer_id);
                } else if let Some(candidate) = self
                    .touch_pointer_down_outside_candidates
                    .remove(pointer_id)
                {
                    if let Some(layer) = self.layers.get(candidate.layer_id) {
                        let foreign_capture_active = self.captured.iter().any(|(pid, node)| {
                            *pid != *pointer_id
                                && self
                                    .node_layer(*node)
                                    .is_some_and(|layer_id| layer_id != candidate.layer_id)
                        });

                        if !foreign_capture_active && !candidate.moved {
                            let hit_root = hit.and_then(|n| self.node_root(n));
                            let hit_is_inside_layer = hit_root == Some(layer.root);
                            let hit_is_inside_branch = hit.is_some_and(|hit| {
                                layer
                                    .pointer_down_outside_branches
                                    .iter()
                                    .copied()
                                    .any(|branch| self.is_descendant(branch, hit))
                            });

                            if !hit_is_inside_layer && !hit_is_inside_branch {
                                self.dispatch_event_to_node_chain_observer(
                                    app,
                                    services,
                                    &input_ctx,
                                    candidate.root,
                                    &candidate.down_event,
                                    &mut invalidation_visited,
                                );
                                needs_redraw = true;
                                suppress_touch_up_outside_dispatch = candidate.consume;
                            }
                        }
                    }
                }
            }

            // Pointer occlusion is a window-level layer substrate mechanism (policy-owned).
            //
            // When active, the runtime must:
            // - suppress hover state for underlay layers (even when scroll is allowed),
            // - optionally suppress hit-tested pointer dispatch for underlay layers depending on
            //   the occlusion mode.
            let mut hit_for_hover = hit;
            if captured.is_none()
                && let Some((occlusion_layer, occlusion)) =
                    self.topmost_pointer_occlusion_layer(barrier_root)
                && occlusion != PointerOcclusion::None
            {
                let occlusion_z = self
                    .layer_order
                    .iter()
                    .position(|id| *id == occlusion_layer);
                let hit_layer_z = hit
                    .and_then(|hit| self.node_layer(hit))
                    .and_then(|layer| self.layer_order.iter().position(|id| *id == layer));

                let hit_is_below_occlusion = match (occlusion_z, hit_layer_z, hit) {
                    (Some(oz), Some(hz), Some(_)) => hz < oz,
                    (Some(_), None, Some(_)) => true,
                    (Some(_), _, None) => true,
                    _ => false,
                };

                if hit_is_below_occlusion {
                    // Match GPUI-style "occluded hover": underlay hover/pressable detection is
                    // disabled while occlusion is active, even when scroll is still allowed.
                    hit_for_hover = None;

                    let blocks_pointer_dispatch = match occlusion {
                        PointerOcclusion::None => false,
                        PointerOcclusion::BlockMouse => true,
                        PointerOcclusion::BlockMouseExceptScroll => !is_scroll_like,
                    };
                    if blocks_pointer_dispatch {
                        suppress_pointer_dispatch = true;
                    }
                }
            }

            if input_ctx.caps.ui.cursor_icons
                && cursor_query_choice.is_none()
                && matches!(event, Event::Pointer(PointerEvent::Move { .. }))
            {
                let mut node = captured.or(hit_for_hover);
                while let Some(id) = node {
                    let (bounds, parent) = self
                        .nodes
                        .get(id)
                        .map(|n| (n.bounds, n.parent))
                        .unwrap_or_default();
                    if let Some(icon) = self.with_widget_mut(id, |widget, _tree| {
                        widget.cursor_icon_at(bounds, pos, &input_ctx)
                    }) {
                        cursor_query_choice = Some(icon);
                        break;
                    }
                    node = parent;
                }
            }

            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) && captured.is_none() {
                if dock_drag_affects_window {
                    // ADR 0072: while a dock drag session is active, outside-press dismissal must
                    // not trigger. The drag owns input arbitration for the window.
                    //
                    // This is intentionally window-global (not pointer-local): a dock drag session
                    // is exclusive for the window, and we do not want secondary pointers to dismiss
                    // overlays or change focus while the drag is in progress.
                    //
                    // Note: overlay policy is expected to close/suspend non-modal overlays when a
                    // dock drag starts; this suppression makes the routing rule durable even if a
                    // layer remains mounted for a close transition.
                    pointer_down_outside = PointerDownOutsideOutcome::default();
                } else {
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
                        &mut invalidation_visited,
                    );
                    if pointer_down_outside.dispatched {
                        needs_redraw = true;
                    }
                }
            }

            let hover_capable = match event {
                Event::Pointer(PointerEvent::Move { pointer_type, .. })
                | Event::Pointer(PointerEvent::Down { pointer_type, .. })
                | Event::Pointer(PointerEvent::Up { pointer_type, .. })
                | Event::Pointer(PointerEvent::Wheel { pointer_type, .. })
                | Event::Pointer(PointerEvent::PinchGesture { pointer_type, .. }) => {
                    pointer_type_supports_hover(*pointer_type)
                }
                _ => false,
            };

            if hover_capable {
                self.update_hover_state_from_hit(
                    app,
                    window,
                    hit_for_hover,
                    hit,
                    &mut invalidation_visited,
                    &mut needs_redraw,
                );
            }
        }

        let mut pointer_hit: Option<NodeId> = None;
        let target = if let Some(captured) = captured {
            Some(captured)
        } else if let Some(target) = internal_drag_target {
            Some(target)
        } else if let Some(pos) = event_position(event) {
            // See the cached hit-test reuse note above.
            let hit = if event_allows_hit_test_path_cache_reuse(event) {
                self.hit_test_layers_cached(hit_test_layer_roots, pos)
            } else {
                self.hit_test_path_cache = None;
                self.hit_test_layers_cached(hit_test_layer_roots, pos)
            };

            let hit = if matches!(event, Event::InternalDrag(_)) {
                (|| {
                    let window = self.window?;
                    crate::declarative::with_window_frame(app, window, |window_frame| {
                        let window_frame = window_frame?;
                        let mut node = hit?;
                        loop {
                            if let Some(record) = window_frame.instances.get(&node)
                                && matches!(
                                    record.instance,
                                    crate::declarative::ElementInstance::InternalDragRegion(p)
                                        if p.enabled
                                )
                            {
                                return Some(node);
                            }
                            node = self.nodes.get(node).and_then(|n| n.parent)?;
                        }
                    })
                })()
                .or(hit)
            } else {
                hit
            };
            pointer_hit = hit;

            if let Event::Pointer(PointerEvent::Move {
                buttons,
                pointer_id,
                ..
            }) = event
                && !buttons.left
                && !buttons.right
                && !buttons.middle
            {
                // When a modal barrier becomes active, the previous pointer-move hit may belong to
                // an underlay layer that is now inactive. Do not synthesize hover-move events into
                // the underlay in that case (e.g. Radix `disableOutsidePointerEvents`).
                let mut last_pointer_move_hit = self
                    .last_pointer_move_hit
                    .get(pointer_id)
                    .copied()
                    .flatten();
                if barrier_root.is_some()
                    && last_pointer_move_hit
                        .is_some_and(|n| !self.node_in_any_layer(n, &active_layers))
                {
                    self.last_pointer_move_hit.remove(pointer_id);
                    last_pointer_move_hit = None;
                }

                if hit != last_pointer_move_hit {
                    synth_pointer_move_prev_target = last_pointer_move_hit;
                    match hit {
                        Some(hit) => {
                            self.last_pointer_move_hit.insert(*pointer_id, Some(hit));
                        }
                        None => {
                            self.last_pointer_move_hit.remove(pointer_id);
                        }
                    }
                }
            }

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
            match event {
                Event::SetTextSelection { .. } => {
                    let selection_node = self.window.and_then(|window| {
                        crate::elements::with_window_state(app, window, |window_state| {
                            window_state
                                .active_text_selection()
                                .and_then(|selection| window_state.node_entry(selection.element))
                                .map(|entry| entry.node)
                        })
                    });
                    selection_node.or(self.focus).or(Some(default_root))
                }
                _ => self.focus.or(Some(default_root)),
            }
        };

        let Some(mut node_id) = target else {
            return;
        };

        if matches!(event, Event::Pointer(PointerEvent::Down { .. }))
            && pointer_down_outside.suppress_hit_test_dispatch
        {
            if needs_redraw {
                self.request_redraw_coalesced(app);
            }
            return;
        }

        if matches!(event, Event::Pointer(PointerEvent::Up { .. }))
            && suppress_touch_up_outside_dispatch
        {
            if needs_redraw {
                self.request_redraw_coalesced(app);
            }
            return;
        }

        if suppress_pointer_dispatch && matches!(event, Event::Pointer(_)) {
            if matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
                self.dispatch_pointer_move_layer_observers(
                    app,
                    services,
                    &input_ctx,
                    barrier_root,
                    event,
                    &mut needs_redraw,
                    &mut invalidation_visited,
                );
            }
            if needs_redraw {
                self.request_redraw_coalesced(app);
            }
            return;
        }

        if cursor_choice.is_none()
            && input_ctx.caps.ui.cursor_icons
            && matches!(event, Event::Pointer(_))
            && let Some(hit) = pointer_hit
        {
            cursor_choice = self.cursor_icon_query_for_pointer_hit(hit, &input_ctx, event);
            cursor_choice_from_query = cursor_choice.is_some();
        }

        if !suppress_pointer_dispatch
            && matches!(
                event,
                Event::Pointer(_)
                    | Event::PointerCancel(_)
                    | Event::ExternalDrag(_)
                    | Event::InternalDrag(_)
            )
        {
            let chain = if event_position(event).is_some() {
                self.build_mapped_event_chain(node_id, event)
            } else {
                self.build_unmapped_event_chain(node_id, event)
            };
            let should_run_capture_phase = match event {
                Event::Pointer(PointerEvent::Down { .. })
                | Event::Pointer(PointerEvent::Up { .. })
                | Event::Pointer(PointerEvent::Wheel { .. })
                | Event::Pointer(PointerEvent::PinchGesture { .. })
                | Event::PointerCancel(..) => true,
                Event::Pointer(PointerEvent::Move { buttons, .. }) => {
                    captured.is_some() || buttons.left || buttons.right || buttons.middle
                }
                _ => false,
            };
            let mut stopped_in_capture = false;
            if should_run_capture_phase {
                let mut capture_ctx = input_ctx.clone();
                capture_ctx.dispatch_phase = InputDispatchPhase::Capture;

                for (node_id, event_for_node) in chain.iter().rev() {
                    let node_id = *node_id;
                    let (
                        invalidations,
                        requested_focus,
                        requested_capture,
                        requested_cursor,
                        notify_requested,
                        notify_requested_location,
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
                            layer_root: tree.node_root(node_id),
                            window: tree.window,
                            pointer_id: event_pointer_id_for_capture,
                            input_ctx: capture_ctx.clone(),
                            prevented_default_actions: &mut prevented_default_actions,
                            children,
                            focus: tree.focus,
                            captured: event_pointer_id_for_capture
                                .and_then(|p| tree.captured.get(&p).copied()),
                            bounds,
                            invalidations: Vec::new(),
                            requested_focus: None,
                            requested_capture: None,
                            requested_cursor: None,
                            notify_requested: false,
                            notify_requested_location: None,
                            stop_propagation: false,
                        };
                        widget.event_capture(&mut cx, event_for_node);
                        (
                            cx.invalidations,
                            cx.requested_focus,
                            cx.requested_capture,
                            cx.requested_cursor,
                            cx.notify_requested,
                            cx.notify_requested_location,
                            cx.stop_propagation,
                        )
                    });

                    if !invalidations.is_empty()
                        || requested_focus.is_some()
                        || requested_capture.is_some()
                        || notify_requested
                    {
                        needs_redraw = true;
                    }

                    for (id, inv) in invalidations {
                        self.mark_invalidation(id, inv);
                    }
                    if notify_requested {
                        self.debug_record_notify_request(
                            app.frame_id(),
                            node_id,
                            notify_requested_location,
                        );
                        self.mark_invalidation_with_source(
                            node_id,
                            Invalidation::Paint,
                            UiDebugInvalidationSource::Notify,
                        );
                    }

                    if let Some(focus) = requested_focus
                        && self.focus_request_is_allowed(app, self.window, &active_layers, focus)
                    {
                        focus_requested = true;
                        if let Some(prev) = self.focus {
                            self.mark_invalidation(prev, Invalidation::Paint);
                        }
                        self.focus = Some(focus);
                        self.mark_invalidation(focus, Invalidation::Paint);
                        self.scroll_node_into_view(app, focus);
                    } else if requested_focus.is_some() {
                        focus_requested = true;
                    }

                    if let Some(capture) = requested_capture {
                        if let Some(pointer_id) = event_pointer_id_for_capture {
                            match capture {
                                Some(node) => {
                                    let allow = !dock_drag_affects_window
                                        || dock_drag_capture_anchor == Some(node);
                                    if allow {
                                        self.captured.insert(pointer_id, node);
                                    }
                                }
                                None => {
                                    self.captured.remove(&pointer_id);
                                }
                            }
                        }
                    }

                    if let Some(requested_cursor) = requested_cursor
                        && (cursor_choice.is_none() || cursor_choice_from_query)
                    {
                        cursor_choice = Some(requested_cursor);
                        cursor_choice_from_query = false;
                    }

                    if stop_propagation {
                        stop_propagation_requested = true;
                        if is_wheel && wheel_stop_node.is_none() {
                            wheel_stop_node = Some(node_id);
                        }
                        stopped_in_capture = true;
                        break;
                    }
                }
            }

            if !stopped_in_capture {
                let mut bubble_ctx = input_ctx.clone();
                bubble_ctx.dispatch_phase = InputDispatchPhase::Bubble;

                for (node_id, event_for_node) in chain {
                    let (
                        invalidations,
                        requested_focus,
                        requested_capture,
                        requested_cursor,
                        notify_requested,
                        notify_requested_location,
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
                            layer_root: tree.node_root(node_id),
                            window: tree.window,
                            pointer_id: event_pointer_id_for_capture,
                            input_ctx: bubble_ctx.clone(),
                            prevented_default_actions: &mut prevented_default_actions,
                            children,
                            focus: tree.focus,
                            captured: event_pointer_id_for_capture
                                .and_then(|p| tree.captured.get(&p).copied()),
                            bounds,
                            invalidations: Vec::new(),
                            requested_focus: None,
                            requested_capture: None,
                            requested_cursor: None,
                            notify_requested: false,
                            notify_requested_location: None,
                            stop_propagation: false,
                        };
                        widget.event(&mut cx, &event_for_node);
                        if cx.requested_cursor.is_none()
                            && matches!(event_for_node, Event::Pointer(_))
                            && cx.input_ctx.caps.ui.cursor_icons
                            && let Some(position) = event_position(&event_for_node)
                        {
                            cx.requested_cursor =
                                widget.cursor_icon_at(bounds, position, &cx.input_ctx);
                        }
                        (
                            cx.invalidations,
                            cx.requested_focus,
                            cx.requested_capture,
                            cx.requested_cursor,
                            cx.notify_requested,
                            cx.notify_requested_location,
                            cx.stop_propagation,
                        )
                    });

                    if !invalidations.is_empty()
                        || requested_focus.is_some()
                        || requested_capture.is_some()
                        || notify_requested
                    {
                        needs_redraw = true;
                    }

                    for (id, inv) in invalidations {
                        self.mark_invalidation(id, inv);
                    }
                    if notify_requested {
                        self.debug_record_notify_request(
                            app.frame_id(),
                            node_id,
                            notify_requested_location,
                        );
                        self.mark_invalidation_with_source(
                            node_id,
                            Invalidation::Paint,
                            UiDebugInvalidationSource::Notify,
                        );
                    }

                    if let Some(focus) = requested_focus
                        && self.focus_request_is_allowed(app, self.window, &active_layers, focus)
                    {
                        focus_requested = true;
                        if let Some(prev) = self.focus {
                            self.mark_invalidation(prev, Invalidation::Paint);
                        }
                        self.focus = Some(focus);
                        self.mark_invalidation(focus, Invalidation::Paint);
                        self.scroll_node_into_view(app, focus);
                    } else if requested_focus.is_some() {
                        focus_requested = true;
                    }

                    if let Some(capture) = requested_capture {
                        if let Some(pointer_id) = event_pointer_id_for_capture {
                            match capture {
                                Some(node) => {
                                    let allow = !dock_drag_affects_window
                                        || dock_drag_capture_anchor == Some(node);
                                    if allow {
                                        self.captured.insert(pointer_id, node);
                                    }
                                }
                                None => {
                                    self.captured.remove(&pointer_id);
                                }
                            }
                        }
                    }

                    if let Some(requested_cursor) = requested_cursor
                        && (cursor_choice.is_none() || cursor_choice_from_query)
                    {
                        cursor_choice = Some(requested_cursor);
                        cursor_choice_from_query = false;
                    }

                    if stop_propagation {
                        stop_propagation_requested = true;
                        if is_wheel && wheel_stop_node.is_none() {
                            wheel_stop_node = Some(node_id);
                        }
                    }

                    let captured_now =
                        event_pointer_id_for_capture.and_then(|p| self.captured.get(&p).copied());
                    if captured_now.is_some() || stop_propagation {
                        break;
                    }
                }
            }
        } else {
            if matches!(event, Event::KeyDown { .. } | Event::KeyUp { .. }) {
                let mut chain: Vec<NodeId> = Vec::new();
                let mut cur = Some(node_id);
                while let Some(id) = cur {
                    chain.push(id);
                    cur = self.nodes.get(id).and_then(|n| n.parent);
                }

                let mut stopped_in_capture = false;
                {
                    let mut capture_ctx = input_ctx.clone();
                    capture_ctx.dispatch_phase = InputDispatchPhase::Capture;

                    for &node_id in chain.iter().rev() {
                        let (
                            invalidations,
                            requested_focus,
                            requested_capture,
                            requested_cursor,
                            notify_requested,
                            notify_requested_location,
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
                                layer_root: tree.node_root(node_id),
                                window: tree.window,
                                pointer_id: event_pointer_id_for_capture,
                                input_ctx: capture_ctx.clone(),
                                prevented_default_actions: &mut prevented_default_actions,
                                children,
                                focus: tree.focus,
                                captured: event_pointer_id_for_capture
                                    .and_then(|p| tree.captured.get(&p).copied()),
                                bounds,
                                invalidations: Vec::new(),
                                requested_focus: None,
                                requested_capture: None,
                                requested_cursor: None,
                                notify_requested: false,
                                notify_requested_location: None,
                                stop_propagation: false,
                            };
                            widget.event_capture(&mut cx, event);
                            (
                                cx.invalidations,
                                cx.requested_focus,
                                cx.requested_capture,
                                cx.requested_cursor,
                                cx.notify_requested,
                                cx.notify_requested_location,
                                cx.stop_propagation,
                            )
                        });

                        if !invalidations.is_empty()
                            || requested_focus.is_some()
                            || requested_capture.is_some()
                            || notify_requested
                        {
                            needs_redraw = true;
                        }

                        for (id, inv) in invalidations {
                            self.mark_invalidation(id, inv);
                        }
                        if notify_requested {
                            self.debug_record_notify_request(
                                app.frame_id(),
                                node_id,
                                notify_requested_location,
                            );
                            self.mark_invalidation_with_source(
                                node_id,
                                Invalidation::Paint,
                                UiDebugInvalidationSource::Notify,
                            );
                        }

                        if let Some(focus) = requested_focus
                            && self.focus_request_is_allowed(
                                app,
                                self.window,
                                &active_layers,
                                focus,
                            )
                        {
                            focus_requested = true;
                            if let Some(prev) = self.focus {
                                self.mark_invalidation(prev, Invalidation::Paint);
                            }
                            self.focus = Some(focus);
                            self.mark_invalidation(focus, Invalidation::Paint);
                            self.scroll_node_into_view(app, focus);
                        } else if requested_focus.is_some() {
                            focus_requested = true;
                        }

                        if let Some(capture) = requested_capture {
                            if let Some(pointer_id) = event_pointer_id_for_capture {
                                match capture {
                                    Some(node) => {
                                        let allow = !dock_drag_affects_window
                                            || dock_drag_capture_anchor == Some(node);
                                        if allow {
                                            self.captured.insert(pointer_id, node);
                                        }
                                    }
                                    None => {
                                        self.captured.remove(&pointer_id);
                                    }
                                }
                            }
                        }

                        if requested_cursor.is_some() && cursor_choice.is_none() {
                            cursor_choice = requested_cursor;
                        }

                        if stop_propagation {
                            stop_propagation_requested = true;
                            stopped_in_capture = true;
                            break;
                        }
                    }
                }
                if !stopped_in_capture {
                    let mut bubble_ctx = input_ctx.clone();
                    bubble_ctx.dispatch_phase = InputDispatchPhase::Bubble;

                    for node_id in chain {
                        let (
                            invalidations,
                            requested_focus,
                            requested_capture,
                            requested_cursor,
                            notify_requested,
                            notify_requested_location,
                            stop_propagation,
                        ) = self.with_widget_mut(node_id, |widget, tree| {
                            let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                            let _ = parent;
                            let (children, bounds) = tree
                                .nodes
                                .get(node_id)
                                .map(|n| (n.children.as_slice(), n.bounds))
                                .unwrap_or((&[][..], Rect::default()));
                            let mut cx = EventCx {
                                app,
                                services: &mut *services,
                                node: node_id,
                                layer_root: tree.node_root(node_id),
                                window: tree.window,
                                pointer_id: event_pointer_id_for_capture,
                                input_ctx: bubble_ctx.clone(),
                                prevented_default_actions: &mut prevented_default_actions,
                                children,
                                focus: tree.focus,
                                captured: event_pointer_id_for_capture
                                    .and_then(|p| tree.captured.get(&p).copied()),
                                bounds,
                                invalidations: Vec::new(),
                                requested_focus: None,
                                requested_capture: None,
                                requested_cursor: None,
                                notify_requested: false,
                                notify_requested_location: None,
                                stop_propagation: false,
                            };
                            widget.event(&mut cx, event);
                            (
                                cx.invalidations,
                                cx.requested_focus,
                                cx.requested_capture,
                                cx.requested_cursor,
                                cx.notify_requested,
                                cx.notify_requested_location,
                                cx.stop_propagation,
                            )
                        });

                        if !invalidations.is_empty()
                            || requested_focus.is_some()
                            || requested_capture.is_some()
                            || notify_requested
                        {
                            needs_redraw = true;
                        }

                        for (id, inv) in invalidations {
                            self.mark_invalidation(id, inv);
                        }
                        if notify_requested {
                            self.debug_record_notify_request(
                                app.frame_id(),
                                node_id,
                                notify_requested_location,
                            );
                            self.mark_invalidation_with_source(
                                node_id,
                                Invalidation::Paint,
                                UiDebugInvalidationSource::Notify,
                            );
                        }

                        if let Some(focus) = requested_focus
                            && self.focus_request_is_allowed(
                                app,
                                self.window,
                                &active_layers,
                                focus,
                            )
                        {
                            focus_requested = true;
                            if let Some(prev) = self.focus {
                                self.mark_invalidation(prev, Invalidation::Paint);
                            }
                            self.focus = Some(focus);
                            self.mark_invalidation(focus, Invalidation::Paint);
                            self.scroll_node_into_view(app, focus);
                        } else if requested_focus.is_some() {
                            focus_requested = true;
                        }

                        if let Some(capture) = requested_capture {
                            if let Some(pointer_id) = event_pointer_id_for_capture {
                                match capture {
                                    Some(node) => {
                                        let allow = !dock_drag_affects_window
                                            || dock_drag_capture_anchor == Some(node);
                                        if allow {
                                            self.captured.insert(pointer_id, node);
                                        }
                                    }
                                    None => {
                                        self.captured.remove(&pointer_id);
                                    }
                                }
                            }
                        }

                        if requested_cursor.is_some() && cursor_choice.is_none() {
                            cursor_choice = requested_cursor;
                        }

                        if stop_propagation {
                            stop_propagation_requested = true;
                            break;
                        }
                    }
                }
            } else {
                loop {
                    let (
                        invalidations,
                        requested_focus,
                        requested_capture,
                        requested_cursor,
                        notify_requested,
                        notify_requested_location,
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
                            layer_root: tree.node_root(node_id),
                            window: tree.window,
                            pointer_id: event_pointer_id_for_capture,
                            input_ctx: input_ctx.clone(),
                            prevented_default_actions: &mut prevented_default_actions,
                            children,
                            focus: tree.focus,
                            captured: event_pointer_id_for_capture
                                .and_then(|p| tree.captured.get(&p).copied()),
                            bounds,
                            invalidations: Vec::new(),
                            requested_focus: None,
                            requested_capture: None,
                            requested_cursor: None,
                            notify_requested: false,
                            notify_requested_location: None,
                            stop_propagation: false,
                        };
                        widget.event(&mut cx, event);
                        (
                            cx.invalidations,
                            cx.requested_focus,
                            cx.requested_capture,
                            cx.requested_cursor,
                            cx.notify_requested,
                            cx.notify_requested_location,
                            cx.stop_propagation,
                            parent,
                        )
                    });
                    if !invalidations.is_empty()
                        || requested_focus.is_some()
                        || requested_capture.is_some()
                        || notify_requested
                    {
                        needs_redraw = true;
                    }

                    for (id, inv) in invalidations {
                        self.mark_invalidation(id, inv);
                    }
                    if notify_requested {
                        self.debug_record_notify_request(
                            app.frame_id(),
                            node_id,
                            notify_requested_location,
                        );
                        self.mark_invalidation_with_source(
                            node_id,
                            Invalidation::Paint,
                            UiDebugInvalidationSource::Notify,
                        );
                    }

                    if let Some(focus) = requested_focus
                        && self.focus_request_is_allowed(app, self.window, &active_layers, focus)
                    {
                        focus_requested = true;
                        if let Some(prev) = self.focus {
                            self.mark_invalidation(prev, Invalidation::Paint);
                        }
                        self.focus = Some(focus);
                        self.mark_invalidation(focus, Invalidation::Paint);
                        self.scroll_node_into_view(app, focus);
                    } else if requested_focus.is_some() {
                        focus_requested = true;
                    }

                    if let Some(capture) = requested_capture {
                        if let Some(pointer_id) = event_pointer_id_for_capture {
                            match capture {
                                Some(node) => {
                                    let allow = !dock_drag_affects_window
                                        || dock_drag_capture_anchor == Some(node);
                                    if allow {
                                        self.captured.insert(pointer_id, node);
                                    }
                                }
                                None => {
                                    self.captured.remove(&pointer_id);
                                }
                            }
                        }
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

                    let captured_now =
                        event_pointer_id_for_capture.and_then(|p| self.captured.get(&p).copied());
                    if captured_now.is_some() || stop_propagation {
                        break;
                    }

                    node_id = match parent {
                        Some(parent) => parent,
                        None => break,
                    };
                }
            }
        }

        if let Event::Pointer(PointerEvent::Down { button, .. }) = event
            && *button == fret_core::MouseButton::Left
            && !focus_requested
            && !prevented_default_actions.contains(fret_runtime::DefaultAction::FocusOnPointerDown)
            && captured.is_none()
            && internal_drag_target.is_none()
            && let Some(window) = self.window
            && let Some(hit) = pointer_hit
        {
            let candidate = self.first_focusable_ancestor_including_declarative(app, window, hit);
            if let Some(focus) = candidate
                && self.focus_request_is_allowed(app, self.window, &active_layers, focus)
            {
                if let Some(prev) = self.focus {
                    self.mark_invalidation(prev, Invalidation::Paint);
                }
                self.focus = Some(focus);
                self.mark_invalidation(focus, Invalidation::Paint);
                self.scroll_node_into_view(app, focus);
                needs_redraw = true;
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

                    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
                        self.app.next_clipboard_token()
                    }
                }

                let mut dismissed_any = false;
                for layer_id in self.visible_layers_in_paint_order() {
                    let Some(layer) = self.layers.get(layer_id) else {
                        continue;
                    };
                    if layer.scroll_dismiss_elements.is_empty() {
                        continue;
                    }
                    let should_dismiss = layer
                        .scroll_dismiss_elements
                        .iter()
                        .copied()
                        .filter_map(|element| {
                            crate::elements::node_for_element(app, window, element)
                        })
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
                    let mut req =
                        crate::action::DismissRequestCx::new(crate::action::DismissReason::Scroll);
                    hook(
                        &mut host,
                        crate::action::ActionCx {
                            window,
                            target: root_element,
                        },
                        &mut req,
                    );
                    dismissed_any = true;
                }

                if dismissed_any {
                    needs_redraw = true;
                }
            }
        }

        if matches!(event, Event::PointerCancel(_))
            && let Some(pointer_id) = event_pointer_id_for_capture
        {
            self.captured.remove(&pointer_id);
        }

        if let Event::PointerCancel(e) = event
            && let Some(window) = self.window
            && pointer_type_supports_hover(e.pointer_type)
        {
            let (prev_element, prev_node, _next_element, _next_node) =
                crate::elements::update_hovered_pressable(app, window, None);
            if prev_node.is_some() {
                needs_redraw = true;
                self.debug_record_hover_edge_pressable();
                if let Some(node) = prev_node {
                    self.mark_invalidation_dedup_with_source(
                        node,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Hover,
                    );
                }
            }

            if let Some(element) = prev_element
                && prev_node.is_some()
            {
                Self::run_pressable_hover_hook(app, window, element, false);
            }

            let (_prev_element, prev_node, _next_element, _next_node) =
                crate::elements::update_hovered_hover_region(app, window, None);
            if prev_node.is_some() {
                needs_redraw = true;
                self.debug_record_hover_edge_hover_region();
                if let Some(node) = prev_node {
                    self.mark_invalidation_dedup_with_source(
                        node,
                        Invalidation::Paint,
                        &mut invalidation_visited,
                        UiDebugInvalidationSource::Hover,
                    );
                }
            }
        }

        if let Event::PointerCancel(e) = event {
            self.touch_pointer_down_outside_candidates
                .remove(&e.pointer_id);
        }

        if defer_keydown_shortcuts_until_after_dispatch
            && !stop_propagation_requested
            && let Event::KeyDown {
                key,
                modifiers,
                repeat,
            } = event
        {
            let focus_is_text_input = self.focus_is_text_input(app);
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
                if needs_redraw {
                    self.request_redraw_coalesced(app);
                }
                return;
            }
        }

        if let Event::Pointer(PointerEvent::Move { .. }) = event
            && let Some(prev) = synth_pointer_move_prev_target
            && captured.is_none()
            && self.node_in_any_layer(prev, &active_layers)
        {
            // Forward a synthetic hover-move to the previously hovered target so retained
            // widgets can clear hover state when the pointer crosses between siblings.
            //
            // We intentionally use observer dispatch to avoid allowing the previous target to
            // mutate focus/capture/cursor routing on the transition frame.
            self.dispatch_event_to_node_chain_observer(
                app,
                services,
                &input_ctx,
                prev,
                event,
                &mut invalidation_visited,
            );
            needs_redraw = true;
        }

        if is_wheel
            && wheel_stop_node.is_some()
            && captured.is_none()
            && let Some(window) = self.window
            && let Event::Pointer(PointerEvent::Wheel {
                position,
                pointer_type,
                ..
            }) = event
            && pointer_type_supports_hover(*pointer_type)
        {
            // Capture scroll-handle-driven invalidations triggered by this wheel event, including
            // out-of-band handle mutations that were not routed through a `Scroll` widget.
            self.invalidate_scroll_handle_bindings_for_changed_handles(
                app,
                crate::layout_pass::LayoutPassKind::Final,
                false,
                false,
            );

            let hit = if event_allows_hit_test_path_cache_reuse(event) {
                self.hit_test_layers_cached(hit_test_layer_roots, *position)
            } else {
                self.hit_test_path_cache = None;
                self.hit_test_layers_cached(hit_test_layer_roots, *position)
            };

            let mut hit_for_hover = hit;
            if let Some((occlusion_layer, occlusion)) =
                self.topmost_pointer_occlusion_layer(barrier_root)
                && occlusion != PointerOcclusion::None
            {
                let occlusion_z = self
                    .layer_order
                    .iter()
                    .position(|id| *id == occlusion_layer);
                let hit_layer_z = hit
                    .and_then(|hit| self.node_layer(hit))
                    .and_then(|layer| self.layer_order.iter().position(|id| *id == layer));
                let hit_is_below_occlusion = match (occlusion_z, hit_layer_z, hit) {
                    (Some(oz), Some(hz), Some(_)) => hz < oz,
                    (Some(_), None, Some(_)) => true,
                    (Some(_), _, None) => true,
                    _ => false,
                };
                if hit_is_below_occlusion {
                    hit_for_hover = None;
                }
            }

            self.update_hover_state_from_hit(
                app,
                window,
                hit_for_hover,
                hit,
                &mut invalidation_visited,
                &mut needs_redraw,
            );
        }

        if input_ctx.caps.ui.cursor_icons
            && let Some(window) = self.window
            && matches!(event, Event::Pointer(_))
        {
            let icon = cursor_choice
                .or(cursor_query_choice)
                .unwrap_or(fret_core::CursorIcon::Default);
            app.push_effect(Effect::CursorSetIcon { window, icon });
        }

        if needs_redraw {
            self.request_redraw_coalesced(app);
        }
        self.dispatch_pointer_move_layer_observers(
            app,
            services,
            &input_ctx,
            barrier_root,
            event,
            &mut needs_redraw,
            &mut invalidation_visited,
        );
        if needs_redraw {
            self.request_redraw_coalesced(app);
        }

        // Keep IME enable/disable tightly coupled to focus changes caused by the event itself.
        let focus_is_text_input = self.focus_is_text_input(app);
        self.set_ime_allowed(app, focus_is_text_input);

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
                focus_is_text_input,
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
            if let Some(mode) = self.focus_text_boundary_mode_override() {
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

            app.with_global_mut(
                fret_runtime::WindowInputContextService::default,
                |svc, _app| {
                    svc.set_snapshot(window, input_ctx.clone());
                },
            );

            self.publish_window_command_action_availability_snapshot(app, &input_ctx);
        }
    }

    pub(super) fn dispatch_event_to_node_chain_observer(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        input_ctx: &InputContext,
        start: NodeId,
        event: &Event,
        invalidation_visited: &mut HashMap<NodeId, u8>,
    ) -> bool {
        let pointer_id_for_capture: Option<fret_core::PointerId> = match event {
            Event::Pointer(PointerEvent::Move { pointer_id, .. })
            | Event::Pointer(PointerEvent::Down { pointer_id, .. })
            | Event::Pointer(PointerEvent::Up { pointer_id, .. })
            | Event::Pointer(PointerEvent::Wheel { pointer_id, .. })
            | Event::Pointer(PointerEvent::PinchGesture { pointer_id, .. }) => Some(*pointer_id),
            Event::PointerCancel(e) => Some(e.pointer_id),
            _ => None,
        };

        let mut pending_invalidations = HashMap::<NodeId, PendingInvalidation>::new();
        let mut did_work = false;

        if event_position(event).is_some() {
            let chain = self.build_mapped_event_chain(start, event);
            for (node_id, event_for_node) in chain {
                let (invalidations, notify_requested, notify_requested_location, _parent) = self
                    .with_widget_mut(node_id, |widget, tree| {
                        let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                        let (children, bounds) = tree
                            .nodes
                            .get(node_id)
                            .map(|n| (n.children.as_slice(), n.bounds))
                            .unwrap_or((&[][..], Rect::default()));
                        let mut observer_ctx = input_ctx.clone();
                        observer_ctx.dispatch_phase = InputDispatchPhase::Preview;
                        let mut cx = crate::widget::ObserverCx {
                            app,
                            services: &mut *services,
                            node: node_id,
                            window: tree.window,
                            pointer_id: pointer_id_for_capture,
                            input_ctx: observer_ctx,
                            children,
                            focus: tree.focus,
                            captured: pointer_id_for_capture
                                .and_then(|p| tree.captured.get(&p).copied()),
                            bounds,
                            invalidations: Vec::new(),
                            notify_requested: false,
                            notify_requested_location: None,
                        };
                        widget.event_observer(&mut cx, &event_for_node);

                        (
                            cx.invalidations,
                            cx.notify_requested,
                            cx.notify_requested_location,
                            parent,
                        )
                    });

                if !invalidations.is_empty() || notify_requested {
                    did_work = true;
                }
                for (id, inv) in invalidations {
                    Self::pending_invalidation_merge(
                        &mut pending_invalidations,
                        id,
                        inv,
                        UiDebugInvalidationSource::Other,
                        UiDebugInvalidationDetail::Unknown,
                    );
                }

                if notify_requested {
                    self.debug_record_notify_request(
                        app.frame_id(),
                        node_id,
                        notify_requested_location,
                    );
                    Self::pending_invalidation_merge(
                        &mut pending_invalidations,
                        node_id,
                        Invalidation::Paint,
                        UiDebugInvalidationSource::Notify,
                        UiDebugInvalidationDetail::from_source(UiDebugInvalidationSource::Notify),
                    );
                }
            }
            self.apply_pending_invalidations(
                std::mem::take(&mut pending_invalidations),
                invalidation_visited,
            );
            return did_work;
        }

        let mut node_id = start;
        loop {
            let (invalidations, notify_requested, notify_requested_location, parent) = self
                .with_widget_mut(node_id, |widget, tree| {
                    let parent = tree.nodes.get(node_id).and_then(|n| n.parent);
                    let (children, bounds) = tree
                        .nodes
                        .get(node_id)
                        .map(|n| (n.children.as_slice(), n.bounds))
                        .unwrap_or((&[][..], Rect::default()));
                    let mut observer_ctx = input_ctx.clone();
                    observer_ctx.dispatch_phase = InputDispatchPhase::Preview;
                    let mut cx = crate::widget::ObserverCx {
                        app,
                        services: &mut *services,
                        node: node_id,
                        window: tree.window,
                        pointer_id: pointer_id_for_capture,
                        input_ctx: observer_ctx,
                        children,
                        focus: tree.focus,
                        captured: pointer_id_for_capture
                            .and_then(|p| tree.captured.get(&p).copied()),
                        bounds,
                        invalidations: Vec::new(),
                        notify_requested: false,
                        notify_requested_location: None,
                    };
                    widget.event_observer(&mut cx, event);

                    (
                        cx.invalidations,
                        cx.notify_requested,
                        cx.notify_requested_location,
                        parent,
                    )
                });

            if !invalidations.is_empty() || notify_requested {
                did_work = true;
            }
            for (id, inv) in invalidations {
                Self::pending_invalidation_merge(
                    &mut pending_invalidations,
                    id,
                    inv,
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::Unknown,
                );
            }

            if notify_requested {
                self.debug_record_notify_request(
                    app.frame_id(),
                    node_id,
                    notify_requested_location,
                );
                Self::pending_invalidation_merge(
                    &mut pending_invalidations,
                    node_id,
                    Invalidation::Paint,
                    UiDebugInvalidationSource::Notify,
                    UiDebugInvalidationDetail::from_source(UiDebugInvalidationSource::Notify),
                );
            }

            node_id = match parent {
                Some(parent) => parent,
                None => break,
            };
        }

        self.apply_pending_invalidations(
            std::mem::take(&mut pending_invalidations),
            invalidation_visited,
        );
        did_work
    }

    fn apply_vector(t: Transform2D, v: Point) -> Point {
        Point::new(Px(t.a * v.x.0 + t.c * v.y.0), Px(t.b * v.x.0 + t.d * v.y.0))
    }

    fn event_with_mapped_position(event: &Event, position: Point, delta: Option<Point>) -> Event {
        match event {
            Event::Pointer(e) => {
                let e = match e {
                    PointerEvent::Move {
                        pointer_id,
                        buttons,
                        modifiers,
                        pointer_type,
                        ..
                    } => PointerEvent::Move {
                        pointer_id: *pointer_id,
                        position,
                        buttons: *buttons,
                        modifiers: *modifiers,
                        pointer_type: *pointer_type,
                    },
                    PointerEvent::Down {
                        pointer_id,
                        button,
                        modifiers,
                        click_count,
                        pointer_type,
                        ..
                    } => PointerEvent::Down {
                        pointer_id: *pointer_id,
                        position,
                        button: *button,
                        modifiers: *modifiers,
                        click_count: *click_count,
                        pointer_type: *pointer_type,
                    },
                    PointerEvent::Up {
                        pointer_id,
                        button,
                        modifiers,
                        is_click,
                        click_count,
                        pointer_type,
                        ..
                    } => PointerEvent::Up {
                        pointer_id: *pointer_id,
                        position,
                        button: *button,
                        modifiers: *modifiers,
                        is_click: *is_click,
                        click_count: *click_count,
                        pointer_type: *pointer_type,
                    },
                    PointerEvent::Wheel {
                        pointer_id,
                        modifiers,
                        pointer_type,
                        ..
                    } => PointerEvent::Wheel {
                        pointer_id: *pointer_id,
                        position,
                        delta: delta.unwrap_or(Point::new(Px(0.0), Px(0.0))),
                        modifiers: *modifiers,
                        pointer_type: *pointer_type,
                    },
                    PointerEvent::PinchGesture {
                        pointer_id,
                        delta,
                        modifiers,
                        pointer_type,
                        ..
                    } => PointerEvent::PinchGesture {
                        pointer_id: *pointer_id,
                        position,
                        delta: *delta,
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
                pointer_id: e.pointer_id,
                position,
                kind: e.kind.clone(),
                modifiers: e.modifiers,
            }),
            Event::PointerCancel(e) => {
                let mut e = e.clone();
                e.position = Some(position);
                Event::PointerCancel(e)
            }
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
            let prepaint = self
                .nodes
                .get(node)
                .and_then(|n| {
                    (!self.inspection_active && !n.invalidation.hit_test)
                        .then_some(n.prepaint_hit_test)
                })
                .flatten();
            if let Some(inv) = prepaint
                .and_then(|p| p.render_transform_inv)
                .or_else(|| self.node_render_transform(node).and_then(|t| t.inverse()))
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

            // Map into the child's coordinate space for the next node in the chain.
            let prepaint = self
                .nodes
                .get(node)
                .and_then(|n| {
                    (!self.inspection_active && !n.invalidation.hit_test)
                        .then_some(n.prepaint_hit_test)
                })
                .flatten();
            if let Some(inv) = prepaint
                .and_then(|p| p.children_render_transform_inv)
                .or_else(|| {
                    self.node_children_render_transform(node)
                        .and_then(|t| t.inverse())
                })
            {
                mapped_pos = inv.apply_point(mapped_pos);
                if let Some(d) = mapped_delta {
                    mapped_delta = Some(Self::apply_vector(inv, d));
                }
            }
        }

        out.reverse();
        out
    }

    fn build_unmapped_event_chain(&self, start: NodeId, event: &Event) -> Vec<(NodeId, Event)> {
        let mut out: Vec<(NodeId, Event)> = Vec::new();
        let mut cur = Some(start);
        while let Some(id) = cur {
            out.push((id, event.clone()));
            cur = self.nodes.get(id).and_then(|n| n.parent);
        }
        out
    }

    fn cursor_icon_query_for_pointer_hit(
        &mut self,
        start: NodeId,
        input_ctx: &InputContext,
        event: &Event,
    ) -> Option<fret_core::CursorIcon> {
        if event_position(event).is_none() {
            return None;
        }

        let chain = self.build_mapped_event_chain(start, event);
        for (node_id, mapped_event) in chain {
            let Some(position) = event_position(&mapped_event) else {
                continue;
            };
            let bounds = self
                .nodes
                .get(node_id)
                .map(|n| n.bounds)
                .unwrap_or_default();
            let requested = self.with_widget_mut(node_id, |widget, _tree| {
                widget.cursor_icon_at(bounds, position, input_ctx)
            });
            if requested.is_some() {
                return requested;
            }
        }

        None
    }
}
