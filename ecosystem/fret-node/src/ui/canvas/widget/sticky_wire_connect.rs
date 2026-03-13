mod finish;
mod plan;

use fret_core::Point;
use fret_ui::UiHost;

use crate::ui::canvas::state::WireDrag;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot};

pub(super) fn connectable_sticky_wire_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    hit_port: Option<crate::core::PortId>,
) -> Option<crate::core::PortId> {
    plan::connectable_sticky_wire_target(canvas, host, snapshot, hit_port)
}

pub(super) fn handle_sticky_wire_connect_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    from_port: crate::core::PortId,
    target_port: crate::core::PortId,
    wire_drag: &mut WireDrag,
    position: Point,
) -> bool {
    match plan::plan_sticky_wire_connect_outcome(canvas, cx.app, snapshot, from_port, target_port) {
        plan::StickyWireConnectOutcome::Apply(ops) => {
            canvas.apply_ops(cx.app, cx.window, ops);
            super::sticky_wire_targets::reset_sticky_wire_state(canvas);
            finish::finish_sticky_wire_pointer_down(cx);
            true
        }
        plan::StickyWireConnectOutcome::Reject(severity, message) => {
            canvas.show_toast(cx.app, cx.window, severity, message);
            wire_drag.pos = position;
            canvas.interaction.wire_drag = Some(wire_drag.clone());
            finish::finish_sticky_wire_pointer_down(cx);
            true
        }
        plan::StickyWireConnectOutcome::Ignore => {
            wire_drag.pos = position;
            canvas.interaction.wire_drag = Some(wire_drag.clone());
            finish::finish_sticky_wire_pointer_down(cx);
            true
        }
    }
}
