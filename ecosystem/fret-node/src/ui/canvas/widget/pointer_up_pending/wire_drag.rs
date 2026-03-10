use fret_core::Point;
use fret_ui::UiHost;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{ViewSnapshot, WireDrag, WireDragKind};

pub(in super::super) fn handle_pending_wire_drag_release<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool {
    let Some(pending) = canvas.interaction.pending_wire_drag.take() else {
        return false;
    };

    if should_promote_pending_wire_drag(snapshot.interaction.connect_on_click, &pending.kind) {
        let kind = pending.kind.clone();
        canvas.interaction.wire_drag = Some(WireDrag {
            kind: pending.kind,
            pos: position,
        });
        canvas.interaction.click_connect = true;
        canvas.emit_connect_start(snapshot, &kind);
    }

    super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}

fn should_promote_pending_wire_drag(connect_on_click: bool, kind: &WireDragKind) -> bool {
    connect_on_click && matches!(kind, WireDragKind::New { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{EdgeId, PortId};
    use crate::rules::EdgeEndpoint;

    fn new_wire_drag() -> WireDragKind {
        WireDragKind::New {
            from: PortId::new(),
            bundle: vec![PortId::new()],
        }
    }

    #[test]
    fn should_promote_pending_wire_drag_requires_click_connect_and_new_drag() {
        assert!(should_promote_pending_wire_drag(true, &new_wire_drag()));
        assert!(!should_promote_pending_wire_drag(false, &new_wire_drag()));
        assert!(!should_promote_pending_wire_drag(
            true,
            &WireDragKind::Reconnect {
                edge: EdgeId::new(),
                endpoint: EdgeEndpoint::From,
                fixed: PortId::new(),
            }
        ));
    }
}
