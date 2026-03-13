use crate::ui::canvas::widget::*;

use super::ConnectionConversionMenuPlan;

pub(super) fn apply_connection_conversion_menu_plan<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    fallback_from: PortId,
    invoked_at: Point,
    plan: ConnectionConversionMenuPlan,
) {
    match plan {
        ConnectionConversionMenuPlan::Apply(ops) => {
            let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
            canvas.apply_ops(cx.app, cx.window, ops);
            canvas.interaction.suspended_wire_drag = None;
            canvas.select_inserted_node(cx.app, node_id);
        }
        ConnectionConversionMenuPlan::Reject(severity, message) => {
            canvas.show_toast(cx.app, cx.window, severity, message);
            canvas.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
        }
        ConnectionConversionMenuPlan::Ignore => {
            canvas.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
        }
    }
}
