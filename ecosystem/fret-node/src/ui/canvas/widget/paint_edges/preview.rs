use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;
use fret_core::scene::DashPatternV1;
use fret_core::scene::DropShadowV1;
use fret_core::{EffectChain, EffectMode, EffectQuality, EffectStep};

use super::support::glow_bounds_for_edge_route;

#[derive(Debug, Clone, Copy)]
pub(super) struct PreviewTargetState {
    hovered_port: Option<PortId>,
    hovered_port_valid: bool,
    hovered_port_convertible: bool,
    focused_port: Option<PortId>,
    focused_port_valid: bool,
    focused_port_convertible: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PreviewWireStyle {
    pub color: Color,
    pub dash: Option<DashPatternV1>,
}

impl PreviewTargetState {
    pub(super) fn from_widget<M: NodeGraphCanvasMiddleware>(
        widget: &NodeGraphCanvasWith<M>,
    ) -> Self {
        Self {
            hovered_port: widget.interaction.hover_port,
            hovered_port_valid: widget.interaction.hover_port_valid,
            hovered_port_convertible: widget.interaction.hover_port_convertible,
            focused_port: widget.interaction.focused_port,
            focused_port_valid: widget.interaction.focused_port_valid,
            focused_port_convertible: widget.interaction.focused_port_convertible,
        }
    }

    pub(super) fn resolve_target(self, geom: &CanvasGeometry, fallback: Point) -> Point {
        let focused_target = self
            .focused_port
            .filter(|_| self.focused_port_valid || self.focused_port_convertible);
        self.hovered_port
            .filter(|_| self.hovered_port_valid || self.hovered_port_convertible)
            .or(focused_target)
            .and_then(|port| geom.port_center(port))
            .unwrap_or(fallback)
    }

    pub(super) fn resolve_style(
        self,
        interaction_hint: crate::ui::InteractionChromeHint,
        preview_color: Color,
    ) -> PreviewWireStyle {
        let invalid = interaction_hint.invalid.unwrap_or_else(|| Color {
            a: 0.95,
            ..Color::from_srgb_hex_rgb(0xe6_59_59)
        });
        let convertible = interaction_hint.convertible.unwrap_or_else(|| Color {
            a: 0.95,
            ..Color::from_srgb_hex_rgb(0xf2_bf_33)
        });
        let preview = interaction_hint.preview_wire.unwrap_or(preview_color);
        let dash_invalid = interaction_hint.dash_invalid;
        let dash_preview = interaction_hint.dash_preview;

        let (color, dash) = if self.hovered_port.is_some()
            && !self.hovered_port_valid
            && !self.hovered_port_convertible
        {
            (invalid, dash_invalid)
        } else if self.hovered_port.is_some()
            && self.hovered_port_convertible
            && !self.hovered_port_valid
        {
            (convertible, dash_preview)
        } else if self.focused_port.is_some()
            && !self.focused_port_valid
            && !self.focused_port_convertible
            && self.hovered_port.is_none()
        {
            (invalid, dash_invalid)
        } else if self.focused_port.is_some()
            && self.focused_port_convertible
            && !self.focused_port_valid
            && self.hovered_port.is_none()
        {
            (convertible, dash_preview)
        } else {
            (preview, dash_preview)
        };

        PreviewWireStyle { color, dash }
    }
}

pub(super) fn push_drop_marker(scene: &mut fret_core::Scene, pos: Point, color: Color, zoom: f32) {
    let z = zoom.max(1.0e-6);
    let r = 7.0 / z;
    let border_w = 2.0 / z;
    let rect = Rect::new(
        Point::new(Px(pos.x.0 - r), Px(pos.y.0 - r)),
        Size::new(Px(2.0 * r), Px(2.0 * r)),
    );
    scene.push(SceneOp::Quad {
        order: DrawOrder(4),
        rect,
        background: fret_core::Paint::TRANSPARENT.into(),
        border: Edges::all(Px(border_w)),
        border_paint: fret_core::Paint::Solid(color).into(),
        corner_radii: Corners::all(Px(r)),
    });

    let arm = 10.0 / z;
    let thick = (2.0 / z).max(0.5 / z);
    let h_rect = Rect::new(
        Point::new(Px(pos.x.0 - arm * 0.5), Px(pos.y.0 - thick * 0.5)),
        Size::new(Px(arm), Px(thick)),
    );
    let v_rect = Rect::new(
        Point::new(Px(pos.x.0 - thick * 0.5), Px(pos.y.0 - arm * 0.5)),
        Size::new(Px(thick), Px(arm)),
    );
    scene.push(SceneOp::Quad {
        order: DrawOrder(4),
        rect: h_rect,
        background: fret_core::Paint::Solid(color).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
    scene.push(SceneOp::Quad {
        order: DrawOrder(4),
        rect: v_rect,
        background: fret_core::Paint::Solid(color).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(0.0)),
    });
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_wire_drag_preview<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        render: &RenderData,
        geom: &CanvasGeometry,
        zoom: f32,
        interaction_hint: crate::ui::InteractionChromeHint,
        outline_budget: &mut WorkBudget,
        outline_budget_skipped: &mut u32,
    ) {
        let Some(wire_drag) = self.interaction.wire_drag.clone() else {
            return;
        };

        let target_state = PreviewTargetState::from_widget(self);
        let to = target_state.resolve_target(geom, wire_drag.pos);
        let style =
            target_state.resolve_style(interaction_hint, self.style.paint.wire_color_preview);

        let mut draw_preview = |from: Point| {
            self.push_preview_wire_path(
                cx,
                from,
                to,
                zoom,
                interaction_hint,
                style,
                outline_budget,
                outline_budget_skipped,
            );
        };

        match &wire_drag.kind {
            WireDragKind::New { from, bundle } => {
                let ports = if bundle.is_empty() {
                    std::slice::from_ref(from)
                } else {
                    bundle.as_slice()
                };
                for port in ports {
                    if let Some(from) = render.port_centers.get(port).copied() {
                        draw_preview(from);
                    }
                }
            }
            WireDragKind::Reconnect { fixed, .. } => {
                if let Some(from) = render.port_centers.get(fixed).copied() {
                    draw_preview(from);
                }
            }
            WireDragKind::ReconnectMany { edges } => {
                for (_edge, _endpoint, fixed) in edges {
                    if let Some(from) = render.port_centers.get(fixed).copied() {
                        draw_preview(from);
                    }
                }
            }
        }
    }

    fn push_preview_wire_path<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        from: Point,
        to: Point,
        zoom: f32,
        interaction_hint: crate::ui::InteractionChromeHint,
        style: PreviewWireStyle,
        outline_budget: &mut WorkBudget,
        outline_budget_skipped: &mut u32,
    ) {
        if let Some(outline) = interaction_hint.wire_outline_preview
            && outline.width_mul.is_finite()
            && outline.width_mul > 1.0e-3
            && outline.color.a > 0.0
        {
            if !outline_budget.try_consume(1) {
                *outline_budget_skipped = outline_budget_skipped.saturating_add(1);
            } else {
                let outline_width = self.style.geometry.wire_width * outline.width_mul.max(0.0);
                if let Some(path) = self.paint_cache.wire_path(
                    cx.services,
                    EdgeRouteKind::Bezier,
                    from,
                    to,
                    zoom,
                    cx.scale_factor,
                    outline_width,
                    style.dash,
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

        let glow = interaction_hint.wire_glow_preview;
        let glow_bounds = glow.and_then(|g| {
            glow_bounds_for_edge_route(
                EdgeRouteKind::Bezier,
                from,
                to,
                zoom,
                self.style.geometry.wire_width,
                g.blur_radius_px,
            )
        });
        let mut glow_pushed = false;
        if let Some(glow) = glow
            && let Some(bounds) = glow_bounds
        {
            let z = zoom.max(1.0e-6);
            let blur_canvas = (glow.blur_radius_px / z).max(0.0);
            let mut glow_color = style.color;
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
            glow_pushed = true;
        }

        if let Some(path) = self.paint_cache.wire_path(
            cx.services,
            EdgeRouteKind::Bezier,
            from,
            to,
            zoom,
            cx.scale_factor,
            self.style.geometry.wire_width,
            style.dash,
        ) {
            cx.scene.push(SceneOp::Path {
                order: DrawOrder(2),
                origin: Point::new(Px(0.0), Px(0.0)),
                path,
                paint: style.color.into(),
            });
        }

        if glow_pushed {
            cx.scene.push(SceneOp::PopEffect);
        }
    }
}
