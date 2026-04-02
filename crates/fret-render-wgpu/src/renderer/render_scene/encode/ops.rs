use super::clip;
use super::draw;
use super::mask;
use super::state::{EncodeState, bounds_of_quad_points, transform_quad_points_px};
use super::*;
use fret_core::Paint;
use slotmap::Key;

fn mix_u64(mut state: u64, value: u64) -> u64 {
    state ^= value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    state = state.rotate_left(7);
    state = state.wrapping_mul(0xD6E8_FEB8_6659_FD93);
    state
}

fn mix_f32(state: u64, value: f32) -> u64 {
    mix_u64(state, u64::from(value.to_bits()))
}

fn mix_transform(mut state: u64, t: Transform2D) -> u64 {
    state = mix_f32(state, t.a);
    state = mix_f32(state, t.b);
    state = mix_f32(state, t.c);
    state = mix_f32(state, t.d);
    state = mix_f32(state, t.tx);
    state = mix_f32(state, t.ty);
    state
}

fn clip_path_mask_cache_key(
    state: &EncodeState<'_>,
    scissor: ScissorRect,
    origin: Point,
    path: fret_core::PathId,
) -> u64 {
    let mut key = 0x6b2d_4f8c_f3a1_6d77u64;
    key = mix_u64(key, path.data().as_ffi());
    key = mix_f32(key, state.scale_factor);
    key = mix_transform(key, state.current_transform());
    key = mix_f32(key, origin.x.0);
    key = mix_f32(key, origin.y.0);
    key = mix_u64(key, u64::from(scissor.x));
    key = mix_u64(key, u64::from(scissor.y));
    key = mix_u64(key, u64::from(scissor.w));
    key = mix_u64(key, u64::from(scissor.h));

    key = mix_u64(key, u64::from(state.clip_head));
    key = mix_u64(key, u64::from(state.clip_count));
    key = mix_u64(key, u64::from(state.mask_head));
    key = mix_u64(key, u64::from(state.mask_count));
    key = mix_u64(key, u64::from(state.mask_scope_head));
    key = mix_u64(key, u64::from(state.mask_scope_count));

    if let Some(sel) = state.mask_image {
        key = mix_u64(key, sel.image.data().as_ffi());
        key = mix_u64(
            key,
            match sel.sampling {
                fret_core::scene::ImageSamplingHint::Default => 0,
                fret_core::scene::ImageSamplingHint::Linear => 1,
                fret_core::scene::ImageSamplingHint::Nearest => 2,
            },
        );
    }

    key
}

pub(super) fn handle_op(renderer: &Renderer, state: &mut EncodeState<'_>, op: &SceneOp) {
    match *op {
        SceneOp::PushTransform { transform } => {
            let current = state.current_transform();
            state.transform_stack.push(current * transform);
        }
        SceneOp::PopTransform => {
            if state.transform_stack.len() > 1 {
                state.transform_stack.pop();
            }
        }

        SceneOp::PushOpacity { opacity } => {
            let current = state.current_opacity();
            state
                .opacity_stack
                .push((current * opacity).clamp(0.0, 1.0));
        }
        SceneOp::PopOpacity => {
            if state.opacity_stack.len() > 1 {
                state.opacity_stack.pop();
            }
        }

        SceneOp::PushLayer { .. } | SceneOp::PopLayer => {
            state.flush_quad_batch();
        }

        SceneOp::PushClipRect { rect } => {
            let _ = clip::push_clip_rect(state, rect);
        }
        SceneOp::PushClipRRect { rect, corner_radii } => {
            let _ = clip::push_clip_rrect(state, rect, corner_radii);
        }
        SceneOp::PushClipPath {
            bounds,
            origin,
            path,
        } => {
            state.flush_quad_batch();

            let scissor = effect_scissor(state, bounds);
            state.current_scissor = scissor;
            state.scissor_stack.push(scissor);

            let mask_uniform_index = state.push_mask_viewport_uniform_snapshot(
                scissor,
                state.clip_head,
                state.clip_count,
                state.mask_head,
                state.mask_count,
                state.mask_scope_head,
                state.mask_scope_count,
            );

            let Some((first_vertex, vertex_count)) =
                draw::encode_clip_path_mask(renderer, state, origin, path)
            else {
                // Degrade to computation-bounds scissor only.
                state.clip_pop_stack.push(ClipPop::NoShader);
                return;
            };

            let cache_key = clip_path_mask_cache_key(state, scissor, origin, path);

            let mask_draw_index = state.clip_path_masks.len() as u32;
            state.clip_path_masks.push(ClipPathMaskDraw {
                scissor,
                uniform_index: mask_uniform_index,
                first_vertex,
                vertex_count,
                cache_key,
            });

            state.effect_markers.push(EffectMarker {
                draw_ix: state.ordered_draws.len(),
                kind: EffectMarkerKind::ClipPathPush {
                    scissor,
                    uniform_index: mask_uniform_index,
                    mask_draw_index,
                },
            });

            state.clip_pop_stack.push(ClipPop::Path);
        }
        SceneOp::PopClip => {
            clip::pop_clip(state);
        }

        SceneOp::PushMask { bounds, mask: m } => {
            let _ = mask::push_mask(renderer, state, bounds, m);
        }
        SceneOp::PopMask => {
            mask::pop_mask(state);
        }

        SceneOp::PushEffect {
            bounds,
            mode,
            chain,
            quality,
        } => {
            state.flush_quad_batch();

            let scissor = effect_scissor(state, bounds);

            let uniform_index = state.push_effect_uniform_snapshot(
                scissor,
                state.clip_head,
                state.clip_count,
                state.mask_head,
                state.mask_count,
            );
            state.effect_markers.push(EffectMarker {
                draw_ix: state.ordered_draws.len(),
                kind: EffectMarkerKind::Push {
                    scissor,
                    uniform_index,
                    mode,
                    chain,
                    quality,
                },
            });

            // Inside an effect scope, masks active at effect entry are excluded from draw shaders
            // and applied by the composite that closes the effect scope.
            state
                .mask_scope_stack
                .push((state.mask_scope_head, state.mask_scope_count));
            state.mask_scope_head = state.mask_head;
            state.mask_scope_count = state.mask_count;
            state.current_uniform_index = state.push_uniform_snapshot(
                state.clip_head,
                state.clip_count,
                state.mask_head,
                state.mask_count,
                state.mask_scope_head,
                state.mask_scope_count,
            );
        }
        SceneOp::PopEffect => {
            state.flush_quad_batch();
            state.effect_markers.push(EffectMarker {
                draw_ix: state.ordered_draws.len(),
                kind: EffectMarkerKind::Pop,
            });

            let (head, count) = state.mask_scope_stack.pop().unwrap_or((0, 0));
            state.mask_scope_head = head;
            state.mask_scope_count = count;
            state.current_uniform_index = state.push_uniform_snapshot(
                state.clip_head,
                state.clip_count,
                state.mask_head,
                state.mask_count,
                state.mask_scope_head,
                state.mask_scope_count,
            );
        }

        SceneOp::PushBackdropSourceGroupV1 {
            bounds,
            pyramid,
            quality,
        } => {
            state.flush_quad_batch();
            let scissor = effect_scissor(state, bounds);
            state.effect_markers.push(EffectMarker {
                draw_ix: state.ordered_draws.len(),
                kind: EffectMarkerKind::BackdropSourceGroupPush {
                    scissor,
                    pyramid,
                    quality,
                },
            });
        }
        SceneOp::PopBackdropSourceGroup => {
            state.flush_quad_batch();
            state.effect_markers.push(EffectMarker {
                draw_ix: state.ordered_draws.len(),
                kind: EffectMarkerKind::BackdropSourceGroupPop,
            });
        }

        SceneOp::PushCompositeGroup { desc } => {
            state.flush_quad_batch();

            let scissor = effect_scissor(state, desc.bounds);

            // Composite group bounds are computation bounds (not an implicit clip), but they must
            // bound GPU work inside the scope. This keeps isolated opacity/groups wasm/mobile
            // friendly by avoiding unbounded intermediate fills.
            state.current_scissor = scissor;
            state.scissor_stack.push(scissor);

            let uniform_index = state.push_effect_uniform_snapshot(
                scissor,
                state.clip_head,
                state.clip_count,
                state.mask_head,
                state.mask_count,
            );
            state.effect_markers.push(EffectMarker {
                draw_ix: state.ordered_draws.len(),
                kind: EffectMarkerKind::CompositeGroupPush {
                    scissor,
                    uniform_index,
                    mode: desc.mode,
                    quality: desc.quality,
                    opacity: desc.opacity.clamp(0.0, 1.0),
                },
            });

            // Inside an isolated compositing group, masks active at group entry are excluded from
            // draw shaders and applied by the composite that closes the group.
            state
                .mask_scope_stack
                .push((state.mask_scope_head, state.mask_scope_count));
            state.mask_scope_head = state.mask_head;
            state.mask_scope_count = state.mask_count;
            state.current_uniform_index = state.push_uniform_snapshot(
                state.clip_head,
                state.clip_count,
                state.mask_head,
                state.mask_count,
                state.mask_scope_head,
                state.mask_scope_count,
            );
        }
        SceneOp::PopCompositeGroup => {
            state.flush_quad_batch();
            state.effect_markers.push(EffectMarker {
                draw_ix: state.ordered_draws.len(),
                kind: EffectMarkerKind::CompositeGroupPop,
            });

            if state.scissor_stack.len() > 1 {
                state.scissor_stack.pop();
                state.current_scissor = *state
                    .scissor_stack
                    .last()
                    .expect("scissor stack must be non-empty");
            }

            let (head, count) = state.mask_scope_stack.pop().unwrap_or((0, 0));
            state.mask_scope_head = head;
            state.mask_scope_count = count;
            state.current_uniform_index = state.push_uniform_snapshot(
                state.clip_head,
                state.clip_count,
                state.mask_head,
                state.mask_count,
                state.mask_scope_head,
                state.mask_scope_count,
            );
        }

        SceneOp::Quad {
            rect,
            background,
            border,
            border_paint,
            corner_radii,
            ..
        } => {
            draw::encode_quad(
                renderer,
                state,
                rect,
                background,
                border,
                border_paint,
                corner_radii,
                None,
            );
        }
        SceneOp::StrokeRRect {
            rect,
            stroke,
            stroke_paint,
            corner_radii,
            style,
            ..
        } => {
            draw::encode_quad(
                renderer,
                state,
                rect,
                Paint::Solid(Color::TRANSPARENT).into(),
                stroke,
                stroke_paint,
                corner_radii,
                style.dash,
            );
        }
        SceneOp::ShadowRRect {
            rect,
            corner_radii,
            offset,
            spread,
            blur_radius,
            color,
            ..
        } => draw::encode_shadow_rrect(
            renderer,
            state,
            rect,
            corner_radii,
            offset,
            spread,
            blur_radius,
            color,
        ),
        SceneOp::Image {
            rect,
            image,
            fit,
            sampling,
            opacity,
            ..
        } => {
            draw::encode_image(renderer, state, rect, image, fit, sampling, opacity);
        }
        SceneOp::ImageRegion {
            rect,
            image,
            uv,
            sampling,
            opacity,
            ..
        } => {
            draw::encode_image_region(renderer, state, rect, image, uv, sampling, opacity);
        }
        SceneOp::MaskImage {
            rect,
            image,
            uv,
            sampling,
            color,
            opacity,
            ..
        } => {
            draw::encode_mask_image(renderer, state, rect, image, uv, sampling, color, opacity);
        }
        SceneOp::SvgMaskIcon {
            rect,
            svg,
            fit,
            color,
            opacity,
            ..
        } => {
            draw::encode_svg_mask_icon(renderer, state, rect, svg, fit, color, opacity);
        }
        SceneOp::SvgImage {
            rect,
            svg,
            fit,
            opacity,
            ..
        } => {
            draw::encode_svg_image(renderer, state, rect, svg, fit, opacity);
        }
        SceneOp::Text {
            origin,
            text,
            paint,
            outline,
            shadow,
            ..
        } => {
            draw::encode_text(renderer, state, origin, text, paint, outline, shadow);
        }
        SceneOp::Path {
            origin,
            path,
            paint,
            ..
        } => {
            draw::encode_path(renderer, state, origin, path, paint);
        }
        SceneOp::ViewportSurface {
            rect,
            target,
            opacity,
            ..
        } => {
            draw::encode_viewport_surface(renderer, state, rect, target, opacity);
        }
    }
}

fn effect_scissor(state: &EncodeState<'_>, bounds: Rect) -> ScissorRect {
    let (x, y, w, h) = rect_to_pixels(bounds, state.scale_factor);
    let bounds_scissor = if w <= 0.0 || h <= 0.0 {
        Some(ScissorRect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        })
    } else {
        let t_px = state.current_transform_px();
        let quad = transform_quad_points_px(t_px, x, y, w, h);
        let (min_x, min_y, max_x, max_y) = bounds_of_quad_points(&quad);
        scissor_from_bounds_px(min_x, min_y, max_x, max_y, state.viewport_size)
    };

    let Some(bounds_scissor) = bounds_scissor else {
        return ScissorRect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
    };

    intersect_scissor(state.current_scissor, bounds_scissor)
}
