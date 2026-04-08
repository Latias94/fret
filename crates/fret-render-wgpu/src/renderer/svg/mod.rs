use super::*;
use std::collections::BTreeSet;

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

pub(super) enum SvgRegistryUnregisterOutcome {
    Missing,
    StillReferenced,
    Removed,
}

pub(super) struct SvgRegistryState {
    pub(super) renderer: SvgRenderer,
    svgs: SlotMap<fret_core::SvgId, SvgEntry>,
    hash_index: HashMap<u64, Vec<fret_core::SvgId>>,
}

impl SvgRegistryState {
    pub(super) fn new() -> Self {
        Self {
            renderer: SvgRenderer::new(),
            svgs: SlotMap::with_key(),
            hash_index: HashMap::new(),
        }
    }

    #[cfg(test)]
    pub(super) fn bytes(&self, svg: fret_core::SvgId) -> Option<&[u8]> {
        self.svgs.get(svg).map(|entry| entry.bytes.as_ref())
    }

    pub(super) fn bytes_arc(&self, svg: fret_core::SvgId) -> Option<Arc<[u8]>> {
        self.svgs.get(svg).map(|entry| Arc::clone(&entry.bytes))
    }

    pub(super) fn contains_text_nodes(&self, svg: fret_core::SvgId) -> bool {
        self.svgs
            .get(svg)
            .is_some_and(|entry| entry.contains_text_nodes)
    }

    pub(super) fn register_svg(&mut self, bytes: &[u8]) -> fret_core::SvgId {
        let hash = hash_bytes(bytes);
        let contains_text_nodes = crate::svg::svg_contains_text_nodes(bytes).unwrap_or(false);
        if let Some(ids) = self.hash_index.get(&hash) {
            for &id in ids {
                let Some(existing) = self.svgs.get(id) else {
                    continue;
                };
                if existing.bytes.as_ref() == bytes {
                    if let Some(entry) = self.svgs.get_mut(id) {
                        entry.refs = entry.refs.saturating_add(1);
                    }
                    return id;
                }
            }
        }

        let id = self.svgs.insert(SvgEntry {
            bytes: Arc::<[u8]>::from(bytes),
            contains_text_nodes,
            refs: 1,
        });
        self.hash_index.entry(hash).or_default().push(id);
        id
    }

    pub(super) fn unregister_svg(&mut self, svg: fret_core::SvgId) -> SvgRegistryUnregisterOutcome {
        let Some(refs) = self.svgs.get(svg).map(|entry| entry.refs) else {
            return SvgRegistryUnregisterOutcome::Missing;
        };

        if refs > 1 {
            if let Some(entry) = self.svgs.get_mut(svg) {
                entry.refs = entry.refs.saturating_sub(1);
            }
            return SvgRegistryUnregisterOutcome::StillReferenced;
        }

        let Some(bytes) = self.svgs.remove(svg).map(|entry| entry.bytes) else {
            return SvgRegistryUnregisterOutcome::Missing;
        };

        let hash = hash_bytes(&bytes);
        if let Some(list) = self.hash_index.get_mut(&hash) {
            list.retain(|id| *id != svg);
            if list.is_empty() {
                self.hash_index.remove(&hash);
            }
        }

        SvgRegistryUnregisterOutcome::Removed
    }
}

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

#[derive(Default)]
struct SvgTextBridgeDiagnosticsAggregate {
    selection_misses: BTreeSet<crate::svg::SvgTextFontSelectionMiss>,
    fallback_records: BTreeSet<crate::svg::SvgTextFontFallbackRecord>,
    missing_glyphs: BTreeSet<crate::svg::SvgTextMissingGlyphRecord>,
}

impl SvgTextBridgeDiagnosticsAggregate {
    fn clear(&mut self) {
        self.selection_misses.clear();
        self.fallback_records.clear();
        self.missing_glyphs.clear();
    }

    fn observe(&mut self, diagnostics: &crate::svg::SvgTextBridgeDiagnostics) {
        self.selection_misses
            .extend(diagnostics.selection_misses.iter().cloned());
        self.fallback_records
            .extend(diagnostics.fallback_records.iter().cloned());
        self.missing_glyphs
            .extend(diagnostics.missing_glyphs.iter().cloned());
    }

    fn to_snapshot(&self, revision: u64) -> crate::SvgTextBridgeDiagnosticsSnapshot {
        crate::svg::SvgTextBridgeDiagnostics {
            selection_misses: self.selection_misses.iter().cloned().collect(),
            fallback_records: self.fallback_records.iter().cloned().collect(),
            missing_glyphs: self.missing_glyphs.iter().cloned().collect(),
        }
        .to_snapshot(revision)
    }
}

pub(super) struct SvgRasterState {
    pub(super) rasters: HashMap<SvgRasterKey, SvgRasterEntry>,
    pub(super) mask_atlas_pages: Vec<Option<SvgMaskAtlasPage>>,
    pub(super) mask_atlas_free: Vec<usize>,
    pub(super) text_bridge: Option<SvgTextFontBridgeState>,
    text_bridge_frame_observed: bool,
    text_bridge_frame_diagnostics: SvgTextBridgeDiagnosticsAggregate,
    text_bridge_last_observed_revision: u64,
    text_bridge_last_snapshot: Option<crate::SvgTextBridgeDiagnosticsSnapshot>,
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
            text_bridge: None,
            text_bridge_frame_observed: false,
            text_bridge_frame_diagnostics: SvgTextBridgeDiagnosticsAggregate::default(),
            text_bridge_last_observed_revision: 0,
            text_bridge_last_snapshot: None,
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

pub(super) struct SvgTextFontBridgeState {
    pub(super) font_stack_key: u64,
    pub(super) fontdb: Arc<usvg::fontdb::Database>,
}

impl SvgRasterState {
    pub(super) fn begin_text_bridge_diagnostics_frame(&mut self) {
        self.text_bridge_frame_observed = false;
        self.text_bridge_frame_diagnostics.clear();
    }

    pub(super) fn note_text_bridge_diagnostics(
        &mut self,
        diagnostics: &crate::svg::SvgTextBridgeDiagnostics,
    ) {
        self.text_bridge_frame_observed = true;
        self.text_bridge_frame_diagnostics.observe(diagnostics);
    }

    pub(super) fn commit_text_bridge_diagnostics_frame(&mut self) {
        if !self.text_bridge_frame_observed {
            return;
        }

        self.text_bridge_last_observed_revision =
            self.text_bridge_last_observed_revision.saturating_add(1);
        self.text_bridge_last_snapshot = Some(
            self.text_bridge_frame_diagnostics
                .to_snapshot(self.text_bridge_last_observed_revision),
        );
        self.text_bridge_frame_observed = false;
        self.text_bridge_frame_diagnostics.clear();
    }

    pub(super) fn invalidate_text_bridge_environment(&mut self) {
        self.text_bridge = None;
        self.text_bridge_frame_observed = false;
        self.text_bridge_frame_diagnostics.clear();
        self.text_bridge_last_snapshot = None;
    }

    pub(super) fn svg_text_bridge_diagnostics_snapshot(
        &self,
    ) -> Option<&crate::SvgTextBridgeDiagnosticsSnapshot> {
        self.text_bridge_last_snapshot.as_ref()
    }

    pub(super) fn take_rasters_for_svg(&mut self, svg: fret_core::SvgId) -> Vec<SvgRasterEntry> {
        let keys_to_remove: Vec<_> = self
            .rasters
            .keys()
            .copied()
            .filter(|key| key.svg == svg)
            .collect();
        let mut entries = Vec::with_capacity(keys_to_remove.len());
        for key in keys_to_remove {
            if let Some(entry) = self.rasters.remove(&key) {
                entries.push(entry);
            }
        }
        entries
    }

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

#[cfg(test)]
mod tests {
    use super::super::types::SvgRasterKind;
    use super::super::{Point, Px, Rect, Size};
    use super::{SvgRasterGpu, SvgRegistryState, SvgRegistryUnregisterOutcome};
    use crate::Renderer;
    use fret_core::SvgService;
    use fret_core::TextCommonFallbackInjection;

    fn bundled_only_svg_text_renderer() -> (crate::WgpuContext, Renderer) {
        unsafe {
            std::env::set_var("FRET_TEXT_SYSTEM_FONTS", "0");
        }

        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);
        let added = renderer.add_fonts(fret_fonts::test_support::face_blobs(
            fret_fonts::default_profile()
                .faces
                .iter()
                .chain(fret_fonts_cjk::default_profile().faces.iter())
                .chain(fret_fonts_emoji::default_profile().faces.iter()),
        ));
        assert!(
            added > 0,
            "expected bundled fonts to load for SVG text raster tests"
        );

        let _ = renderer.set_text_font_families(&crate::TextFontFamilyConfig {
            common_fallback_injection: TextCommonFallbackInjection::CommonFallback,
            ui_sans: vec!["Inter".to_string()],
            ui_mono: vec!["JetBrains Mono".to_string()],
            ..Default::default()
        });

        (ctx, renderer)
    }

    #[test]
    fn registry_deduplicates_svg_bytes_and_tracks_refcounts() {
        let mut state = SvgRegistryState::new();
        let bytes = br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#;

        let first = state.register_svg(bytes);
        let second = state.register_svg(bytes);

        assert_eq!(first, second);
        assert_eq!(state.bytes(first), Some(bytes.as_slice()));
        assert!(matches!(
            state.unregister_svg(first),
            SvgRegistryUnregisterOutcome::StillReferenced
        ));
        assert_eq!(state.bytes(first), Some(bytes.as_slice()));
        assert!(matches!(
            state.unregister_svg(first),
            SvgRegistryUnregisterOutcome::Removed
        ));
        assert_eq!(state.bytes(first), None);
        assert!(matches!(
            state.unregister_svg(first),
            SvgRegistryUnregisterOutcome::Missing
        ));
    }

    #[test]
    fn registry_records_svg_text_presence() {
        let mut state = SvgRegistryState::new();
        let outline =
            br#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="8" height="8"/></svg>"#;
        let text =
            br#"<svg xmlns="http://www.w3.org/2000/svg"><text x="1" y="6">Fret</text></svg>"#;

        let outline_id = state.register_svg(outline);
        let text_id = state.register_svg(text);

        assert!(!state.contains_text_nodes(outline_id));
        assert!(state.contains_text_nodes(text_id));
    }

    #[test]
    fn raster_key_tracks_text_font_stack_only_for_text_svgs() {
        let ctx = pollster::block_on(crate::WgpuContext::new()).expect("wgpu context");
        let mut renderer = Renderer::new(&ctx.adapter, &ctx.device);

        let outline_svg = renderer.register_svg(
            br#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="8" height="8"/></svg>"#,
        );
        let text_svg = renderer.register_svg(
            br#"<svg xmlns="http://www.w3.org/2000/svg"><text x="1" y="6">Fret</text></svg>"#,
        );

        let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(32.0), Px(16.0)));
        let outline_key0 = renderer.svg_raster_key(
            outline_svg,
            rect,
            1.0,
            SvgRasterKind::Rgba,
            fret_core::SvgFit::Contain,
        );
        let text_key0 = renderer.svg_raster_key(
            text_svg,
            rect,
            1.0,
            SvgRasterKind::Rgba,
            fret_core::SvgFit::Contain,
        );

        assert_eq!(outline_key0.text_font_stack_key, 0);
        assert_ne!(text_key0.text_font_stack_key, 0);

        assert!(renderer.set_text_locale(Some("zh-CN")));

        let outline_key1 = renderer.svg_raster_key(
            outline_svg,
            rect,
            1.0,
            SvgRasterKind::Rgba,
            fret_core::SvgFit::Contain,
        );
        let text_key1 = renderer.svg_raster_key(
            text_svg,
            rect,
            1.0,
            SvgRasterKind::Rgba,
            fret_core::SvgFit::Contain,
        );

        assert_eq!(outline_key0, outline_key1);
        assert_ne!(text_key0, text_key1);
    }

    #[test]
    fn ensure_svg_raster_allows_text_when_bridge_diagnostics_are_clean() {
        let (ctx, mut renderer) = bundled_only_svg_text_renderer();
        let svg = renderer.register_svg(
            br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 24"><text x="4" y="18" font-family="Inter" font-size="16">A&#x4E2D;</text></svg>"#,
        );
        let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(24.0)));
        let gpu = SvgRasterGpu {
            device: &ctx.device,
            queue: &ctx.queue,
        };

        renderer
            .svg_raster_state
            .begin_text_bridge_diagnostics_frame();
        let raster = renderer.ensure_svg_raster(
            &gpu,
            svg,
            rect,
            1.0,
            SvgRasterKind::Rgba,
            fret_core::SvgFit::Contain,
        );
        renderer
            .svg_raster_state
            .commit_text_bridge_diagnostics_frame();

        assert!(
            raster.is_some(),
            "expected renderer SVG raster path to allow text when the bridge diagnostics are clean"
        );
        let snapshot = renderer
            .svg_text_bridge_diagnostics_snapshot()
            .expect("clean text SVG should publish a bridge snapshot");
        assert!(snapshot.is_clean());
        assert_eq!(snapshot.revision, 1);
        assert!(snapshot.selection_misses.is_empty());
        assert!(snapshot.missing_glyphs.is_empty());
    }

    #[test]
    fn ensure_svg_raster_keeps_rejecting_text_when_bridge_diagnostics_are_not_clean() {
        let (ctx, mut renderer) = bundled_only_svg_text_renderer();
        let svg = renderer.register_svg(
            br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 24"><text x="4" y="18" font-family="Inter" font-size="16">&#x0378;</text></svg>"#,
        );
        let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(24.0)));
        let gpu = SvgRasterGpu {
            device: &ctx.device,
            queue: &ctx.queue,
        };

        renderer
            .svg_raster_state
            .begin_text_bridge_diagnostics_frame();
        let raster = renderer.ensure_svg_raster(
            &gpu,
            svg,
            rect,
            1.0,
            SvgRasterKind::Rgba,
            fret_core::SvgFit::Contain,
        );
        renderer
            .svg_raster_state
            .commit_text_bridge_diagnostics_frame();

        assert!(
            raster.is_none(),
            "expected renderer SVG raster path to keep rejecting text when bridge diagnostics report unresolved glyphs"
        );
        let snapshot = renderer
            .svg_text_bridge_diagnostics_snapshot()
            .expect("failed text SVG should still publish a bridge snapshot");
        assert!(!snapshot.is_clean());
        assert_eq!(snapshot.revision, 1);
        assert_eq!(snapshot.missing_glyphs.len(), 1);
    }

    #[test]
    fn svg_text_bridge_snapshot_clears_when_text_environment_changes() {
        let (ctx, mut renderer) = bundled_only_svg_text_renderer();
        let svg = renderer.register_svg(
            br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 24"><text x="4" y="18" font-family="Inter" font-size="16">Fret</text></svg>"#,
        );
        let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(64.0), Px(24.0)));
        let gpu = SvgRasterGpu {
            device: &ctx.device,
            queue: &ctx.queue,
        };

        renderer
            .svg_raster_state
            .begin_text_bridge_diagnostics_frame();
        let _ = renderer.ensure_svg_raster(
            &gpu,
            svg,
            rect,
            1.0,
            SvgRasterKind::Rgba,
            fret_core::SvgFit::Contain,
        );
        renderer
            .svg_raster_state
            .commit_text_bridge_diagnostics_frame();

        assert!(renderer.svg_text_bridge_diagnostics_snapshot().is_some());
        assert!(renderer.set_text_locale(Some("zh-CN")));
        assert!(renderer.svg_text_bridge_diagnostics_snapshot().is_none());
    }
}
