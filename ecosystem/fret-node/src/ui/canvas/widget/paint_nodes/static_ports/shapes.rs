use crate::ui::PortShapeHint;
use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

pub(super) fn paint_static_port_shapes<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    scene: &mut fret_core::Scene,
    services: &mut dyn fret_core::UiServices,
    scale_factor: f32,
    render: &RenderData,
    zoom: f32,
) {
    for (port_id, rect, color, hint) in &render.pins {
        let outer_rect = *rect;
        let fill_rect = super::geometry::scaled_inner_port_rect(outer_rect, hint.inner_scale);
        let color = *color;
        let shape = hint.shape.unwrap_or(PortShapeHint::Circle);
        let dir = render.port_labels.get(port_id).map(|label| label.dir);

        if hint.inner_scale.unwrap_or(1.0) > 0.0 {
            super::fill::paint_static_port_fill(
                canvas,
                scene,
                services,
                scale_factor,
                fill_rect,
                shape,
                dir,
                color,
                zoom,
            );
        }

        if let Some(stroke) = hint.stroke {
            let width = hint.stroke_width.unwrap_or(1.0);
            if width.is_finite() && width > 0.0 {
                super::stroke::paint_static_port_stroke(
                    canvas,
                    scene,
                    services,
                    scale_factor,
                    outer_rect,
                    shape,
                    dir,
                    stroke,
                    width,
                    zoom,
                );
            }
        }
    }
}
