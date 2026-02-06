use super::super::*;
use super::SvgRasterGpu;
use fret_core::AlphaMode;
use fret_core::time::Instant;

use crate::images::{ImageColorSpace, ImageDescriptor};
use crate::svg::{SMOOTH_SVG_SCALE_FACTOR, upload_alpha_mask, upload_rgba_image};

impl Renderer {
    pub(in crate::renderer) fn ensure_svg_raster(
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
            if self.perf_enabled {
                self.perf_svg_raster_cache_hits = self.perf_svg_raster_cache_hits.saturating_add(1);
            }
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
        if self.perf_enabled {
            self.perf_svg_raster_cache_misses = self.perf_svg_raster_cache_misses.saturating_add(1);
        }
        if self.svg_perf_enabled {
            self.svg_perf.cache_misses = self.svg_perf.cache_misses.saturating_add(1);
        }

        let bytes = self.svgs.get(svg).map(|e| e.bytes.as_ref())?;
        let target_box_px = (key.target_w, key.target_h);

        let (image, uv, size_px, approx_bytes, storage) = match kind {
            SvgRasterKind::AlphaMask => {
                let t_raster = self.svg_perf_enabled.then(Instant::now);
                let mask = self
                    .svg_renderer
                    .render_alpha_mask_fit_mode(bytes, target_box_px, SMOOTH_SVG_SCALE_FACTOR, fit)
                    .ok()?;
                if let Some(t_raster) = t_raster {
                    self.svg_perf.alpha_raster_count =
                        self.svg_perf.alpha_raster_count.saturating_add(1);
                    self.svg_perf.alpha_raster += t_raster.elapsed();
                }

                let t_insert = self.svg_perf_enabled.then(Instant::now);
                if let Some((image, uv, size_px, page_index, alloc_id)) =
                    self.insert_svg_alpha_mask_into_atlas(gpu.device, gpu.queue, &mask)
                {
                    if let Some(t_insert) = t_insert {
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
                    if let Some(t_insert) = t_insert {
                        self.svg_perf.alpha_atlas_write += t_insert.elapsed();
                    }
                    let t_upload = self.svg_perf_enabled.then(Instant::now);
                    let uploaded = upload_alpha_mask(gpu.device, gpu.queue, &mask);
                    if let Some(t_upload) = t_upload {
                        self.svg_perf.alpha_standalone_uploads =
                            self.svg_perf.alpha_standalone_uploads.saturating_add(1);
                        self.svg_perf.alpha_standalone_upload += t_upload.elapsed();
                    }
                    let image = self.register_image(ImageDescriptor {
                        view: uploaded.view.clone(),
                        size: uploaded.size_px,
                        format: wgpu::TextureFormat::R8Unorm,
                        color_space: ImageColorSpace::Linear,
                        alpha_mode: AlphaMode::Straight,
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
                let t_raster = self.svg_perf_enabled.then(Instant::now);
                let rgba = self
                    .svg_renderer
                    .render_rgba_fit_mode(bytes, target_box_px, SMOOTH_SVG_SCALE_FACTOR, fit)
                    .ok()?;
                if let Some(t_raster) = t_raster {
                    self.svg_perf.rgba_raster_count =
                        self.svg_perf.rgba_raster_count.saturating_add(1);
                    self.svg_perf.rgba_raster += t_raster.elapsed();
                }
                let t_upload = self.svg_perf_enabled.then(Instant::now);
                let uploaded = upload_rgba_image(gpu.device, gpu.queue, &rgba);
                if let Some(t_upload) = t_upload {
                    self.svg_perf.rgba_uploads = self.svg_perf.rgba_uploads.saturating_add(1);
                    self.svg_perf.rgba_upload += t_upload.elapsed();
                }
                let image = self.register_image(ImageDescriptor {
                    view: uploaded.view.clone(),
                    size: uploaded.size_px,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    color_space: ImageColorSpace::Srgb,
                    alpha_mode: AlphaMode::Straight,
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
