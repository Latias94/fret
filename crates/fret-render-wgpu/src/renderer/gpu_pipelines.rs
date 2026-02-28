use super::Renderer;
use super::gpu_globals::GpuGlobals;
use super::types::QuadPipelineKey;

pub(super) struct CustomEffectPipelines {
    pub(super) unmasked: wgpu::RenderPipeline,
    pub(super) masked: wgpu::RenderPipeline,
    pub(super) mask: wgpu::RenderPipeline,
}

pub(super) struct GpuPipelines {
    pub(super) quad_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) quad_pipelines: std::collections::HashMap<QuadPipelineKey, wgpu::RenderPipeline>,

    pub(super) viewport_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) viewport_pipeline: Option<wgpu::RenderPipeline>,

    pub(super) mask_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) mask_pipeline: Option<wgpu::RenderPipeline>,

    pub(super) text_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) text_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) text_outline_pipeline: Option<wgpu::RenderPipeline>,

    pub(super) text_color_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) text_color_pipeline: Option<wgpu::RenderPipeline>,

    pub(super) text_subpixel_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) text_subpixel_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) text_subpixel_outline_pipeline: Option<wgpu::RenderPipeline>,

    pub(super) path_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) path_pipeline: Option<wgpu::RenderPipeline>,

    pub(super) path_msaa_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) path_msaa_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) path_msaa_pipeline_sample_count: Option<u32>,

    pub(super) path_clip_mask_pipeline: Option<wgpu::RenderPipeline>,

    pub(super) composite_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) composite_pipelines: [Option<wgpu::RenderPipeline>; fret_core::BlendMode::COUNT],
    pub(super) composite_mask_pipelines:
        [Option<wgpu::RenderPipeline>; fret_core::BlendMode::COUNT],
    pub(super) composite_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) clip_mask_pipeline: Option<wgpu::RenderPipeline>,

    pub(super) blit_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) blit_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) blit_srgb_encode_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) blit_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) blur_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) blur_h_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) blur_v_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) blur_h_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) blur_v_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) blur_h_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) blur_v_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) blit_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) scale_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) downsample_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) upscale_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) upscale_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) upscale_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) scale_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) scale_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) backdrop_warp_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) backdrop_warp_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) backdrop_warp_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) backdrop_warp_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) backdrop_warp_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) backdrop_warp_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) backdrop_warp_image_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) backdrop_warp_image_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) backdrop_warp_image_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) backdrop_warp_image_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) backdrop_warp_image_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) color_adjust_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) color_adjust_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) color_adjust_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) color_adjust_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) color_adjust_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) color_adjust_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) color_matrix_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) color_matrix_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) color_matrix_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) color_matrix_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) color_matrix_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) color_matrix_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) alpha_threshold_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) alpha_threshold_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) alpha_threshold_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) alpha_threshold_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) alpha_threshold_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) alpha_threshold_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) dither_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) dither_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) dither_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) dither_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) dither_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) dither_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) noise_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) noise_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) noise_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) noise_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) noise_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) noise_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) drop_shadow_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) drop_shadow_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) drop_shadow_masked_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) drop_shadow_mask_pipeline: Option<wgpu::RenderPipeline>,
    pub(super) drop_shadow_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) drop_shadow_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) custom_effect_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) custom_effect_pipelines:
        std::collections::HashMap<fret_core::EffectId, CustomEffectPipelines>,
    pub(super) custom_effect_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) custom_effect_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) custom_effect_v2_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) custom_effect_v2_pipelines:
        std::collections::HashMap<fret_core::EffectId, CustomEffectPipelines>,
    pub(super) custom_effect_v2_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) custom_effect_v2_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub(super) custom_effect_v3_pipeline_format: Option<wgpu::TextureFormat>,
    pub(super) custom_effect_v3_pipelines:
        std::collections::HashMap<fret_core::EffectId, CustomEffectPipelines>,
    pub(super) custom_effect_v3_bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub(super) custom_effect_v3_mask_bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl Default for GpuPipelines {
    fn default() -> Self {
        Self {
            quad_pipeline_format: None,
            quad_pipelines: std::collections::HashMap::new(),
            viewport_pipeline_format: None,
            viewport_pipeline: None,
            mask_pipeline_format: None,
            mask_pipeline: None,
            text_pipeline_format: None,
            text_pipeline: None,
            text_outline_pipeline: None,
            text_color_pipeline_format: None,
            text_color_pipeline: None,
            text_subpixel_pipeline_format: None,
            text_subpixel_pipeline: None,
            text_subpixel_outline_pipeline: None,
            path_pipeline_format: None,
            path_pipeline: None,
            path_msaa_pipeline_format: None,
            path_msaa_pipeline: None,
            path_msaa_pipeline_sample_count: None,
            path_clip_mask_pipeline: None,
            composite_pipeline_format: None,
            composite_pipelines: [const { None }; fret_core::BlendMode::COUNT],
            composite_mask_pipelines: [const { None }; fret_core::BlendMode::COUNT],
            composite_mask_bind_group_layout: None,
            clip_mask_pipeline: None,
            blit_pipeline_format: None,
            blit_pipeline: None,
            blit_srgb_encode_pipeline: None,
            blit_bind_group_layout: None,
            blur_pipeline_format: None,
            blur_h_pipeline: None,
            blur_v_pipeline: None,
            blur_h_masked_pipeline: None,
            blur_v_masked_pipeline: None,
            blur_h_mask_pipeline: None,
            blur_v_mask_pipeline: None,
            blit_mask_bind_group_layout: None,
            scale_pipeline_format: None,
            downsample_pipeline: None,
            upscale_pipeline: None,
            upscale_masked_pipeline: None,
            upscale_mask_pipeline: None,
            scale_bind_group_layout: None,
            scale_mask_bind_group_layout: None,
            backdrop_warp_pipeline_format: None,
            backdrop_warp_pipeline: None,
            backdrop_warp_masked_pipeline: None,
            backdrop_warp_mask_pipeline: None,
            backdrop_warp_bind_group_layout: None,
            backdrop_warp_mask_bind_group_layout: None,
            backdrop_warp_image_pipeline: None,
            backdrop_warp_image_masked_pipeline: None,
            backdrop_warp_image_mask_pipeline: None,
            backdrop_warp_image_bind_group_layout: None,
            backdrop_warp_image_mask_bind_group_layout: None,
            color_adjust_pipeline_format: None,
            color_adjust_pipeline: None,
            color_adjust_masked_pipeline: None,
            color_adjust_mask_pipeline: None,
            color_adjust_bind_group_layout: None,
            color_adjust_mask_bind_group_layout: None,
            color_matrix_pipeline_format: None,
            color_matrix_pipeline: None,
            color_matrix_masked_pipeline: None,
            color_matrix_mask_pipeline: None,
            color_matrix_bind_group_layout: None,
            color_matrix_mask_bind_group_layout: None,
            alpha_threshold_pipeline_format: None,
            alpha_threshold_pipeline: None,
            alpha_threshold_masked_pipeline: None,
            alpha_threshold_mask_pipeline: None,
            alpha_threshold_bind_group_layout: None,
            alpha_threshold_mask_bind_group_layout: None,

            dither_pipeline_format: None,
            dither_pipeline: None,
            dither_masked_pipeline: None,
            dither_mask_pipeline: None,
            dither_bind_group_layout: None,
            dither_mask_bind_group_layout: None,

            noise_pipeline_format: None,
            noise_pipeline: None,
            noise_masked_pipeline: None,
            noise_mask_pipeline: None,
            noise_bind_group_layout: None,
            noise_mask_bind_group_layout: None,

            drop_shadow_pipeline_format: None,
            drop_shadow_pipeline: None,
            drop_shadow_masked_pipeline: None,
            drop_shadow_mask_pipeline: None,
            drop_shadow_bind_group_layout: None,
            drop_shadow_mask_bind_group_layout: None,

            custom_effect_pipeline_format: None,
            custom_effect_pipelines: std::collections::HashMap::new(),
            custom_effect_bind_group_layout: None,
            custom_effect_mask_bind_group_layout: None,

            custom_effect_v2_pipeline_format: None,
            custom_effect_v2_pipelines: std::collections::HashMap::new(),
            custom_effect_v2_bind_group_layout: None,
            custom_effect_v2_mask_bind_group_layout: None,

            custom_effect_v3_pipeline_format: None,
            custom_effect_v3_pipelines: std::collections::HashMap::new(),
            custom_effect_v3_bind_group_layout: None,
            custom_effect_v3_mask_bind_group_layout: None,
        }
    }
}

impl GpuPipelines {
    pub(super) fn ensure_quad_pipelines(&mut self, format: wgpu::TextureFormat) {
        if self.quad_pipeline_format == Some(format) {
            return;
        }

        self.quad_pipeline_format = Some(format);
        self.quad_pipelines.clear();
    }

    pub(super) fn quad_pipeline_ref(&self, key: &QuadPipelineKey) -> Option<&wgpu::RenderPipeline> {
        self.quad_pipelines.get(key)
    }

    pub(super) fn quad_pipeline_inserted(
        &mut self,
        key: QuadPipelineKey,
        pipeline: wgpu::RenderPipeline,
    ) -> &wgpu::RenderPipeline {
        self.quad_pipelines.insert(key, pipeline);
        self.quad_pipelines
            .get(&key)
            .expect("quad pipeline must exist")
    }

    pub(super) fn ensure_viewport_pipeline(
        &mut self,
        device: &wgpu::Device,
        globals: &GpuGlobals,
        format: wgpu::TextureFormat,
    ) {
        if self.viewport_pipeline_format == Some(format) && self.viewport_pipeline.is_some() {
            return;
        }

        let create_span = tracing::enabled!(tracing::Level::TRACE)
            .then(|| {
                let reason = if self.viewport_pipeline.is_none() {
                    "missing"
                } else {
                    "format_changed"
                };
                tracing::trace_span!(
                    "fret.renderer.pipeline.create.viewport",
                    format = ?format,
                    reason
                )
            })
            .unwrap_or_else(tracing::Span::none);
        let _create_guard = create_span.enter();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fret viewport shader"),
            source: wgpu::ShaderSource::Wgsl(super::shaders::VIEWPORT_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fret viewport pipeline layout"),
            bind_group_layouts: &[
                &globals.uniform_bind_group_layout,
                &globals.viewport_bind_group_layout,
            ],
            immediate_size: 0,
        });

        let vertex_stride = std::mem::size_of::<super::ViewportVertex>() as wgpu::BufferAddress;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fret viewport pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 16,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 20,
                            shader_location: 3,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        self.viewport_pipeline_format = Some(format);
        self.viewport_pipeline = Some(pipeline);
    }

    pub(super) fn viewport_pipeline_ref(&self) -> Option<&wgpu::RenderPipeline> {
        self.viewport_pipeline.as_ref()
    }
}

impl Renderer {
    pub(in crate::renderer) fn ensure_viewport_pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) {
        self.pipelines
            .ensure_viewport_pipeline(device, &self.globals, format);
    }

    pub(in crate::renderer) fn viewport_pipeline_ref(&self) -> &wgpu::RenderPipeline {
        self.pipelines
            .viewport_pipeline_ref()
            .expect("viewport pipeline must exist")
    }
}
