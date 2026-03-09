use crate::ui::PortShapeHint;
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
            let (blob, metrics) = self.paint_cache.text_blob(
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
                paint: self.style.paint.context_menu_text.into(),
                outline: None,
                shadow: None,
            });
        }
    }

    pub(super) fn paint_static_port_shapes(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        render: &RenderData,
        zoom: f32,
    ) {
        for (port_id, rect, color, hint) in &render.pins {
            let outer_rect = *rect;
            let fill_rect = scaled_inner_port_rect(outer_rect, hint.inner_scale);
            let color = *color;
            let shape = hint.shape.unwrap_or(PortShapeHint::Circle);
            let dir = render.port_labels.get(port_id).map(|label| label.dir);

            if hint.inner_scale.unwrap_or(1.0) > 0.0 {
                self.paint_static_port_fill(
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
                    self.paint_static_port_stroke(
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

    #[allow(clippy::too_many_arguments)]
    fn paint_static_port_fill(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        rect: Rect,
        shape: PortShapeHint,
        dir: Option<PortDirection>,
        color: Color,
        zoom: f32,
    ) {
        match shape {
            PortShapeHint::Circle => {
                let radius = Px(0.5 * rect.size.width.0);
                scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect,
                    background: fret_core::Paint::Solid(color).into(),
                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),
                    corner_radii: Corners::all(radius),
                });
            }
            PortShapeHint::Diamond | PortShapeHint::Triangle => {
                if let Some(path) = self.paint_cache.port_shape_fill_path(
                    services,
                    shape,
                    rect.size,
                    dir,
                    zoom,
                    scale_factor,
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
                        background: fret_core::Paint::Solid(color).into(),
                        border: Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT.into(),
                        corner_radii: Corners::all(radius),
                    });
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_static_port_stroke(
        &mut self,
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
                if let Some(path) = self.paint_cache.port_shape_stroke_path(
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
}

fn scaled_inner_port_rect(rect: Rect, inner_scale: Option<f32>) -> Rect {
    let mut fill_rect = rect;
    if let Some(scale) = inner_scale
        && scale.is_finite()
        && scale > 0.0
    {
        let scale = scale.clamp(0.05, 1.0);
        let center_x = rect.origin.x.0 + 0.5 * rect.size.width.0;
        let center_y = rect.origin.y.0 + 0.5 * rect.size.height.0;
        let width = rect.size.width.0 * scale;
        let height = rect.size.height.0 * scale;
        fill_rect = Rect::new(
            Point::new(Px(center_x - 0.5 * width), Px(center_y - 0.5 * height)),
            Size::new(Px(width), Px(height)),
        );
    }
    fill_rect
}
