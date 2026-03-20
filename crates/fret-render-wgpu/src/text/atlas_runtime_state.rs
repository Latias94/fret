use super::atlas::{GlyphAtlas, GlyphAtlasEntry, GlyphAtlasInsertError, GlyphKey};

pub(crate) struct TextAtlasRuntimeState {
    pub(crate) mask_atlas: GlyphAtlas,
    pub(crate) color_atlas: GlyphAtlas,
    pub(crate) subpixel_atlas: GlyphAtlas,
    pub(crate) atlas_bind_group_layout: wgpu::BindGroupLayout,
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

    fn atlas_for_key(&self, key: GlyphKey) -> &GlyphAtlas {
        match key.kind {
            super::GlyphQuadKind::Mask => &self.mask_atlas,
            super::GlyphQuadKind::Color => &self.color_atlas,
            super::GlyphQuadKind::Subpixel => &self.subpixel_atlas,
        }
    }

    pub(super) fn atlas_mut_for_key(&mut self, key: GlyphKey) -> &mut GlyphAtlas {
        match key.kind {
            super::GlyphQuadKind::Mask => &mut self.mask_atlas,
            super::GlyphQuadKind::Color => &mut self.color_atlas,
            super::GlyphQuadKind::Subpixel => &mut self.subpixel_atlas,
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
}
