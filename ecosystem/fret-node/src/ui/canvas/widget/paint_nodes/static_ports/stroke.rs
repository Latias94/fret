use crate::ui::PortShapeHint;
use crate::ui::canvas::widget::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_static_port_stroke<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    scene: &mut fret_core::Scene,
    services: &mut dyn fret_core::UiServices,
    scale_factor: f32,
    rect: Rect,
    shape: PortShapeHint,
    dir: Option<PortDirection>,
    color: Color,
    width: f32,
    zoom: f32,
) {
    match shape {
        PortShapeHint::Circle => {
            let radius = Px(0.5 * rect.size.width.0);
            scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect,
                background: fret_core::Paint::TRANSPARENT.into(),
                border: Edges::all(Px(width / zoom)),
                border_paint: fret_core::Paint::Solid(color).into(),
                corner_radii: Corners::all(radius),
            });
        }
        PortShapeHint::Diamond | PortShapeHint::Triangle => {
            if let Some(path) = canvas.paint_cache.port_shape_stroke_path(
                services,
                shape,
                rect.size,
                dir,
                zoom,
                scale_factor,
                width,
            ) {
                scene.push(SceneOp::Path {
                    order: DrawOrder(4),
                    origin: rect.origin,
                    path,
                    paint: color.into(),
                });
            } else {
                let radius = Px(0.5 * rect.size.width.0);
                scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect,
                    background: fret_core::Paint::TRANSPARENT.into(),
                    border: Edges::all(Px(width / zoom)),
                    border_paint: fret_core::Paint::Solid(color).into(),
                    corner_radii: Corners::all(radius),
                });
            }
        }
    }
}
