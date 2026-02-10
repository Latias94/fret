pub(super) use super::super::*;

mod clip;
mod draw;
mod ops;
mod state;

use state::EncodeState;

impl Renderer {
    pub(super) fn encode_scene_ops_into(
        &mut self,
        scene: &Scene,
        scale_factor: f32,
        viewport_size: (u32, u32),
        output_is_srgb: bool,
        encoding: &mut SceneEncoding,
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
            self.material_paint_budget_per_frame,
            self.material_distinct_budget_per_frame,
        );

        for op in scene.ops() {
            ops::handle_op(self, &mut state, op);
        }

        state.flush_quad_batch();
    }
}
