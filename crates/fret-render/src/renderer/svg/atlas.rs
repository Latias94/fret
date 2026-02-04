use super::super::*;
use super::SvgMaskAtlasInsert;
use crate::images::{ImageColorSpace, ImageDescriptor};
use crate::svg::SvgAlphaMask;
use fret_core::AlphaMode;

impl Renderer {
    pub(in crate::renderer) fn ensure_svg_mask_atlas_page(
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
            alpha_mode: AlphaMode::Straight,
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

    pub(in crate::renderer) fn insert_svg_alpha_mask_into_atlas(
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

    pub(in crate::renderer) fn evict_svg_mask_atlas_page(&mut self, page_index: usize) {
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
        if self.perf_enabled {
            self.perf_svg_mask_atlas_page_evictions =
                self.perf_svg_mask_atlas_page_evictions.saturating_add(1);
            self.perf_svg_mask_atlas_entries_evicted = self
                .perf_svg_mask_atlas_entries_evicted
                .saturating_add(keys_to_remove.len() as u64);
        }
        for k in keys_to_remove {
            let _ = self.svg_rasters.remove(&k);
        }

        self.svg_mask_atlas_bytes = self.svg_mask_atlas_bytes.saturating_sub(page.bytes());
        let _ = self.unregister_image(page.image);

        self.svg_mask_atlas_free.push(page_index);
    }
}
