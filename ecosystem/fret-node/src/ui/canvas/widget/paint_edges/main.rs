use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;
use fret_core::scene::DashPatternV1;
use fret_core::scene::DropShadowV1;
use fret_core::scene::PaintBindingV1;
use fret_core::{EffectChain, EffectMode, EffectQuality, EffectStep};

use super::markers::WireHighlightPaint;
use super::preview::push_drop_marker;
use super::support::{glow_bounds_for_custom_path, glow_bounds_for_edge_route, stable_hash_u64};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_edges<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        render: &RenderData,
        geom: &CanvasGeometry,
        zoom: f32,
        view_interacting: bool,
    ) {
        #[derive(Debug, Clone)]
        struct EdgePaint {
            id: EdgeId,
            from: Point,
            to: Point,
            color: Color,
            paint: PaintBindingV1,
            width: f32,
            route: EdgeRouteKind,
            dash: Option<DashPatternV1>,
            start_marker: Option<crate::ui::presenter::EdgeMarker>,
            end_marker: Option<crate::ui::presenter::EdgeMarker>,
            selected: bool,
            hovered: bool,
        }

        let interaction_hint = if let Some(skin) = self.skin.as_ref() {
            self.graph
                .read_ref(cx.app, |g| skin.interaction_chrome_hint(g, &self.style))
                .ok()
                .unwrap_or_default()
        } else {
            crate::ui::InteractionChromeHint::default()
        };

        let edge_insert_marker_request = self
            .interaction
            .edge_insert_drag
            .as_ref()
            .map(|d| (d.edge, d.pos));
        let insert_node_drag_preview = self.interaction.insert_node_drag_preview.as_ref();

        let mut edges_normal: Vec<EdgePaint> = Vec::new();
        let mut edges_selected: Vec<EdgePaint> = Vec::new();
        let mut edges_hovered: Vec<EdgePaint> = Vec::new();
        let elevate_edges_on_select = snapshot.interaction.elevate_edges_on_select;

        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
        let custom_paths = self.collect_custom_edge_paths(&*cx.app, &render.edges, zoom);
        let edge_insert_marker: Option<(Point, Color)> =
            edge_insert_marker_request.and_then(|(edge_id, pos)| {
                render.edges.iter().find(|e| e.id == edge_id).map(|e| {
                    (
                        custom_paths
                            .get(&edge_id)
                            .map(|custom| {
                                closest_point_on_path(&custom.commands, bezier_steps, pos)
                            })
                            .unwrap_or_else(|| {
                                closest_point_on_edge_route(
                                    e.hint.route,
                                    e.from,
                                    e.to,
                                    zoom,
                                    bezier_steps,
                                    pos,
                                )
                            }),
                        e.color,
                    )
                })
            });

        let insert_node_drag_marker: Option<(Point, Color)> =
            insert_node_drag_preview.as_ref().map(|p| {
                if let Some(edge_id) = p.edge
                    && let Some(edge) = render.edges.iter().find(|e| e.id == edge_id)
                {
                    (
                        custom_paths
                            .get(&edge_id)
                            .map(|custom| {
                                closest_point_on_path(&custom.commands, bezier_steps, p.pos)
                            })
                            .unwrap_or_else(|| {
                                closest_point_on_edge_route(
                                    edge.hint.route,
                                    edge.from,
                                    edge.to,
                                    zoom,
                                    bezier_steps,
                                    p.pos,
                                )
                            }),
                        edge.color,
                    )
                } else {
                    (p.pos, self.style.paint.wire_color_preview)
                }
            });

        for edge in &render.edges {
            let mut width = self.style.geometry.wire_width * edge.hint.width_mul.max(0.0);
            if edge.selected {
                width *= self.style.paint.wire_width_selected_mul;
            }
            if edge.hovered {
                width *= self.style.paint.wire_width_hover_mul;
            }

            let route = edge.hint.route;

            let paint = EdgePaint {
                id: edge.id,
                from: edge.from,
                to: edge.to,
                color: edge.color,
                paint: edge.paint,
                width,
                route,
                dash: edge.hint.dash,
                start_marker: edge.hint.start_marker.clone(),
                end_marker: edge.hint.end_marker.clone(),
                selected: edge.selected,
                hovered: edge.hovered,
            };

            if edge.hovered {
                edges_hovered.push(paint);
            } else if edge.selected && elevate_edges_on_select {
                edges_selected.push(paint);
            } else {
                edges_normal.push(paint);
            }
        }

        let marker_budget_limit = Self::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut marker_budget = WorkBudget::new(marker_budget_limit);
        let mut marker_budget_skipped: u32 = 0;
        let mut wire_budget = WorkBudget::new(u32::MAX / 2);
        let outline_budget_limit =
            Self::EDGE_WIRE_OUTLINE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut outline_budget = WorkBudget::new(outline_budget_limit);
        let mut outline_budget_skipped: u32 = 0;
        let highlight_budget_limit =
            Self::EDGE_WIRE_HIGHLIGHT_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut highlight_budget = WorkBudget::new(highlight_budget_limit);
        let mut highlight_budget_skipped: u32 = 0;

        for edge in edges_normal
            .into_iter()
            .chain(edges_selected)
            .chain(edges_hovered)
        {
            let outline = if edge.selected {
                interaction_hint.wire_outline_selected
            } else {
                interaction_hint.wire_outline_base
            };
            if let Some(outline) = outline
                && outline.width_mul.is_finite()
                && outline.width_mul > 1.0e-3
                && outline.color.a > 0.0
            {
                if !outline_budget.try_consume(1) {
                    outline_budget_skipped = outline_budget_skipped.saturating_add(1);
                } else {
                    let outline_width = edge.width * outline.width_mul.max(0.0);
                    if let Some(custom) = custom_paths.get(&edge.id) {
                        let dash = edge
                            .dash
                            .map(|p| (p.dash.0.to_bits(), p.gap.0.to_bits(), p.phase.0.to_bits()));
                        let key = (custom.cache_key, outline_width.to_bits(), dash);
                        let outline_cache_key = stable_hash_u64(1, &key);
                        if let Some(path) = self.paint_cache.wire_path_from_commands(
                            cx.services,
                            outline_cache_key,
                            &custom.commands,
                            zoom,
                            cx.scale_factor,
                            outline_width,
                            edge.dash,
                        ) {
                            cx.scene.push(SceneOp::Path {
                                order: DrawOrder(2),
                                origin: Point::new(Px(0.0), Px(0.0)),
                                path,
                                paint: outline.color.into(),
                            });
                        }
                    } else if let Some(path) = self.paint_cache.wire_path(
                        cx.services,
                        edge.route,
                        edge.from,
                        edge.to,
                        zoom,
                        cx.scale_factor,
                        outline_width,
                        edge.dash,
                    ) {
                        cx.scene.push(SceneOp::Path {
                            order: DrawOrder(2),
                            origin: Point::new(Px(0.0), Px(0.0)),
                            path,
                            paint: outline.color.into(),
                        });
                    }
                }
            }

            let glow = edge
                .selected
                .then_some(interaction_hint.wire_glow_selected)
                .flatten();

            let glow_bounds = glow.and_then(|g| {
                if let Some(custom) = custom_paths.get(&edge.id) {
                    glow_bounds_for_custom_path(
                        &custom.commands,
                        zoom,
                        edge.width,
                        g.blur_radius_px,
                    )
                } else {
                    glow_bounds_for_edge_route(
                        edge.route,
                        edge.from,
                        edge.to,
                        zoom,
                        edge.width,
                        g.blur_radius_px,
                    )
                }
            });

            let mut glow_pushed = false;
            if let Some(glow) = glow
                && let Some(bounds) = glow_bounds
            {
                let z = zoom.max(1.0e-6);
                let blur_canvas = (glow.blur_radius_px / z).max(0.0);
                let mut color = edge.color;
                let alpha_mul = if glow.alpha_mul.is_finite() {
                    glow.alpha_mul.clamp(0.0, 1.0)
                } else {
                    0.0
                };
                color.a = (color.a * alpha_mul).clamp(0.0, 1.0);
                cx.scene.push(SceneOp::PushEffect {
                    bounds,
                    mode: EffectMode::FilterContent,
                    chain: EffectChain::from_steps(&[EffectStep::DropShadowV1(
                        DropShadowV1 {
                            offset_px: Point::new(Px(0.0), Px(0.0)),
                            blur_radius_px: Px(blur_canvas),
                            downsample: glow.downsample,
                            color,
                        }
                        .sanitize(),
                    )]),
                    quality: EffectQuality::Auto,
                });
                glow_pushed = true;
            }

            let highlight_hint = if edge.hovered {
                interaction_hint.wire_highlight_hovered
            } else if edge.selected {
                interaction_hint.wire_highlight_selected
            } else {
                None
            };
            let highlight = if let Some(hint) = highlight_hint {
                let width_mul = if hint.width_mul.is_finite() {
                    hint.width_mul.clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let alpha_mul = if hint.alpha_mul.is_finite() {
                    hint.alpha_mul.clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let width = edge.width * width_mul;
                if width.is_finite() && width > 1.0e-3 && alpha_mul > 1.0e-6 {
                    if !highlight_budget.try_consume(1) {
                        highlight_budget_skipped = highlight_budget_skipped.saturating_add(1);
                        None
                    } else {
                        let mut color = hint.color.unwrap_or_else(|| {
                            let t = 0.45;
                            Color {
                                r: edge.color.r + (1.0 - edge.color.r) * t,
                                g: edge.color.g + (1.0 - edge.color.g) * t,
                                b: edge.color.b + (1.0 - edge.color.b) * t,
                                a: edge.color.a,
                            }
                        });
                        color.a = (color.a * alpha_mul).clamp(0.0, 1.0);
                        (color.a > 0.0).then_some(WireHighlightPaint { width, color })
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let (_stop, skipped) = if let Some(custom) = custom_paths.get(&edge.id) {
                let fallback = Point::new(
                    Px(edge.to.x.0 - edge.from.x.0),
                    Px(edge.to.y.0 - edge.from.y.0),
                );
                let (t0, t1) =
                    path_start_end_tangents(&custom.commands).unwrap_or((fallback, fallback));
                self.push_edge_custom_wire_and_markers_budgeted(
                    cx.scene,
                    cx.services,
                    custom.cache_key,
                    &custom.commands,
                    t0,
                    t1,
                    zoom,
                    cx.scale_factor,
                    edge.from,
                    edge.to,
                    edge.paint,
                    edge.color,
                    edge.width,
                    edge.dash,
                    highlight,
                    edge.start_marker.as_ref(),
                    edge.end_marker.as_ref(),
                    &mut wire_budget,
                    &mut marker_budget,
                    false,
                )
            } else {
                self.push_edge_wire_and_markers_budgeted(
                    cx.scene,
                    cx.services,
                    zoom,
                    cx.scale_factor,
                    edge.route,
                    edge.from,
                    edge.to,
                    edge.paint,
                    edge.color,
                    edge.width,
                    edge.dash,
                    highlight,
                    edge.start_marker.as_ref(),
                    edge.end_marker.as_ref(),
                    &mut wire_budget,
                    &mut marker_budget,
                    false,
                )
            };
            marker_budget_skipped = marker_budget_skipped.saturating_add(skipped);

            if glow_pushed {
                cx.scene.push(SceneOp::PopEffect);
            }
        }

        super::super::redraw_request::request_paint_redraw_if(cx, marker_budget_skipped > 0);
        super::super::redraw_request::request_paint_redraw_if(cx, outline_budget_skipped > 0);
        super::super::redraw_request::request_paint_redraw_if(cx, highlight_budget_skipped > 0);

        if let Some((pos, color)) = edge_insert_marker {
            push_drop_marker(cx.scene, pos, color, zoom);
        }
        if let Some((pos, color)) = insert_node_drag_marker {
            push_drop_marker(cx.scene, pos, color, zoom);
        }

        if render
            .edges
            .iter()
            .any(|e| e.hint.label.as_ref().is_some_and(|s| !s.is_empty()))
        {
            let label_budget_limit =
                Self::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
            let mut label_budget = WorkBudget::new(label_budget_limit);
            let (next_edge, skipped_by_budget) = self.paint_edge_labels_static_budgeted(
                cx.scene,
                cx.services,
                cx.scale_factor,
                &render.edges,
                (!custom_paths.is_empty()).then_some(&custom_paths),
                bezier_steps,
                zoom,
                0,
                &mut label_budget,
            );
            let mut label_budget_skipped: u32 = 0;
            if skipped_by_budget && next_edge < render.edges.len() {
                label_budget_skipped = 1;
                super::super::redraw_request::request_paint_redraw(cx);
            }

            if let Some(window) = cx.window {
                let frame_id = cx.app.frame_id().0;
                let key = CanvasCacheKey {
                    window: window.data().as_ffi(),
                    node: cx.node.data().as_ffi(),
                    name: "fret-node.canvas.edge_labels_budget",
                };
                cx.app
                    .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                        registry.record_work_budget(
                            key,
                            frame_id,
                            label_budget.used().saturating_add(label_budget_skipped),
                            label_budget_limit,
                            label_budget.used(),
                            label_budget_skipped,
                        );
                    });
            }
        }
        if let Some(window) = cx.window {
            let frame_id = cx.app.frame_id().0;
            let key = CanvasCacheKey {
                window: window.data().as_ffi(),
                node: cx.node.data().as_ffi(),
                name: "fret-node.canvas.edge_markers_budget",
            };
            cx.app
                .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                    registry.record_work_budget(
                        key,
                        frame_id,
                        marker_budget.used().saturating_add(marker_budget_skipped),
                        marker_budget_limit,
                        marker_budget.used(),
                        marker_budget_skipped,
                    );
                });
        }

        self.paint_wire_drag_preview(
            cx,
            render,
            geom,
            zoom,
            interaction_hint,
            &mut outline_budget,
            &mut outline_budget_skipped,
        );
    }
}
