use super::*;

impl<H: UiHost> UiTree<H> {
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
                kind: e.kind,
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

    pub(in crate::tree::dispatch) fn build_mapped_event_chain(
        &self,
        start: NodeId,
        event: &Event,
    ) -> Vec<(NodeId, Event)> {
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

    pub(in crate::tree::dispatch) fn build_unmapped_event_chain(
        &self,
        start: NodeId,
        event: &Event,
    ) -> Vec<(NodeId, Event)> {
        let mut out: Vec<(NodeId, Event)> = Vec::new();
        let mut cur = Some(start);
        while let Some(id) = cur {
            out.push((id, event.clone()));
            cur = self.nodes.get(id).and_then(|n| n.parent);
        }
        out
    }

    pub(in crate::tree::dispatch) fn cursor_icon_query_for_pointer_hit(
        &mut self,
        start: NodeId,
        input_ctx: &InputContext,
        event: &Event,
    ) -> Option<fret_core::CursorIcon> {
        event_position(event)?;

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

pub(super) fn pointer_cancel_event_for_capture_switch(
    event: &Event,
    pointer_id: fret_core::PointerId,
) -> Event {
    let (position, buttons, modifiers, pointer_type) = match event {
        Event::Pointer(PointerEvent::Move {
            position,
            buttons,
            modifiers,
            pointer_type,
            ..
        }) => (Some(*position), *buttons, *modifiers, *pointer_type),
        Event::Pointer(PointerEvent::Down {
            position,
            modifiers,
            pointer_type,
            ..
        }) => (
            Some(*position),
            fret_core::MouseButtons::default(),
            *modifiers,
            *pointer_type,
        ),
        Event::Pointer(PointerEvent::Up {
            position,
            modifiers,
            pointer_type,
            ..
        }) => (
            Some(*position),
            fret_core::MouseButtons::default(),
            *modifiers,
            *pointer_type,
        ),
        Event::Pointer(PointerEvent::Wheel {
            position,
            modifiers,
            pointer_type,
            ..
        }) => (
            Some(*position),
            fret_core::MouseButtons::default(),
            *modifiers,
            *pointer_type,
        ),
        Event::Pointer(PointerEvent::PinchGesture {
            position,
            modifiers,
            pointer_type,
            ..
        }) => (
            Some(*position),
            fret_core::MouseButtons::default(),
            *modifiers,
            *pointer_type,
        ),
        Event::PointerCancel(e) => (e.position, e.buttons, e.modifiers, e.pointer_type),
        _ => (
            event_position(event),
            fret_core::MouseButtons::default(),
            fret_core::Modifiers::default(),
            fret_core::PointerType::Unknown,
        ),
    };

    Event::PointerCancel(fret_core::PointerCancelEvent {
        pointer_id,
        position,
        buttons,
        modifiers,
        pointer_type,
        // We do not have a dedicated cancel reason for capture switches yet.
        // Treat this as a best-effort cancellation signal to clear pressed/drag state.
        reason: fret_core::PointerCancelReason::LeftWindow,
    })
}
