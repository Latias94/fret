fn apply_dst_local_scissor(
    rp: &mut wgpu::RenderPass<'_>,
    dst_scissor: Option<super::LocalScissorRect>,
    dst_size: (u32, u32),
) -> bool {
    if dst_size.0 == 0 || dst_size.1 == 0 {
        return false;
    }

    let Some(scissor) = dst_scissor.map(|s| s.0) else {
        return false;
    };
    if scissor.w == 0 || scissor.h == 0 {
        return false;
    }

    let x0 = scissor.x.min(dst_size.0);
    let y0 = scissor.y.min(dst_size.1);
    let x1 = scissor.x.saturating_add(scissor.w).min(dst_size.0);
    let y1 = scissor.y.saturating_add(scissor.h).min(dst_size.1);

    let w = x1.saturating_sub(x0);
    let h = y1.saturating_sub(y0);
    if w == 0 || h == 0 {
        return false;
    }

    rp.set_scissor_rect(x0, y0, w, h);
    true
}

pub(super) fn run_fullscreen_triangle_pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &str,
    pipeline: &wgpu::RenderPipeline,
    dst_view: &wgpu::TextureView,
    dst_size: (u32, u32),
    load: wgpu::LoadOp<wgpu::Color>,
    bind_group0: &wgpu::BindGroup,
    bind_group0_offsets: &[u32],
    dst_scissor: Option<super::LocalScissorRect>,
    perf: Option<&mut super::RenderPerfStats>,
) {
    let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: dst_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    rp.set_pipeline(pipeline);
    rp.set_bind_group(0, bind_group0, bind_group0_offsets);
    let has_scissor = dst_scissor.is_some();
    let _ = apply_dst_local_scissor(&mut rp, dst_scissor, dst_size);
    rp.draw(0..3, 0..1);

    if let Some(perf) = perf {
        perf.pipeline_switches = perf.pipeline_switches.saturating_add(1);
        perf.pipeline_switches_fullscreen = perf.pipeline_switches_fullscreen.saturating_add(1);
        perf.bind_group_switches = perf.bind_group_switches.saturating_add(1);
        perf.texture_bind_group_switches = perf.texture_bind_group_switches.saturating_add(1);
        if has_scissor {
            perf.scissor_sets = perf.scissor_sets.saturating_add(1);
        }
        perf.draw_calls = perf.draw_calls.saturating_add(1);
        perf.fullscreen_draw_calls = perf.fullscreen_draw_calls.saturating_add(1);
    }
}

pub(super) fn run_fullscreen_triangle_pass_uniform_texture(
    encoder: &mut wgpu::CommandEncoder,
    label: &str,
    pipeline: &wgpu::RenderPipeline,
    dst_view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    uniform_bind_group: &wgpu::BindGroup,
    uniform_offsets: &[u32],
    texture_bind_group: &wgpu::BindGroup,
    texture_offsets: &[u32],
    dst_scissor: Option<super::LocalScissorRect>,
    dst_size: (u32, u32),
    perf: Option<&mut super::RenderPerfStats>,
) {
    let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: dst_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    rp.set_pipeline(pipeline);
    rp.set_bind_group(0, uniform_bind_group, uniform_offsets);
    rp.set_bind_group(1, texture_bind_group, texture_offsets);

    let applied_scissor = apply_dst_local_scissor(&mut rp, dst_scissor, dst_size);

    rp.draw(0..3, 0..1);

    if let Some(perf) = perf {
        perf.pipeline_switches = perf.pipeline_switches.saturating_add(1);
        perf.pipeline_switches_fullscreen = perf.pipeline_switches_fullscreen.saturating_add(1);

        perf.bind_group_switches = perf.bind_group_switches.saturating_add(2);
        perf.uniform_bind_group_switches = perf.uniform_bind_group_switches.saturating_add(1);
        perf.texture_bind_group_switches = perf.texture_bind_group_switches.saturating_add(1);

        if applied_scissor {
            perf.scissor_sets = perf.scissor_sets.saturating_add(1);
        }
        perf.draw_calls = perf.draw_calls.saturating_add(1);
        perf.fullscreen_draw_calls = perf.fullscreen_draw_calls.saturating_add(1);
    }
}

pub(super) fn run_clip_mask_triangle_pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &str,
    pipeline: &wgpu::RenderPipeline,
    dst_view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    uniform_bind_group: &wgpu::BindGroup,
    uniform_offsets: &[u32],
    param_bind_group: &wgpu::BindGroup,
    param_offsets: &[u32],
    dst_scissor: Option<super::LocalScissorRect>,
    dst_size: (u32, u32),
    perf: Option<&mut super::RenderPerfStats>,
) {
    let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: dst_view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    rp.set_pipeline(pipeline);
    rp.set_bind_group(0, uniform_bind_group, uniform_offsets);
    rp.set_bind_group(1, param_bind_group, param_offsets);

    let applied_scissor = apply_dst_local_scissor(&mut rp, dst_scissor, dst_size);

    rp.draw(0..3, 0..1);

    if let Some(perf) = perf {
        perf.pipeline_switches = perf.pipeline_switches.saturating_add(1);
        perf.pipeline_switches_clip_mask = perf.pipeline_switches_clip_mask.saturating_add(1);

        perf.bind_group_switches = perf.bind_group_switches.saturating_add(2);
        perf.uniform_bind_group_switches = perf.uniform_bind_group_switches.saturating_add(1);
        perf.texture_bind_group_switches = perf.texture_bind_group_switches.saturating_add(1);

        if applied_scissor {
            perf.scissor_sets = perf.scissor_sets.saturating_add(1);
        }
        perf.draw_calls = perf.draw_calls.saturating_add(1);
        perf.clip_mask_draw_calls = perf.clip_mask_draw_calls.saturating_add(1);
    }
}

pub(super) fn create_texture_bind_group(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(texture_view),
        }],
    })
}

pub(super) fn create_texture_uniform_bind_group(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
    uniform_binding: wgpu::BindingResource<'_>,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: uniform_binding,
            },
        ],
    })
}
