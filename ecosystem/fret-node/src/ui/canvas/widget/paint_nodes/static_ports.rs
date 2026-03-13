#[path = "static_ports/fill.rs"]
mod fill;
#[path = "static_ports/geometry.rs"]
mod geometry;
#[path = "static_ports/labels.rs"]
mod labels;
#[path = "static_ports/shapes.rs"]
mod shapes;
#[path = "static_ports/stroke.rs"]
mod stroke;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;
use fret_core::TextStyle;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_static_port_labels(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        render: &RenderData,
        node_text_style: &TextStyle,
        zoom: f32,
        pin_r: f32,
        pin_gap: f32,
    ) {
        labels::paint_static_port_labels(
            self,
            scene,
            services,
            scale_factor,
            render,
            node_text_style,
            zoom,
            pin_r,
            pin_gap,
        );
    }

    pub(super) fn paint_static_port_shapes(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        render: &RenderData,
        zoom: f32,
    ) {
        shapes::paint_static_port_shapes(self, scene, services, scale_factor, render, zoom);
    }
}
