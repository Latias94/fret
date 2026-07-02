pub(super) use super::super::*;

mod clip;
mod draw;
mod mask;
mod ops;
mod state;

use state::EncodeState;

use fret_core::time::Instant;

impl Renderer {
    pub(in crate::renderer) fn encode_scene_ops_into(
        &mut self,
        scene: &Scene,
        scale_factor: f32,
        viewport_size: (u32, u32),
        output_is_srgb: bool,
        encoding: &mut SceneEncoding,
        perf_enabled: bool,
        encode_family_profile_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) {
        self.encode_scene_op_slice_into(
            scene.ops(),
            None,
            scale_factor,
            viewport_size,
            output_is_srgb,
            encoding,
            perf_enabled,
            encode_family_profile_enabled,
            frame_perf,
        );
    }

    pub(in crate::renderer) fn encode_scene_op_slice_into(
        &mut self,
        ops: &[SceneOp],
        initial_transform: Option<Transform2D>,
        scale_factor: f32,
        viewport_size: (u32, u32),
        output_is_srgb: bool,
        encoding: &mut SceneEncoding,
        perf_enabled: bool,
        encode_family_profile_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) {
        encoding.clear();
        let (text_gamma_ratios, text_grayscale_enhanced_contrast, text_subpixel_enhanced_contrast) =
            self.text_system.text_quality_uniforms();
        let mut state = EncodeState::new(
            encoding,
            scale_factor,
            viewport_size,
            output_is_srgb,
            text_gamma_ratios,
            text_grayscale_enhanced_contrast,
            text_subpixel_enhanced_contrast,
            self.material_effect_state.material_paint_budget_per_frame,
            self.material_effect_state
                .material_distinct_budget_per_frame,
        );
        if let Some(transform) = initial_transform
            && transform != Transform2D::IDENTITY
        {
            state.set_initial_transform(transform);
        }

        for op in ops {
            if perf_enabled {
                let family = encode_scene_family(op);
                let start = encode_family_profile_enabled.then(Instant::now);
                ops::handle_op(self, &mut state, op, perf_enabled, frame_perf);
                frame_perf.record_encode_scene_family(family, start.map(|start| start.elapsed()));
            } else {
                ops::handle_op(self, &mut state, op, perf_enabled, frame_perf);
            }
        }

        if perf_enabled {
            let start = encode_family_profile_enabled.then(Instant::now);
            state.flush_quad_batch();
            frame_perf.record_encode_scene_family(
                EncodeSceneFamily::Flush,
                start.map(|start| start.elapsed()),
            );
        } else {
            state.flush_quad_batch();
        }
    }
}

fn encode_scene_family(op: &SceneOp) -> EncodeSceneFamily {
    match *op {
        SceneOp::PushTransform { .. }
        | SceneOp::PopTransform
        | SceneOp::PushOpacity { .. }
        | SceneOp::PopOpacity
        | SceneOp::PushLayer { .. }
        | SceneOp::PopLayer => EncodeSceneFamily::Stack,

        SceneOp::PushClipRect { .. }
        | SceneOp::PushClipRRect { .. }
        | SceneOp::PushClipPath { .. }
        | SceneOp::PopClip => EncodeSceneFamily::Clip,

        SceneOp::PushMask { .. }
        | SceneOp::PopMask
        | SceneOp::MaskImage { .. }
        | SceneOp::SvgMaskIcon { .. } => EncodeSceneFamily::Mask,

        SceneOp::PushEffect { .. }
        | SceneOp::PopEffect
        | SceneOp::PushBackdropSourceGroupV1 { .. }
        | SceneOp::PopBackdropSourceGroup
        | SceneOp::PushCompositeGroup { .. }
        | SceneOp::PopCompositeGroup => EncodeSceneFamily::Effect,

        SceneOp::Quad { .. }
        | SceneOp::StrokeRRect { .. }
        | SceneOp::ShadowRRect { .. }
        | SceneOp::VertexColorQuad { .. }
        | SceneOp::VertexColorTriangle { .. } => EncodeSceneFamily::Quad,

        SceneOp::Image { .. }
        | SceneOp::ImageRegion { .. }
        | SceneOp::ImageQuad { .. }
        | SceneOp::ImageTriangle { .. }
        | SceneOp::SvgImage { .. } => EncodeSceneFamily::Image,

        SceneOp::Text { .. } => EncodeSceneFamily::Text,
        SceneOp::Path { .. } => EncodeSceneFamily::Path,
        SceneOp::ViewportSurface { .. } => EncodeSceneFamily::Viewport,
    }
}
