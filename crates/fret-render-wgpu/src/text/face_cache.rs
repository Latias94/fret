use fret_render_text::{FontFaceKey, GlyphFontData};
use std::{collections::HashMap, sync::Arc};

#[derive(Default)]
pub(crate) struct TextFaceCacheState {
    pub(crate) font_data_by_face: HashMap<(u64, u32), GlyphFontData>,
    pub(crate) font_instance_coords_by_face: HashMap<FontFaceKey, Arc<[i16]>>,
    pub(crate) font_face_family_name_cache: HashMap<(u64, u32), String>,
}

impl TextFaceCacheState {
    pub(crate) fn clear(&mut self) {
        self.font_data_by_face.clear();
        self.font_instance_coords_by_face.clear();
        self.font_face_family_name_cache.clear();
    }
}
