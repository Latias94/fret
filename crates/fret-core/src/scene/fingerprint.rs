use super::*;

fn mix_u64(mut state: u64, value: u64) -> u64 {
    // A lightweight, deterministic mixing function (not cryptographic).
    // We want stability across platforms and reasonable avalanche for small changes.
    state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = state.rotate_left(7);
    state = state.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    state
}

fn mix_f32(state: u64, value: f32) -> u64 {
    mix_u64(state, u64::from(value.to_bits()))
}

fn mix_px(state: u64, value: crate::Px) -> u64 {
    mix_f32(state, value.0)
}

fn mix_point(mut state: u64, p: Point) -> u64 {
    state = mix_px(state, p.x);
    state = mix_px(state, p.y);
    state
}

fn mix_rect(mut state: u64, r: Rect) -> u64 {
    state = mix_point(state, r.origin);
    state = mix_px(state, r.size.width);
    state = mix_px(state, r.size.height);
    state
}

fn mix_color(mut state: u64, c: Color) -> u64 {
    state = mix_f32(state, c.r);
    state = mix_f32(state, c.g);
    state = mix_f32(state, c.b);
    state = mix_f32(state, c.a);
    state
}

fn mix_edges(mut state: u64, e: Edges) -> u64 {
    state = mix_px(state, e.top);
    state = mix_px(state, e.right);
    state = mix_px(state, e.bottom);
    state = mix_px(state, e.left);
    state
}

fn mix_corners(mut state: u64, c: Corners) -> u64 {
    state = mix_px(state, c.top_left);
    state = mix_px(state, c.top_right);
    state = mix_px(state, c.bottom_right);
    state = mix_px(state, c.bottom_left);
    state
}

pub(super) fn mix_scene_op(state: u64, op: SceneOp) -> u64 {
    match op {
        SceneOp::PushTransform { transform } => {
            let mut state = mix_u64(state, 100);
            state = mix_f32(state, transform.a);
            state = mix_f32(state, transform.b);
            state = mix_f32(state, transform.c);
            state = mix_f32(state, transform.d);
            state = mix_f32(state, transform.tx);
            mix_f32(state, transform.ty)
        }
        SceneOp::PopTransform => mix_u64(state, 101),
        SceneOp::PushOpacity { opacity } => {
            let state = mix_u64(state, 102);
            mix_f32(state, opacity)
        }
        SceneOp::PopOpacity => mix_u64(state, 103),
        SceneOp::PushLayer { layer } => {
            let state = mix_u64(state, 104);
            mix_u64(state, u64::from(layer))
        }
        SceneOp::PopLayer => mix_u64(state, 105),
        SceneOp::PushClipRect { rect } => {
            let state = mix_u64(state, 1);
            mix_rect(state, rect)
        }
        SceneOp::PushClipRRect { rect, corner_radii } => {
            let mut state = mix_u64(state, 13);
            state = mix_rect(state, rect);
            mix_corners(state, corner_radii)
        }
        SceneOp::PopClip => mix_u64(state, 2),
        SceneOp::PushEffect {
            bounds,
            mode,
            chain,
            quality,
        } => {
            let mut state = mix_u64(state, 106);
            state = mix_rect(state, bounds);
            state = mix_u64(
                state,
                match mode {
                    EffectMode::FilterContent => 1,
                    EffectMode::Backdrop => 2,
                },
            );
            state = mix_u64(
                state,
                match quality {
                    EffectQuality::Auto => 1,
                    EffectQuality::Low => 2,
                    EffectQuality::Medium => 3,
                    EffectQuality::High => 4,
                },
            );

            for step in chain.iter() {
                state = match step {
                    EffectStep::GaussianBlur {
                        radius_px,
                        downsample,
                    } => {
                        let mut state = mix_u64(state, 1);
                        state = mix_px(state, radius_px);
                        mix_u64(state, u64::from(downsample))
                    }
                    EffectStep::ColorAdjust {
                        saturation,
                        brightness,
                        contrast,
                    } => {
                        let mut state = mix_u64(state, 2);
                        state = mix_f32(state, saturation);
                        state = mix_f32(state, brightness);
                        mix_f32(state, contrast)
                    }
                    EffectStep::Pixelate { scale } => mix_u64(mix_u64(state, 3), u64::from(scale)),
                    EffectStep::Dither { mode } => mix_u64(
                        mix_u64(state, 4),
                        match mode {
                            DitherMode::Bayer4x4 => 1,
                        },
                    ),
                };
            }

            state
        }
        SceneOp::PopEffect => mix_u64(state, 107),
        SceneOp::Quad {
            order,
            rect,
            background,
            border,
            border_color,
            corner_radii,
        } => {
            let mut state = mix_u64(state, 3);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_color(state, background);
            state = mix_edges(state, border);
            state = mix_color(state, border_color);
            mix_corners(state, corner_radii)
        }
        SceneOp::Image {
            order,
            rect,
            image,
            opacity,
        } => {
            let mut state = mix_u64(state, 4);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, image.data().as_ffi());
            mix_f32(state, opacity)
        }
        SceneOp::ImageRegion {
            order,
            rect,
            image,
            uv,
            opacity,
        } => {
            let mut state = mix_u64(state, 7);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, image.data().as_ffi());
            state = mix_f32(state, uv.u0);
            state = mix_f32(state, uv.v0);
            state = mix_f32(state, uv.u1);
            state = mix_f32(state, uv.v1);
            mix_f32(state, opacity)
        }
        SceneOp::MaskImage {
            order,
            rect,
            image,
            uv,
            color,
            opacity,
        } => {
            let mut state = mix_u64(state, 9);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, image.data().as_ffi());
            state = mix_f32(state, uv.u0);
            state = mix_f32(state, uv.v0);
            state = mix_f32(state, uv.u1);
            state = mix_f32(state, uv.v1);
            state = mix_color(state, color);
            mix_f32(state, opacity)
        }
        SceneOp::SvgMaskIcon {
            order,
            rect,
            svg,
            fit,
            color,
            opacity,
        } => {
            let mut state = mix_u64(state, 10);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, svg.data().as_ffi());
            state = mix_u64(
                state,
                match fit {
                    SvgFit::Contain => 1,
                    SvgFit::Width => 2,
                    SvgFit::Stretch => 3,
                },
            );
            state = mix_color(state, color);
            mix_f32(state, opacity)
        }
        SceneOp::SvgImage {
            order,
            rect,
            svg,
            fit,
            opacity,
        } => {
            let mut state = mix_u64(state, 11);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, svg.data().as_ffi());
            state = mix_u64(
                state,
                match fit {
                    SvgFit::Contain => 1,
                    SvgFit::Width => 2,
                    SvgFit::Stretch => 3,
                },
            );
            mix_f32(state, opacity)
        }
        SceneOp::Text {
            order,
            origin,
            text,
            color,
        } => {
            let mut state = mix_u64(state, 5);
            state = mix_u64(state, u64::from(order.0));
            state = mix_point(state, origin);
            state = mix_u64(state, text.data().as_ffi());
            mix_color(state, color)
        }
        SceneOp::Path {
            order,
            origin,
            path,
            color,
        } => {
            let mut state = mix_u64(state, 8);
            state = mix_u64(state, u64::from(order.0));
            state = mix_point(state, origin);
            state = mix_u64(state, path.data().as_ffi());
            mix_color(state, color)
        }
        SceneOp::ViewportSurface {
            order,
            rect,
            target,
            opacity,
        } => {
            let mut state = mix_u64(state, 6);
            state = mix_u64(state, u64::from(order.0));
            state = mix_rect(state, rect);
            state = mix_u64(state, target.data().as_ffi());
            mix_f32(state, opacity)
        }
    }
}
