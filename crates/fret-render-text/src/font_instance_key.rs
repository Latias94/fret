use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontFaceKey {
    pub font_data_id: u64,
    pub face_index: u32,
    pub variation_key: u64,
    pub synthesis_embolden: bool,
    /// Faux italic/oblique skew in degrees, applied at rasterization time.
    pub synthesis_skew_degrees: i8,
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
