use crate::ui::canvas::widget::*;
use fret_core::scene::PaintBindingV1;
use fret_core::scene::PaintEvalSpaceV1;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_edge_overlays_selected_hovered<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        zoom: f32,
    ) {
        let hovered_edge = self.interaction.hover_edge;
        let selected_edges: HashSet<EdgeId> = snapshot.selected_edges.iter().copied().collect();

        let mut overlay_edges: Vec<EdgeId> = snapshot.selected_edges.clone();
        if let Some(hovered) = hovered_edge
            && !selected_edges.contains(&hovered)
        {
            overlay_edges.push(hovered);
        }

        if overlay_edges.is_empty() {
            return;
        }

        fn marker_paint_binding_for_wire(paint: PaintBindingV1, color: Color) -> PaintBindingV1 {
            if paint.eval_space == PaintEvalSpaceV1::StrokeS01 {
                color.into()
            } else {
                paint
            }
        }

        struct OverlayEdgeDraw {
            from: Point,
            to: Point,
            hint: EdgeRenderHint,
            paint: PaintBindingV1,
            marker_paint: PaintBindingV1,
            width: f32,
        }

        let presenter = &self.presenter;
        let edge_types = self.edge_types.as_ref();
        let style = &self.style;

        let mut edges_to_draw: Vec<OverlayEdgeDraw> = Vec::new();
        let _ = self.graph.read_ref(cx.app, |g| {
            for edge_id in &overlay_edges {
                let Some(edge) = g.edges.get(edge_id) else {
                    continue;
                };
                let Some(from) = geom.port_center(edge.from) else {
                    continue;
                };
                let Some(to) = geom.port_center(edge.to) else {
                    continue;
                };

                let base_hint = presenter.edge_render_hint(g, *edge_id, style);
                let hint = if let Some(edge_types) = edge_types {
                    edge_types.apply(g, *edge_id, style, base_hint)
                } else {
                    base_hint
                }
                .normalized();

                let mut hint = hint;
                let paint_override = self
                    .paint_overrides
                    .as_ref()
                    .and_then(|o| o.edge_paint_override(*edge_id));
                if let Some(ov) = paint_override {
                    if let Some(dash) = ov.dash {
                        hint.dash = Some(dash);
                    }
                    if let Some(w) = ov.stroke_width_mul {
                        hint.width_mul = hint.width_mul * w;
                    }
                }
                hint = hint.normalized();

                let mut color = presenter.edge_color(g, *edge_id, style);
                if let Some(override_color) = hint.color {
                    color = override_color;
                }

                let mut paint: PaintBindingV1 = color.into();
                if let Some(ov) = paint_override {
                    if let Some(p) = ov.stroke_paint {
                        paint = p;
                    }
                }

                let mut width = style.geometry.wire_width * hint.width_mul;
                let is_selected = selected_edges.contains(edge_id);
                let is_hovered = hovered_edge == Some(*edge_id);
                if is_selected {
                    width *= style.paint.wire_width_selected_mul;
                }
                if is_hovered {
                    width *= style.paint.wire_width_hover_mul;
                }

                edges_to_draw.push(OverlayEdgeDraw {
                    from,
                    to,
                    hint,
                    paint,
                    marker_paint: marker_paint_binding_for_wire(paint, color),
                    width,
                });
            }
            Some(())
        });

        let mut marker_budget = WorkBudget::new(u32::MAX / 2);
        for edge in edges_to_draw {
            if let Some(path) = self.paint_cache.wire_path(
                cx.services,
                edge.hint.route,
                edge.from,
                edge.to,
                zoom,
                cx.scale_factor,
                edge.width,
                edge.hint.dash,
            ) {
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    paint: edge.paint,
                });
            }

            if let Some(marker) = edge.hint.end_marker.as_ref() {
                let (path, _skipped) = self.paint_cache.edge_end_marker_path_budgeted(
                    cx.services,
                    edge.hint.route,
                    edge.from,
                    edge.to,
                    zoom,
                    cx.scale_factor,
                    marker,
                    self.style.geometry.pin_radius,
                    &mut marker_budget,
                );
                if let Some(path) = path {
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        paint: edge.marker_paint,
                    });
                }
            }

            if let Some(marker) = edge.hint.start_marker.as_ref() {
                let (path, _skipped) = self.paint_cache.edge_start_marker_path_budgeted(
                    cx.services,
                    edge.hint.route,
                    edge.from,
                    edge.to,
                    zoom,
                    cx.scale_factor,
                    marker,
                    self.style.geometry.pin_radius,
                    &mut marker_budget,
                );
                if let Some(path) = path {
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        paint: edge.marker_paint,
                    });
                }
            }
        }
    }
}
