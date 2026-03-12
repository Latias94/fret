mod edge;
mod group;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

use crate::core::{EdgeId, GroupId};
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn select_group_context_target<H: UiHost>(
        &mut self,
        host: &mut H,
        group_id: GroupId,
    ) {
        self.update_view_state(host, |view_state| {
            group::select_group_context_target_in_view_state(view_state, group_id);
        });
    }

    pub(in crate::ui::canvas::widget) fn select_edge_context_target<H: UiHost>(
        &mut self,
        host: &mut H,
        edge_id: EdgeId,
    ) {
        self.update_view_state(host, |view_state| {
            edge::select_edge_context_target_in_view_state(view_state, edge_id);
        });
    }
}
