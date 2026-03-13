use super::super::support::{glow_bounds_for_custom_path, glow_bounds_for_edge_route};
use super::*;
use fret_core::scene::DropShadowV1;
use fret_core::{EffectChain, EffectMode, EffectQuality, EffectStep};

fn glow_hint(
    interaction_hint: crate::ui::InteractionChromeHint,
    edge_selected: bool,
) -> Option<crate::ui::WireGlowHint> {
    edge_selected
        .then_some(interaction_hint.wire_glow_selected)
        .flatten()
}

fn glow_bounds(
    custom: Option<&crate::ui::edge_types::EdgeCustomPath>,
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    width: f32,
    glow: crate::ui::WireGlowHint,
) -> Option<Rect> {
    if let Some(custom) = custom {
        glow_bounds_for_custom_path(&custom.commands, zoom, width, glow.blur_radius_px)
    } else {
        glow_bounds_for_edge_route(route, from, to, zoom, width, glow.blur_radius_px)
    }
}

fn glow_color(color: Color, alpha_mul: f32) -> Color {
    let mut glow_color = color;
    glow_color.a = (glow_color.a * alpha_mul).clamp(0.0, 1.0);
    glow_color
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_edge_glow<H: UiHost, M: NodeGraphCanvasMiddleware>(
    _canvas: &mut NodeGraphCanvasWith<M>,
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
    let Some(glow) = glow_hint(interaction_hint, edge_selected) else {
        return false;
    };
    let Some(bounds) = glow_bounds(custom, route, from, to, zoom, width, glow) else {
        return false;
    };

    let z = zoom.max(1.0e-6);
    let blur_canvas = (glow.blur_radius_px / z).max(0.0);
    let alpha_mul = if glow.alpha_mul.is_finite() {
        glow.alpha_mul.clamp(0.0, 1.0)
    } else {
        0.0
    };
    cx.scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::DropShadowV1(
            DropShadowV1 {
                offset_px: Point::new(Px(0.0), Px(0.0)),
                blur_radius_px: Px(blur_canvas),
                downsample: glow.downsample,
                color: glow_color(color, alpha_mul),
            }
            .sanitize(),
        )]),
        quality: EffectQuality::Auto,
    });
    true
}
