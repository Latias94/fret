use super::*;

pub(in crate::renderer) struct SvgRasterGpu<'a> {
    pub(in crate::renderer) device: &'a wgpu::Device,
    pub(in crate::renderer) queue: &'a wgpu::Queue,
}

pub(in crate::renderer) type SvgMaskAtlasInsert = (
    fret_core::ImageId,
    fret_core::UvRect,
    (u32, u32),
    usize,
    etagere::AllocId,
);

#[derive(Default, Clone, Copy)]
pub(super) struct SvgFramePerfCounters {
    pub(super) raster_cache_hits: u64,
    pub(super) raster_cache_misses: u64,
    pub(super) raster_budget_evictions: u64,
    pub(super) mask_atlas_page_evictions: u64,
    pub(super) mask_atlas_entries_evicted: u64,
}

#[derive(Clone, Copy)]
struct SvgRasterOccupancy {
    pages_live: usize,
    rasters_live: usize,
    standalone_bytes_live: u64,
    atlas_bytes_live: u64,
    atlas_used_px: u64,
    atlas_capacity_px: u64,
}

pub(super) struct SvgRasterState {
    pub(super) rasters: HashMap<SvgRasterKey, SvgRasterEntry>,
    pub(super) mask_atlas_pages: Vec<Option<SvgMaskAtlasPage>>,
    pub(super) mask_atlas_free: Vec<usize>,
    pub(super) raster_bytes: u64,
    pub(super) mask_atlas_bytes: u64,
    pub(super) raster_budget_bytes: u64,
    pub(super) raster_epoch: u64,
    pub(super) perf_enabled: bool,
    pub(super) perf: SvgPerfStats,
    pub(super) frame_perf: SvgFramePerfCounters,
}

impl Default for SvgRasterState {
    fn default() -> Self {
        Self {
            rasters: HashMap::new(),
            mask_atlas_pages: Vec::new(),
            mask_atlas_free: Vec::new(),
            raster_bytes: 0,
            mask_atlas_bytes: 0,
            raster_budget_bytes: 64 * 1024 * 1024,
            raster_epoch: 0,
            perf_enabled: false,
            perf: SvgPerfStats::default(),
            frame_perf: SvgFramePerfCounters::default(),
        }
    }
}

impl SvgRasterState {
    pub(super) fn reset_frame_perf_counters(&mut self) {
        self.frame_perf = SvgFramePerfCounters::default();
    }

    pub(super) fn bump_raster_epoch(&mut self) -> u64 {
        self.raster_epoch = self.raster_epoch.wrapping_add(1);
        self.raster_epoch
    }

    pub(super) fn set_perf_enabled(&mut self, enabled: bool) {
        self.perf_enabled = enabled;
        self.perf = SvgPerfStats::default();
    }

    pub(super) fn raster_budget_bytes(&self) -> u64 {
        self.raster_budget_bytes
    }

    pub(super) fn set_raster_budget_bytes(&mut self, bytes: u64) {
        self.raster_budget_bytes = bytes.max(1024);
    }

    pub(super) fn take_perf_snapshot(&mut self) -> Option<SvgPerfSnapshot> {
        if !self.perf_enabled {
            return None;
        }

        let occupancy = self.occupancy();
        let snap = SvgPerfSnapshot {
            frames: self.perf.frames,
            prepare_svg_ops_us: self.perf.prepare_svg_ops.as_micros() as u64,
            cache_hits: self.perf.cache_hits,
            cache_misses: self.perf.cache_misses,
            alpha_raster_count: self.perf.alpha_raster_count,
            alpha_raster_us: self.perf.alpha_raster.as_micros() as u64,
            rgba_raster_count: self.perf.rgba_raster_count,
            rgba_raster_us: self.perf.rgba_raster.as_micros() as u64,
            alpha_atlas_inserts: self.perf.alpha_atlas_inserts,
            alpha_atlas_write_us: self.perf.alpha_atlas_write.as_micros() as u64,
            alpha_standalone_uploads: self.perf.alpha_standalone_uploads,
            alpha_standalone_upload_us: self.perf.alpha_standalone_upload.as_micros() as u64,
            rgba_uploads: self.perf.rgba_uploads,
            rgba_upload_us: self.perf.rgba_upload.as_micros() as u64,
            atlas_pages_live: occupancy.pages_live,
            svg_rasters_live: occupancy.rasters_live,
            svg_standalone_bytes_live: occupancy.standalone_bytes_live,
            svg_mask_atlas_bytes_live: occupancy.atlas_bytes_live,
            svg_mask_atlas_used_px: occupancy.atlas_used_px,
            svg_mask_atlas_capacity_px: occupancy.atlas_capacity_px,
        };

        self.perf = SvgPerfStats::default();
        Some(snap)
    }

    pub(super) fn write_frame_perf(&self, frame_perf: &mut RenderPerfStats) {
        let occupancy = self.occupancy();
        frame_perf.svg_raster_budget_bytes = self.raster_budget_bytes;
        frame_perf.svg_rasters_live = occupancy.rasters_live as u64;
        frame_perf.svg_standalone_bytes_live = occupancy.standalone_bytes_live;
        frame_perf.svg_mask_atlas_pages_live = occupancy.pages_live as u64;
        frame_perf.svg_mask_atlas_bytes_live = occupancy.atlas_bytes_live;
        frame_perf.svg_mask_atlas_used_px = occupancy.atlas_used_px;
        frame_perf.svg_mask_atlas_capacity_px = occupancy.atlas_capacity_px;
        frame_perf.svg_raster_cache_hits = self.frame_perf.raster_cache_hits;
        frame_perf.svg_raster_cache_misses = self.frame_perf.raster_cache_misses;
        frame_perf.svg_raster_budget_evictions = self.frame_perf.raster_budget_evictions;
        frame_perf.svg_mask_atlas_page_evictions = self.frame_perf.mask_atlas_page_evictions;
        frame_perf.svg_mask_atlas_entries_evicted = self.frame_perf.mask_atlas_entries_evicted;
    }

    fn occupancy(&self) -> SvgRasterOccupancy {
        let pages_live = self
            .mask_atlas_pages
            .iter()
            .filter(|page| page.is_some())
            .count();
        let atlas_capacity_px = u64::from(pages_live as u32)
            .saturating_mul(u64::from(SVG_MASK_ATLAS_PAGE_SIZE_PX))
            .saturating_mul(u64::from(SVG_MASK_ATLAS_PAGE_SIZE_PX));
        let atlas_used_px = self
            .rasters
            .values()
            .filter_map(|entry| match entry.storage {
                SvgRasterStorage::MaskAtlas { page_index, .. } => Some((page_index, entry.size_px)),
                SvgRasterStorage::Standalone { .. } => None,
            })
            .filter(|(page_index, _)| {
                self.mask_atlas_pages
                    .get(*page_index)
                    .is_some_and(|p| p.is_some())
            })
            .fold(0u64, |acc, (_, (w, h))| {
                let pad = u64::from(SVG_MASK_ATLAS_PADDING_PX.saturating_mul(2));
                let w_pad = u64::from(w).saturating_add(pad);
                let h_pad = u64::from(h).saturating_add(pad);
                acc.saturating_add(w_pad.saturating_mul(h_pad))
            });

        SvgRasterOccupancy {
            pages_live,
            rasters_live: self.rasters.len(),
            standalone_bytes_live: self.raster_bytes,
            atlas_bytes_live: self.mask_atlas_bytes,
            atlas_used_px,
            atlas_capacity_px,
        }
    }
}

mod atlas;
mod cache;
mod prepare;
mod raster;
