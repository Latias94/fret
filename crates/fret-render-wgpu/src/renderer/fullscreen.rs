pub(super) fn run_fullscreen_triangle_pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &str,
    pipeline: &wgpu::RenderPipeline,
    dst_view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    bind_group0: &wgpu::BindGroup,
    bind_group0_offsets: &[u32],
    dst_scissor: Option<super::ScissorRect>,
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
    if let Some(scissor) = dst_scissor
        && scissor.w != 0
        && scissor.h != 0
    {
        rp.set_scissor_rect(scissor.x, scissor.y, scissor.w, scissor.h);
    }
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
