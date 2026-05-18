use super::{AtlasSelector, TextAtlasRuntimeState};
use crate::text::DebugGlyphAtlasLookup;

impl TextAtlasRuntimeState {
    pub(in crate::text) fn mask_dimensions(&self) -> (u32, u32) {
        self.dimensions(AtlasSelector::Mask)
    }

    pub(in crate::text) fn color_dimensions(&self) -> (u32, u32) {
        self.dimensions(AtlasSelector::Color)
    }

    pub(in crate::text) fn subpixel_dimensions(&self) -> (u32, u32) {
        self.dimensions(AtlasSelector::Subpixel)
    }

    pub(in crate::text) fn debug_lookup_mask_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.debug_lookup_entry(AtlasSelector::Mask, page, x, y, w, h)
    }

    pub(in crate::text) fn debug_lookup_color_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.debug_lookup_entry(AtlasSelector::Color, page, x, y, w, h)
    }

    pub(in crate::text) fn debug_lookup_subpixel_entry(
        &self,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.debug_lookup_entry(AtlasSelector::Subpixel, page, x, y, w, h)
    }

    fn debug_lookup_entry(
        &self,
        selector: AtlasSelector,
        page: u16,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
    ) -> Option<DebugGlyphAtlasLookup> {
        self.atlas(selector).debug_lookup_entry(page, x, y, w, h)
    }

    fn dimensions(&self, selector: AtlasSelector) -> (u32, u32) {
        self.atlas(selector).debug_dimensions()
    }
}
