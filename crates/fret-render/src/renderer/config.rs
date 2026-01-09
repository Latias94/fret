use super::*;
use crate::text::TextFontFamilyConfig;

impl Renderer {
    pub fn set_svg_perf_enabled(&mut self, enabled: bool) {
        self.svg_perf_enabled = enabled;
        self.svg_perf = SvgPerfStats::default();
    }

    /// Drop all cached SVG rasterizations (standalone rasters and alpha-mask atlas pages) while
    /// keeping the underlying registered SVG bytes (`SvgId`) intact.
    ///
    /// This is the GPUI-style explicit lifecycle knob: apps can decide when to reclaim memory and
    /// accept the cost of re-rasterizing later.
    pub fn clear_svg_raster_cache(&mut self) {
        let rasters = std::mem::take(&mut self.svg_rasters);
        for (_, entry) in rasters {
            if matches!(entry.storage, SvgRasterStorage::Standalone { .. }) {
                let _ = self.unregister_image(entry.image);
            }
        }
        self.svg_raster_bytes = 0;

        for idx in 0..self.svg_mask_atlas_pages.len() {
            self.evict_svg_mask_atlas_page(idx);
        }
        self.svg_mask_atlas_pages.clear();
        self.svg_mask_atlas_free.clear();
        self.svg_mask_atlas_bytes = 0;
    }

    /// Drop only the SVG alpha-mask atlas pages and their cached entries.
    ///
    /// This is a cheap explicit “defragment/rebuild” knob: the next `SceneOp::SvgMaskIcon` usage
    /// will re-pack masks into fresh pages.
    pub fn clear_svg_mask_atlas_cache(&mut self) {
        self.svg_rasters
            .retain(|_, entry| matches!(entry.storage, SvgRasterStorage::Standalone { .. }));

        for idx in 0..self.svg_mask_atlas_pages.len() {
            self.evict_svg_mask_atlas_page(idx);
        }
        self.svg_mask_atlas_pages.clear();
        self.svg_mask_atlas_free.clear();
        self.svg_mask_atlas_bytes = 0;
    }

    pub fn take_svg_perf_snapshot(&mut self) -> Option<SvgPerfSnapshot> {
        if !self.svg_perf_enabled {
            return None;
        }

        let pages_live = self
            .svg_mask_atlas_pages
            .iter()
            .filter(|p| p.is_some())
            .count();
        let rasters_live = self.svg_rasters.len();
        let standalone_bytes_live = self.svg_raster_bytes;
        let atlas_bytes_live = self.svg_mask_atlas_bytes;
        let atlas_capacity_px = u64::from(pages_live as u32)
            .saturating_mul(u64::from(SVG_MASK_ATLAS_PAGE_SIZE_PX))
            .saturating_mul(u64::from(SVG_MASK_ATLAS_PAGE_SIZE_PX));
        let atlas_used_px = self
            .svg_rasters
            .values()
            .filter_map(|e| match e.storage {
                SvgRasterStorage::MaskAtlas { page_index, .. } => Some((page_index, e.size_px)),
                SvgRasterStorage::Standalone { .. } => None,
            })
            .filter(|(page_index, _)| {
                self.svg_mask_atlas_pages
                    .get(*page_index)
                    .is_some_and(|p| p.is_some())
            })
            .fold(0u64, |acc, (_, (w, h))| {
                let pad = u64::from(SVG_MASK_ATLAS_PADDING_PX.saturating_mul(2));
                let w_pad = u64::from(w).saturating_add(pad);
                let h_pad = u64::from(h).saturating_add(pad);
                acc.saturating_add(w_pad.saturating_mul(h_pad))
            });

        let snap = SvgPerfSnapshot {
            frames: self.svg_perf.frames,
            prepare_svg_ops_us: self.svg_perf.prepare_svg_ops.as_micros() as u64,

            cache_hits: self.svg_perf.cache_hits,
            cache_misses: self.svg_perf.cache_misses,

            alpha_raster_count: self.svg_perf.alpha_raster_count,
            alpha_raster_us: self.svg_perf.alpha_raster.as_micros() as u64,
            rgba_raster_count: self.svg_perf.rgba_raster_count,
            rgba_raster_us: self.svg_perf.rgba_raster.as_micros() as u64,

            alpha_atlas_inserts: self.svg_perf.alpha_atlas_inserts,
            alpha_atlas_write_us: self.svg_perf.alpha_atlas_write.as_micros() as u64,
            alpha_standalone_uploads: self.svg_perf.alpha_standalone_uploads,
            alpha_standalone_upload_us: self.svg_perf.alpha_standalone_upload.as_micros() as u64,
            rgba_uploads: self.svg_perf.rgba_uploads,
            rgba_upload_us: self.svg_perf.rgba_upload.as_micros() as u64,

            atlas_pages_live: pages_live,
            svg_rasters_live: rasters_live,
            svg_standalone_bytes_live: standalone_bytes_live,
            svg_mask_atlas_bytes_live: atlas_bytes_live,
            svg_mask_atlas_used_px: atlas_used_px,
            svg_mask_atlas_capacity_px: atlas_capacity_px,
        };

        self.svg_perf = SvgPerfStats::default();
        Some(snap)
    }

    pub fn svg_raster_budget_bytes(&self) -> u64 {
        self.svg_raster_budget_bytes
    }

    pub fn set_svg_raster_budget_bytes(&mut self, bytes: u64) {
        // Keep a small non-zero floor so callers can't accidentally force unbounded thrash.
        self.svg_raster_budget_bytes = bytes.max(1024);
    }

    pub fn path_msaa_samples(&self) -> u32 {
        self.path_msaa_samples
    }

    pub fn set_path_msaa_samples(&mut self, samples: u32) {
        let samples = samples.max(1);
        let samples = samples.min(16);
        if samples == 1 {
            self.path_msaa_samples = 1;
            return;
        }

        // wgpu requires sample counts to be powers of two. Prefer a conservative downgrade to the
        // nearest supported-shape value (rather than rounding up to a potentially unsupported count).
        let pow2_floor = 1u32 << (31 - samples.leading_zeros());
        self.path_msaa_samples = pow2_floor.max(1);
    }

    pub fn debug_offscreen_blit_enabled(&self) -> bool {
        self.debug_offscreen_blit_enabled
    }

    pub fn set_debug_offscreen_blit_enabled(&mut self, enabled: bool) {
        self.debug_offscreen_blit_enabled = enabled;
    }

    pub fn debug_pixelate_scale(&self) -> u32 {
        self.debug_pixelate_scale
    }

    pub fn set_debug_pixelate_scale(&mut self, scale: u32) {
        // 0 disables the debug pixelate path; otherwise clamp to a sane upper bound.
        self.debug_pixelate_scale = scale.min(128);
    }

    pub fn debug_blur_radius(&self) -> u32 {
        self.debug_blur_radius
    }

    pub fn set_debug_blur_radius(&mut self, radius: u32) {
        // 0 disables the debug blur path; otherwise clamp to a sane upper bound.
        self.debug_blur_radius = radius.min(64);
    }

    pub fn set_text_font_families(&mut self, config: &TextFontFamilyConfig) -> bool {
        self.text_system.set_font_families(config)
    }

    /// Returns a sorted list of available font family names (best-effort).
    pub fn all_font_names(&self) -> Vec<String> {
        self.text_system.all_font_names()
    }

    /// Adds font bytes (TTF/OTF/TTC) to the renderer text system.
    ///
    /// Returns the number of newly loaded faces.
    pub fn add_fonts(&mut self, fonts: impl IntoIterator<Item = Vec<u8>>) -> usize {
        self.text_system.add_fonts(fonts)
    }

    pub fn text_font_stack_key(&self) -> u64 {
        self.text_system.font_stack_key()
    }

    pub(super) fn effective_path_msaa_samples(&self, format: wgpu::TextureFormat) -> u32 {
        let requested = self.path_msaa_samples.max(1);
        if requested == 1 {
            return 1;
        }

        let features = self.adapter.get_texture_format_features(format);
        if !features
            .allowed_usages
            .contains(wgpu::TextureUsages::RENDER_ATTACHMENT)
        {
            return 1;
        }

        // When MSAA is enabled we render into an intermediate and then sample from the resolved
        // texture in the composite pass, so the format must be sampleable and support resolves.
        if !features
            .allowed_usages
            .contains(wgpu::TextureUsages::TEXTURE_BINDING)
            || !features
                .flags
                .contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)
        {
            return 1;
        }

        for candidate in [16u32, 8, 4, 2] {
            if candidate <= requested && features.flags.sample_count_supported(candidate) {
                return candidate;
            }
        }
        1
    }
}
