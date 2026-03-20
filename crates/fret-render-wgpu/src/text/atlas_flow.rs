use super::TextSystem;
use super::atlas::{GlyphKey, GlyphKeyBuckets, subpixel_bin_as_float};
use fret_core::scene::{Scene, SceneOp};

impl TextSystem {
    pub fn atlas_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.atlas_runtime.atlas_bind_group_layout()
    }

    pub fn mask_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.atlas_runtime.mask_bind_group(page)
    }

    pub fn color_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.atlas_runtime.color_bind_group(page)
    }

    pub fn subpixel_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.atlas_runtime.subpixel_bind_group(page)
    }

    pub fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        self.atlas_runtime.flush_uploads(queue);
    }

    pub fn prepare_for_scene(&mut self, scene: &Scene, frame_index: u64) {
        let ring_len = self.pin_state.ring_len();
        if ring_len == 0 {
            return;
        }
        let bucket = (frame_index as usize) % ring_len;

        let (old_mask, old_color, old_subpixel) = self.pin_state.take_bucket(bucket);
        self.atlas_runtime
            .dec_pin_bucket(&old_mask, &old_color, &old_subpixel);

        let mut pinned_keys = GlyphKeyBuckets::default();

        for op in scene.ops() {
            let SceneOp::Text { text, .. } = *op else {
                continue;
            };
            let Some(blob) = self.blob_state.blobs.get(text) else {
                continue;
            };
            for glyph in blob.shape().glyphs() {
                pinned_keys.insert(glyph.key);
            }
        }

        let epoch = frame_index;
        let (new_mask, new_color, new_subpixel) = pinned_keys.into_pin_bucket();

        for &key in &new_mask {
            self.ensure_glyph_in_atlas(key, epoch);
        }
        for &key in &new_color {
            self.ensure_glyph_in_atlas(key, epoch);
        }
        for &key in &new_subpixel {
            self.ensure_glyph_in_atlas(key, epoch);
        }

        self.atlas_runtime
            .inc_pin_bucket(&new_mask, &new_color, &new_subpixel);

        self.pin_state
            .append_bucket(bucket, new_mask, new_color, new_subpixel);
    }

    pub(super) fn ensure_glyph_in_atlas(&mut self, key: GlyphKey, epoch: u64) {
        if self.atlas_runtime.touch_if_present(key, epoch) {
            return;
        }

        self.ensure_parley_glyph(key, epoch);
    }

    fn ensure_parley_glyph(&mut self, key: GlyphKey, epoch: u64) {
        let Some(font_data) = self
            .face_cache
            .font_data_by_face
            .get(&(key.font.font_data_id(), key.font.face_index()))
        else {
            return;
        };

        let Some(font_ref) =
            parley::swash::FontRef::from_index(font_data.bytes(), key.font.face_index() as usize)
        else {
            return;
        };
        let Ok(glyph_id) = u16::try_from(key.glyph_id) else {
            return;
        };

        let font_size = f32::from_bits(key.size_bits).max(1.0);
        let mut scaler_builder = self
            .parley_scale
            .builder(font_ref)
            .size(font_size)
            .hint(false);
        if let Some(coords) = self.face_cache.font_instance_coords_by_face.get(&key.font) {
            scaler_builder = scaler_builder.normalized_coords(coords.iter());
        }
        let mut scaler = scaler_builder.build();

        let offset_px = parley::swash::zeno::Vector::new(
            subpixel_bin_as_float(key.x_bin),
            subpixel_bin_as_float(key.y_bin),
        );
        let mut render = parley::swash::scale::Render::new(&[
            parley::swash::scale::Source::ColorOutline(0),
            parley::swash::scale::Source::ColorBitmap(parley::swash::scale::StrikeWith::BestFit),
            parley::swash::scale::Source::Outline,
        ]);
        render.offset(offset_px);

        if key.font.synthesis_embolden() {
            let strength = (font_size / 48.0).clamp(0.25, 1.0);
            render.embolden(strength);
        }

        if key.font.synthesis_skew_degrees() != 0 {
            let angle =
                parley::swash::zeno::Angle::from_degrees(key.font.synthesis_skew_degrees() as f32);
            let t = parley::swash::zeno::Transform::skew(angle, parley::swash::zeno::Angle::ZERO);
            render.transform(Some(t));
        }

        let Some(image) = render.render(&mut scaler, glyph_id) else {
            return;
        };
        if image.placement.width == 0 || image.placement.height == 0 {
            return;
        }

        let Some(bytes_per_pixel) = key.bytes_per_pixel_for_image_content(image.content) else {
            return;
        };

        self.atlas_runtime.cache_glyph(
            key,
            image.placement.width,
            image.placement.height,
            image.placement.left,
            image.placement.top,
            bytes_per_pixel,
            image.data,
            epoch,
        );
    }
}
