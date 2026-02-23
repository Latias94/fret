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
        self.ensure_uniform_capacity(device, uniforms.len());
        let uniform_bytes_written = self.uniforms.write_viewport_uniforms(queue, uniforms) as u64;
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf
                .uniform_bytes
                .saturating_add(uniform_bytes_written);
        }

        self.ensure_clip_capacity(device, clips.len().max(1));
        let clip_bytes_written = self.uniforms.write_clips(queue, clips) as u64;
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf.uniform_bytes.saturating_add(clip_bytes_written);
        }

        self.ensure_mask_capacity(device, masks.len().max(1));
        let mask_bytes_written = self.uniforms.write_masks(queue, masks) as u64;
        if perf_enabled {
            frame_perf.uniform_bytes = frame_perf.uniform_bytes.saturating_add(mask_bytes_written);
        }

        self.prepare_viewport_bind_groups(device, ordered_draws);
        self.prepare_image_bind_groups(device, ordered_draws);
        self.prepare_uniform_mask_image_bind_groups(device, uniform_mask_images);
    }
}
