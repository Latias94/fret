use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

impl<H: UiHost> super::pass_scene_adapter::PaintRootPassSceneCx<H> for PaintCx<'_, H> {
    fn paint_root_pass_groups_static<M>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        groups: &[(Rect, Arc<str>, bool)],
        zoom: f32,
    ) where
        M: NodeGraphCanvasMiddleware,
    {
        canvas.paint_groups_static(self.scene, self.services, self.scale_factor, groups, zoom);
    }

    fn paint_root_pass_groups_selected_overlay<M>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        groups: &[(Rect, Arc<str>, bool)],
        zoom: f32,
    ) where
        M: NodeGraphCanvasMiddleware,
    {
        canvas.paint_groups_selected_overlay(self.scene, groups, zoom);
    }

    fn paint_root_pass_nodes_static<M>(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        render: &RenderData,
        zoom: f32,
    ) where
        M: NodeGraphCanvasMiddleware,
    {
        canvas.paint_nodes_static(self.scene, self.services, self.scale_factor, render, zoom);
    }
}
