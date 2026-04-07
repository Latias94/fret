use crate::ui::canvas::widget::*;

use super::delta;

pub(super) fn auto_pan_should_tick<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    bounds: Rect,
) -> bool {
    if super::super::menu_session::has_active_menu_session(&canvas.interaction) {
        return false;
    }
    let Some(pos) = canvas.interaction.last_pos else {
        return false;
    };

    let wants_node_drag = snapshot.interaction.auto_pan.on_node_drag
        && (canvas.interaction.node_drag.is_some()
            || canvas.interaction.group_drag.is_some()
            || canvas.interaction.group_resize.is_some());
    let wants_connect =
        snapshot.interaction.auto_pan.on_connect && canvas.interaction.wire_drag.is_some();

    if !wants_node_drag && !wants_connect {
        return false;
    }

    let delta = delta::auto_pan_delta(snapshot, pos, bounds);
    delta.x != 0.0 || delta.y != 0.0
}
