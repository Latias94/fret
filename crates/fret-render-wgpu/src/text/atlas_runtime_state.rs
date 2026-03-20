use super::atlas::{GlyphAtlas, GlyphKey};

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

    pub(super) fn atlas_for_key(&self, key: GlyphKey) -> &GlyphAtlas {
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
}
