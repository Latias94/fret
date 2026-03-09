use crate::ui::canvas::widget::*;
use fret_core::scene::DashPatternV1;
use fret_core::scene::DropShadowV1;
use fret_core::{EffectChain, EffectMode, EffectQuality, EffectStep};

use super::markers::WireHighlightPaint;
use super::support::{glow_bounds_for_custom_path, glow_bounds_for_edge_route, stable_hash_u64};

#[derive(Debug, Clone, Copy)]
pub(super) struct EdgeChromeState {
    pub glow_pushed: bool,
    pub highlight: Option<WireHighlightPaint>,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn prepare_edge_chrome<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        custom: Option<&crate::ui::edge_types::EdgeCustomPath>,
        interaction_hint: crate::ui::InteractionChromeHint,
        edge_selected: bool,
        edge_hovered: bool,
        edge_id: EdgeId,
        from: Point,
        to: Point,
        route: EdgeRouteKind,
        color: Color,
        width: f32,
        dash: Option<DashPatternV1>,
        zoom: f32,
        outline_budget: &mut WorkBudget,
        outline_budget_skipped: &mut u32,
        highlight_budget: &mut WorkBudget,
        highlight_budget_skipped: &mut u32,
    ) -> EdgeChromeState {
        self.push_edge_outline(
            cx,
            custom,
            interaction_hint,
            edge_selected,
            edge_id,
            from,
            to,
            route,
            width,
            dash,
            zoom,
            outline_budget,
            outline_budget_skipped,
        );

        let glow_pushed = self.push_edge_glow(
            cx,
            custom,
            interaction_hint,
            edge_selected,
            route,
            from,
            to,
            color,
            width,
            zoom,
        );

        let highlight = self.resolve_edge_highlight(
            interaction_hint,
            edge_selected,
            edge_hovered,
            color,
            width,
            highlight_budget,
            highlight_budget_skipped,
        );

        EdgeChromeState {
            glow_pushed,
            highlight,
        }
    }

    fn push_edge_outline<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        custom: Option<&crate::ui::edge_types::EdgeCustomPath>,
        interaction_hint: crate::ui::InteractionChromeHint,
        edge_selected: bool,
        edge_id: EdgeId,
        from: Point,
        to: Point,
        route: EdgeRouteKind,
        width: f32,
        dash: Option<DashPatternV1>,
        zoom: f32,
        outline_budget: &mut WorkBudget,
        outline_budget_skipped: &mut u32,
    ) {
        let outline = if edge_selected {
            interaction_hint.wire_outline_selected
        } else {
            interaction_hint.wire_outline_base
        };
        let Some(outline) = outline else {
            return;
        };
        if !outline.width_mul.is_finite() || outline.width_mul <= 1.0e-3 || outline.color.a <= 0.0 {
            return;
        }
        if !outline_budget.try_consume(1) {
            *outline_budget_skipped = outline_budget_skipped.saturating_add(1);
            return;
        }

        let outline_width = width * outline.width_mul.max(0.0);
        if let Some(custom) = custom {
            let dash_key =
                dash.map(|p| (p.dash.0.to_bits(), p.gap.0.to_bits(), p.phase.0.to_bits()));
            let key = (custom.cache_key, edge_id, outline_width.to_bits(), dash_key);
            let outline_cache_key = stable_hash_u64(1, &key);
            if let Some(path) = self.paint_cache.wire_path_from_commands(
                cx.services,
                outline_cache_key,
                &custom.commands,
                zoom,
                cx.scale_factor,
                outline_width,
                dash,
            ) {
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    paint: outline.color.into(),
                });
            }
            return;
        }

        if let Some(path) = self.paint_cache.wire_path(
            cx.services,
            route,
            from,
            to,
            zoom,
            cx.scale_factor,
            outline_width,
            dash,
        ) {
            cx.scene.push(SceneOp::Path {
                order: DrawOrder(2),
                origin: Point::new(Px(0.0), Px(0.0)),
                path,
                paint: outline.color.into(),
            });
        }
    }

    fn push_edge_glow<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        custom: Option<&crate::ui::edge_types::EdgeCustomPath>,
        interaction_hint: crate::ui::InteractionChromeHint,
        edge_selected: bool,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        color: Color,
        width: f32,
        zoom: f32,
    ) -> bool {
        let glow = edge_selected
            .then_some(interaction_hint.wire_glow_selected)
            .flatten();
        let glow_bounds = glow.and_then(|glow| {
            if let Some(custom) = custom {
                glow_bounds_for_custom_path(&custom.commands, zoom, width, glow.blur_radius_px)
            } else {
                glow_bounds_for_edge_route(route, from, to, zoom, width, glow.blur_radius_px)
            }
        });

        let Some(glow) = glow else {
            return false;
        };
        let Some(bounds) = glow_bounds else {
            return false;
        };

        let z = zoom.max(1.0e-6);
        let blur_canvas = (glow.blur_radius_px / z).max(0.0);
        let mut glow_color = color;
        let alpha_mul = if glow.alpha_mul.is_finite() {
            glow.alpha_mul.clamp(0.0, 1.0)
        } else {
            0.0
        };
        glow_color.a = (glow_color.a * alpha_mul).clamp(0.0, 1.0);
        cx.scene.push(SceneOp::PushEffect {
            bounds,
            mode: EffectMode::FilterContent,
            chain: EffectChain::from_steps(&[EffectStep::DropShadowV1(
                DropShadowV1 {
                    offset_px: Point::new(Px(0.0), Px(0.0)),
                    blur_radius_px: Px(blur_canvas),
                    downsample: glow.downsample,
                    color: glow_color,
                }
                .sanitize(),
            )]),
            quality: EffectQuality::Auto,
        });
        true
    }

    fn resolve_edge_highlight(
        &mut self,
        interaction_hint: crate::ui::InteractionChromeHint,
        edge_selected: bool,
        edge_hovered: bool,
        color: Color,
        width: f32,
        highlight_budget: &mut WorkBudget,
        highlight_budget_skipped: &mut u32,
    ) -> Option<WireHighlightPaint> {
        let highlight_hint = if edge_hovered {
            interaction_hint.wire_highlight_hovered
        } else if edge_selected {
            interaction_hint.wire_highlight_selected
        } else {
            None
        };
        let hint = highlight_hint?;

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
        let highlight_width = width * width_mul;
        if !highlight_width.is_finite() || highlight_width <= 1.0e-3 || alpha_mul <= 1.0e-6 {
            return None;
        }
        if !highlight_budget.try_consume(1) {
            *highlight_budget_skipped = highlight_budget_skipped.saturating_add(1);
            return None;
        }

        let mut highlight_color = hint.color.unwrap_or_else(|| {
            let t = 0.45;
            Color {
                r: color.r + (1.0 - color.r) * t,
                g: color.g + (1.0 - color.g) * t,
                b: color.b + (1.0 - color.b) * t,
                a: color.a,
            }
        });
        highlight_color.a = (highlight_color.a * alpha_mul).clamp(0.0, 1.0);
        (highlight_color.a > 0.0).then_some(WireHighlightPaint {
            width: highlight_width,
            color: highlight_color,
        })
    }
}
