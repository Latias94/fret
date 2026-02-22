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

        let textures = GpuTextures::new(mask_image_identity_texture, material_catalog_texture);
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

        let drop_shadow_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret drop-shadow params buffer"),
            size: 256,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            adapter: adapter.clone(),
            uniform_bind_group,
            uniforms,
            globals,
            textures,
            pipelines: GpuPipelines::default(),
            quad_instances,
            path_paints,
            text_paints,
            viewport_vertices,
            text_vertices,
            clip_mask_param_buffer,
            clip_mask_param_bind_group,
            clip_mask_param_bind_group_layout,
            scale_param_buffer,
            scale_param_stride,
            scale_param_capacity,
            backdrop_warp_param_buffer,
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
            drop_shadow_pipeline_format: None,
            drop_shadow_pipeline: None,
            drop_shadow_masked_pipeline: None,
            drop_shadow_mask_pipeline: None,
            drop_shadow_bind_group_layout: None,
            drop_shadow_mask_bind_group_layout: None,
            drop_shadow_param_buffer,
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
            clip_path_mask_cache: ClipPathMaskCache::new((256 * 1024 * 1024) / 8),
            perf_enabled: false,
            perf_svg_raster_cache_hits: 0,
            perf_svg_raster_cache_misses: 0,
            perf_svg_raster_budget_evictions: 0,
            perf_svg_mask_atlas_page_evictions: 0,
            perf_svg_mask_atlas_entries_evicted: 0,
            perf_pending_render_target_updates_requested_by_ingest: [0;
                fret_render_core::RenderTargetIngestStrategy::COUNT],
            perf_pending_render_target_updates_by_ingest: [0;
                fret_render_core::RenderTargetIngestStrategy::COUNT],
            perf_pending_render_target_updates_ingest_fallbacks: 0,
            perf_pending_render_target_metadata_degradations_color_encoding_dropped: 0,
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
            bind_group_caches: BindGroupCaches::default(),
            render_target_revisions: HashMap::new(),
            render_targets_generation: 0,
            image_revisions: HashMap::new(),
            images_generation: 0,
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
        let mut desc = desc;
        if Self::render_target_color_encoding_conflicts_with_portable_rgb_assumption(
            desc.color_space,
            desc.metadata.color_encoding,
        ) {
            desc.metadata.color_encoding = fret_render_core::RenderTargetColorEncoding::default();
            if self.perf_enabled {
                self.perf_pending_render_target_metadata_degradations_color_encoding_dropped = self
                    .perf_pending_render_target_metadata_degradations_color_encoding_dropped
                    .saturating_add(1);
            }
        }
        if self.perf_enabled {
            let effective_ix =
                render_target_ingest_strategy_perf_index(desc.metadata.ingest_strategy);
            self.perf_pending_render_target_updates_by_ingest[effective_ix] =
                self.perf_pending_render_target_updates_by_ingest[effective_ix].saturating_add(1);

            let requested_ix =
                render_target_ingest_strategy_perf_index(desc.metadata.requested_ingest_strategy);
            self.perf_pending_render_target_updates_requested_by_ingest[requested_ix] = self
                .perf_pending_render_target_updates_requested_by_ingest[requested_ix]
                .saturating_add(1);

            if desc.metadata.requested_ingest_strategy != desc.metadata.ingest_strategy {
                self.perf_pending_render_target_updates_ingest_fallbacks = self
                    .perf_pending_render_target_updates_ingest_fallbacks
                    .saturating_add(1);
            }
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
        self.bind_group_caches.invalidate_image(id);
        self.images_generation = self.images_generation.saturating_add(1);
        true
    }

    pub fn unregister_image(&mut self, id: fret_core::ImageId) -> bool {
        if !self.images.unregister(id) {
            return false;
        }
        self.image_revisions.remove(&id);
        self.bind_group_caches.invalidate_image(id);
        self.images_generation = self.images_generation.saturating_add(1);
        true
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
            if self.perf_enabled {
                self.perf_pending_render_target_metadata_degradations_color_encoding_dropped = self
                    .perf_pending_render_target_metadata_degradations_color_encoding_dropped
                    .saturating_add(1);
            }
        }
        if self.perf_enabled {
            let effective_ix =
                render_target_ingest_strategy_perf_index(desc.metadata.ingest_strategy);
            self.perf_pending_render_target_updates_by_ingest[effective_ix] =
                self.perf_pending_render_target_updates_by_ingest[effective_ix].saturating_add(1);

            let requested_ix =
                render_target_ingest_strategy_perf_index(desc.metadata.requested_ingest_strategy);
            self.perf_pending_render_target_updates_requested_by_ingest[requested_ix] = self
                .perf_pending_render_target_updates_requested_by_ingest[requested_ix]
                .saturating_add(1);

            if desc.metadata.requested_ingest_strategy != desc.metadata.ingest_strategy {
                self.perf_pending_render_target_updates_ingest_fallbacks = self
                    .perf_pending_render_target_updates_ingest_fallbacks
                    .saturating_add(1);
            }
        }
        if !self.render_targets.update(id, desc) {
            return false;
        }
        let next = self.render_target_revisions.get(&id).copied().unwrap_or(0) + 1;
        self.render_target_revisions.insert(id, next);
        self.bind_group_caches.invalidate_render_target(id);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        true
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
        if !self.render_targets.unregister(id) {
            return false;
        }
        self.render_target_revisions.remove(&id);
        self.bind_group_caches.invalidate_render_target(id);
        self.render_targets_generation = self.render_targets_generation.saturating_add(1);
        true
    }

    pub(super) fn ensure_mask_image_identity_uploaded(&mut self, queue: &wgpu::Queue) {
        self.textures.ensure_mask_image_identity_uploaded(queue);
    }

    pub(super) fn ensure_material_catalog_uploaded(&mut self, queue: &wgpu::Queue) {
        self.textures.ensure_material_catalog_uploaded(queue);
    }
}
