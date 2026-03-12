mod checks;
mod target;

use fret_core::{MouseButton, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_sticky_wire_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    let (mut wire_drag, from_port) = match checks::prepare_sticky_wire_pointer_down(canvas, button)
    {
        checks::StickyWirePointerDownPrep::NotHandled => return false,
        checks::StickyWirePointerDownPrep::Handled => return true,
        checks::StickyWirePointerDownPrep::Ready {
            wire_drag,
            from_port,
        } => (wire_drag, from_port),
    };

    let (geom, index, target) =
        target::sticky_wire_target(canvas, cx.app, snapshot, position, zoom);

    if let Some(target_port) = target {
        if super::sticky_wire_connect::handle_sticky_wire_connect_target(
            canvas,
            cx,
            snapshot,
            from_port,
            target_port,
            &mut wire_drag,
            position,
        ) {
            return true;
        }
    }

    super::sticky_wire_targets::handle_sticky_wire_non_port_target(
        canvas,
        cx,
        snapshot,
        geom.as_ref(),
        index.as_ref(),
        from_port,
        position,
        zoom,
    )
}
