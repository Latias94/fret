use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneValidationErrorKind {
    TransformUnderflow,
    OpacityUnderflow,
    LayerUnderflow,
    ClipUnderflow,
    EffectUnderflow,
    NonFiniteTransform,
    NonFiniteOpacity,
    NonFiniteOpData,
    UnbalancedTransformStack { remaining: usize },
    UnbalancedOpacityStack { remaining: usize },
    UnbalancedLayerStack { remaining: usize },
    UnbalancedClipStack { remaining: usize },
    UnbalancedEffectStack { remaining: usize },
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
                    for i in 0..n {
                        if !g.stops[i].offset.is_finite() || !color_is_finite(g.stops[i].color) {
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
                    for i in 0..n {
                        if !g.stops[i].offset.is_finite() || !color_is_finite(g.stops[i].color) {
                            return false;
                        }
                    }
                    true
                }
            }
        }

        fn uv_is_finite(uv: UvRect) -> bool {
            uv.u0.is_finite() && uv.v0.is_finite() && uv.u1.is_finite() && uv.v1.is_finite()
        }

        let mut transform_depth: usize = 0;
        let mut opacity_depth: usize = 0;
        let mut layer_depth: usize = 0;
        let mut clip_depth: usize = 0;
        let mut effect_depth: usize = 0;

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
                SceneOp::PushClipRect { .. } | SceneOp::PushClipRRect { .. } => {
                    let ok = match op {
                        SceneOp::PushClipRect { rect } => rect_is_finite(rect),
                        SceneOp::PushClipRRect { rect, corner_radii } => {
                            rect_is_finite(rect) && corners_is_finite(corner_radii)
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
                            EffectStep::ColorAdjust {
                                saturation,
                                brightness,
                                contrast,
                            } => {
                                saturation.is_finite()
                                    && brightness.is_finite()
                                    && contrast.is_finite()
                            }
                            EffectStep::Pixelate { scale } => scale > 0,
                            EffectStep::Dither { .. } => true,
                        };
                        if !ok {
                            return Err(SceneValidationError {
                                index,
                                op,
                                kind: SceneValidationErrorKind::NonFiniteOpData,
                            });
                        }
                    }

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
                    effect_depth = effect_depth.saturating_sub(1);
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
                SceneOp::Text { origin, color, .. } => {
                    if !point_is_finite(origin) || !color_is_finite(color) {
                        return Err(SceneValidationError {
                            index,
                            op,
                            kind: SceneValidationErrorKind::NonFiniteOpData,
                        });
                    }
                }
                SceneOp::Path { origin, color, .. } => {
                    if !point_is_finite(origin) || !color_is_finite(color) {
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
        if effect_depth != 0 {
            return Err(SceneValidationError {
                index: self.ops.len(),
                op: SceneOp::PopEffect,
                kind: SceneValidationErrorKind::UnbalancedEffectStack {
                    remaining: effect_depth,
                },
            });
        }

        Ok(())
    }
}
