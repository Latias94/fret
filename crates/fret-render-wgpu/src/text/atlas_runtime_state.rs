use super::atlas::{GlyphAtlas, GlyphAtlasEntry, GlyphAtlasInsertError, GlyphKey};
use super::{DebugGlyphAtlasLookup, TextAtlasPerfSnapshot};

pub(crate) struct TextAtlasRuntimeState {
    mask_atlas: GlyphAtlas,
    color_atlas: GlyphAtlas,
    subpixel_atlas: GlyphAtlas,
    atlas_bind_group_layout: wgpu::BindGroupLayout,
}

impl TextAtlasRuntimeState {
    pub(crate) fn new(
        mask_atlas: GlyphAtlas,
        color_atlas: GlyphAtlas,
        subpixel_atlas: GlyphAtlas,
        atlas_bind_group_layout: wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            mask_atlas,
            color_atlas,
            subpixel_atlas,
            atlas_bind_group_layout,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.mask_atlas.reset();
        self.color_atlas.reset();
        self.subpixel_atlas.reset();
    }

    pub(crate) fn begin_frame_diagnostics(&mut self) {
        self.mask_atlas.begin_frame_diagnostics();
        self.color_atlas.begin_frame_diagnostics();
        self.subpixel_atlas.begin_frame_diagnostics();
    }

    pub(crate) fn atlas_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.atlas_bind_group_layout
    }

    pub(crate) fn mask_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.mask_atlas.bind_group(page)
    }

    pub(crate) fn color_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.color_atlas.bind_group(page)
    }

    pub(crate) fn subpixel_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.subpixel_atlas.bind_group(page)
    }

    pub(crate) fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        self.mask_atlas.flush_uploads(queue);
        self.color_atlas.flush_uploads(queue);
        self.subpixel_atlas.flush_uploads(queue);
    }

    pub(super) fn diagnostics_snapshots(
        &self,
    ) -> (
        fret_core::RendererGlyphAtlasPerfSnapshot,
        fret_core::RendererGlyphAtlasPerfSnapshot,
        fret_core::RendererGlyphAtlasPerfSnapshot,
    ) {
        (
            self.mask_atlas.diagnostics_snapshot(),
            self.color_atlas.diagnostics_snapshot(),
            self.subpixel_atlas.diagnostics_snapshot(),
        )
    }

    pub(super) fn take_perf_snapshot(&mut self) -> TextAtlasPerfSnapshot {
        let mask = self.mask_atlas.take_perf_snapshot();
        let color = self.color_atlas.take_perf_snapshot();
        let subpixel = self.subpixel_atlas.take_perf_snapshot();

        TextAtlasPerfSnapshot {
            uploads: mask.uploads + color.uploads + subpixel.uploads,
            upload_bytes: mask.upload_bytes + color.upload_bytes + subpixel.upload_bytes,
            evicted_glyphs: mask.evicted_glyphs + color.evicted_glyphs + subpixel.evicted_glyphs,
            evicted_pages: mask.evicted_pages + color.evicted_pages + subpixel.evicted_pages,
            evicted_page_glyphs: mask.evicted_page_glyphs
                + color.evicted_page_glyphs
                + subpixel.evicted_page_glyphs,
            resets: mask.resets + color.resets + subpixel.resets,
        }
    }

    pub(super) fn combined_revision(&self) -> u64 {
        self.mask_atlas
            .revision()
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ self.color_atlas.revision().rotate_left(1)
            ^ self.subpixel_atlas.revision().rotate_left(2)
    }

    pub(crate) fn mask_dimensions(&self) -> (u32, u32) {
        self.mask_atlas.dimensions()
    }

    pub(crate) fn color_dimensions(&self) -> (u32, u32) {
        self.color_atlas.dimensions()
    }

    pub(crate) fn subpixel_dimensions(&self) -> (u32, u32) {
        self.subpixel_atlas.dimensions()
    }

    fn atlas_for_key(&self, key: GlyphKey) -> &GlyphAtlas {
        if key.is_color() {
            &self.color_atlas
        } else if key.is_subpixel() {
            &self.subpixel_atlas
        } else {
            &self.mask_atlas
        }
    }

    fn atlas_mut_for_key(&mut self, key: GlyphKey) -> &mut GlyphAtlas {
        if key.is_color() {
            &mut self.color_atlas
        } else if key.is_subpixel() {
            &mut self.subpixel_atlas
        } else {
            &mut self.mask_atlas
        }
    }

    pub(super) fn dimensions_for_key(&self, key: GlyphKey) -> (u32, u32) {
        self.atlas_for_key(key).dimensions()
    }

    pub(super) fn entry(&self, key: GlyphKey) -> Option<GlyphAtlasEntry> {
        self.atlas_for_key(key).entry(key)
    }

    pub(super) fn get(&mut self, key: GlyphKey, epoch: u64) -> Option<GlyphAtlasEntry> {
        self.atlas_mut_for_key(key).get(key, epoch)
    }

    pub(super) fn dec_pin_bucket(
        &mut self,
        mask: &[GlyphKey],
        color: &[GlyphKey],
        subpixel: &[GlyphKey],
    ) {
        self.mask_atlas.dec_live_refs(mask);
        self.color_atlas.dec_live_refs(color);
        self.subpixel_atlas.dec_live_refs(subpixel);
    }

    pub(super) fn inc_pin_bucket(
        &mut self,
        mask: &[GlyphKey],
        color: &[GlyphKey],
        subpixel: &[GlyphKey],
    ) {
        self.mask_atlas.inc_live_refs(mask);
        self.color_atlas.inc_live_refs(color);
        self.subpixel_atlas.inc_live_refs(subpixel);
    }

    pub(super) fn get_or_insert(
        &mut self,
        key: GlyphKey,
        w: u32,
        h: u32,
        placement_left: i32,
        placement_top: i32,
        bytes_per_pixel: u32,
        data: Vec<u8>,
        epoch: u64,
    ) -> Result<GlyphAtlasEntry, GlyphAtlasInsertError> {
        self.atlas_mut_for_key(key).get_or_insert(
            key,
            w,
            h,
            placement_left,
            placement_top,
            bytes_per_pixel,
            data,
            epoch,
        )
    }

    #[cfg(test)]
    pub(super) fn pending_upload_bytes_for_key(&self, key: GlyphKey) -> Option<Vec<u8>> {
        let entry = self.entry(key)?;
        self.atlas_for_key(key)
            .pending_upload_bytes_for_entry(entry)
    }

    pub(crate) fn debug_lookup_mask_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.debug_lookup_entry_in_atlas(&self.mask_atlas, page, x, y, w, h)
    }

    pub(crate) fn debug_lookup_color_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.debug_lookup_entry_in_atlas(&self.color_atlas, page, x, y, w, h)
    }

    pub(crate) fn debug_lookup_subpixel_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.debug_lookup_entry_in_atlas(&self.subpixel_atlas, page, x, y, w, h)
    }

    fn debug_lookup_entry_in_atlas(
        &self,
        atlas: &GlyphAtlas,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        let k = atlas.find_key_for_bounds(page, x, y, w, h)?;

        Some(DebugGlyphAtlasLookup::new(
            k.font.font_data_id(),
            k.font.face_index(),
            k.font.variation_key(),
            k.font.synthesis_embolden(),
            k.font.synthesis_skew_degrees(),
            k.glyph_id,
            k.size_bits,
            k.x_bin,
            k.y_bin,
            k.kind_label(),
        ))
    }
}
