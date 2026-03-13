use crate::ui::canvas::widget::*;

use super::BackgroundInsertMenuPlan;

pub(super) fn apply_background_insert_menu_plan<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    plan: BackgroundInsertMenuPlan,
) {
    match plan {
        BackgroundInsertMenuPlan::Apply(ops) => {
            let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
            if canvas.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                canvas.select_inserted_node(cx.app, node_id);
            }
        }
        BackgroundInsertMenuPlan::Reject(sev, msg) => {
            canvas.show_toast(cx.app, cx.window, sev, msg);
        }
        BackgroundInsertMenuPlan::Ignore => {}
    }
}
