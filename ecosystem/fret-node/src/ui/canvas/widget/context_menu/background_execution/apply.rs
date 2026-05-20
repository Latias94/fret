use crate::ui::canvas::widget::*;

use super::BackgroundInsertMenuPlan;

pub(super) fn apply_background_insert_menu_plan<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::BackgroundInsertMenuCx<H>,
    plan: BackgroundInsertMenuPlan,
) {
    match plan {
        BackgroundInsertMenuPlan::Apply(ops) => {
            let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
            let window = cx.window();
            if canvas.commit_ops(cx.host(), window, Some("Insert Node"), ops) {
                canvas.select_inserted_node(cx.host(), node_id);
            }
        }
        BackgroundInsertMenuPlan::Reject(sev, msg) => {
            let window = cx.window();
            canvas.show_toast(cx.host(), window, sev, msg);
        }
        BackgroundInsertMenuPlan::Ignore => {}
    }
}
