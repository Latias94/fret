use crate::ui::canvas::widget::*;

use super::ConnectionInsertMenuPlan;

pub(super) fn apply_connection_insert_menu_plan<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    fallback_from: PortId,
    invoked_at: Point,
    plan: ConnectionInsertMenuPlan,
) {
    match plan {
        ConnectionInsertMenuPlan::Apply(planned) => {
            let workflow::WireDropInsertPlan {
                ops,
                created_node,
                continue_from,
                toast,
            } = planned;
            if canvas.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                canvas.select_inserted_node(cx.app, created_node);
                if let Some((severity, message)) = toast {
                    canvas.show_toast(cx.app, cx.window, severity, message);
                }
                canvas.resume_connection_insert_wire_drag(
                    cx,
                    fallback_from,
                    invoked_at,
                    continue_from,
                );
            } else {
                canvas.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
            }
        }
        ConnectionInsertMenuPlan::Reject(severity, message) => {
            canvas.show_toast(cx.app, cx.window, severity, message);
            canvas.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
        }
        ConnectionInsertMenuPlan::Ignore => {
            canvas.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
        }
    }
}
