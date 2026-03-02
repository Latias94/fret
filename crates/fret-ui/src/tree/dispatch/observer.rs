use super::*;
use std::collections::HashMap;

use super::PendingInvalidation;

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn dispatch_event_to_node_chain_observer(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        input_ctx: &InputContext,
        start: NodeId,
        event: &Event,
        snapshot: Option<&UiDispatchSnapshot>,
        invalidation_visited: &mut impl InvalidationVisited,
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
            let chain = self.build_mapped_event_chain(start, event, snapshot);
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
}
