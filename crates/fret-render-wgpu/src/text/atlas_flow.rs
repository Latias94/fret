use super::TextSystem;
use super::atlas::{GlyphKey, GlyphKeyBuckets};
use super::prepare::{build_glyph_scaler, glyph_render_at_bins};
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
        let epoch = frame_index;

        self.release_pin_bucket(bucket);
        let pinned_keys = self.collect_scene_pinned_keys(scene);
        let (new_mask, new_color, new_subpixel) = pinned_keys.into_pin_bucket();
        self.prewarm_pin_bucket(&new_mask, &new_color, &new_subpixel, epoch);
        self.activate_pin_bucket(bucket, new_mask, new_color, new_subpixel);
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

        let font_size = parley_glyph_font_size(key);
        let normalized_coords = self.cached_face_normalized_coords(key);
        let mut scaler = build_glyph_scaler(
            &mut self.parley_scale,
            font_ref,
            font_size,
            normalized_coords.as_deref(),
        );

        let mut render = glyph_render_at_bins(key.x_bin, key.y_bin);
        apply_parley_glyph_synthesis(&mut render, key, font_size);

        let Some(image) = render.render(&mut scaler, glyph_id) else {
            return;
        };
        self.cache_rendered_parley_glyph(key, image, epoch);
    }

    fn cache_rendered_parley_glyph(
        &mut self,
        key: GlyphKey,
        image: parley::swash::scale::image::Image,
        epoch: u64,
    ) {
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

    fn cached_face_normalized_coords(&self, key: GlyphKey) -> Option<std::sync::Arc<[i16]>> {
        self.face_cache
            .font_instance_coords_by_face
            .get(&key.font)
            .cloned()
    }

    fn release_pin_bucket(&mut self, bucket: usize) {
        let (old_mask, old_color, old_subpixel) = self.pin_state.take_bucket(bucket);
        self.atlas_runtime
            .dec_pin_bucket(&old_mask, &old_color, &old_subpixel);
    }

    fn collect_scene_pinned_keys(&self, scene: &Scene) -> GlyphKeyBuckets {
        let mut pinned_keys = GlyphKeyBuckets::default();
        for op in scene.ops() {
            let SceneOp::Text { text, .. } = *op else {
                continue;
            };
            let Some(blob) = self.blob_state.blobs.get(text) else {
                continue;
            };
            self.collect_blob_pinned_keys(blob.shape().glyphs(), &mut pinned_keys);
        }
        pinned_keys
    }

    fn collect_blob_pinned_keys(
        &self,
        glyphs: &[super::GlyphInstance],
        pinned_keys: &mut GlyphKeyBuckets,
    ) {
        for glyph in glyphs {
            pinned_keys.insert(glyph.key);
        }
    }

    fn prewarm_pin_bucket(
        &mut self,
        mask: &[GlyphKey],
        color: &[GlyphKey],
        subpixel: &[GlyphKey],
        epoch: u64,
    ) {
        self.ensure_glyphs_in_atlas(mask, epoch);
        self.ensure_glyphs_in_atlas(color, epoch);
        self.ensure_glyphs_in_atlas(subpixel, epoch);
    }

    fn ensure_glyphs_in_atlas(&mut self, keys: &[GlyphKey], epoch: u64) {
        for &key in keys {
            self.ensure_glyph_in_atlas(key, epoch);
        }
    }

    fn activate_pin_bucket(
        &mut self,
        bucket: usize,
        mask: Vec<GlyphKey>,
        color: Vec<GlyphKey>,
        subpixel: Vec<GlyphKey>,
    ) {
        self.atlas_runtime.inc_pin_bucket(&mask, &color, &subpixel);
        self.pin_state.append_bucket(bucket, mask, color, subpixel);
    }
}

fn parley_glyph_font_size(key: GlyphKey) -> f32 {
    f32::from_bits(key.size_bits).max(1.0)
}

fn apply_parley_glyph_synthesis(
    render: &mut parley::swash::scale::Render,
    key: GlyphKey,
    font_size: f32,
) {
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
}
