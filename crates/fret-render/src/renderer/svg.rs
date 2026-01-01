use super::*;
use std::time::Instant;

use crate::images::{ImageColorSpace, ImageDescriptor};
use crate::svg::{SMOOTH_SVG_SCALE_FACTOR, SvgAlphaMask, upload_alpha_mask, upload_rgba_image};

pub(super) struct SvgRasterGpu<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
}

pub(super) type SvgMaskAtlasInsert = (
    fret_core::ImageId,
    UvRect,
    (u32, u32),
    usize,
    etagere::AllocId,
);

impl Renderer {
    pub(super) fn svg_target_box_px(rect: Rect, scale_factor: f32) -> (u32, u32) {
        let w = (rect.size.width.0 * scale_factor).ceil().max(1.0);
        let h = (rect.size.height.0 * scale_factor).ceil().max(1.0);
        (w as u32, h as u32)
    }

    pub(super) fn svg_raster_key(
        svg: fret_core::SvgId,
        rect: Rect,
        scale_factor: f32,
        kind: SvgRasterKind,
        fit: fret_core::SvgFit,
    ) -> SvgRasterKey {
        let (target_w, target_h) = Self::svg_target_box_px(rect, scale_factor);
        SvgRasterKey {
            svg,
            target_w,
            target_h,
            smooth_scale_bits: SMOOTH_SVG_SCALE_FACTOR.to_bits(),
            kind,
            fit,
        }
    }

    pub(super) fn prepare_svg_ops(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        scale_factor: f32,
    ) {
        #[cfg(debug_assertions)]
        if let Err(e) = scene.validate() {
            panic!("invalid scene: {e}");
        }

        let gpu = SvgRasterGpu { device, queue };
        let mut transform_stack: Vec<Transform2D> = vec![Transform2D::IDENTITY];

        let current_transform_scale = |t: Transform2D| -> f32 {
            if let Some((s, _)) = t.as_translation_uniform_scale()
                && s.is_finite()
                && s > 0.0
            {
                return s;
            }

            let sx = (t.a * t.a + t.b * t.b).sqrt();
            let sy = (t.c * t.c + t.d * t.d).sqrt();
            let s = sx.max(sy);
            if s.is_finite() && s > 0.0 { s } else { 1.0 }
        };

        for op in scene.ops() {
            match op {
                SceneOp::PushTransform { transform } => {
                    let current = *transform_stack
                        .last()
                        .expect("transform stack must be non-empty");
                    transform_stack.push(current * *transform);
                }
                SceneOp::PopTransform => {
                    if transform_stack.len() > 1 {
                        transform_stack.pop();
                    }
                }
                SceneOp::PushOpacity { .. }
                | SceneOp::PopOpacity
                | SceneOp::PushLayer { .. }
                | SceneOp::PopLayer
                | SceneOp::PushClipRect { .. }
                | SceneOp::PushClipRRect { .. }
                | SceneOp::PopClip => {}
                SceneOp::SvgMaskIcon { rect, svg, fit, .. } => {
                    let s = current_transform_scale(
                        *transform_stack
                            .last()
                            .expect("transform stack must be non-empty"),
                    );
                    let rect = Rect::new(
                        rect.origin,
                        Size::new(Px(rect.size.width.0 * s), Px(rect.size.height.0 * s)),
                    );
                    let _ = self.ensure_svg_raster(
                        &gpu,
                        *svg,
                        rect,
                        scale_factor,
                        SvgRasterKind::AlphaMask,
                        *fit,
                    );
                }
                SceneOp::SvgImage { rect, svg, fit, .. } => {
                    let s = current_transform_scale(
                        *transform_stack
                            .last()
                            .expect("transform stack must be non-empty"),
                    );
                    let rect = Rect::new(
                        rect.origin,
                        Size::new(Px(rect.size.width.0 * s), Px(rect.size.height.0 * s)),
                    );
                    let _ = self.ensure_svg_raster(
                        &gpu,
                        *svg,
                        rect,
                        scale_factor,
                        SvgRasterKind::Rgba,
                        *fit,
                    );
                }
                SceneOp::Quad { .. }
                | SceneOp::Image { .. }
                | SceneOp::ImageRegion { .. }
                | SceneOp::MaskImage { .. }
                | SceneOp::Text { .. }
                | SceneOp::Path { .. }
                | SceneOp::ViewportSurface { .. } => {}
            }
        }

        self.prune_svg_rasters();
    }

    pub(super) fn bump_svg_raster_epoch(&mut self) -> u64 {
        self.svg_raster_epoch = self.svg_raster_epoch.wrapping_add(1);
        self.svg_raster_epoch
    }

    pub(super) fn prune_svg_rasters(&mut self) {
        if self.svg_raster_bytes <= self.svg_raster_budget_bytes {
            return;
        }

        // Best-effort eviction: never evict entries used in the current frame.
        let cur_epoch = self.svg_raster_epoch;

        while self.svg_raster_bytes > self.svg_raster_budget_bytes {
            let mut victim_standalone: Option<(SvgRasterKey, u64)> = None;
            for (k, v) in &self.svg_rasters {
                if v.last_used_epoch == cur_epoch {
                    continue;
                }
                if !matches!(&v.storage, SvgRasterStorage::Standalone { .. }) {
                    continue;
                }
                match victim_standalone {
                    None => victim_standalone = Some((*k, v.last_used_epoch)),
                    Some((_, best_epoch)) => {
                        if v.last_used_epoch < best_epoch {
                            victim_standalone = Some((*k, v.last_used_epoch));
                        }
                    }
                }
            }

            let Some((victim_key, _)) = victim_standalone else {
                // Cache is over budget but all standalone entries were used this frame (or there
                // are no standalone entries). Keep correctness and allow a temporary overshoot.
                break;
            };

            if let Some(entry) = self.svg_rasters.remove(&victim_key) {
                self.drop_svg_raster_entry(entry);
            } else {
                break;
            }
        }
    }

    pub(super) fn drop_svg_raster_entry(&mut self, entry: SvgRasterEntry) {
        match entry.storage {
            SvgRasterStorage::Standalone { .. } => {
                self.svg_raster_bytes = self.svg_raster_bytes.saturating_sub(entry.approx_bytes);
                let _ = self.unregister_image(entry.image);
            }
            SvgRasterStorage::MaskAtlas {
                page_index,
                alloc_id,
            } => {
                let Some(page) = self
                    .svg_mask_atlas_pages
                    .get_mut(page_index)
                    .and_then(|p| p.as_mut())
                else {
                    return;
                };
                page.allocator.deallocate(alloc_id);
                page.entries = page.entries.saturating_sub(1);
                if page.entries == 0 {
                    self.evict_svg_mask_atlas_page(page_index);
                }
            }
        }
    }

    pub(super) fn ensure_svg_mask_atlas_page(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> usize {
        let size_px = (SVG_MASK_ATLAS_PAGE_SIZE_PX, SVG_MASK_ATLAS_PAGE_SIZE_PX);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret svg mask atlas"),
            size: wgpu::Extent3d {
                width: size_px.0,
                height: size_px.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let image = self.register_image(ImageDescriptor {
            view: view.clone(),
            size: size_px,
            format: wgpu::TextureFormat::R8Unorm,
            color_space: ImageColorSpace::Linear,
        });

        let zeros = vec![0u8; (size_px.0 as usize) * (size_px.1 as usize)];
        write_r8_texture_region(queue, &texture, (0, 0), size_px, &zeros);

        let page = SvgMaskAtlasPage {
            image,
            size_px,
            allocator: etagere::BucketedAtlasAllocator::new(etagere::Size::new(
                size_px.0 as i32,
                size_px.1 as i32,
            )),
            entries: 0,
            last_used_epoch: self.svg_raster_epoch,
            _texture: texture,
        };

        let idx = self.svg_mask_atlas_free.pop().unwrap_or_else(|| {
            self.svg_mask_atlas_pages.push(None);
            self.svg_mask_atlas_pages.len() - 1
        });
        self.svg_mask_atlas_pages[idx] = Some(page);

        self.svg_mask_atlas_bytes = self
            .svg_mask_atlas_bytes
            .saturating_add(u64::from(size_px.0).saturating_mul(u64::from(size_px.1)));
        idx
    }

    pub(super) fn insert_svg_alpha_mask_into_atlas(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mask: &SvgAlphaMask,
    ) -> Option<SvgMaskAtlasInsert> {
        let (w, h) = mask.size_px;
        if w == 0 || h == 0 {
            return None;
        }

        let pad = SVG_MASK_ATLAS_PADDING_PX;
        let w_pad = w.saturating_add(pad.saturating_mul(2));
        let h_pad = h.saturating_add(pad.saturating_mul(2));
        if w_pad == 0
            || h_pad == 0
            || w_pad > SVG_MASK_ATLAS_PAGE_SIZE_PX
            || h_pad > SVG_MASK_ATLAS_PAGE_SIZE_PX
        {
            return None;
        }

        let size = etagere::Size::new(w_pad as i32, h_pad as i32);

        let mut alloc: Option<(usize, etagere::Allocation)> = None;
        for (idx, page) in self.svg_mask_atlas_pages.iter_mut().enumerate() {
            let Some(page) = page.as_mut() else {
                continue;
            };
            if let Some(allocation) = page.allocator.allocate(size) {
                alloc = Some((idx, allocation));
                break;
            }
        }
        if alloc.is_none() {
            let page_index = self.ensure_svg_mask_atlas_page(device, queue);
            let page = self.svg_mask_atlas_pages[page_index]
                .as_mut()
                .expect("atlas page exists");
            let allocation = page.allocator.allocate(size)?;
            alloc = Some((page_index, allocation));
        }

        let (page_index, allocation) = alloc?;
        let page = self.svg_mask_atlas_pages[page_index]
            .as_mut()
            .expect("atlas page exists");
        let Ok(x) = u32::try_from(allocation.rectangle.min.x) else {
            page.allocator.deallocate(allocation.id);
            return None;
        };
        let Ok(y) = u32::try_from(allocation.rectangle.min.y) else {
            page.allocator.deallocate(allocation.id);
            return None;
        };

        let mut padded = vec![0u8; (w_pad as usize) * (h_pad as usize)];
        let max_x = w.saturating_sub(1);
        let max_y = h.saturating_sub(1);
        for yy in 0..h_pad {
            let src_y = yy.saturating_sub(pad).min(max_y);
            for xx in 0..w_pad {
                let src_x = xx.saturating_sub(pad).min(max_x);
                let dst = (yy as usize) * (w_pad as usize) + (xx as usize);
                let src = (src_y as usize) * (w as usize) + (src_x as usize);
                padded[dst] = mask.alpha[src];
            }
        }

        write_r8_texture_region(queue, &page._texture, (x, y), (w_pad, h_pad), &padded);

        let page_w = page.size_px.0 as f32;
        let page_h = page.size_px.1 as f32;
        let u0 = (x + pad) as f32 / page_w;
        let v0 = (y + pad) as f32 / page_h;
        let u1 = (x + pad + w) as f32 / page_w;
        let v1 = (y + pad + h) as f32 / page_h;

        page.entries += 1;
        page.last_used_epoch = self.svg_raster_epoch;

        Some((
            page.image,
            UvRect { u0, v0, u1, v1 },
            (w, h),
            page_index,
            allocation.id,
        ))
    }

    pub(super) fn evict_svg_mask_atlas_page(&mut self, page_index: usize) {
        let Some(page) = self
            .svg_mask_atlas_pages
            .get_mut(page_index)
            .and_then(|p| p.take())
        else {
            return;
        };

        let mut keys_to_remove: Vec<SvgRasterKey> = Vec::new();
        for (k, v) in &self.svg_rasters {
            let is_page = match &v.storage {
                SvgRasterStorage::MaskAtlas {
                    page_index: idx, ..
                } => *idx == page_index,
                SvgRasterStorage::Standalone { .. } => false,
            };
            if is_page {
                keys_to_remove.push(*k);
            }
        }
        for k in keys_to_remove {
            let _ = self.svg_rasters.remove(&k);
        }

        self.svg_mask_atlas_bytes = self.svg_mask_atlas_bytes.saturating_sub(page.bytes());
        let _ = self.unregister_image(page.image);

        self.svg_mask_atlas_free.push(page_index);
    }

    pub(super) fn ensure_svg_raster(
        &mut self,
        gpu: &SvgRasterGpu<'_>,
        svg: fret_core::SvgId,
        rect: Rect,
        scale_factor: f32,
        kind: SvgRasterKind,
        fit: fret_core::SvgFit,
    ) -> Option<(fret_core::ImageId, fret_core::UvRect, (u32, u32))> {
        let key = Self::svg_raster_key(svg, rect, scale_factor, kind, fit);
        if self.svg_rasters.contains_key(&key) {
            if self.svg_perf_enabled {
                self.svg_perf.cache_hits = self.svg_perf.cache_hits.saturating_add(1);
            }
            let (image, uv, size_px, page_index) = {
                let e = self.svg_rasters.get_mut(&key)?;
                e.last_used_epoch = self.svg_raster_epoch;
                let page_index = match &e.storage {
                    SvgRasterStorage::MaskAtlas { page_index, .. } => Some(*page_index),
                    SvgRasterStorage::Standalone { .. } => None,
                };
                (e.image, e.uv, e.size_px, page_index)
            };
            if let Some(page_index) = page_index
                && let Some(Some(page)) = self.svg_mask_atlas_pages.get_mut(page_index)
            {
                page.last_used_epoch = self.svg_raster_epoch;
            }
            return Some((image, uv, size_px));
        }
        if self.svg_perf_enabled {
            self.svg_perf.cache_misses = self.svg_perf.cache_misses.saturating_add(1);
        }

        let bytes = self.svgs.get(svg)?;
        let target_box_px = (key.target_w, key.target_h);

        let (image, uv, size_px, approx_bytes, storage) = match kind {
            SvgRasterKind::AlphaMask => {
                let t_raster = Instant::now();
                let mask = self
                    .svg_renderer
                    .render_alpha_mask_fit_mode(bytes, target_box_px, SMOOTH_SVG_SCALE_FACTOR, fit)
                    .ok()?;
                if self.svg_perf_enabled {
                    self.svg_perf.alpha_raster_count =
                        self.svg_perf.alpha_raster_count.saturating_add(1);
                    self.svg_perf.alpha_raster += t_raster.elapsed();
                }

                let t_insert = Instant::now();
                if let Some((image, uv, size_px, page_index, alloc_id)) =
                    self.insert_svg_alpha_mask_into_atlas(gpu.device, gpu.queue, &mask)
                {
                    if self.svg_perf_enabled {
                        self.svg_perf.alpha_atlas_inserts =
                            self.svg_perf.alpha_atlas_inserts.saturating_add(1);
                        self.svg_perf.alpha_atlas_write += t_insert.elapsed();
                    }
                    (
                        image,
                        uv,
                        size_px,
                        0,
                        SvgRasterStorage::MaskAtlas {
                            page_index,
                            alloc_id,
                        },
                    )
                } else {
                    if self.svg_perf_enabled {
                        self.svg_perf.alpha_atlas_write += t_insert.elapsed();
                    }
                    let t_upload = Instant::now();
                    let uploaded = upload_alpha_mask(gpu.device, gpu.queue, &mask);
                    if self.svg_perf_enabled {
                        self.svg_perf.alpha_standalone_uploads =
                            self.svg_perf.alpha_standalone_uploads.saturating_add(1);
                        self.svg_perf.alpha_standalone_upload += t_upload.elapsed();
                    }
                    let image = self.register_image(ImageDescriptor {
                        view: uploaded.view.clone(),
                        size: uploaded.size_px,
                        format: wgpu::TextureFormat::R8Unorm,
                        color_space: ImageColorSpace::Linear,
                    });
                    let approx_bytes =
                        u64::from(uploaded.size_px.0).saturating_mul(u64::from(uploaded.size_px.1));
                    (
                        image,
                        UvRect::FULL,
                        uploaded.size_px,
                        approx_bytes,
                        SvgRasterStorage::Standalone {
                            _texture: uploaded.texture,
                        },
                    )
                }
            }
            SvgRasterKind::Rgba => {
                let t_raster = Instant::now();
                let rgba = self
                    .svg_renderer
                    .render_rgba_fit_mode(bytes, target_box_px, SMOOTH_SVG_SCALE_FACTOR, fit)
                    .ok()?;
                if self.svg_perf_enabled {
                    self.svg_perf.rgba_raster_count =
                        self.svg_perf.rgba_raster_count.saturating_add(1);
                    self.svg_perf.rgba_raster += t_raster.elapsed();
                }
                let t_upload = Instant::now();
                let uploaded = upload_rgba_image(gpu.device, gpu.queue, &rgba);
                if self.svg_perf_enabled {
                    self.svg_perf.rgba_uploads = self.svg_perf.rgba_uploads.saturating_add(1);
                    self.svg_perf.rgba_upload += t_upload.elapsed();
                }
                let image = self.register_image(ImageDescriptor {
                    view: uploaded.view.clone(),
                    size: uploaded.size_px,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    color_space: ImageColorSpace::Srgb,
                });
                let approx_bytes = u64::from(uploaded.size_px.0)
                    .saturating_mul(u64::from(uploaded.size_px.1))
                    .saturating_mul(4);
                (
                    image,
                    UvRect::FULL,
                    uploaded.size_px,
                    approx_bytes,
                    SvgRasterStorage::Standalone {
                        _texture: uploaded.texture,
                    },
                )
            }
        };

        let is_standalone = matches!(&storage, SvgRasterStorage::Standalone { .. });
        self.svg_rasters.insert(
            key,
            SvgRasterEntry {
                image,
                uv,
                size_px,
                approx_bytes,
                last_used_epoch: self.svg_raster_epoch,
                storage,
            },
        );
        if is_standalone {
            self.svg_raster_bytes = self.svg_raster_bytes.saturating_add(approx_bytes);
        }

        Some((image, uv, size_px))
    }
}
