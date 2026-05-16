use super::atlas::GlyphKey;

pub(crate) struct TextPinState {
    mask: Vec<Vec<GlyphKey>>,
    color: Vec<Vec<GlyphKey>>,
    subpixel: Vec<Vec<GlyphKey>>,
    atlas_reset_generation: u64,
}

impl TextPinState {
    pub(crate) fn with_ring_len(ring_len: usize) -> Self {
        Self {
            mask: vec![Vec::new(); ring_len],
            color: vec![Vec::new(); ring_len],
            subpixel: vec![Vec::new(); ring_len],
            atlas_reset_generation: 0,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.mask.iter_mut().for_each(|bucket| bucket.clear());
        self.color.iter_mut().for_each(|bucket| bucket.clear());
        self.subpixel.iter_mut().for_each(|bucket| bucket.clear());
    }

    pub(crate) fn clear_for_atlas_reset_generation(&mut self, generation: u64) {
        if self.atlas_reset_generation == generation {
            return;
        }
        self.clear();
        self.atlas_reset_generation = generation;
    }

    pub(crate) fn ring_len(&self) -> usize {
        self.mask
            .len()
            .min(self.color.len())
            .min(self.subpixel.len())
    }

    pub(crate) fn bucket(&self, bucket: usize) -> Option<(&[GlyphKey], &[GlyphKey], &[GlyphKey])> {
        if bucket >= self.ring_len() {
            return None;
        }
        Some((
            &self.mask[bucket],
            &self.color[bucket],
            &self.subpixel[bucket],
        ))
    }

    pub(crate) fn replace_bucket(
        &mut self,
        bucket: usize,
        mask: Vec<GlyphKey>,
        color: Vec<GlyphKey>,
        subpixel: Vec<GlyphKey>,
    ) {
        self.mask[bucket] = mask;
        self.color[bucket] = color;
        self.subpixel[bucket] = subpixel;
    }
}
