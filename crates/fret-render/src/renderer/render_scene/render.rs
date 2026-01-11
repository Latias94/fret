use super::super::*;
use fret_core::time::Instant;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

fn debug_render_path_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED
        .get_or_init(|| std::env::var_os("FRET_DEBUG_RENDER_PATH").is_some_and(|v| !v.is_empty()))
}

fn debug_render_path_should_log() -> bool {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed) < 32
}

fn debug_render_path_log_adapter(renderer: &Renderer) {
    static LOGGED: OnceLock<()> = OnceLock::new();
    if !debug_render_path_enabled() {
        return;
    }
    LOGGED.get_or_init(|| {
        let info = renderer.adapter.get_info();
        eprintln!(
            "render_scene: adapter name={:?} backend={:?} device_type={:?} vendor={:#x} device={:#x}",
            info.name, info.backend, info.device_type, info.vendor, info.device
        );
    });
}

fn path_intermediate_format(format: wgpu::TextureFormat) -> wgpu::TextureFormat {
    match format {
        // Prefer a linear intermediate to avoid relying on MSAA resolves into sRGB swapchain formats.
        // Composite into the final output format to apply any required color space conversions.
        //
        // Additionally, prefer RGBA8 for BGRA swapchains: we've observed cases where MSAA resolves
        // into BGRA8 intermediates produce no visible output on some Vulkan drivers, while RGBA8
        // works reliably.
        wgpu::TextureFormat::Bgra8UnormSrgb | wgpu::TextureFormat::Bgra8Unorm => {
            wgpu::TextureFormat::Rgba8Unorm
        }
        wgpu::TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8Unorm,
        other => other,
    }
}

impl Renderer {
    pub fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        params: RenderSceneParams<'_>,
    ) -> wgpu::CommandBuffer {
        let RenderSceneParams {
            format,
            target_view,
            scene,
            clear,
            scale_factor,
            viewport_size,
        } = params;

        #[cfg(debug_assertions)]
        if let Err(e) = scene.validate() {
            panic!("invalid scene: {e}");
        }

        self.ensure_viewport_pipeline(device, format);
        self.ensure_pipeline(device, format);
        self.ensure_text_pipeline(device, format);
        self.ensure_text_color_pipeline(device, format);
        self.ensure_mask_pipeline(device, format);
        self.ensure_path_pipeline(device, format);
        debug_render_path_log_adapter(self);
        let _backend = self.adapter.get_info().backend;
        let output_is_srgb = format.is_srgb();
        let intermediate_format = path_intermediate_format(format);
        let mut path_samples = if output_is_srgb {
            // The MSAA path implementation renders into an offscreen texture and then composites
            // into the main pass. When the final render target is not sRGB, the fragment shader
            // performs manual linear->sRGB encoding. Applying that transform twice (offscreen +
            // composite) would be incorrect, so we currently disable MSAA for non-sRGB outputs.
            //
            // Additionally, some backends/drivers are fragile when resolving MSAA into sRGB
            // textures. Prefer a linear intermediate for compatibility.
            self.effective_path_msaa_samples(intermediate_format)
        } else {
            1
        };
        if let Some(requested) = std::env::var_os("FRET_FORCE_PATH_MSAA_SAMPLES")
            .and_then(|v| v.to_string_lossy().trim().parse::<u32>().ok())
        {
            // Debug override to quickly validate whether rendering issues are MSAA-related.
            // Any non-zero value is clamped to a power-of-two, with `1` disabling the MSAA path
            // intermediate/composite path entirely.
            let requested = requested.max(1).min(16);
            let pow2_floor = 1u32 << (31 - requested.leading_zeros());
            path_samples = pow2_floor.max(1);
        }
        if debug_render_path_enabled() && debug_render_path_should_log() {
            eprintln!(
                "render_scene: format={:?} output_is_srgb={} path_msaa_samples={} intermediate_format={:?}",
                format, output_is_srgb, path_samples, intermediate_format
            );
        }
        if path_samples > 1 {
            self.ensure_composite_pipeline(device, format);
            self.ensure_path_msaa_pipeline(device, intermediate_format, path_samples);
            self.ensure_path_intermediate(device, viewport_size, intermediate_format, path_samples);
        }

        self.text_system.flush_uploads(queue);
        if self.svg_perf_enabled {
            self.svg_perf.frames = self.svg_perf.frames.saturating_add(1);
        }
        self.bump_svg_raster_epoch();
        let svg_prepare_start = self.svg_perf_enabled.then(Instant::now);
        self.prepare_svg_ops(device, queue, scene, scale_factor);
        if let Some(svg_prepare_start) = svg_prepare_start {
            self.svg_perf.prepare_svg_ops += svg_prepare_start.elapsed();
        }

        let key = SceneEncodingCacheKey {
            format,
            viewport_size,
            scale_factor_bits: scale_factor.to_bits(),
            scene_fingerprint: scene.fingerprint(),
            scene_ops_len: scene.ops_len(),
            render_targets_generation: self.render_targets_generation,
            images_generation: self.images_generation,
        };

        let cache_hit = self.scene_encoding_cache_key == Some(key);
        let encoding = if cache_hit {
            std::mem::take(&mut self.scene_encoding_cache)
        } else {
            let mut encoding = std::mem::take(&mut self.scene_encoding_scratch);
            encoding.clear();
            self.encode_scene_ops_into(
                scene,
                scale_factor,
                viewport_size,
                output_is_srgb,
                &mut encoding,
            );

            // Preserve the old cache's allocations for reuse.
            self.scene_encoding_scratch = std::mem::take(&mut self.scene_encoding_cache);
            self.scene_encoding_cache_key = Some(key);
            encoding
        };
        if debug_render_path_enabled() && debug_render_path_should_log() {
            let mut paths = 0usize;
            let mut quads = 0usize;
            let mut viewports = 0usize;
            let mut images = 0usize;
            let mut texts = 0usize;
            let mut masks = 0usize;
            for d in &encoding.ordered_draws {
                match d {
                    OrderedDraw::Path(_) => paths += 1,
                    OrderedDraw::Quad(_) => quads += 1,
                    OrderedDraw::Viewport(_) => viewports += 1,
                    OrderedDraw::Image(_) => images += 1,
                    OrderedDraw::Text(_) => texts += 1,
                    OrderedDraw::Mask(_) => masks += 1,
                }
            }
            eprintln!(
                "render_scene: ordered_draws total={} paths={} quads={} viewports={} images={} texts={} masks={}",
                encoding.ordered_draws.len(),
                paths,
                quads,
                viewports,
                images,
                texts,
                masks
            );
        }

        self.ensure_uniform_capacity(device, encoding.uniforms.len());
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        let mut uniform_bytes =
            vec![0u8; (self.uniform_stride * encoding.uniforms.len() as u64) as usize];
        for (i, u) in encoding.uniforms.iter().enumerate() {
            let offset = (self.uniform_stride * i as u64) as usize;
            uniform_bytes[offset..offset + uniform_size as usize]
                .copy_from_slice(bytemuck::bytes_of(u));
        }
        queue.write_buffer(&self.uniform_buffer, 0, &uniform_bytes);

        self.ensure_clip_capacity(device, encoding.clips.len().max(1));
        if !encoding.clips.is_empty() {
            queue.write_buffer(&self.clip_buffer, 0, bytemuck::cast_slice(&encoding.clips));
        }

        self.prepare_viewport_bind_groups(device, &encoding.ordered_draws);
        self.prepare_image_bind_groups(device, &encoding.ordered_draws);

        let instances = &encoding.instances;
        let viewport_vertices = &encoding.viewport_vertices;
        let text_vertices = &encoding.text_vertices;
        let path_vertices = &encoding.path_vertices;

        self.ensure_instance_capacity(device, instances.len());
        self.ensure_viewport_vertex_capacity(device, viewport_vertices.len());
        self.ensure_text_vertex_capacity(device, text_vertices.len());
        self.ensure_path_vertex_capacity(device, path_vertices.len());

        let instance_buffer_index = self.instance_buffer_index;
        self.instance_buffer_index = (self.instance_buffer_index + 1) % self.instance_buffers.len();
        let instance_buffer = self.instance_buffers[instance_buffer_index].clone();
        if !instances.is_empty() {
            queue.write_buffer(&instance_buffer, 0, bytemuck::cast_slice(instances));
        }

        let viewport_vertex_buffer_index = self.viewport_vertex_buffer_index;
        self.viewport_vertex_buffer_index =
            (self.viewport_vertex_buffer_index + 1) % self.viewport_vertex_buffers.len();
        let viewport_vertex_buffer =
            self.viewport_vertex_buffers[viewport_vertex_buffer_index].clone();
        if !viewport_vertices.is_empty() {
            queue.write_buffer(
                &viewport_vertex_buffer,
                0,
                bytemuck::cast_slice(viewport_vertices),
            );
        }

        let text_vertex_buffer_index = self.text_vertex_buffer_index;
        self.text_vertex_buffer_index =
            (self.text_vertex_buffer_index + 1) % self.text_vertex_buffers.len();
        let text_vertex_buffer = self.text_vertex_buffers[text_vertex_buffer_index].clone();
        if !text_vertices.is_empty() {
            queue.write_buffer(&text_vertex_buffer, 0, bytemuck::cast_slice(text_vertices));
        }

        let path_vertex_buffer_index = self.path_vertex_buffer_index;
        self.path_vertex_buffer_index =
            (self.path_vertex_buffer_index + 1) % self.path_vertex_buffers.len();
        let path_vertex_buffer = self.path_vertex_buffers[path_vertex_buffer_index].clone();
        if !path_vertices.is_empty() {
            queue.write_buffer(&path_vertex_buffer, 0, bytemuck::cast_slice(path_vertices));
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fret renderer encoder"),
        });

        {
            #[derive(Clone, Copy)]
            struct PathRun {
                start: usize,
                end: usize,
                union: ScissorRect,
                uniform_index: u32,
                composite_vertex_base: u32,
            }

            let mut path_runs: Vec<PathRun> = Vec::new();
            if path_samples > 1 {
                let mut idx = 0usize;
                while idx < encoding.ordered_draws.len() {
                    let OrderedDraw::Path(first) = &encoding.ordered_draws[idx] else {
                        idx += 1;
                        continue;
                    };

                    let mut union = first.scissor;
                    let uniform_index = first.uniform_index;
                    let mut end = idx + 1;
                    while end < encoding.ordered_draws.len() {
                        match &encoding.ordered_draws[end] {
                            OrderedDraw::Path(d) if d.uniform_index == uniform_index => {
                                union = union_scissor(union, d.scissor);
                                end += 1;
                            }
                            _ => break,
                        }
                    }

                    path_runs.push(PathRun {
                        start: idx,
                        end,
                        union,
                        uniform_index,
                        composite_vertex_base: 0,
                    });
                    idx = end;
                }
            }

            // Pre-render all path batches into offscreen textures so we can run a single main pass.
            // This avoids relying on `LoadOp::Load` across multiple surface render passes, which
            // some Vulkan swapchain paths have been observed to handle unreliably.
            if path_samples > 1 && !path_runs.is_empty() {
                let intermediate_info = self
                    .path_intermediate
                    .as_ref()
                    .map(|i| (i.format, i.sample_count, i.msaa_view.is_some()));

                if let Some((intermediate_format, sample_count, has_msaa_view)) = intermediate_info
                    && sample_count > 1
                    && has_msaa_view
                    && self.path_msaa_pipeline.is_some()
                {
                    for run_index in 0..path_runs.len() {
                        self.ensure_path_composite_target(
                            device,
                            run_index,
                            viewport_size,
                            intermediate_format,
                        );
                    }

                    let path_msaa_pipeline = self
                        .path_msaa_pipeline
                        .as_ref()
                        .expect("path msaa pipeline must exist when enabled");
                    let intermediate = self
                        .path_intermediate
                        .as_ref()
                        .expect("path intermediate must exist when enabled");
                    let msaa_view = intermediate
                        .msaa_view
                        .as_ref()
                        .expect("msaa_view must exist when sample_count > 1");

                    for (run_index, run) in path_runs.iter().enumerate() {
                        let target = &self.path_composite_targets[run_index];
                        let mut path_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("fret path batch pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: msaa_view,
                                    depth_slice: None,
                                    resolve_target: Some(&target.view),
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                                multiview_mask: None,
                            });

                        path_pass.set_pipeline(path_msaa_pipeline);
                        path_pass.set_vertex_buffer(0, path_vertex_buffer.slice(..));

                        let mut active_uniform_offset: Option<u32> = None;
                        for j in run.start..run.end {
                            let OrderedDraw::Path(draw) = &encoding.ordered_draws[j] else {
                                unreachable!();
                            };
                            if draw.scissor.w == 0 || draw.scissor.h == 0 {
                                continue;
                            }

                            path_pass.set_scissor_rect(
                                draw.scissor.x,
                                draw.scissor.y,
                                draw.scissor.w,
                                draw.scissor.h,
                            );

                            let uniform_offset =
                                (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                            if active_uniform_offset != Some(uniform_offset) {
                                path_pass.set_bind_group(
                                    0,
                                    &self.uniform_bind_group,
                                    &[uniform_offset],
                                );
                                active_uniform_offset = Some(uniform_offset);
                            }

                            path_pass.draw(
                                draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                                0..1,
                            );
                        }
                    }
                } else {
                    path_runs.clear();
                }
            }

            if path_samples > 1 && !path_runs.is_empty() {
                let mut composite_vertices: Vec<ViewportVertex> =
                    Vec::with_capacity(path_runs.len().saturating_mul(6));
                let vw = viewport_size.0.max(1) as f32;
                let vh = viewport_size.1.max(1) as f32;

                for run in &mut path_runs {
                    run.composite_vertex_base =
                        composite_vertices.len().min(u32::MAX as usize) as u32;

                    let union = run.union;
                    let x0 = union.x as f32;
                    let y0 = union.y as f32;
                    let x1 = (union.x + union.w) as f32;
                    let y1 = (union.y + union.h) as f32;

                    let u0 = x0 / vw;
                    let v0 = y0 / vh;
                    let u1 = x1 / vw;
                    let v1 = y1 / vh;

                    let vertices: [ViewportVertex; 6] = [
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [u0, v0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y0],
                            uv: [u1, v0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [u1, v1],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y0],
                            uv: [u0, v0],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x1, y1],
                            uv: [u1, v1],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                        ViewportVertex {
                            pos_px: [x0, y1],
                            uv: [u0, v1],
                            opacity: 1.0,
                            _pad: [0.0; 3],
                        },
                    ];
                    composite_vertices.extend_from_slice(&vertices);
                }

                self.ensure_path_composite_vertex_buffer(device, composite_vertices.len());
                queue.write_buffer(
                    &self.path_composite_vertices,
                    0,
                    bytemuck::cast_slice(&composite_vertices),
                );
            }

            enum ActivePipeline {
                None,
                Quad,
                Viewport,
                TextMask,
                TextColor,
                Mask,
                Composite,
                Path,
            }

            let quad_pipeline = self
                .quad_pipeline
                .as_ref()
                .expect("quad pipeline must exist");
            let viewport_pipeline = self
                .viewport_pipeline
                .as_ref()
                .expect("viewport pipeline must exist");
            let text_pipeline = self
                .text_pipeline
                .as_ref()
                .expect("text pipeline must exist");
            let text_color_pipeline = self
                .text_color_pipeline
                .as_ref()
                .expect("text color pipeline must exist");
            let mask_pipeline = self
                .mask_pipeline
                .as_ref()
                .expect("mask pipeline must exist");
            let composite_pipeline = self.composite_pipeline.as_ref();
            let path_pipeline = self
                .path_pipeline
                .as_ref()
                .expect("path pipeline must exist");

            let mut active_pipeline = ActivePipeline::None;

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

            let mut pass = begin_main_pass(&mut encoder, target_view, wgpu::LoadOp::Clear(clear.0));
            let mut active_uniform_offset: Option<u32> = None;

            let mut i = 0usize;
            let mut path_run_cursor = 0usize;
            while i < encoding.ordered_draws.len() {
                let item = &encoding.ordered_draws[i];

                if path_samples > 1
                    && let Some(run) = path_runs.get(path_run_cursor)
                    && run.start == i
                {
                    let union = run.union;
                    if union.w > 0 && union.h > 0 {
                        let target = &self.path_composite_targets[path_run_cursor];

                        let composite_pipeline =
                            composite_pipeline.expect("composite pipeline must exist");
                        pass.set_pipeline(composite_pipeline);
                        let uniform_offset =
                            (u64::from(run.uniform_index) * self.uniform_stride) as u32;
                        pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                        pass.set_bind_group(1, &target.bind_group, &[]);
                        let vertex_size = std::mem::size_of::<ViewportVertex>() as u64;
                        let base = u64::from(run.composite_vertex_base) * vertex_size;
                        let len = 6 * vertex_size;
                        pass.set_vertex_buffer(
                            0,
                            self.path_composite_vertices.slice(base..base + len),
                        );
                        pass.set_scissor_rect(union.x, union.y, union.w, union.h);
                        pass.draw(0..6, 0..1);
                        active_pipeline = ActivePipeline::Composite;
                    }

                    i = run.end;
                    path_run_cursor += 1;
                    continue;
                }

                match item {
                    OrderedDraw::Quad(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Quad) {
                            pass.set_pipeline(quad_pipeline);
                            pass.set_vertex_buffer(0, instance_buffer.slice(..));
                            active_pipeline = ActivePipeline::Quad;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            0..6,
                            draw.first_instance..(draw.first_instance + draw.instance_count),
                        );
                    }
                    OrderedDraw::Viewport(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Viewport) {
                            pass.set_pipeline(viewport_pipeline);
                            pass.set_vertex_buffer(0, viewport_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Viewport;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        let Some((_, bind_group)) = self.viewport_bind_groups.get(&draw.target)
                        else {
                            // Missing bind group should only happen if the target vanished
                            // between encoding and rendering.
                            i += 1;
                            continue;
                        };
                        pass.set_bind_group(1, bind_group, &[]);
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                    }
                    OrderedDraw::Image(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Viewport) {
                            pass.set_pipeline(viewport_pipeline);
                            pass.set_vertex_buffer(0, viewport_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Viewport;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        let Some((_, bind_group)) = self.image_bind_groups.get(&draw.image) else {
                            i += 1;
                            continue;
                        };
                        pass.set_bind_group(1, bind_group, &[]);
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                    }
                    OrderedDraw::Mask(draw) => {
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Mask) {
                            pass.set_pipeline(mask_pipeline);
                            pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Mask;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        let Some((_, bind_group)) = self.image_bind_groups.get(&draw.image) else {
                            i += 1;
                            continue;
                        };
                        pass.set_bind_group(1, bind_group, &[]);
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
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
                                    pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.mask_atlas_bind_group(),
                                        &[],
                                    );
                                    active_pipeline = ActivePipeline::TextMask;
                                }
                            }
                            TextDrawKind::Color => {
                                if !matches!(active_pipeline, ActivePipeline::TextColor) {
                                    pass.set_pipeline(text_color_pipeline);
                                    pass.set_vertex_buffer(0, text_vertex_buffer.slice(..));
                                    pass.set_bind_group(
                                        1,
                                        self.text_system.color_atlas_bind_group(),
                                        &[],
                                    );
                                    active_pipeline = ActivePipeline::TextColor;
                                }
                            }
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                    }
                    OrderedDraw::Path(draw) => {
                        if path_samples > 1 {
                            i += 1;
                            continue;
                        }
                        if draw.scissor.w == 0 || draw.scissor.h == 0 {
                            i += 1;
                            continue;
                        }

                        if !matches!(active_pipeline, ActivePipeline::Path) {
                            pass.set_pipeline(path_pipeline);
                            pass.set_vertex_buffer(0, path_vertex_buffer.slice(..));
                            active_pipeline = ActivePipeline::Path;
                        }

                        let uniform_offset =
                            (u64::from(draw.uniform_index) * self.uniform_stride) as u32;
                        if active_uniform_offset != Some(uniform_offset) {
                            pass.set_bind_group(0, &self.uniform_bind_group, &[uniform_offset]);
                            active_uniform_offset = Some(uniform_offset);
                        }
                        pass.set_scissor_rect(
                            draw.scissor.x,
                            draw.scissor.y,
                            draw.scissor.w,
                            draw.scissor.h,
                        );
                        pass.draw(
                            draw.first_vertex..(draw.first_vertex + draw.vertex_count),
                            0..1,
                        );
                    }
                }

                i += 1;
            }
        }

        let cmd = encoder.finish();

        // Keep the most recent encoding for potential reuse on the next frame.
        if cache_hit {
            self.scene_encoding_cache_key = Some(key);
        }
        self.scene_encoding_cache = encoding;
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::path_intermediate_format;

    #[test]
    fn path_intermediate_format_drops_srgb_suffix() {
        assert_eq!(
            path_intermediate_format(wgpu::TextureFormat::Bgra8UnormSrgb),
            wgpu::TextureFormat::Rgba8Unorm
        );
        assert_eq!(
            path_intermediate_format(wgpu::TextureFormat::Bgra8Unorm),
            wgpu::TextureFormat::Rgba8Unorm
        );
        assert_eq!(
            path_intermediate_format(wgpu::TextureFormat::Rgba8UnormSrgb),
            wgpu::TextureFormat::Rgba8Unorm
        );
    }

    #[test]
    fn path_intermediate_format_preserves_non_srgb() {
        assert_eq!(
            path_intermediate_format(wgpu::TextureFormat::Bgra8Unorm),
            wgpu::TextureFormat::Rgba8Unorm
        );
        assert_eq!(
            path_intermediate_format(wgpu::TextureFormat::Rgba16Float),
            wgpu::TextureFormat::Rgba16Float
        );
    }
}
