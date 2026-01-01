use super::clip;
use super::draw;
use super::state::EncodeState;
use super::*;

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
        SceneOp::PopClip => {
            clip::pop_clip(state);
        }

        SceneOp::Quad {
            rect,
            background,
            border,
            border_color,
            corner_radii,
            ..
        } => {
            draw::encode_quad(state, rect, background, border, border_color, corner_radii);
        }
        SceneOp::Image {
            rect,
            image,
            opacity,
            ..
        } => {
            draw::encode_image(renderer, state, rect, image, opacity);
        }
        SceneOp::ImageRegion {
            rect,
            image,
            uv,
            opacity,
            ..
        } => {
            draw::encode_image_region(renderer, state, rect, image, uv, opacity);
        }
        SceneOp::MaskImage {
            rect,
            image,
            uv,
            color,
            opacity,
            ..
        } => {
            draw::encode_mask_image(renderer, state, rect, image, uv, color, opacity);
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
            color,
            ..
        } => {
            draw::encode_text(renderer, state, origin, text, color);
        }
        SceneOp::Path {
            origin,
            path,
            color,
            ..
        } => {
            draw::encode_path(renderer, state, origin, path, color);
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
