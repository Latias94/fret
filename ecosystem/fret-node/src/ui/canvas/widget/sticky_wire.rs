use fret_core::{MouseButton, Point};
use fret_ui::UiHost;

use super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{ViewSnapshot, WireDragKind};

pub(super) fn handle_sticky_wire_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if button != MouseButton::Left
        || !canvas.interaction.sticky_wire
        || canvas.interaction.wire_drag.is_none()
    {
        return false;
    }

    let Some(mut wire_drag) = canvas.interaction.wire_drag.take() else {
        super::sticky_wire_targets::reset_sticky_wire_state(canvas);
        return true;
    };

    let from_port = match &wire_drag.kind {
        WireDragKind::New { from, .. } => *from,
        _ => {
            canvas.interaction.wire_drag = Some(wire_drag);
            return true;
        }
    };

    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    let mut scratch = HitTestScratch::default();
    let mut hit_test = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
    let hit_port = canvas.hit_port(&mut hit_test, position);
    let target = super::sticky_wire_connect::connectable_sticky_wire_target(
        canvas, cx.app, snapshot, hit_port,
    );

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
