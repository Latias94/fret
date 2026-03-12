use crate::ui::NodeShadowHint;

use super::*;
use fret_core::scene::DropShadowV1;
use fret_core::{EffectChain, EffectMode, EffectQuality, EffectStep};

pub(super) fn push_static_node_shadow(
    scene: &mut fret_core::Scene,
    rect: Rect,
    zoom: f32,
    shadow: Option<NodeShadowHint>,
) -> bool {
    let Some(shadow) = shadow else {
        return false;
    };
    let Some((bounds, drop_shadow)) = shadow_to_drop_shadow_canvas_units(rect, zoom, shadow) else {
        return false;
    };

    scene.push(SceneOp::PushEffect {
        bounds,
        mode: EffectMode::FilterContent,
        chain: EffectChain::from_steps(&[EffectStep::DropShadowV1(drop_shadow)]),
        quality: EffectQuality::Auto,
    });
    true
}

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
