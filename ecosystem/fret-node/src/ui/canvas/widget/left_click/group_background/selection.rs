use fret_ui::UiHost;

use crate::core::GroupId;
use crate::ui::canvas::widget::*;

pub(super) fn select_group_for_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    group: GroupId,
    multi_selection_pressed: bool,
) {
    canvas.update_view_state(cx.app, |s| {
        if multi_selection_pressed {
            if let Some(ix) = s.selected_groups.iter().position(|id| *id == group) {
                s.selected_groups.remove(ix);
            } else {
                s.selected_groups.push(group);
            }
        } else if !s.selected_groups.iter().any(|id| *id == group) {
            s.selected_nodes.clear();
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_groups.push(group);
        }
        s.group_draw_order.retain(|id| *id != group);
        s.group_draw_order.push(group);
    });
}
