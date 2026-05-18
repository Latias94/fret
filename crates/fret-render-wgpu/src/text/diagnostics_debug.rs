use super::{DebugGlyphAtlasLookup, TextSystem};

impl TextSystem {
    pub(crate) fn debug_mask_atlas_dims(&self) -> (u32, u32) {
        self.atlas_runtime.mask_dimensions()
    }

    pub(crate) fn debug_color_atlas_dims(&self) -> (u32, u32) {
        self.atlas_runtime.color_dimensions()
    }

    pub(crate) fn debug_subpixel_atlas_dims(&self) -> (u32, u32) {
        self.atlas_runtime.subpixel_dimensions()
    }

    pub(crate) fn debug_lookup_mask_glyph_atlas_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.atlas_runtime.debug_lookup_mask_entry(page, x, y, w, h)
    }

    pub(crate) fn debug_lookup_color_glyph_atlas_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.atlas_runtime
            .debug_lookup_color_entry(page, x, y, w, h)
    }

    pub(crate) fn debug_lookup_subpixel_glyph_atlas_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.atlas_runtime
            .debug_lookup_subpixel_entry(page, x, y, w, h)
    }
}
