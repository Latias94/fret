use super::super::super::frame_targets::FrameTargets;
use super::super::super::*;
use super::super::executor::{RecordPassCtx, RecordPassResources, RenderSceneExecutor};
use super::super::helpers::{ensure_color_dst_view_owned, set_scissor_rect_absolute};

pub(in super::super) struct SceneDrawRangePassArgs<'a> {
    pub(in super::super) device: &'a wgpu::Device,
    pub(in super::super) format: wgpu::TextureFormat,
    pub(in super::super) target_view: &'a wgpu::TextureView,
    pub(in super::super) usage: wgpu::TextureUsages,
    pub(in super::super) encoder: &'a mut wgpu::CommandEncoder,
    pub(in super::super) frame_targets: &'a mut FrameTargets,
    pub(in super::super) encoding: &'a SceneEncoding,
    pub(in super::super) render_space_offset_u32: u32,
    pub(in super::super) perf_enabled: bool,
    pub(in super::super) frame_perf: &'a mut RenderPerfStats,
    pub(in super::super) plan: &'a RenderPlan,
    pub(in super::super) scene_pass: &'a SceneDrawRangePass,
    pub(in super::super) viewport_vertex_buffer: &'a wgpu::Buffer,
    pub(in super::super) text_vertex_buffer: &'a wgpu::Buffer,
    pub(in super::super) path_vertex_buffer: &'a wgpu::Buffer,
    pub(in super::super) quad_instance_bind_group: &'a wgpu::BindGroup,
    pub(in super::super) text_paint_bind_group: &'a wgpu::BindGroup,
    pub(in super::super) path_paint_bind_group: &'a wgpu::BindGroup,
}

pub(in super::super) fn record_scene_draw_range_pass(
    exec: &mut RenderSceneExecutor<'_>,
    ctx: &RecordPassCtx<'_>,
    resources: &RecordPassResources<'_>,
    scene_pass: &SceneDrawRangePass,
) {
    let device = exec.device;
    let format = exec.format;
    let target_view = exec.target_view;
    let usage = exec.usage;
    let encoder = &mut *exec.encoder;
    let frame_targets = &mut *exec.frame_targets;
    let encoding = exec.encoding;
    let perf_enabled = exec.perf_enabled;
    let frame_perf = &mut *exec.frame_perf;

    let renderer = &mut *exec.renderer;

    let mut args = SceneDrawRangePassArgs {
        device,
        format,
        target_view,
        usage,
        encoder,
        frame_targets,
        encoding,
        render_space_offset_u32: ctx.render_space_offset_u32,
        perf_enabled,
        frame_perf,
        plan: ctx.plan,
        scene_pass,
        viewport_vertex_buffer: resources.viewport_vertex_buffer,
        text_vertex_buffer: resources.text_vertex_buffer,
        path_vertex_buffer: resources.path_vertex_buffer,
        quad_instance_bind_group: resources.quad_instance_bind_group,
        text_paint_bind_group: resources.text_paint_bind_group,
        path_paint_bind_group: resources.path_paint_bind_group,
    };
    renderer.record_scene_draw_range_pass(&mut args);
}

impl Renderer {
    pub(in super::super) fn record_scene_draw_range_pass(
        &mut self,
        args: &mut SceneDrawRangePassArgs<'_>,
    ) {
        let device = args.device;
        let format = args.format;
        let target_view = args.target_view;
        let usage = args.usage;
        let encoder = &mut *args.encoder;
        let frame_targets = &mut *args.frame_targets;
        let encoding = args.encoding;
        let render_space_offset_u32 = args.render_space_offset_u32;
        let perf_enabled = args.perf_enabled;
        let frame_perf = &mut *args.frame_perf;
        let plan = args.plan;
        let scene_pass = args.scene_pass;
        let viewport_vertex_buffer = args.viewport_vertex_buffer;
        let text_vertex_buffer = args.text_vertex_buffer;
        let path_vertex_buffer = args.path_vertex_buffer;
        let quad_instance_bind_group = args.quad_instance_bind_group;
        let text_paint_bind_group = args.text_paint_bind_group;
        let path_paint_bind_group = args.path_paint_bind_group;

        debug_assert!(scene_pass.segment.0 < plan.segments.len());
        let target_origin = scene_pass.target_origin;
        let target_size = scene_pass.target_size;
        let load = scene_pass.load;
        let pass_target_view_owned = ensure_color_dst_view_owned(
            frame_targets,
            &mut self.intermediate_pool,
            device,
            scene_pass.target,
            target_size,
            format,
            usage,
            "SceneDrawRange",
        );
        let pass_target_view = pass_target_view_owned.as_ref().unwrap_or(target_view);

        {
            enum ActivePipeline {
                None,
                Quad,
                Viewport,
                TextMask,
                TextMaskOutline,
                TextColor,
                TextSubpixel,
                TextSubpixelOutline,
                Mask,
                Path,
            }

            for item in &encoding.ordered_draws[scene_pass.draw_range.clone()] {
                let OrderedDraw::Quad(draw) = item else {
                    continue;
                };
                let _ = self.quad_pipeline(device, format, draw.pipeline);
            }

            let viewport_pipeline = self
                .viewport_pipeline
                .as_ref()
                .expect("viewport pipeline must exist");
            let text_pipeline = self
                .text_pipeline
                .as_ref()
                .expect("text pipeline must exist");
            let text_outline_pipeline = self
                .text_outline_pipeline
                .as_ref()
                .expect("text outline pipeline must exist");
            let text_color_pipeline = self
                .text_color_pipeline
                .as_ref()
                .expect("text color pipeline must exist");
            let text_subpixel_pipeline = self
                .text_subpixel_pipeline
                .as_ref()
                .expect("text subpixel pipeline must exist");
            let text_subpixel_outline_pipeline = self
                .text_subpixel_outline_pipeline
                .as_ref()
                .expect("text subpixel outline pipeline must exist");
            let mask_pipeline = self
                .mask_pipeline
                .as_ref()
                .expect("mask pipeline must exist");
            let path_pipeline = self
                .path_pipeline
                .as_ref()
                .expect("path pipeline must exist");

            let mut active_pipeline = ActivePipeline::None;
            let mut active_text_page: Option<u16> = None;
            let mut active_quad_pipeline: Option<QuadPipelineKey> = None;

            fn begin_main_pass<'a>(
                encoder: &'a mut wgpu::CommandEncoder,
                target_view: &'a wgpu::TextureView,
                load: wgpu::LoadOp<wgpu::Color>,
            ) -> wgpu::RenderPass<'a> {
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("fret renderer pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target_view,
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
                })
            }

            let mut pass = begin_main_pass(encoder, pass_target_view, load);
            let mut active_uniform_offset: Option<u32> = None;
            let mut active_mask_image: Option<UniformMaskImageSelection> = None;
            let mut active_scissor: Option<ScissorRect> = None;

            let mut set_scissor =
                |pass: &mut wgpu::RenderPass<'_>,
                 scissor: ScissorRect,
                 frame_perf: &mut RenderPerfStats| {
                    if active_scissor != Some(scissor) {
                        if set_scissor_rect_absolute(pass, scissor, target_origin, target_size)
                            && perf_enabled
                        {
                            frame_perf.scissor_sets = frame_perf.scissor_sets.saturating_add(1);
                        }
                        active_scissor = Some(scissor);
                    }
                };

            let mut set_uniform =
                |pass: &mut wgpu::RenderPass<'_>,
                 uniform_bind_group: &wgpu::BindGroup,
                 uniform_offset: u32,
                 mask_image: Option<UniformMaskImageSelection>,
                 frame_perf: &mut RenderPerfStats| {
                    if active_uniform_offset != Some(uniform_offset)
                        || active_mask_image != mask_image
                    {
                        pass.set_bind_group(
                            0,
                            uniform_bind_group,
                            &[uniform_offset, render_space_offset_u32],
                        );
                        if perf_enabled {
                            frame_perf.bind_group_switches =
                                frame_perf.bind_group_switches.saturating_add(1);
                            frame_perf.uniform_bind_group_switches =
                                frame_perf.uniform_bind_group_switches.saturating_add(1);
                        }
                        active_uniform_offset = Some(uniform_offset);
                        active_mask_image = mask_image;
                    }
                };

            let mut i = scene_pass.draw_range.start;
            while i < scene_pass.draw_range.end {
                let item = &encoding.ordered_draws[i];

                match item {
                    OrderedDraw::Quad(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        let quad_pipeline = self
                            .quad_pipelines
                            .get(&draw.pipeline)
                            .expect("quad pipeline must exist");
                        if !matches!(active_pipeline, ActivePipeline::Quad)
                            || active_quad_pipeline != Some(draw.pipeline)
                        {
                            pass.set_pipeline(quad_pipeline);
                            if perf_enabled {
                                frame_perf.pipeline_switches =
                                    frame_perf.pipeline_switches.saturating_add(1);
                                frame_perf.pipeline_switches_quad =
                                    frame_perf.pipeline_switches_quad.saturating_add(1);
                            }
                            if !matches!(active_pipeline, ActivePipeline::Quad) {
                                pass.set_bind_group(1, quad_instance_bind_group, &[]);
                                if perf_enabled {
                                    frame_perf.bind_group_switches =
                                        frame_perf.bind_group_switches.saturating_add(1);
                                }
                                active_pipeline = ActivePipeline::Quad;
                            }
                            active_quad_pipeline = Some(draw.pipeline);
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        let mask_image = encoding
                            .uniform_mask_images
                            .get(draw.uniform_index as usize)
                            .copied()
                            .flatten();
                        let uniform_bind_group =
                            self.pick_uniform_bind_group_for_mask_image(mask_image);

                        set_uniform(
                            &mut pass,
                            uniform_bind_group,
                            uniform_offset,
                            mask_image,
                            frame_perf,
                        );
                        set_scissor(&mut pass, draw.scissor, frame_perf);
                        pass.draw(
                            0..6,
                            draw.first_instance..(draw.first_instance + draw.instance_count),
                        );
                        if perf_enabled {
                            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                            frame_perf.quad_draw_calls =
                                frame_perf.quad_draw_calls.saturating_add(1);
                        }
                    }
                    OrderedDraw::Viewport(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Viewport) {
                            pass.set_pipeline(viewport_pipeline);
                            if perf_enabled {
                                frame_perf.pipeline_switches =
                                    frame_perf.pipeline_switches.saturating_add(1);
                                frame_perf.pipeline_switches_viewport =
                                    frame_perf.pipeline_switches_viewport.saturating_add(1);
                            }
                            pass.set_vertex_buffer(0, viewport_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Viewport;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        let mask_image = encoding
                            .uniform_mask_images
                            .get(draw.uniform_index as usize)
                            .copied()
                            .flatten();
                        let uniform_bind_group =
                            self.pick_uniform_bind_group_for_mask_image(mask_image);

                        set_uniform(
                            &mut pass,
                            uniform_bind_group,
                            uniform_offset,
                            mask_image,
                            frame_perf,
                        );
                        let Some(bind_group) =
                            self.bind_group_caches.get_viewport_bind_group(draw.target)
                        else {
                            // Missing bind group should only happen if the target vanished
                            // between encoding and rendering.
                            i += 1;
                            continue;
                        };
                        pass.set_bind_group(1, bind_group, &[]);
                        if perf_enabled {
                            frame_perf.bind_group_switches =
                                frame_perf.bind_group_switches.saturating_add(1);
                            frame_perf.texture_bind_group_switches =
                                frame_perf.texture_bind_group_switches.saturating_add(1);
                        }
                        set_scissor(&mut pass, draw.scissor, frame_perf);
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                        if perf_enabled {
                            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                            frame_perf.viewport_draw_calls =
                                frame_perf.viewport_draw_calls.saturating_add(1);
                            let metadata = self
                                .render_targets
                                .metadata(draw.target)
                                .unwrap_or_default();
                            match metadata.ingest_strategy {
                                fret_render_core::RenderTargetIngestStrategy::Unknown => {
                                    frame_perf.viewport_draw_calls_ingest_unknown = frame_perf
                                        .viewport_draw_calls_ingest_unknown
                                        .saturating_add(1);
                                }
                                fret_render_core::RenderTargetIngestStrategy::Owned => {
                                    frame_perf.viewport_draw_calls_ingest_owned = frame_perf
                                        .viewport_draw_calls_ingest_owned
                                        .saturating_add(1);
                                }
                                fret_render_core::RenderTargetIngestStrategy::ExternalZeroCopy => {
                                    frame_perf.viewport_draw_calls_ingest_external_zero_copy =
                                        frame_perf
                                            .viewport_draw_calls_ingest_external_zero_copy
                                            .saturating_add(1);
                                }
                                fret_render_core::RenderTargetIngestStrategy::GpuCopy => {
                                    frame_perf.viewport_draw_calls_ingest_gpu_copy = frame_perf
                                        .viewport_draw_calls_ingest_gpu_copy
                                        .saturating_add(1);
                                }
                                fret_render_core::RenderTargetIngestStrategy::CpuUpload => {
                                    frame_perf.viewport_draw_calls_ingest_cpu_upload = frame_perf
                                        .viewport_draw_calls_ingest_cpu_upload
                                        .saturating_add(1);
                                }
                            }
                        }
                    }
                    OrderedDraw::Image(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Viewport) {
                            pass.set_pipeline(viewport_pipeline);
                            if perf_enabled {
                                frame_perf.pipeline_switches =
                                    frame_perf.pipeline_switches.saturating_add(1);
                                frame_perf.pipeline_switches_viewport =
                                    frame_perf.pipeline_switches_viewport.saturating_add(1);
                            }
                            pass.set_vertex_buffer(0, viewport_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Viewport;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        let mask_image = encoding
                            .uniform_mask_images
                            .get(draw.uniform_index as usize)
                            .copied()
                            .flatten();
                        let uniform_bind_group =
                            self.pick_uniform_bind_group_for_mask_image(mask_image);

                        set_uniform(
                            &mut pass,
                            uniform_bind_group,
                            uniform_offset,
                            mask_image,
                            frame_perf,
                        );
                        let Some(bind_group) =
                            self.pick_image_bind_group(draw.image, draw.sampling)
                        else {
                            i += 1;
                            continue;
                        };
                        pass.set_bind_group(1, bind_group, &[]);
                        if perf_enabled {
                            frame_perf.bind_group_switches =
                                frame_perf.bind_group_switches.saturating_add(1);
                            frame_perf.texture_bind_group_switches =
                                frame_perf.texture_bind_group_switches.saturating_add(1);
                        }
                        set_scissor(&mut pass, draw.scissor, frame_perf);
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                        if perf_enabled {
                            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                            frame_perf.image_draw_calls =
                                frame_perf.image_draw_calls.saturating_add(1);
                        }
                    }
                    OrderedDraw::Mask(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Mask) {
                            pass.set_pipeline(mask_pipeline);
                            if perf_enabled {
                                frame_perf.pipeline_switches =
                                    frame_perf.pipeline_switches.saturating_add(1);
                                frame_perf.pipeline_switches_mask =
                                    frame_perf.pipeline_switches_mask.saturating_add(1);
                            }
                            pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Mask;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        let mask_image = encoding
                            .uniform_mask_images
                            .get(draw.uniform_index as usize)
                            .copied()
                            .flatten();
                        let uniform_bind_group =
                            self.pick_uniform_bind_group_for_mask_image(mask_image);

                        set_uniform(
                            &mut pass,
                            uniform_bind_group,
                            uniform_offset,
                            mask_image,
                            frame_perf,
                        );
                        let Some(bind_group) =
                            self.pick_image_bind_group(draw.image, draw.sampling)
                        else {
                            i += 1;
                            continue;
                        };
                        pass.set_bind_group(1, bind_group, &[]);
                        if perf_enabled {
                            frame_perf.bind_group_switches =
                                frame_perf.bind_group_switches.saturating_add(1);
                            frame_perf.texture_bind_group_switches =
                                frame_perf.texture_bind_group_switches.saturating_add(1);
                        }
                        set_scissor(&mut pass, draw.scissor, frame_perf);
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                        if perf_enabled {
                            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                            frame_perf.mask_draw_calls =
                                frame_perf.mask_draw_calls.saturating_add(1);
                        }
                    }
                    OrderedDraw::Text(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        match draw.kind {
                            TextDrawKind::Mask => {
                                if !matches!(active_pipeline, ActivePipeline::TextMask) {
                                    pass.set_pipeline(text_pipeline);
                                    if perf_enabled {
                                        frame_perf.pipeline_switches =
                                            frame_perf.pipeline_switches.saturating_add(1);
                                        frame_perf.pipeline_switches_text_mask = frame_perf
                                            .pipeline_switches_text_mask
                                            .saturating_add(1);
                                    }
                                    pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.mask_atlas_bind_group(draw.atlas_page),
                                        &[],
                                    );
                                    pass.set_bind_group(2, text_paint_bind_group, &[]);
                                    if perf_enabled {
                                        frame_perf.bind_group_switches =
                                            frame_perf.bind_group_switches.saturating_add(1);
                                        frame_perf.texture_bind_group_switches = frame_perf
                                            .texture_bind_group_switches
                                            .saturating_add(1);
                                    }
                                    active_pipeline = ActivePipeline::TextMask;
                                    active_text_page = Some(draw.atlas_page);
                                } else if active_text_page != Some(draw.atlas_page) {
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.mask_atlas_bind_group(draw.atlas_page),
                                        &[],
                                    );
                                    if perf_enabled {
                                        frame_perf.bind_group_switches =
                                            frame_perf.bind_group_switches.saturating_add(1);
                                        frame_perf.texture_bind_group_switches = frame_perf
                                            .texture_bind_group_switches
                                            .saturating_add(1);
                                    }
                                    active_text_page = Some(draw.atlas_page);
                                }
                            }
                            TextDrawKind::MaskOutline => {
                                if !matches!(active_pipeline, ActivePipeline::TextMaskOutline) {
                                    pass.set_pipeline(text_outline_pipeline);
                                    if perf_enabled {
                                        frame_perf.pipeline_switches =
                                            frame_perf.pipeline_switches.saturating_add(1);
                                        frame_perf.pipeline_switches_text_mask = frame_perf
                                            .pipeline_switches_text_mask
                                            .saturating_add(1);
                                    }
                                    pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.mask_atlas_bind_group(draw.atlas_page),
                                        &[],
                                    );
                                    pass.set_bind_group(2, text_paint_bind_group, &[]);
                                    if perf_enabled {
                                        frame_perf.bind_group_switches =
                                            frame_perf.bind_group_switches.saturating_add(1);
                                        frame_perf.texture_bind_group_switches = frame_perf
                                            .texture_bind_group_switches
                                            .saturating_add(1);
                                    }
                                    active_pipeline = ActivePipeline::TextMaskOutline;
                                    active_text_page = Some(draw.atlas_page);
                                } else if active_text_page != Some(draw.atlas_page) {
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.mask_atlas_bind_group(draw.atlas_page),
                                        &[],
                                    );
                                    if perf_enabled {
                                        frame_perf.bind_group_switches =
                                            frame_perf.bind_group_switches.saturating_add(1);
                                        frame_perf.texture_bind_group_switches = frame_perf
                                            .texture_bind_group_switches
                                            .saturating_add(1);
                                    }
                                    active_text_page = Some(draw.atlas_page);
                                }
                            }
                            TextDrawKind::Color => {
                                if !matches!(active_pipeline, ActivePipeline::TextColor) {
                                    pass.set_pipeline(text_color_pipeline);
                                    if perf_enabled {
                                        frame_perf.pipeline_switches =
                                            frame_perf.pipeline_switches.saturating_add(1);
                                        frame_perf.pipeline_switches_text_color = frame_perf
                                            .pipeline_switches_text_color
                                            .saturating_add(1);
                                    }
                                    pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.color_atlas_bind_group(draw.atlas_page),
                                        &[],
                                    );
                                    pass.set_bind_group(2, text_paint_bind_group, &[]);
                                    if perf_enabled {
                                        frame_perf.bind_group_switches =
                                            frame_perf.bind_group_switches.saturating_add(1);
                                        frame_perf.texture_bind_group_switches = frame_perf
                                            .texture_bind_group_switches
                                            .saturating_add(1);
                                    }
                                    active_pipeline = ActivePipeline::TextColor;
                                    active_text_page = Some(draw.atlas_page);
                                } else if active_text_page != Some(draw.atlas_page) {
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.color_atlas_bind_group(draw.atlas_page),
                                        &[],
                                    );
                                    if perf_enabled {
                                        frame_perf.bind_group_switches =
                                            frame_perf.bind_group_switches.saturating_add(1);
                                        frame_perf.texture_bind_group_switches = frame_perf
                                            .texture_bind_group_switches
                                            .saturating_add(1);
                                    }
                                    active_text_page = Some(draw.atlas_page);
                                }
                            }
                            TextDrawKind::Subpixel => {
                                if !matches!(active_pipeline, ActivePipeline::TextSubpixel) {
                                    pass.set_pipeline(text_subpixel_pipeline);
                                    if perf_enabled {
                                        frame_perf.pipeline_switches =
                                            frame_perf.pipeline_switches.saturating_add(1);
                                        frame_perf.pipeline_switches_text_subpixel = frame_perf
                                            .pipeline_switches_text_subpixel
                                            .saturating_add(1);
                                    }
                                    pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.subpixel_atlas_bind_group(draw.atlas_page),
                                        &[],
                                    );
                                    pass.set_bind_group(2, text_paint_bind_group, &[]);
                                    if perf_enabled {
                                        frame_perf.bind_group_switches =
                                            frame_perf.bind_group_switches.saturating_add(1);
                                        frame_perf.texture_bind_group_switches = frame_perf
                                            .texture_bind_group_switches
                                            .saturating_add(1);
                                    }
                                    active_pipeline = ActivePipeline::TextSubpixel;
                                    active_text_page = Some(draw.atlas_page);
                                } else if active_text_page != Some(draw.atlas_page) {
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.subpixel_atlas_bind_group(draw.atlas_page),
                                        &[],
                                    );
                                    if perf_enabled {
                                        frame_perf.bind_group_switches =
                                            frame_perf.bind_group_switches.saturating_add(1);
                                        frame_perf.texture_bind_group_switches = frame_perf
                                            .texture_bind_group_switches
                                            .saturating_add(1);
                                    }
                                    active_text_page = Some(draw.atlas_page);
                                }
                            }
                            TextDrawKind::SubpixelOutline => {
                                if !matches!(active_pipeline, ActivePipeline::TextSubpixelOutline) {
                                    pass.set_pipeline(text_subpixel_outline_pipeline);
                                    if perf_enabled {
                                        frame_perf.pipeline_switches =
                                            frame_perf.pipeline_switches.saturating_add(1);
                                        frame_perf.pipeline_switches_text_subpixel = frame_perf
                                            .pipeline_switches_text_subpixel
                                            .saturating_add(1);
                                    }
                                    pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.subpixel_atlas_bind_group(draw.atlas_page),
                                        &[],
                                    );
                                    pass.set_bind_group(2, text_paint_bind_group, &[]);
                                    if perf_enabled {
                                        frame_perf.bind_group_switches =
                                            frame_perf.bind_group_switches.saturating_add(1);
                                        frame_perf.texture_bind_group_switches = frame_perf
                                            .texture_bind_group_switches
                                            .saturating_add(1);
                                    }
                                    active_pipeline = ActivePipeline::TextSubpixelOutline;
                                    active_text_page = Some(draw.atlas_page);
                                } else if active_text_page != Some(draw.atlas_page) {
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.subpixel_atlas_bind_group(draw.atlas_page),
                                        &[],
                                    );
                                    if perf_enabled {
                                        frame_perf.bind_group_switches =
                                            frame_perf.bind_group_switches.saturating_add(1);
                                        frame_perf.texture_bind_group_switches = frame_perf
                                            .texture_bind_group_switches
                                            .saturating_add(1);
                                    }
                                    active_text_page = Some(draw.atlas_page);
                                }
                            }
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        let mask_image = encoding
                            .uniform_mask_images
                            .get(draw.uniform_index as usize)
                            .copied()
                            .flatten();
                        let uniform_bind_group =
                            self.pick_uniform_bind_group_for_mask_image(mask_image);

                        set_uniform(
                            &mut pass,
                            uniform_bind_group,
                            uniform_offset,
                            mask_image,
                            frame_perf,
                        );
                        set_scissor(&mut pass, draw.scissor, frame_perf);
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            draw.paint_index..(draw.paint_index + 1),
                        );
                        if perf_enabled {
                            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                            frame_perf.text_draw_calls =
                                frame_perf.text_draw_calls.saturating_add(1);
                        }
                    }
                    OrderedDraw::Path(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Path) {
                            pass.set_pipeline(path_pipeline);
                            if perf_enabled {
                                frame_perf.pipeline_switches =
                                    frame_perf.pipeline_switches.saturating_add(1);
                                frame_perf.pipeline_switches_path =
                                    frame_perf.pipeline_switches_path.saturating_add(1);
                            }
                            pass.set_bind_group(1, path_paint_bind_group, &[]);
                            if perf_enabled {
                                frame_perf.bind_group_switches =
                                    frame_perf.bind_group_switches.saturating_add(1);
                            }
                            pass.set_vertex_buffer(0, path_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Path;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        let mask_image = encoding
                            .uniform_mask_images
                            .get(draw.uniform_index as usize)
                            .copied()
                            .flatten();
                        let uniform_bind_group =
                            self.pick_uniform_bind_group_for_mask_image(mask_image);

                        set_uniform(
                            &mut pass,
                            uniform_bind_group,
                            uniform_offset,
                            mask_image,
                            frame_perf,
                        );
                        set_scissor(&mut pass, draw.scissor, frame_perf);
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            draw.paint_index..(draw.paint_index + 1),
                        );
                        if perf_enabled {
                            frame_perf.draw_calls = frame_perf.draw_calls.saturating_add(1);
                            frame_perf.path_draw_calls =
                                frame_perf.path_draw_calls.saturating_add(1);
                        }
                    }
                }

                i += 1;
            }
        }
    }
}
