use super::super::super::frame_targets::FrameTargets;
use super::super::super::*;
use super::super::helpers::set_scissor_rect_absolute;

impl Renderer {
    pub(in super::super) fn record_path_clip_mask_pass(
        &mut self,
        device: &wgpu::Device,
        usage: wgpu::TextureUsages,
        frame_index: u64,
        frame_targets: &mut FrameTargets,
        encoder: &mut wgpu::CommandEncoder,
        encoding: &SceneEncoding,
        path_vertex_buffer: &wgpu::Buffer,
        render_space_offset_u32: u32,
        perf_enabled: bool,
        frame_perf: &mut RenderPerfStats,
        mask_pass: &PathClipMaskPass,
    ) {
        let target_size = mask_pass.dst_size;

        let (pass_target_texture, pass_target_view) = frame_targets.ensure_target_with_texture(
            &mut self.intermediate_pool,
            device,
            mask_pass.dst,
            target_size,
            wgpu::TextureFormat::R8Unorm,
            usage,
        );

        if self.clip_path_mask_cache.try_copy_into(
            encoder,
            mask_pass.cache_key,
            target_size,
            pass_target_texture,
            frame_index,
        ) {
            if perf_enabled {
                frame_perf.clip_path_mask_cache_hits =
                    frame_perf.clip_path_mask_cache_hits.saturating_add(1);
            }
            return;
        }

        let uniform_offset = (mask_pass.uniform_index as u64).saturating_mul(self.uniform_stride);

        let vertex_size = std::mem::size_of::<PathVertex>() as u64;
        let first = (mask_pass.first_vertex as u64).saturating_mul(vertex_size);
        let size = (mask_pass.vertex_count as u64).saturating_mul(vertex_size);

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fret path clip-mask pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &pass_target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: mask_pass.load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let pipeline = self
                .path_clip_mask_pipeline
                .as_ref()
                .expect("path clip-mask pipeline must exist");
            rp.set_pipeline(pipeline);
            let mask_image = encoding
                .uniform_mask_images
                .get(mask_pass.uniform_index as usize)
                .copied()
                .flatten();
            let uniform_bind_group = self.pick_uniform_bind_group_for_mask_image(mask_image);
            rp.set_bind_group(
                0,
                uniform_bind_group,
                &[uniform_offset as u32, render_space_offset_u32],
            );

            if size != 0 {
                rp.set_vertex_buffer(0, path_vertex_buffer.slice(first..first + size));
                let _ = set_scissor_rect_absolute(
                    &mut rp,
                    mask_pass.scissor,
                    mask_pass.dst_origin,
                    mask_pass.dst_size,
                );
                rp.draw(0..mask_pass.vertex_count, 0..1);
            }
        }

        if perf_enabled {
            frame_perf.clip_mask_draw_calls = frame_perf.clip_mask_draw_calls.saturating_add(1);
            frame_perf.clip_path_mask_cache_misses =
                frame_perf.clip_path_mask_cache_misses.saturating_add(1);
        }

        self.clip_path_mask_cache.store_from(
            &mut self.intermediate_pool,
            device,
            encoder,
            mask_pass.cache_key,
            target_size,
            pass_target_texture,
            frame_index,
        );
    }
}
