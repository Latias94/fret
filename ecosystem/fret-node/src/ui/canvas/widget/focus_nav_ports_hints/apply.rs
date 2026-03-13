use super::evaluate::FocusedPortHintOutcome;
use crate::ui::canvas::widget::*;

pub(super) fn apply_focused_port_hints<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    target: PortId,
    outcome: FocusedPortHintOutcome,
) {
    if canvas.interaction.wire_drag.is_some() && canvas.interaction.focused_port == Some(target) {
        canvas.interaction.focused_port_valid = outcome.valid;
        canvas.interaction.focused_port_convertible = outcome.convertible;
    }
}
