use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_nodes_static(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        render: &RenderData,
        zoom: f32,
    ) {
        let mut node_text_style = self.style.geometry.context_menu_text_style.clone();
        node_text_style.size = Px(node_text_style.size.0 / zoom);
        if let Some(line_height) = node_text_style.line_height.as_mut() {
            line_height.0 /= zoom;
        }

        let corner = Px(self.style.paint.node_corner_radius / zoom);
        let title_pad = self.style.geometry.node_padding / zoom;
        let title_h = self.style.geometry.node_header_height / zoom;

        for (node, rect, is_selected, title, body, pin_rows, _resize_handles, hint) in &render.nodes
        {
            self.paint_static_node(
                scene,
                services,
                scale_factor,
                *node,
                *rect,
                *is_selected,
                title,
                body.as_ref(),
                *pin_rows,
                *hint,
                &node_text_style,
                zoom,
                corner,
                title_pad,
                title_h,
            );
        }

        let pin_r = self.style.geometry.pin_radius / zoom;
        let pin_gap = 8.0 / zoom;
        self.paint_static_port_labels(
            scene,
            services,
            scale_factor,
            render,
            &node_text_style,
            zoom,
            pin_r,
            pin_gap,
        );
        self.paint_static_port_shapes(scene, services, scale_factor, render, zoom);
    }
}
