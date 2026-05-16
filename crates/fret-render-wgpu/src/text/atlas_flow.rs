use super::TextSystem;
use super::atlas::{GlyphKey, GlyphKeyBuckets};
use super::prepare::{
    build_glyph_scaler_from_face_bytes, glyph_render_at_bins, render_glyph_image,
};
use fret_core::scene::Scene;

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
        self.pin_state
            .clear_for_atlas_reset_generation(self.atlas_runtime.reset_generation());

        let pinned_keys = self.collect_scene_pinned_keys(scene);
        let Some((old_mask, old_color, old_subpixel)) = self.pin_state.bucket(bucket) else {
            return;
        };
        let delta = pinned_keys.retain_delta_from_existing(old_mask, old_color, old_subpixel);
        let (retain_mask, retain_color, retain_subpixel) = delta.retained;
        let (mut add_mask, mut add_color, mut add_subpixel) = delta.added;
        let (remove_mask, remove_color, remove_subpixel) = delta.removed;

        self.atlas_runtime
            .dec_pin_bucket(&remove_mask, &remove_color, &remove_subpixel);
        self.prewarm_pin_bucket(&add_mask, &add_color, &add_subpixel, epoch);

        add_mask.retain(|key| self.atlas_runtime.contains_key(*key));
        add_color.retain(|key| self.atlas_runtime.contains_key(*key));
        add_subpixel.retain(|key| self.atlas_runtime.contains_key(*key));

        self.atlas_runtime
            .inc_pin_bucket(&add_mask, &add_color, &add_subpixel);

        self.pin_state.replace_bucket(
            bucket,
            append_pin_bucket(retain_mask, add_mask),
            append_pin_bucket(retain_color, add_color),
            append_pin_bucket(retain_subpixel, add_subpixel),
        );
    }

    pub(super) fn ensure_glyph_in_atlas(&mut self, key: GlyphKey, epoch: u64) {
        if self.atlas_runtime.touch_if_present(key, epoch) {
            return;
        }

        self.ensure_parley_glyph(key, epoch);
    }

    fn ensure_parley_glyph(&mut self, key: GlyphKey, epoch: u64) {
        let Some(font_data) =
            self.cloned_font_data_for_face(key.font.font_data_id(), key.font.face_index())
        else {
            return;
        };

        let Ok(glyph_id) = u16::try_from(key.glyph_id) else {
            return;
        };

        let font_size = parley_glyph_font_size(key);
        let normalized_coords = self.cloned_face_normalized_coords(key.font);
        let Some(mut scaler) = build_glyph_scaler_from_face_bytes(
            &mut self.parley_scale,
            font_data.bytes(),
            key.font.face_index(),
            font_size,
            normalized_coords.as_deref(),
        ) else {
            return;
        };

        let mut render = glyph_render_at_bins(key.x_bin, key.y_bin);
        apply_parley_glyph_synthesis(&mut render, key, font_size);

        let Some(image) = render_glyph_image(render, &mut scaler, glyph_id) else {
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

    fn collect_scene_pinned_keys(&self, scene: &Scene) -> GlyphKeyBuckets {
        let mut mask_capacity = 0usize;
        let mut color_capacity = 0usize;
        let mut subpixel_capacity = 0usize;

        for &text in scene.text_blob_ids() {
            let Some(blob) = self.blob_state.blobs.get(text) else {
                continue;
            };
            let (mask, color, subpixel) = blob.shape().pin_keys().bucket_lens();
            mask_capacity = mask_capacity.saturating_add(mask);
            color_capacity = color_capacity.saturating_add(color);
            subpixel_capacity = subpixel_capacity.saturating_add(subpixel);
        }

        let mut pinned_keys =
            GlyphKeyBuckets::with_capacities(mask_capacity, color_capacity, subpixel_capacity);
        for &text in scene.text_blob_ids() {
            let Some(blob) = self.blob_state.blobs.get(text) else {
                continue;
            };
            pinned_keys.extend_pin_keys(blob.shape().pin_keys());
        }
        pinned_keys
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
}

fn append_pin_bucket(mut retained: Vec<GlyphKey>, mut added: Vec<GlyphKey>) -> Vec<GlyphKey> {
    retained.append(&mut added);
    retained
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
