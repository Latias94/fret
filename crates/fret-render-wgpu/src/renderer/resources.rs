use super::*;
use crate::images::ImageDescriptor;
use crate::targets::RenderTargetDescriptor;

impl Renderer {
    pub fn new(adapter: &wgpu::Adapter, device: &wgpu::Device) -> Self {
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        // wgpu requires uniform dynamic offsets to be aligned to 256 bytes.
        let uniform_stride = uniform_size.div_ceil(256) * 256;
        let uniform_capacity = 256usize;

        let render_space_size = std::mem::size_of::<RenderSpaceUniform>() as u64;
        let render_space_stride = render_space_size.div_ceil(256) * 256;
        // RenderSpace is dynamic (per pass) and must not be overwritten within a frame.
        // Allocate enough slots for typical worst-case RenderPlan pass counts.
        let render_space_capacity = 2048usize;
        let clip_capacity = 1024usize;
        let clip_entry_size = std::mem::size_of::<ClipRRectUniform>() as u64;
        let mask_capacity = 1024usize;
        let mask_entry_size = std::mem::size_of::<MaskGradientUniform>() as u64;

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret quad uniforms layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: Some(
                                std::num::NonZeroU64::new(uniform_size).unwrap(),
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                std::num::NonZeroU64::new(clip_entry_size).unwrap(),
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                std::num::NonZeroU64::new(mask_entry_size).unwrap(),
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: Some(
                                std::num::NonZeroU64::new(render_space_size).unwrap(),
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
            });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret quad uniforms buffer"),
            size: uniform_stride * uniform_capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let render_space_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret render-space uniform buffer"),
            size: render_space_stride * render_space_capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let clip_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret clip stack buffer"),
            size: clip_entry_size.saturating_mul(clip_capacity as u64).max(4),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mask_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret mask stack buffer"),
            size: mask_entry_size.saturating_mul(mask_capacity as u64).max(4),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms = UniformResources::new(
            uniform_buffer,
            uniform_stride,
            uniform_capacity,
            render_space_buffer,
            render_space_stride,
            render_space_capacity,
            clip_buffer,
            clip_capacity,
            mask_buffer,
            mask_capacity,
        );

        let material_catalog_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret material catalog texture array"),
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let material_catalog_view =
            material_catalog_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("fret material catalog texture array view"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                ..Default::default()
            });
        let material_catalog_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fret material catalog sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let mask_image_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fret mask image sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let mask_image_sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fret mask image sampler (nearest)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let mask_image_identity_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret mask image identity texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mask_image_identity_view =
            mask_image_identity_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let custom_effect_input_fallback_texture =
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fret custom effect input fallback texture"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
        let custom_effect_input_fallback_view = custom_effect_input_fallback_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let viewport_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret viewport texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                ],
            });

        let viewport_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fret viewport sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let image_sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fret image sampler (nearest)"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let textures = GpuTextures::new(
            mask_image_identity_texture,
            material_catalog_texture,
            custom_effect_input_fallback_texture,
        );
        let globals = GpuGlobals {
            uniform_bind_group_layout,
            viewport_bind_group_layout,
            viewport_sampler,
            image_sampler_nearest,
            material_catalog_view,
            material_catalog_sampler,
            mask_image_sampler,
            mask_image_sampler_nearest,
            mask_image_identity_view,
            custom_effect_input_fallback_view,
        };

        let uniform_bind_group = super::bind_group_builders::UniformBindGroupGlobals {
            layout: &globals.uniform_bind_group_layout,
            material_catalog_view: &globals.material_catalog_view,
            material_catalog_sampler: &globals.material_catalog_sampler,
            mask_image_sampler: &globals.mask_image_sampler,
            mask_image_identity_view: &globals.mask_image_identity_view,
        }
        .create(
            device,
            "fret quad uniforms bind group",
            &uniforms.uniform_buffer,
            &uniforms.clip_buffer,
            &uniforms.mask_buffer,
            &uniforms.render_space_buffer,
        );

        let clip_mask_param_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret clip-mask params bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::new(16).unwrap()),
                    },
                    count: None,
                }],
            });
        let clip_mask_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret clip-mask params buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let clip_mask_param_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret clip-mask params bind group"),
            layout: &clip_mask_param_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: clip_mask_param_buffer.as_entire_binding(),
            }],
        });

        let text_system = TextSystem::new(device);

        const FRAMES_IN_FLIGHT: usize = 3;
        let quad_instance_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret quad instances bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let instance_usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let quad_instances = buffers::StorageRingBuffer::<QuadInstance>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            quad_instance_bind_group_layout,
            "fret quad instances",
            instance_usage,
        );

        let path_paint_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret path paints bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let paint_usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let path_paints = buffers::StorageRingBuffer::<PaintGpu>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            path_paint_bind_group_layout,
            "fret path paints",
            paint_usage,
        );

        let text_paint_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("fret text paints bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let text_paints = buffers::StorageRingBuffer::<PaintGpu>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            text_paint_bind_group_layout,
            "fret text paints",
            paint_usage,
        );

        let vertex_usage = wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
        let viewport_vertices = buffers::RingBuffer::<ViewportVertex>::new(
            device,
            FRAMES_IN_FLIGHT,
            64 * 6,
            "fret viewport vertices",
            vertex_usage,
        );
        let text_vertices = buffers::RingBuffer::<TextVertex>::new(
            device,
            FRAMES_IN_FLIGHT,
            512 * 6,
            "fret text vertices",
            vertex_usage,
        );
        let path_vertices = buffers::RingBuffer::<PathVertex>::new(
            device,
            FRAMES_IN_FLIGHT,
            1024,
            "fret path vertices",
            vertex_usage,
        );

        let scale_param_size = std::mem::size_of::<ScaleParamsUniform>() as u64;
        let scale_param_stride = scale_param_size.div_ceil(256) * 256;
        let scale_param_capacity = 64usize;
        let scale_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret scale params buffer"),
            size: scale_param_stride * scale_param_capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let color_adjust_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret color-adjust params buffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let backdrop_warp_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret backdrop-warp params buffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let color_matrix_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret color-matrix params buffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let alpha_threshold_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret alpha-threshold params buffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let noise_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret noise params buffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let drop_shadow_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret drop-shadow params buffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let custom_effect_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret custom-effect params buffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let custom_effect_v2_input_meta_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret custom-effect v2 input meta buffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let custom_effect_v3_meta_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret custom-effect v3 meta buffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let effect_params = super::gpu_effect_params::GpuEffectParams {
            clip_mask_param_buffer,
            clip_mask_param_bind_group,
            clip_mask_param_bind_group_layout,
            scale_param_buffer,
            scale_param_stride,
            scale_param_capacity,
            backdrop_warp_param_buffer,
            color_adjust_param_buffer,
            color_matrix_param_buffer,
            alpha_threshold_param_buffer,
            noise_param_buffer,
            drop_shadow_param_buffer,
            custom_effect_param_buffer,
            custom_effect_v2_input_meta_buffer,
            custom_effect_v3_meta_buffer,
        };

        let render_plan_strict_output_clear =
            std::env::var("FRET_RENDER_PLAN_STRICT_OUTPUT_CLEAR").is_ok_and(|v| v != "0");

        Self {
            adapter: adapter.clone(),
            uniform_bind_group,
            uniforms,
            viewport_uniform_bytes_scratch: Vec::new(),
            render_space_bytes_scratch: Vec::new(),
            plan_quad_vertices_scratch: Vec::new(),
            plan_quad_vertex_bases_scratch: Vec::new(),
            render_plan_reporting_state: RenderPlanReportingState::default(),
            render_plan_strict_output_clear,
            globals,
            textures,
            effect_params,
            pipelines: GpuPipelines::default(),
            custom_effect_v3_pyramid: v3_pyramid::CustomEffectV3PyramidState::default(),
            quad_instances,
            path_paints,
            text_paints,
            viewport_vertices,
            text_vertices,
            path_vertices,
            text_system,
            path_state: PathState::new(device),
            svg_registry_state: svg::SvgRegistryState::new(),
            svg_raster_state: svg::SvgRasterState::default(),
            clip_path_mask_cache: ClipPathMaskCache::new((256 * 1024 * 1024) / 8),
            diagnostics_state: DiagnosticsState::default(),
            path_msaa_samples: 4,
            debug_offscreen_blit_enabled: false,
            debug_pixelate_scale: 0,
            debug_blur_radius: 0,
            debug_blur_scissor: None,
            intermediate_state: IntermediateState::default(),
            gpu_resources: super::gpu_resources::GpuResources::default(),
            scene_encoding_cache: super::scene_encoding_cache::SceneEncodingCache::default(),
            material_effect_state: MaterialEffectState::default(),
        }
    }

    pub fn register_render_target(
        &mut self,
        desc: RenderTargetDescriptor,
    ) -> fret_core::RenderTargetId {
        let mut desc = desc;
        if Self::render_target_color_encoding_conflicts_with_portable_rgb_assumption(
            desc.color_space,
            desc.metadata.color_encoding,
        ) {
            desc.metadata.color_encoding = fret_render_core::RenderTargetColorEncoding::default();
            self.diagnostics_state
                .note_render_target_metadata_degradation_color_encoding_dropped();
        }
        self.diagnostics_state.note_render_target_update(
            desc.metadata.requested_ingest_strategy,
            desc.metadata.ingest_strategy,
        );
        self.gpu_resources.register_render_target(desc)
    }

    pub fn register_image(&mut self, desc: ImageDescriptor) -> fret_core::ImageId {
        self.gpu_resources.register_image(desc)
    }

    pub fn update_image(&mut self, id: fret_core::ImageId, desc: ImageDescriptor) -> bool {
        self.gpu_resources.update_image(id, desc)
    }

    pub fn unregister_image(&mut self, id: fret_core::ImageId) -> bool {
        self.gpu_resources.unregister_image(id)
    }

    pub fn update_render_target(
        &mut self,
        id: fret_core::RenderTargetId,
        desc: RenderTargetDescriptor,
    ) -> bool {
        let mut desc = desc;
        if Self::render_target_color_encoding_conflicts_with_portable_rgb_assumption(
            desc.color_space,
            desc.metadata.color_encoding,
        ) {
            desc.metadata.color_encoding = fret_render_core::RenderTargetColorEncoding::default();
            self.diagnostics_state
                .note_render_target_metadata_degradation_color_encoding_dropped();
        }
        self.diagnostics_state.note_render_target_update(
            desc.metadata.requested_ingest_strategy,
            desc.metadata.ingest_strategy,
        );
        self.gpu_resources.update_render_target(id, desc)
    }

    fn render_target_color_encoding_conflicts_with_portable_rgb_assumption(
        color_space: fret_render_core::RenderTargetColorSpace,
        encoding: fret_render_core::RenderTargetColorEncoding,
    ) -> bool {
        use fret_render_core::RenderTargetColorPrimaries;
        use fret_render_core::RenderTargetColorRange;
        use fret_render_core::RenderTargetMatrixCoefficients;
        use fret_render_core::RenderTargetTransferFunction;

        if encoding == fret_render_core::RenderTargetColorEncoding::default() {
            return false;
        }

        let expected_transfer = match color_space {
            fret_render_core::RenderTargetColorSpace::Srgb => RenderTargetTransferFunction::Srgb,
            fret_render_core::RenderTargetColorSpace::Linear => {
                RenderTargetTransferFunction::Linear
            }
        };

        if encoding.primaries != RenderTargetColorPrimaries::Unknown
            && encoding.primaries != RenderTargetColorPrimaries::Bt709
        {
            return true;
        }
        if encoding.transfer != RenderTargetTransferFunction::Unknown
            && encoding.transfer != expected_transfer
        {
            return true;
        }
        if encoding.matrix != RenderTargetMatrixCoefficients::Unknown
            && encoding.matrix != RenderTargetMatrixCoefficients::Rgb
        {
            return true;
        }
        if encoding.range != RenderTargetColorRange::Unknown
            && encoding.range != RenderTargetColorRange::Full
        {
            return true;
        }

        false
    }

    pub fn unregister_render_target(&mut self, id: fret_core::RenderTargetId) -> bool {
        self.gpu_resources.unregister_render_target(id)
    }

    pub(super) fn ensure_mask_image_identity_uploaded(&mut self, queue: &wgpu::Queue) {
        self.textures.ensure_mask_image_identity_uploaded(queue);
    }

    pub(super) fn ensure_material_catalog_uploaded(&mut self, queue: &wgpu::Queue) {
        self.textures.ensure_material_catalog_uploaded(queue);
    }

    pub(super) fn ensure_custom_effect_input_fallback_uploaded(&mut self, queue: &wgpu::Queue) {
        self.textures
            .ensure_custom_effect_input_fallback_uploaded(queue);
    }
}
