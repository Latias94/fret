use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontFaceKey {
    font_data_id: u64,
    face_index: u32,
    variation_key: u64,
    synthesis_embolden: bool,
    /// Faux italic/oblique skew in degrees, applied at rasterization time.
    synthesis_skew_degrees: i8,
}

impl FontFaceKey {
    pub fn new(
        font_data_id: u64,
        face_index: u32,
        variation_key: u64,
        synthesis_embolden: bool,
        synthesis_skew_degrees: i8,
    ) -> Self {
        Self {
            font_data_id,
            face_index,
            variation_key,
            synthesis_embolden,
            synthesis_skew_degrees,
        }
    }

    pub fn font_data_id(&self) -> u64 {
        self.font_data_id
    }

    pub fn face_index(&self) -> u32 {
        self.face_index
    }

    pub fn variation_key(&self) -> u64 {
        self.variation_key
    }

    pub fn synthesis_embolden(&self) -> bool {
        self.synthesis_embolden
    }

    pub fn synthesis_skew_degrees(&self) -> i8 {
        self.synthesis_skew_degrees
    }

    pub fn into_parts(self) -> (u64, u32, u64, bool, i8) {
        (
            self.font_data_id,
            self.face_index,
            self.variation_key,
            self.synthesis_embolden,
            self.synthesis_skew_degrees,
        )
    }
}

pub fn variation_key_from_normalized_coords(coords: &[i16]) -> u64 {
    if coords.is_empty() {
        return 0;
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    "fret.text.font_instance.v0".hash(&mut hasher);
    coords.hash(&mut hasher);
    let key = hasher.finish();
    if key == 0 { 1 } else { key }
}
