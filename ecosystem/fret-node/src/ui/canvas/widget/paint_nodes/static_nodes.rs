use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;
use crate::ui::{NodeShadowHint, PortShapeHint};
use fret_core::scene::DropShadowV1;
use fret_core::{EffectChain, EffectMode, EffectQuality, EffectStep};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_nodes_static(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        render: &RenderData,
        zoom: f32,
    ) {
        fn shadow_to_drop_shadow_canvas_units(
            rect: Rect,
            zoom: f32,
            shadow: NodeShadowHint,
        ) -> Option<(Rect, DropShadowV1)> {
            let z = if zoom.is_finite() && zoom > 0.0 {
                zoom
            } else {
                1.0
            };

            if !shadow.offset_x_px.is_finite()
                || !shadow.offset_y_px.is_finite()
                || !shadow.blur_radius_px.is_finite()
            {
                return None;
            }

            let blur_canvas = (shadow.blur_radius_px / z).max(0.0);
            let ox_canvas = shadow.offset_x_px / z;
            let oy_canvas = shadow.offset_y_px / z;

            let pad_x = blur_canvas + ox_canvas.abs();
            let pad_y = blur_canvas + oy_canvas.abs();

            let bounds = Rect::new(
                Point::new(Px(rect.origin.x.0 - pad_x), Px(rect.origin.y.0 - pad_y)),
                Size::new(
                    Px(rect.size.width.0 + 2.0 * pad_x),
                    Px(rect.size.height.0 + 2.0 * pad_y),
                ),
            );

            Some((
                bounds,
                DropShadowV1 {
                    offset_px: Point::new(Px(ox_canvas), Px(oy_canvas)),
                    blur_radius_px: Px(blur_canvas),
                    downsample: shadow.downsample,
                    color: shadow.color,
                }
                .sanitize(),
            ))
        }

        let mut node_text_style = self.style.geometry.context_menu_text_style.clone();
        node_text_style.size = Px(node_text_style.size.0 / zoom);
        if let Some(lh) = node_text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let corner = Px(self.style.paint.node_corner_radius / zoom);
        let title_pad = self.style.geometry.node_padding / zoom;
        let title_h = self.style.geometry.node_header_height / zoom;

        for (node, rect, is_selected, title, body, pin_rows, _resize_handles, hint) in &render.nodes
        {
            let rect = *rect;
            let background = hint.background.unwrap_or(self.style.paint.node_background);
            let border = if *is_selected {
                hint.border_selected
                    .or(hint.border)
                    .unwrap_or(self.style.paint.node_border_selected)
            } else {
                hint.border.unwrap_or(self.style.paint.node_border)
            };
            let border_w = Px(1.0 / zoom);

            let paint_override = self
                .paint_overrides
                .as_ref()
                .and_then(|o| o.node_paint_override(*node));

            let body_background: fret_core::scene::PaintBindingV1 = paint_override
                .as_ref()
                .and_then(|o| o.body_background)
                .unwrap_or_else(|| fret_core::Paint::Solid(background).into());

            let header_background: Option<fret_core::scene::PaintBindingV1> = paint_override
                .as_ref()
                .and_then(|o| o.header_background)
                .or_else(|| {
                    hint.header_background
                        .map(|c| fret_core::Paint::Solid(c).into())
                });

            let border_paint: fret_core::scene::PaintBindingV1 = paint_override
                .as_ref()
                .and_then(|o| o.border_paint)
                .unwrap_or_else(|| fret_core::Paint::Solid(border).into());

            let shadow = hint.shadow;
            if let Some(shadow) = shadow
                && let Some((bounds, drop_shadow)) =
                    shadow_to_drop_shadow_canvas_units(rect, zoom, shadow)
            {
                scene.push(SceneOp::PushEffect {
                    bounds,
                    mode: EffectMode::FilterContent,
                    chain: EffectChain::from_steps(&[EffectStep::DropShadowV1(drop_shadow)]),
                    quality: EffectQuality::Auto,
                });
            }

            scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: body_background,

                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),

                corner_radii: Corners::all(corner),
            });

            if let Some(paint) = header_background {
                scene.push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: Rect::new(
                        rect.origin,
                        Size::new(rect.size.width, Px(title_h.min(rect.size.height.0))),
                    ),
                    background: paint,

                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),

                    corner_radii: Corners {
                        top_left: corner,
                        top_right: corner,
                        bottom_right: Px(0.0),
                        bottom_left: Px(0.0),
                    },
                });
            }

            scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: fret_core::Paint::TRANSPARENT.into(),

                border: Edges::all(border_w),
                border_paint,

                corner_radii: Corners::all(corner),
            });

            if shadow.is_some() {
                scene.push(SceneOp::PopEffect);
            }

            if !title.is_empty() {
                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    scale_factor: effective_scale_factor(scale_factor, zoom),
                };
                let (blob, metrics) = self.paint_cache.text_blob(
                    services,
                    title.clone(),
                    &node_text_style,
                    constraints,
                );

                let text_x = Px(rect.origin.x.0 + title_pad);
                let inner_y = rect.origin.y.0 + (title_h - metrics.size.height.0) * 0.5;
                let text_y = Px(inner_y + metrics.baseline.0);
                scene.push(SceneOp::Text {
                    order: DrawOrder(4),
                    origin: Point::new(text_x, text_y),
                    text: blob,
                    paint: (hint
                        .title_text
                        .unwrap_or(self.style.paint.context_menu_text))
                    .into(),
                    outline: None,
                    shadow: None,
                });
            }

            if let Some(body) = body
                && !body.is_empty()
            {
                let pin_rows = (*pin_rows).max(0) as f32;
                let body_top = rect.origin.y.0
                    + (self.style.geometry.node_header_height
                        + self.style.geometry.node_padding
                        + pin_rows * self.style.geometry.pin_row_height
                        + self.style.geometry.node_padding)
                        / zoom;

                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    scale_factor: effective_scale_factor(scale_factor, zoom),
                };
                let (blob, metrics) = self.paint_cache.text_blob(
                    services,
                    body.clone(),
                    &node_text_style,
                    constraints,
                );

                let text_x = Px(rect.origin.x.0 + title_pad);
                let inner_y = body_top + metrics.baseline.0;
                scene.push(SceneOp::Text {
                    order: DrawOrder(4),
                    origin: Point::new(text_x, Px(inner_y)),
                    text: blob,
                    paint: (self.style.paint.context_menu_text).into(),
                    outline: None,
                    shadow: None,
                });
            }
        }

        let pin_r = self.style.geometry.pin_radius / zoom;
        let pin_gap = 8.0 / zoom;

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
                &node_text_style,
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
                paint: (self.style.paint.context_menu_text).into(),
                outline: None,
                shadow: None,
            });
        }

        for (port_id, rect, color, hint) in &render.pins {
            let outer_rect = *rect;
            let mut fill_rect = outer_rect;
            let color = *color;
            let shape = hint.shape.unwrap_or(PortShapeHint::Circle);
            let dir = render.port_labels.get(port_id).map(|l| l.dir);

            if let Some(scale) = hint.inner_scale {
                if scale.is_finite() {
                    if scale > 0.0 {
                        let scale = scale.clamp(0.05, 1.0);
                        let cx = outer_rect.origin.x.0 + 0.5 * outer_rect.size.width.0;
                        let cy = outer_rect.origin.y.0 + 0.5 * outer_rect.size.height.0;
                        let w = outer_rect.size.width.0 * scale;
                        let h = outer_rect.size.height.0 * scale;
                        fill_rect = Rect::new(
                            Point::new(Px(cx - 0.5 * w), Px(cy - 0.5 * h)),
                            Size::new(Px(w), Px(h)),
                        );
                    }
                }
            }

            if hint.inner_scale.unwrap_or(1.0) > 0.0 {
                match shape {
                    PortShapeHint::Circle => {
                        let r = Px(0.5 * fill_rect.size.width.0);
                        scene.push(SceneOp::Quad {
                            order: DrawOrder(4),
                            rect: fill_rect,
                            background: fret_core::Paint::Solid(color).into(),

                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT.into(),

                            corner_radii: Corners::all(r),
                        });
                    }
                    PortShapeHint::Diamond | PortShapeHint::Triangle => {
                        if let Some(path) = self.paint_cache.port_shape_fill_path(
                            services,
                            shape,
                            fill_rect.size,
                            dir,
                            zoom,
                            scale_factor,
                        ) {
                            scene.push(SceneOp::Path {
                                order: DrawOrder(4),
                                origin: fill_rect.origin,
                                path,
                                paint: color.into(),
                            });
                        } else {
                            let r = Px(0.5 * fill_rect.size.width.0);
                            scene.push(SceneOp::Quad {
                                order: DrawOrder(4),
                                rect: fill_rect,
                                background: fret_core::Paint::Solid(color).into(),

                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),

                                corner_radii: Corners::all(r),
                            });
                        }
                    }
                }
            }

            if let Some(stroke) = hint.stroke {
                let w = hint.stroke_width.unwrap_or(1.0);
                if w.is_finite() && w > 0.0 {
                    match shape {
                        PortShapeHint::Circle => {
                            let r = Px(0.5 * outer_rect.size.width.0);
                            scene.push(SceneOp::Quad {
                                order: DrawOrder(4),
                                rect: outer_rect,
                                background: fret_core::Paint::TRANSPARENT.into(),

                                border: Edges::all(Px(w / zoom)),
                                border_paint: fret_core::Paint::Solid(stroke).into(),

                                corner_radii: Corners::all(r),
                            });
                        }
                        PortShapeHint::Diamond | PortShapeHint::Triangle => {
                            if let Some(path) = self.paint_cache.port_shape_stroke_path(
                                services,
                                shape,
                                outer_rect.size,
                                dir,
                                zoom,
                                scale_factor,
                                w,
                            ) {
                                scene.push(SceneOp::Path {
                                    order: DrawOrder(4),
                                    origin: outer_rect.origin,
                                    path,
                                    paint: stroke.into(),
                                });
                            } else {
                                let r = Px(0.5 * outer_rect.size.width.0);
                                scene.push(SceneOp::Quad {
                                    order: DrawOrder(4),
                                    rect: outer_rect,
                                    background: fret_core::Paint::TRANSPARENT.into(),

                                    border: Edges::all(Px(w / zoom)),
                                    border_paint: fret_core::Paint::Solid(stroke).into(),

                                    corner_radii: Corners::all(r),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
