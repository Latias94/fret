use super::super::support::glow_bounds_for_edge_route;
use super::*;
use fret_core::scene::DropShadowV1;
use fret_core::{EffectChain, EffectMode, EffectQuality, EffectStep};

fn push_preview_outline<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    from: Point,
    to: Point,
    zoom: f32,
    outline: crate::ui::WireOutlineHint,
    dash: Option<DashPatternV1>,
    outline_budget: &mut WorkBudget,
    outline_budget_skipped: &mut u32,
) {
    if !outline.width_mul.is_finite() || outline.width_mul <= 1.0e-3 || outline.color.a <= 0.0 {
        return;
    }

    if !outline_budget.try_consume(1) {
        *outline_budget_skipped = outline_budget_skipped.saturating_add(1);
        return;
    }

    let outline_width = canvas.style.geometry.wire_width * outline.width_mul.max(0.0);
    if let Some(path) = canvas.paint_cache.wire_path(
        cx.services,
        EdgeRouteKind::Bezier,
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

fn push_preview_glow<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    from: Point,
    to: Point,
    zoom: f32,
    wire_width: f32,
    glow: crate::ui::WireGlowHint,
    style: PreviewWireStyle,
) -> bool {
    let Some(bounds) = glow_bounds_for_edge_route(
        EdgeRouteKind::Bezier,
        from,
        to,
        zoom,
        wire_width,
        glow.blur_radius_px,
    ) else {
        return false;
    };

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
    true
}

pub(super) fn push_preview_wire_path<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    from: Point,
    to: Point,
    zoom: f32,
    interaction_hint: crate::ui::InteractionChromeHint,
    style: PreviewWireStyle,
    outline_budget: &mut WorkBudget,
    outline_budget_skipped: &mut u32,
) {
    if let Some(outline) = interaction_hint.wire_outline_preview {
        push_preview_outline(
            canvas,
            cx,
            from,
            to,
            zoom,
            outline,
            style.dash,
            outline_budget,
            outline_budget_skipped,
        );
    }

    let glow_pushed = interaction_hint.wire_glow_preview.is_some_and(|glow| {
        push_preview_glow(
            cx,
            from,
            to,
            zoom,
            canvas.style.geometry.wire_width,
            glow,
            style,
        )
    });

    if let Some(path) = canvas.paint_cache.wire_path(
        cx.services,
        EdgeRouteKind::Bezier,
        from,
        to,
        zoom,
        cx.scale_factor,
        canvas.style.geometry.wire_width,
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
