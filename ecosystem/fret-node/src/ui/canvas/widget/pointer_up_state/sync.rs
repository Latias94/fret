use fret_core::{Modifiers, Point};

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(in super::super) fn sync_pointer_up_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
    modifiers: Modifiers,
) {
    canvas.interaction.last_pos = Some(position);
    canvas.interaction.last_modifiers = modifiers;
    canvas.interaction.last_canvas_pos = Some(canvas_point_from_pointer_position(position));
}

fn canvas_point_from_pointer_position(position: Point) -> crate::core::CanvasPoint {
    crate::core::CanvasPoint {
        x: position.x.0,
        y: position.y.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Px;

    #[test]
    fn canvas_point_from_pointer_position_transfers_axes() {
        let position = Point::new(Px(18.0), Px(-4.5));
        assert_eq!(
            canvas_point_from_pointer_position(position),
            crate::core::CanvasPoint { x: 18.0, y: -4.5 }
        );
    }
}
