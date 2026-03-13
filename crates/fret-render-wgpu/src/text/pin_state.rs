use super::atlas::GlyphKey;

pub(crate) struct TextPinState {
    mask: Vec<Vec<GlyphKey>>,
    color: Vec<Vec<GlyphKey>>,
    subpixel: Vec<Vec<GlyphKey>>,
}

impl TextPinState {
    pub(crate) fn with_ring_len(ring_len: usize) -> Self {
        Self {
            mask: vec![Vec::new(); ring_len],
            color: vec![Vec::new(); ring_len],
            subpixel: vec![Vec::new(); ring_len],
        }
    }

    pub(crate) fn clear(&mut self) {
        self.mask.iter_mut().for_each(|bucket| bucket.clear());
        self.color.iter_mut().for_each(|bucket| bucket.clear());
        self.subpixel.iter_mut().for_each(|bucket| bucket.clear());
    }

    pub(crate) fn ring_len(&self) -> usize {
        self.mask
            .len()
            .min(self.color.len())
            .min(self.subpixel.len())
    }

    pub(crate) fn take_bucket(
        &mut self,
        bucket: usize,
    ) -> (Vec<GlyphKey>, Vec<GlyphKey>, Vec<GlyphKey>) {
        (
            std::mem::take(&mut self.mask[bucket]),
            std::mem::take(&mut self.color[bucket]),
            std::mem::take(&mut self.subpixel[bucket]),
        )
    }

    pub(crate) fn append_bucket(
        &mut self,
        bucket: usize,
        mut mask: Vec<GlyphKey>,
        mut color: Vec<GlyphKey>,
        mut subpixel: Vec<GlyphKey>,
    ) {
        self.mask[bucket].append(&mut mask);
        self.color[bucket].append(&mut color);
        self.subpixel[bucket].append(&mut subpixel);
    }
}
