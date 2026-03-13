use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;
use fret_core::TextStyle;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_static_port_labels<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    scene: &mut fret_core::Scene,
    services: &mut dyn fret_core::UiServices,
    scale_factor: f32,
    render: &RenderData,
    node_text_style: &TextStyle,
    zoom: f32,
    pin_r: f32,
    pin_gap: f32,
) {
    for (port_id, info) in &render.port_labels {
        let Some(center) = render.port_centers.get(port_id).copied() else {
            continue;
        };
        let port_constraints = TextConstraints {
            max_width: Some(info.max_width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            scale_factor: effective_scale_factor(scale_factor, zoom),
        };
        let (blob, metrics) = canvas.paint_cache.text_blob(
            services,
            info.label.clone(),
            node_text_style,
            port_constraints,
        );

        let y = Px(center.y.0 - 0.5 * metrics.size.height.0 + metrics.baseline.0);
        let x = match info.dir {
            PortDirection::In => Px(center.x.0 + pin_r + pin_gap),
            PortDirection::Out => Px(center.x.0 - pin_r - pin_gap - metrics.size.width.0),
        };

        scene.push(SceneOp::Text {
            order: DrawOrder(4),
            origin: Point::new(x, y),
            text: blob,
            paint: canvas.style.paint.context_menu_text.into(),
            outline: None,
            shadow: None,
        });
    }
}
