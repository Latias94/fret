mod edge;
mod group;

use crate::core::{EdgeId, GroupId};
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn hit_group_context_target<H: UiHost>(
        &self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        position: Point,
        zoom: f32,
    ) -> Option<GroupId> {
        group::hit_group_context_target(self, host, snapshot, position, zoom)
    }

    pub(in crate::ui::canvas::widget) fn hit_edge_context_target<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        position: Point,
        zoom: f32,
    ) -> Option<EdgeId> {
        edge::hit_edge_context_target(self, host, snapshot, position, zoom)
    }
}
