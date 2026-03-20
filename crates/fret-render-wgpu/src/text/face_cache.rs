use super::TextSystem;
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

impl TextSystem {
    pub(super) fn cached_font_data_for_face(
        &self,
        font_data_id: u64,
        face_index: u32,
    ) -> Option<&GlyphFontData> {
        self.face_cache
            .font_data_by_face
            .get(&(font_data_id, face_index))
    }

    pub(super) fn cloned_font_data_for_face(
        &self,
        font_data_id: u64,
        face_index: u32,
    ) -> Option<GlyphFontData> {
        self.cached_font_data_for_face(font_data_id, face_index)
            .cloned()
    }

    pub(super) fn cached_face_normalized_coords(&self, face_key: FontFaceKey) -> &[i16] {
        self.face_cache
            .font_instance_coords_by_face
            .get(&face_key)
            .map_or(&[], |coords| coords.as_ref())
    }

    pub(super) fn cloned_face_normalized_coords(
        &self,
        face_key: FontFaceKey,
    ) -> Option<Arc<[i16]>> {
        self.face_cache
            .font_instance_coords_by_face
            .get(&face_key)
            .cloned()
    }

    pub(super) fn cache_face_font_data(&mut self, font_data: &GlyphFontData) {
        self.face_cache
            .font_data_by_face
            .entry((font_data.data_id(), font_data.face_index()))
            .or_insert_with(|| font_data.clone());
    }

    pub(super) fn cache_face_normalized_coords(
        &mut self,
        face_key: FontFaceKey,
        normalized_coords: &Arc<[i16]>,
    ) {
        if normalized_coords.is_empty() {
            return;
        }

        self.face_cache
            .font_instance_coords_by_face
            .entry(face_key)
            .or_insert_with(|| normalized_coords.clone());
    }
}
