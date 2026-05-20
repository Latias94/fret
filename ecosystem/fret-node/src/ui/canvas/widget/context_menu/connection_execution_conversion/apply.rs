use crate::ui::canvas::widget::*;

use super::ConnectionConversionMenuPlan;

pub(super) fn apply_connection_conversion_menu_plan<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::ConnectionConversionMenuCx<H>,
    fallback_from: PortId,
    invoked_at: Point,
    plan: ConnectionConversionMenuPlan,
) {
    match plan {
        ConnectionConversionMenuPlan::Apply(ops) => {
            let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
            let window = cx.window();
            canvas.apply_ops(cx.host(), window, ops);
            canvas.interaction.suspended_wire_drag = None;
            canvas.select_inserted_node(cx.host(), node_id);
        }
        ConnectionConversionMenuPlan::Reject(severity, message) => {
            let window = cx.window();
            canvas.show_toast(cx.host(), window, severity, message);
            cx.restore_connection_conversion_wire_drag(canvas, fallback_from, invoked_at);
        }
        ConnectionConversionMenuPlan::Ignore => {
            cx.restore_connection_conversion_wire_drag(canvas, fallback_from, invoked_at);
        }
    }
}
