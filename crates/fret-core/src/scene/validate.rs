use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneValidationErrorKind {
    TransformUnderflow,
    OpacityUnderflow,
    LayerUnderflow,
    ClipUnderflow,
    MaskUnderflow,
    EffectUnderflow,
    CompositeGroupUnderflow,
    NonFiniteTransform,
    NonFiniteOpacity,
    NonFiniteOpData,
    UnbalancedTransformStack { remaining: usize },
    UnbalancedOpacityStack { remaining: usize },
    UnbalancedLayerStack { remaining: usize },
    UnbalancedClipStack { remaining: usize },
    UnbalancedMaskStack { remaining: usize },
    UnbalancedEffectStack { remaining: usize },
    UnbalancedCompositeGroupStack { remaining: usize },
    EffectMaskCrossing,
    CompositeGroupMaskCrossing,
}

#[derive(Debug, Clone, Copy)]
pub struct SceneValidationError {
    pub index: usize,
    pub op: SceneOp,
    pub kind: SceneValidationErrorKind,
}

impl std::fmt::Display for SceneValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "scene validation error at op index {}: {:?}",
            self.index, self.kind
        )
    }
}

impl std::error::Error for SceneValidationError {}

impl SceneRecording {
    #[allow(clippy::result_large_err)]
    pub fn validate(&self) -> Result<(), SceneValidationError> {
        fn px_is_finite(px: crate::Px) -> bool {
            px.0.is_finite()
        }

        fn point_is_finite(p: Point) -> bool {
            px_is_finite(p.x) && px_is_finite(p.y)
        }

        fn rect_is_finite(r: Rect) -> bool {
            point_is_finite(r.origin) && px_is_finite(r.size.width) && px_is_finite(r.size.height)
        }

        fn corners_is_finite(c: Corners) -> bool {
            px_is_finite(c.top_left)
                && px_is_finite(c.top_right)
                && px_is_finite(c.bottom_right)
                && px_is_finite(c.bottom_left)
        }

        fn edges_is_finite(e: Edges) -> bool {
            px_is_finite(e.top)
                && px_is_finite(e.right)
                && px_is_finite(e.bottom)
                && px_is_finite(e.left)
        }

        fn color_is_finite(c: Color) -> bool {
            c.r.is_finite() && c.g.is_finite() && c.b.is_finite() && c.a.is_finite()
        }

        fn paint_is_finite(p: Paint) -> bool {
            match p {
                Paint::Solid(c) => color_is_finite(c),
                Paint::LinearGradient(g) => {
                    if !g.start.x.0.is_finite()
                        || !g.start.y.0.is_finite()
                        || !g.end.x.0.is_finite()
                        || !g.end.y.0.is_finite()
                    {
                        return false;
                    }
                    let n = usize::from(g.stop_count).min(MAX_STOPS);
                    for s in g.stops.iter().take(n) {
                        if !s.offset.is_finite() || !color_is_finite(s.color) {
                            return false;
                        }
                    }
                    true
                }
                Paint::RadialGradient(g) => {
                    if !g.center.x.0.is_finite()
                        || !g.center.y.0.is_finite()
                        || !g.radius.width.0.is_finite()
                        || !g.radius.height.0.is_finite()
                    {
                        return false;
                    }
                    let n = usize::from(g.stop_count).min(MAX_STOPS);
                    for s in g.stops.iter().take(n) {
                        if !s.offset.is_finite() || !color_is_finite(s.color) {
                            return false;
                        }
                    }
                    true
                }
                Paint::SweepGradient(g) => {
                    if !g.center.x.0.is_finite()
                        || !g.center.y.0.is_finite()
                        || !g.start_angle_turns.is_finite()
                        || !g.end_angle_turns.is_finite()
                    {
                        return false;
                    }
                    let n = usize::from(g.stop_count).min(MAX_STOPS);
                    for s in g.stops.iter().take(n) {
                        if !s.offset.is_finite() || !color_is_finite(s.color) {
                            return false;
                        }
                    }
                    true
                }
                Paint::Material { params, .. } => params.is_finite(),
            }
        }

        fn text_shadow_is_finite(s: TextShadowV1) -> bool {
            point_is_finite(s.offset) && color_is_finite(s.color)
        }

        fn mask_is_finite(m: Mask) -> bool {
            match m {
                Mask::LinearGradient(g) => {
                    if !g.start.x.0.is_finite()
                        || !g.start.y.0.is_finite()
                        || !g.end.x.0.is_finite()
                        || !g.end.y.0.is_finite()
                    {
                        return false;
                    }
                    let n = usize::from(g.stop_count).min(MAX_STOPS);
                    for s in g.stops.iter().take(n) {
                        if !s.offset.is_finite() || !color_is_finite(s.color) {
                            return false;
                        }
                    }
                    true
                }
                Mask::RadialGradient(g) => {
                    if !g.center.x.0.is_finite()
                        || !g.center.y.0.is_finite()
                        || !g.radius.width.0.is_finite()
                        || !g.radius.height.0.is_finite()
                    {
                        return false;
                    }
                    let n = usize::from(g.stop_count).min(MAX_STOPS);
                    for s in g.stops.iter().take(n) {
                        if !s.offset.is_finite() || !color_is_finite(s.color) {
                            return false;
                        }
                    }
                    true
                }
                Mask::Image { uv, .. } => uv_is_finite(uv),
            }
        }

        fn uv_is_finite(uv: UvRect) -> bool {
            uv.u0.is_finite() && uv.v0.is_finite() && uv.u1.is_finite() && uv.v1.is_finite()
        }

        let mut transform_depth: usize = 0;
        let mut opacity_depth: usize = 0;
        let mut layer_depth: usize = 0;
        let mut clip_depth: usize = 0;
        let mut mask_depth: usize = 0;
        let mut effect_depth: usize = 0;
        let mut effect_mask_depths: Vec<usize> = Vec::new();
        let mut composite_group_depth: usize = 0;
        let mut composite_group_mask_depths: Vec<usize> = Vec::new();

        for (index, &op) in self.ops.iter().enumerate() {
            match op {
                SceneOp::PushTransform { transform } => {
                    if !transform.a.is_finite()
                        || !transform.b.is_finite()
                        || !transform.c.is_finite()
                        || !transform.d.is_finite()
                        || !transform.tx.is_finite()
                        || !transform.ty.is_finite()
                    {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteTransform,
                        });
                    }
                    transform_depth = transform_depth.saturating_add(1);
                }
                SceneOp::PopTransform => {
                    if transform_depth == 0 {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::TransformUnderflow,
                        });
                    }
                    transform_depth = transform_depth.saturating_sub(1);
                }
                SceneOp::PushOpacity { opacity } => {
                    if !opacity.is_finite() {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpacity,
                        });
                    }
                    opacity_depth = opacity_depth.saturating_add(1);
                }
                SceneOp::PopOpacity => {
                    if opacity_depth == 0 {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::OpacityUnderflow,
                        });
                    }
                    opacity_depth = opacity_depth.saturating_sub(1);
                }
                SceneOp::PushLayer { .. } => {
                    layer_depth = layer_depth.saturating_add(1);
                }
                SceneOp::PopLayer => {
                    if layer_depth == 0 {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::LayerUnderflow,
                        });
                    }
                    layer_depth = layer_depth.saturating_sub(1);
                }
                SceneOp::PushClipRect { .. }
                | SceneOp::PushClipRRect { .. }
                | SceneOp::PushClipPath { .. } => {
                    let ok = match op {
                        SceneOp::PushClipRect { rect } => rect_is_finite(rect),
                        SceneOp::PushClipRRect { rect, corner_radii } => {
                            rect_is_finite(rect) && corners_is_finite(corner_radii)
                        }
                        SceneOp::PushClipPath { bounds, origin, .. } => {
                            rect_is_finite(bounds) && point_is_finite(origin)
                        }
                        _ => unreachable!(),
                    };
                    if !ok {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                    clip_depth = clip_depth.saturating_add(1);
                }
                SceneOp::PopClip => {
                    if clip_depth == 0 {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::ClipUnderflow,
                        });
                    }
                    clip_depth = clip_depth.saturating_sub(1);
                }
                SceneOp::PushMask { bounds, mask } => {
                    if !rect_is_finite(bounds) || !mask_is_finite(mask) {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                    mask_depth = mask_depth.saturating_add(1);
                }
                SceneOp::PopMask => {
                    if mask_depth == 0 {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::MaskUnderflow,
                        });
                    }
                    mask_depth = mask_depth.saturating_sub(1);
                }
                SceneOp::PushEffect { bounds, chain, .. } => {
                    if !rect_is_finite(bounds) {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }

                    for step in chain.iter() {
                        let ok = match step {
                            EffectStep::GaussianBlur {
                                radius_px,
                                downsample,
                            } => px_is_finite(radius_px) && downsample > 0,
                            EffectStep::DropShadowV1(s) => {
                                px_is_finite(s.offset_px.x)
                                    && px_is_finite(s.offset_px.y)
                                    && px_is_finite(s.blur_radius_px)
                                    && s.downsample > 0
                                    && s.color.r.is_finite()
                                    && s.color.g.is_finite()
                                    && s.color.b.is_finite()
                                    && s.color.a.is_finite()
                            }
                            EffectStep::BackdropWarpV1(w) => {
                                px_is_finite(w.strength_px)
                                    && px_is_finite(w.scale_px)
                                    && w.scale_px.0 > 0.0
                                    && w.phase.is_finite()
                                    && px_is_finite(w.chromatic_aberration_px)
                            }
                            EffectStep::BackdropWarpV2(w) => {
                                let base_ok = px_is_finite(w.base.strength_px)
                                    && px_is_finite(w.base.scale_px)
                                    && w.base.scale_px.0 > 0.0
                                    && w.base.phase.is_finite()
                                    && px_is_finite(w.base.chromatic_aberration_px);
                                if !base_ok {
                                    false
                                } else {
                                    match w.field {
                                        BackdropWarpFieldV2::Procedural => true,
                                        BackdropWarpFieldV2::ImageDisplacementMap {
                                            uv, ..
                                        } => {
                                            uv.u0.is_finite()
                                                && uv.v0.is_finite()
                                                && uv.u1.is_finite()
                                                && uv.v1.is_finite()
                                        }
                                    }
                                }
                            }
                            EffectStep::NoiseV1(n) => {
                                n.strength.is_finite()
                                    && px_is_finite(n.scale_px)
                                    && n.scale_px.0 > 0.0
                                    && n.phase.is_finite()
                            }
                            EffectStep::ColorAdjust {
                                saturation,
                                brightness,
                                contrast,
                            } => {
                                saturation.is_finite()
                                    && brightness.is_finite()
                                    && contrast.is_finite()
                            }
                            EffectStep::ColorMatrix { m } => m.iter().all(|v| v.is_finite()),
                            EffectStep::AlphaThreshold { cutoff, soft } => {
                                cutoff.is_finite() && soft.is_finite() && soft >= 0.0
                            }
                            EffectStep::Pixelate { scale } => scale > 0,
                            EffectStep::Dither { .. } => true,
                            EffectStep::CustomV1 {
                                params,
                                max_sample_offset_px,
                                ..
                            } => params.is_finite() && px_is_finite(max_sample_offset_px),
                            EffectStep::CustomV2 {
                                params,
                                max_sample_offset_px,
                                input_image,
                                ..
                            } => {
                                let base_ok =
                                    params.is_finite() && px_is_finite(max_sample_offset_px);
                                if !base_ok {
                                    false
                                } else if let Some(input) = input_image {
                                    input.uv.u0.is_finite()
                                        && input.uv.v0.is_finite()
                                        && input.uv.u1.is_finite()
                                        && input.uv.v1.is_finite()
                                } else {
                                    true
                                }
                            }
                            EffectStep::CustomV3 {
                                params,
                                max_sample_offset_px,
                                user0,
                                user1,
                                sources,
                                ..
                            } => {
                                let base_ok =
                                    params.is_finite() && px_is_finite(max_sample_offset_px);
                                if !base_ok {
                                    false
                                } else {
                                    let input_ok = |input: Option<CustomEffectImageInputV1>| {
                                        input.is_none_or(|input| {
                                            input.uv.u0.is_finite()
                                                && input.uv.v0.is_finite()
                                                && input.uv.u1.is_finite()
                                                && input.uv.v1.is_finite()
                                        })
                                    };
                                    if !input_ok(user0) || !input_ok(user1) {
                                        false
                                    } else if let Some(req) = sources.pyramid {
                                        req.max_levels > 0
                                            && px_is_finite(req.max_radius_px)
                                            && req.max_radius_px.0 >= 0.0
                                    } else {
                                        true
                                    }
                                }
                            }
                        };
                        if !ok {
                            return Err(SceneValidationError {
                                index,
                                op,
                                kind: SceneValidationErrorKind::NonFiniteOpData,
                            });
                        }
                    }

                    effect_mask_depths.push(mask_depth);
                    effect_depth = effect_depth.saturating_add(1);
                }
                SceneOp::PopEffect => {
                    if effect_depth == 0 {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::EffectUnderflow,
                        });
                    }
                    let expected = effect_mask_depths.pop().unwrap_or(mask_depth);
                    if expected != mask_depth {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::EffectMaskCrossing,
                        });
                    }
                    effect_depth = effect_depth.saturating_sub(1);
                }
                SceneOp::PushCompositeGroup { desc } => {
                    if !rect_is_finite(desc.bounds) {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                    composite_group_mask_depths.push(mask_depth);
                    composite_group_depth = composite_group_depth.saturating_add(1);
                }
                SceneOp::PopCompositeGroup => {
                    if composite_group_depth == 0 {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::CompositeGroupUnderflow,
                        });
                    }
                    let expected = composite_group_mask_depths.pop().unwrap_or(mask_depth);
                    if expected != mask_depth {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::CompositeGroupMaskCrossing,
                        });
                    }
                    composite_group_depth = composite_group_depth.saturating_sub(1);
                }
                SceneOp::Quad {
                    rect,
                    background,
                    border,
                    border_paint,
                    corner_radii,
                    ..
                } => {
                    if !rect_is_finite(rect)
                        || !paint_is_finite(background)
                        || !edges_is_finite(border)
                        || !paint_is_finite(border_paint)
                        || !corners_is_finite(corner_radii)
                    {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
                SceneOp::StrokeRRect {
                    rect,
                    stroke,
                    stroke_paint,
                    corner_radii,
                    style,
                    ..
                } => {
                    let mut ok = rect_is_finite(rect)
                        && edges_is_finite(stroke)
                        && paint_is_finite(stroke_paint)
                        && corners_is_finite(corner_radii);
                    if let Some(dash) = style.dash {
                        ok = ok
                            && dash.dash.0.is_finite()
                            && dash.gap.0.is_finite()
                            && dash.phase.0.is_finite();
                    }
                    if !ok {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
                SceneOp::Image { rect, opacity, .. } => {
                    if !rect_is_finite(rect) || !opacity.is_finite() {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
                SceneOp::ImageRegion {
                    rect, uv, opacity, ..
                } => {
                    if !rect_is_finite(rect) || !uv_is_finite(uv) || !opacity.is_finite() {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
                SceneOp::MaskImage {
                    rect,
                    uv,
                    color,
                    opacity,
                    ..
                } => {
                    if !rect_is_finite(rect)
                        || !uv_is_finite(uv)
                        || !color_is_finite(color)
                        || !opacity.is_finite()
                    {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
                SceneOp::SvgMaskIcon {
                    rect,
                    color,
                    opacity,
                    ..
                } => {
                    if !rect_is_finite(rect) || !color_is_finite(color) || !opacity.is_finite() {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
                SceneOp::SvgImage { rect, opacity, .. } => {
                    if !rect_is_finite(rect) || !opacity.is_finite() {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
                SceneOp::Text {
                    origin,
                    paint,
                    outline,
                    shadow,
                    ..
                } => {
                    if !point_is_finite(origin)
                        || !paint_is_finite(paint)
                        || outline
                            .is_some_and(|o| !paint_is_finite(o.paint) || !o.width_px.0.is_finite())
                        || shadow.is_some_and(|s| !text_shadow_is_finite(s))
                    {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
                SceneOp::Path { origin, paint, .. } => {
                    if !point_is_finite(origin) || !paint_is_finite(paint) {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
                SceneOp::ViewportSurface { rect, opacity, .. } => {
                    if !rect_is_finite(rect) || !opacity.is_finite() {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
            }
        }

        if transform_depth != 0 {
            return Err(SceneValidationError {
                index: self.ops.len(),
                op: SceneOp::PopTransform,
                kind: SceneValidationErrorKind::UnbalancedTransformStack {
                    remaining: transform_depth,
                },
            });
        }
        if opacity_depth != 0 {
            return Err(SceneValidationError {
                index: self.ops.len(),
                op: SceneOp::PopOpacity,
                kind: SceneValidationErrorKind::UnbalancedOpacityStack {
                    remaining: opacity_depth,
                },
            });
        }
        if layer_depth != 0 {
            return Err(SceneValidationError {
                index: self.ops.len(),
                op: SceneOp::PopLayer,
                kind: SceneValidationErrorKind::UnbalancedLayerStack {
                    remaining: layer_depth,
                },
            });
        }
        if clip_depth != 0 {
            return Err(SceneValidationError {
                index: self.ops.len(),
                op: SceneOp::PopClip,
                kind: SceneValidationErrorKind::UnbalancedClipStack {
                    remaining: clip_depth,
                },
            });
        }
        if mask_depth != 0 {
            return Err(SceneValidationError {
                index: self.ops.len(),
                op: SceneOp::PopMask,
                kind: SceneValidationErrorKind::UnbalancedMaskStack {
                    remaining: mask_depth,
                },
            });
        }
        if effect_depth != 0 {
            return Err(SceneValidationError {
                index: self.ops.len(),
                op: SceneOp::PopEffect,
                kind: SceneValidationErrorKind::UnbalancedEffectStack {
                    remaining: effect_depth,
                },
            });
        }
        if composite_group_depth != 0 {
            return Err(SceneValidationError {
                index: self.ops.len(),
                op: SceneOp::PopCompositeGroup,
                kind: SceneValidationErrorKind::UnbalancedCompositeGroupStack {
                    remaining: composite_group_depth,
                },
            });
        }

        Ok(())
    }
}
