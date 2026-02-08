use super::*;
use crate::images::ImageDescriptor;
use crate::targets::RenderTargetDescriptor;

impl Renderer {
    pub fn new(adapter: &wgpu::Adapter, device: &wgpu::Device) -> Self {
        let uniform_size = std::mem::size_of::<ViewportUniform>() as u64;
        // wgpu requires uniform dynamic offsets to be aligned to 256 bytes.
        let uniform_stride = uniform_size.div_ceil(256) * 256;
        let uniform_capacity = 256usize;
        let clip_capacity = 1024usize;
        let clip_entry_size = std::mem::size_of::<ClipRRectUniform>() as u64;

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
                ],
            });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret quad uniforms buffer"),
            size: uniform_stride * uniform_capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let clip_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("fret clip stack buffer"),
            size: clip_entry_size.saturating_mul(clip_capacity as u64).max(4),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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
        let instance_capacity = 1024;
        let instance_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret quad instances #{i}")),
                    size: (instance_capacity * std::mem::size_of::<QuadInstance>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        let viewport_vertex_capacity = 64 * 6;
        let viewport_vertex_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret viewport vertices #{i}")),
                    size: (viewport_vertex_capacity * std::mem::size_of::<ViewportVertex>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        let text_vertex_capacity = 512 * 6;
        let text_vertex_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret text vertices #{i}")),
                    size: (text_vertex_capacity * std::mem::size_of::<TextVertex>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        let path_vertex_capacity = 1024;
        let path_vertex_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("fret path vertices #{i}")),
                    size: (path_vertex_capacity * std::mem::size_of::<PathVertex>()) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

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

        Self {
            adapter: adapter.clone(),
            uniform_buffer,
            uniform_bind_group,
            uniform_bind_group_layout,
            uniform_stride,
            uniform_capacity,
            clip_buffer,
            clip_capacity,
            quad_pipeline_format: None,
            quad_pipeline: None,
            viewport_pipeline_format: None,
            viewport_pipeline: None,
            viewport_bind_group_layout,
            viewport_sampler,
            instance_buffers,
            instance_buffer_index: 0,
            instance_capacity,
            viewport_vertex_buffers,
            viewport_vertex_buffer_index: 0,
            viewport_vertex_capacity,
            text_pipeline_format: None,
            text_pipeline: None,
            text_color_pipeline_format: None,
            text_color_pipeline: None,
            text_subpixel_pipeline_format: None,
            text_subpixel_pipeline: None,
            mask_pipeline_format: None,
            mask_pipeline: None,
            text_vertex_buffers,
            text_vertex_buffer_index: 0,
            text_vertex_capacity,
            path_pipeline_format: None,
            path_pipeline: None,
            path_msaa_pipeline_format: None,
            path_msaa_pipeline: None,
            path_msaa_pipeline_sample_count: None,
            composite_pipeline_format: None,
            composite_pipeline: None,
            composite_mask_pipeline: None,
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
            path_vertex_buffers,
            path_vertex_buffer_index: 0,
            path_vertex_capacity,
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
            perf: RenderPerfStats::default(),
            last_frame_perf: None,
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
            scene_encoding_cache_key: None,
            scene_encoding_cache: SceneEncoding::default(),
            scene_encoding_scratch: SceneEncoding::default(),
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
        self.images_generation = self.images_generation.saturating_add(1);
        true
    }

    pub fn unregister_image(&mut self, id: fret_core::ImageId) -> bool {
        if !self.images.unregister(id) {
            return false;
        }
        self.image_revisions.remove(&id);
        self.image_bind_groups.remove(&id);
        self.images_generation = self.images_generation.saturating_add(1);
        true
    }

    pub fn update_render_target(
        &mut self,
        id: fret_core::RenderTargetId,
        desc: RenderTargetDescriptor,
    ) -> bool {
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
}
