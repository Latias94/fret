use crate::ui::canvas::widget::*;

use super::{finish, target};

pub(super) fn handle_edge_reroute_double_click<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    click_count: u8,
    zoom: f32,
) -> bool {
    if click_count != 2
        || !snapshot.interaction.reroute_on_edge_double_click
        || super::super::menu_session::has_active_menu_session(&canvas.interaction)
    {
        return false;
    }

    let Some(edge_id) = target::edge_double_click_target(canvas, cx, snapshot, position, zoom)
    else {
        return false;
    };

    let outcome = canvas.plan_canvas_split_edge_reroute(cx.app, edge_id, position);
    canvas.execute_split_edge_reroute_outcome(cx.app, cx.window, Some("Insert Reroute"), outcome);
    finish::finish_double_click(cx);
    true
}
