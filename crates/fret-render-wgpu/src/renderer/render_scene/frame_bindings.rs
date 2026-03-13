use super::super::*;

impl Renderer {
    pub(super) fn upload_frame_uniforms_and_prepare_bind_groups(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uniforms: &[ViewportUniform],
        clips: &[ClipRRectUniform],
        masks: &[MaskGradientUniform],
        ordered_draws: &[OrderedDraw],
        uniform_mask_images: &[Option<UniformMaskImageSelection>],
        perf_enabled: bool,
        frame_perf: &mut RenderPerfStats,
    ) {
        self.frame_binding_state
            .upload_frame_uniforms_and_prepare_bind_groups(
                device,
                queue,
                &self.globals,
                &mut self.gpu_resources,
                &mut self.frame_scratch_state,
                uniforms,
                clips,
                masks,
                ordered_draws,
                uniform_mask_images,
                perf_enabled,
                frame_perf,
            );
    }
}
