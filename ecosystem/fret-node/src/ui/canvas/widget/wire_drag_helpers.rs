use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn start_sticky_wire_drag_from_port<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        from: PortId,
        pos: Point,
    ) {
        self.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from,
                bundle: Vec::new(),
            },
            pos,
        });
        self.interaction.sticky_wire = true;
        self.interaction.sticky_wire_ignore_next_up = true;
        super::focus_session::clear_hover_port_hints(&mut self.interaction);
        cx.capture_pointer(cx.node);
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
    }

    pub(super) fn restore_suspended_wire_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: Option<PortId>,
        fallback_pos: Point,
    ) {
        if let Some(wire_drag) = self.interaction.suspended_wire_drag.take() {
            self.interaction.wire_drag = Some(wire_drag);
            self.interaction.sticky_wire = true;
            self.interaction.sticky_wire_ignore_next_up = true;
            super::focus_session::clear_hover_port_hints(&mut self.interaction);
            cx.capture_pointer(cx.node);
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            return;
        }

        if let Some(from) = fallback_from {
            self.start_sticky_wire_drag_from_port(cx, from, fallback_pos);
        }
    }

    pub(super) fn wire_drag_suppresses_edge(kind: &WireDragKind, edge_id: EdgeId) -> bool {
        match kind {
            WireDragKind::Reconnect { edge, .. } => *edge == edge_id,
            WireDragKind::ReconnectMany { edges } => {
                edges.iter().any(|(edge, ..)| *edge == edge_id)
            }
            _ => false,
        }
    }
}
