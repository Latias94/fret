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
                        visibility: wgpu::ShaderStages::VERTEX,
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

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fret quad uniforms bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: Some(std::num::NonZeroU64::new(uniform_size).unwrap()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &clip_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &mask_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&material_catalog_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&material_catalog_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &render_space_buffer,
                        offset: 0,
                        size: Some(std::num::NonZeroU64::new(render_space_size).unwrap()),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&mask_image_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&mask_image_identity_view),
                },
            ],
        });

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

        let path_composite_vertex_capacity = 64 * 6;
        let path_composite_vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret path composite vertices"),
            size: (path_composite_vertex_capacity * std::mem::size_of::<ViewportVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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

        Self {
            adapter: adapter.clone(),
            uniform_buffer,
            uniform_bind_group,
            uniform_bind_group_layout,
            mask_image_sampler,
            mask_image_sampler_nearest,
            _mask_image_identity_texture: mask_image_identity_texture,
            mask_image_identity_view,
            mask_image_identity_uploaded: false,
            render_space_buffer,
            render_space_stride,
            render_space_capacity,
            uniform_stride,
            uniform_capacity,
            clip_buffer,
            clip_capacity,
            mask_buffer,
            mask_capacity,
            material_catalog_texture,
            material_catalog_uploaded: false,
            quad_pipeline_format: None,
            quad_pipelines: HashMap::new(),
            viewport_pipeline_format: None,
            viewport_pipeline: None,
            viewport_bind_group_layout,
            viewport_sampler,
            image_sampler_nearest,
            quad_instances,
            path_paints,
            text_paints,
            viewport_vertices,
            text_pipeline_format: None,
            text_pipeline: None,
            text_color_pipeline_format: None,
            text_color_pipeline: None,
            text_subpixel_pipeline_format: None,
            text_subpixel_pipeline: None,
            mask_pipeline_format: None,
            mask_pipeline: None,
            text_vertices,
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
            clip_mask_param_buffer,
            clip_mask_param_bind_group,
            clip_mask_param_bind_group_layout,
            blit_pipeline_format: None,
            blit_pipeline: None,
            blit_bind_group_layout: None,
            blit_mask_bind_group_layout: None,
            blur_pipeline_format: None,
            blur_h_pipeline: None,
            blur_v_pipeline: None,
            blur_h_masked_pipeline: None,
            blur_v_masked_pipeline: None,
            blur_h_mask_pipeline: None,
            blur_v_mask_pipeline: None,
            scale_pipeline_format: None,
            downsample_pipeline: None,
            upscale_pipeline: None,
            upscale_masked_pipeline: None,
            upscale_mask_pipeline: None,
            scale_bind_group_layout: None,
            scale_mask_bind_group_layout: None,
            scale_param_buffer,
            scale_param_stride,
            scale_param_capacity,
            color_adjust_pipeline_format: None,
            color_adjust_pipeline: None,
            color_adjust_masked_pipeline: None,
            color_adjust_mask_pipeline: None,
            color_adjust_bind_group_layout: None,
            color_adjust_mask_bind_group_layout: None,
            color_adjust_param_buffer,
            color_matrix_pipeline_format: None,
            color_matrix_pipeline: None,
            color_matrix_masked_pipeline: None,
            color_matrix_mask_pipeline: None,
            color_matrix_bind_group_layout: None,
            color_matrix_mask_bind_group_layout: None,
            color_matrix_param_buffer,
            alpha_threshold_pipeline_format: None,
            alpha_threshold_pipeline: None,
            alpha_threshold_masked_pipeline: None,
            alpha_threshold_mask_pipeline: None,
            alpha_threshold_bind_group_layout: None,
            alpha_threshold_mask_bind_group_layout: None,
            alpha_threshold_param_buffer,
            path_vertices,
            path_intermediate: None,
            path_composite_vertices,
            path_composite_vertex_capacity,
            text_system,
            paths: SlotMap::with_key(),
            path_cache: HashMap::new(),
            path_cache_capacity: 2048,
            path_cache_epoch: 0,
            svg_renderer: SvgRenderer::new(),
            svgs: SlotMap::with_key(),
            svg_hash_index: HashMap::new(),
            svg_rasters: HashMap::new(),
            svg_mask_atlas_pages: Vec::new(),
            svg_mask_atlas_free: Vec::new(),
            svg_raster_bytes: 0,
            svg_mask_atlas_bytes: 0,
            svg_raster_budget_bytes: 64 * 1024 * 1024,
            svg_raster_epoch: 0,
            svg_perf_enabled: false,
            svg_perf: SvgPerfStats::default(),
            perf_enabled: false,
            perf_svg_raster_cache_hits: 0,
            perf_svg_raster_cache_misses: 0,
            perf_svg_raster_budget_evictions: 0,
            perf_svg_mask_atlas_page_evictions: 0,
            perf_svg_mask_atlas_entries_evicted: 0,
            perf_pending_render_target_updates_by_ingest: [0;
                fret_render_core::RenderTargetIngestStrategy::COUNT],
            perf: RenderPerfStats::default(),
            last_frame_perf: None,
            last_render_plan_segment_report: None,
            render_scene_frame_index: 0,
            path_msaa_samples: 4,
            debug_offscreen_blit_enabled: false,
            debug_pixelate_scale: 0,
            debug_blur_radius: 0,
            debug_blur_scissor: None,
            intermediate_budget_bytes: 256 * 1024 * 1024,
            intermediate_perf_enabled: false,
            intermediate_perf: IntermediatePerfStats::default(),
            intermediate_pool: IntermediatePool::default(),
            render_targets: RenderTargetRegistry::default(),
            images: ImageRegistry::default(),
            viewport_bind_groups: HashMap::new(),
            render_target_revisions: HashMap::new(),
            render_targets_generation: 0,
            image_bind_groups: HashMap::new(),
            image_revisions: HashMap::new(),
            images_generation: 0,
            uniform_mask_image_bind_groups: HashMap::new(),
            scene_encoding_cache_key: None,
            scene_encoding_cache: SceneEncoding::default(),
            scene_encoding_scratch: SceneEncoding::default(),

            materials: SlotMap::with_key(),
            materials_by_desc: HashMap::new(),
            material_paint_budget_per_frame: 50_000,
            material_distinct_budget_per_frame: 256,
        }
    }

    pub(super) fn bump_path_cache_epoch(&mut self) -> u64 {
        self.path_cache_epoch = self.path_cache_epoch.wrapping_add(1);
        self.path_cache_epoch
    }

    pub(super) fn prune_path_cache(&mut self) {
        if self.path_cache.len() <= self.path_cache_capacity {
            return;
        }

        // Simple O(n) eviction: drop least-recently-used entries with refs == 0.
        // This keeps the implementation small and deterministic for MVP-PATH-2.
        while self.path_cache.len() > self.path_cache_capacity {
            let mut victim: Option<(PathCacheKey, CachedPathEntry)> = None;
            for (k, v) in &self.path_cache {
                if v.refs != 0 {
                    continue;
                }
                let replace = match victim {
                    None => true,
                    Some((_, cur)) => v.last_used_epoch < cur.last_used_epoch,
                };
                if replace {
                    victim = Some((*k, *v));
                }
            }

            let Some((key, entry)) = victim else {
                break;
            };

            self.path_cache.remove(&key);
            self.paths.remove(entry.id);
        }
    }

    pub fn register_render_target(
        &mut self,
        desc: RenderTargetDescriptor,
    ) -> fret_core::RenderTargetId {
        if self.perf_enabled {
            let ix = render_target_ingest_strategy_perf_index(desc.metadata.ingest_strategy);
            self.perf_pending_render_target_updates_by_ingest[ix] =
                self.perf_pending_render_target_updates_by_ingest[ix].saturating_add(1);
        }
        let id = self.render_targets.register(desc);
        self.render_target_revisions.insert(id, 1);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        id
    }

    pub fn register_image(&mut self, desc: ImageDescriptor) -> fret_core::ImageId {
        let id = self.images.register(desc);
        self.image_revisions.insert(id, 1);
        self.images_generation = self.images_generation.saturating_add(1);
        id
    }

    pub fn update_image(&mut self, id: fret_core::ImageId, desc: ImageDescriptor) -> bool {
        if !self.images.update(id, desc) {
            return false;
        }
        let next = self.image_revisions.get(&id).copied().unwrap_or(0) + 1;
        self.image_revisions.insert(id, next);
        self.image_bind_groups.remove(&id);
        self.uniform_mask_image_bind_groups.remove(&id);
        self.images_generation = self.images_generation.saturating_add(1);
        true
    }

    pub fn unregister_image(&mut self, id: fret_core::ImageId) -> bool {
        if !self.images.unregister(id) {
            return false;
        }
        self.image_revisions.remove(&id);
        self.image_bind_groups.remove(&id);
        self.uniform_mask_image_bind_groups.remove(&id);
        self.images_generation = self.images_generation.saturating_add(1);
        true
    }

    pub fn update_render_target(
        &mut self,
        id: fret_core::RenderTargetId,
        desc: RenderTargetDescriptor,
    ) -> bool {
        if self.perf_enabled {
            let ix = render_target_ingest_strategy_perf_index(desc.metadata.ingest_strategy);
            self.perf_pending_render_target_updates_by_ingest[ix] =
                self.perf_pending_render_target_updates_by_ingest[ix].saturating_add(1);
        }
        if !self.render_targets.update(id, desc) {
            return false;
        }
        let next = self.render_target_revisions.get(&id).copied().unwrap_or(0) + 1;
        self.render_target_revisions.insert(id, next);
        self.viewport_bind_groups.remove(&id);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        true
    }

    pub fn unregister_render_target(&mut self, id: fret_core::RenderTargetId) -> bool {
        if !self.render_targets.unregister(id) {
            return false;
        }
        self.render_target_revisions.remove(&id);
        self.viewport_bind_groups.remove(&id);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        true
    }

    pub(super) fn ensure_mask_image_identity_uploaded(&mut self, queue: &wgpu::Queue) {
        if self.mask_image_identity_uploaded {
            return;
        }

        // Use a 1x1 `R8Unorm` texture filled with 1.0 coverage as the default mask-image source.
        // This keeps `Mask::Image` deterministic even if an image source disappears between
        // encoding and rendering.
        let bytes_per_row = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let mut bytes = vec![0u8; bytes_per_row as usize];
        bytes[0] = 255;

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self._mask_image_identity_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        self.mask_image_identity_uploaded = true;
    }

    pub(super) fn ensure_material_catalog_uploaded(&mut self, queue: &wgpu::Queue) {
        if self.material_catalog_uploaded {
            return;
        }

        // Layer 0: hash noise (portable and deterministic).
        // Layer 1: Bayer 8x8 repeated (portable and deterministic).
        let w = 64u32;
        let h = 64u32;
        let bytes_per_pixel = 4usize;
        let bytes_per_row = (w as usize) * bytes_per_pixel;

        fn bayer8x8(x: u32, y: u32) -> u8 {
            const M: [[u8; 8]; 8] = [
                [0, 48, 12, 60, 3, 51, 15, 63],
                [32, 16, 44, 28, 35, 19, 47, 31],
                [8, 56, 4, 52, 11, 59, 7, 55],
                [40, 24, 36, 20, 43, 27, 39, 23],
                [2, 50, 14, 62, 1, 49, 13, 61],
                [34, 18, 46, 30, 33, 17, 45, 29],
                [10, 58, 6, 54, 9, 57, 5, 53],
                [42, 26, 38, 22, 41, 25, 37, 21],
            ];
            M[(y & 7) as usize][(x & 7) as usize]
        }

        fn hash_noise_u8(x: u32, y: u32) -> u8 {
            let mut v = x ^ (y.wrapping_mul(0x9e3779b9));
            v ^= v >> 16;
            v = v.wrapping_mul(0x7feb352d);
            v ^= v >> 15;
            v = v.wrapping_mul(0x846ca68b);
            v ^= v >> 16;
            (v & 0xff) as u8
        }

        for layer in 0..2u32 {
            let mut rgba = vec![0u8; (w as usize) * (h as usize) * bytes_per_pixel];
            for yy in 0..h {
                for xx in 0..w {
                    let v = match layer {
                        0 => hash_noise_u8(xx, yy),
                        _ => bayer8x8(xx, yy).saturating_mul(4),
                    };
                    let i = (yy as usize) * bytes_per_row + (xx as usize) * bytes_per_pixel;
                    rgba[i] = v;
                    rgba[i + 1] = v;
                    rgba[i + 2] = v;
                    rgba[i + 3] = 255;
                }
            }

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.material_catalog_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some((w as usize * bytes_per_pixel) as u32),
                    rows_per_image: Some(h),
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.material_catalog_uploaded = true;
    }
}
