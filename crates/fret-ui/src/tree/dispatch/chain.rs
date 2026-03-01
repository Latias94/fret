use super::*;
use std::collections::HashMap;

use super::PendingInvalidation;
use super::event_chain::pointer_cancel_event_for_capture_switch;

impl<H: UiHost> UiTree<H> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::tree::dispatch) fn dispatch_event_to_node_chain(
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
        let event_window_position = event_position(event);
        let event_window_wheel_delta = match event {
            Event::Pointer(PointerEvent::Wheel { delta, .. }) => Some(*delta),
            _ => None,
        };

        let (active_roots, _barrier_root) = self.active_input_layers();
        if event_position(event).is_some() {
            let chain = self.build_mapped_event_chain(start, event);
            let pointer_hit_is_text_input =
                if matches!(event, Event::Pointer(PointerEvent::Down { .. }))
                    && let Some(window) = self.window
                {
                    chain.iter().any(|(node_id, _)| {
                        crate::declarative::element_record_for_node(app, window, *node_id)
                            .is_some_and(|record| {
                                matches!(
                                    &record.instance,
                                    crate::declarative::ElementInstance::TextInput(_)
                                        | crate::declarative::ElementInstance::TextArea(_)
                                        | crate::declarative::ElementInstance::TextInputRegion(_)
                                )
                            })
                    })
                } else {
                    false
                };
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
                        scale_factor: tree.last_layout_scale_factor.unwrap_or(1.0),
                        event_window_position,
                        event_window_wheel_delta,
                        input_ctx: input_ctx.clone(),
                        pointer_hit_is_text_input,
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
                    // Avoid scrolling during pointer-driven focus changes:
                    // programmatic scroll-to-focus can move content under a stationary cursor,
                    // causing pointer activation to miss/cancel (especially for nested pressables).
                    //
                    // Keyboard traversal still scrolls focused nodes into view.
                    if !matches!(event, Event::Pointer(_) | Event::PointerCancel(_)) {
                        self.scroll_node_into_view(app, focus);
                    }
                }

                if let Some(capture) = requested_capture
                    && capture.is_none_or(|n| self.node_in_any_layer(n, &active_roots))
                    && let Some(pointer_id) = pointer_id_for_capture
                {
                    if let Some(new_capture) = capture
                        && !matches!(event, Event::PointerCancel(_))
                        && let Some(old_capture) = self.captured.get(&pointer_id).copied()
                        && old_capture != new_capture
                        && self.node_in_any_layer(old_capture, &active_roots)
                    {
                        // When a component steals pointer capture mid-sequence (e.g. gesture arena
                        // outcomes), cancel the previous capture target so pressables/widgets can
                        // clear any "pressed" state.
                        let cancel_event =
                            pointer_cancel_event_for_capture_switch(event, pointer_id);
                        let _ = self.dispatch_event_to_node_chain(
                            app,
                            services,
                            input_ctx,
                            old_capture,
                            &cancel_event,
                            needs_redraw,
                            invalidation_visited,
                        );
                    }

                    match capture {
                        Some(node) => {
                            self.captured.insert(pointer_id, node);
                        }
                        None => {
                            self.captured.remove(&pointer_id);
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
                    scale_factor: tree.last_layout_scale_factor.unwrap_or(1.0),
                    event_window_position,
                    event_window_wheel_delta,
                    input_ctx: input_ctx.clone(),
                    pointer_hit_is_text_input: false,
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
                // Avoid scrolling during pointer-driven focus changes:
                // programmatic scroll-to-focus can move content under a stationary cursor,
                // causing pointer activation to miss/cancel (especially for nested pressables).
                //
                // Keyboard traversal still scrolls focused nodes into view.
                if !matches!(event, Event::Pointer(_) | Event::PointerCancel(_)) {
                    self.scroll_node_into_view(app, focus);
                }
            }

            if let Some(capture) = requested_capture
                && capture.is_none_or(|n| self.node_in_any_layer(n, &active_roots))
                && let Some(pointer_id) = pointer_id_for_capture
            {
                if let Some(new_capture) = capture
                    && !matches!(event, Event::PointerCancel(_))
                    && let Some(old_capture) = self.captured.get(&pointer_id).copied()
                    && old_capture != new_capture
                    && self.node_in_any_layer(old_capture, &active_roots)
                {
                    let cancel_event = pointer_cancel_event_for_capture_switch(event, pointer_id);
                    let _ = self.dispatch_event_to_node_chain(
                        app,
                        services,
                        input_ctx,
                        old_capture,
                        &cancel_event,
                        needs_redraw,
                        invalidation_visited,
                    );
                }

                match capture {
                    Some(node) => {
                        self.captured.insert(pointer_id, node);
                    }
                    None => {
                        self.captured.remove(&pointer_id);
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
}
