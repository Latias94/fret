use super::GlyphQuadKind;
use super::atlas::GlyphAtlas;

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

    pub(crate) fn atlas(&self, kind: GlyphQuadKind) -> &GlyphAtlas {
        match kind {
            GlyphQuadKind::Mask => &self.mask_atlas,
            GlyphQuadKind::Color => &self.color_atlas,
            GlyphQuadKind::Subpixel => &self.subpixel_atlas,
        }
    }

    pub(crate) fn atlas_mut(&mut self, kind: GlyphQuadKind) -> &mut GlyphAtlas {
        match kind {
            GlyphQuadKind::Mask => &mut self.mask_atlas,
            GlyphQuadKind::Color => &mut self.color_atlas,
            GlyphQuadKind::Subpixel => &mut self.subpixel_atlas,
        }
    }
}
