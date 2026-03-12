use fret_core::MouseButton;

use crate::core::PortId;
use crate::ui::canvas::state::{WireDrag, WireDragKind};
use crate::ui::canvas::widget::*;

pub(super) enum StickyWirePointerDownPrep {
    NotHandled,
    Handled,
    Ready {
        wire_drag: WireDrag,
        from_port: PortId,
    },
}

pub(super) fn prepare_sticky_wire_pointer_down<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    button: MouseButton,
) -> StickyWirePointerDownPrep {
    if !should_prepare_sticky_wire_pointer_down(
        button,
        canvas.interaction.sticky_wire,
        canvas.interaction.wire_drag.is_some(),
    ) {
        return StickyWirePointerDownPrep::NotHandled;
    }

    let Some(wire_drag) = canvas.interaction.wire_drag.take() else {
        super::super::sticky_wire_targets::reset_sticky_wire_state(canvas);
        return StickyWirePointerDownPrep::Handled;
    };

    match sticky_wire_from_port(&wire_drag.kind) {
        Some(from_port) => StickyWirePointerDownPrep::Ready {
            wire_drag,
            from_port,
        },
        _ => {
            canvas.interaction.wire_drag = Some(wire_drag);
            StickyWirePointerDownPrep::Handled
        }
    }
}

fn should_prepare_sticky_wire_pointer_down(
    button: MouseButton,
    sticky_wire: bool,
    has_wire_drag: bool,
) -> bool {
    button == MouseButton::Left && sticky_wire && has_wire_drag
}

fn sticky_wire_from_port(kind: &WireDragKind) -> Option<PortId> {
    match kind {
        WireDragKind::New { from, .. } => Some(*from),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
