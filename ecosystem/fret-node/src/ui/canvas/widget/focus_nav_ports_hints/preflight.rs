use crate::interaction::NodeGraphConnectionMode;
use crate::ui::canvas::state::WireDrag;
use crate::ui::canvas::widget::*;

pub(super) struct FocusedPortHintInput {
    pub target: PortId,
    pub wire_drag: WireDrag,
    pub mode: NodeGraphConnectionMode,
}

pub(super) fn collect_hint_refresh_input<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) -> Option<FocusedPortHintInput> {
    let snapshot = canvas.sync_view_state(host);
    let target = canvas.interaction.focused_port?;
    let wire_drag = canvas.interaction.wire_drag.clone()?;

    Some(FocusedPortHintInput {
        target,
        wire_drag,
        mode: snapshot.interaction.connection_mode,
    })
}
