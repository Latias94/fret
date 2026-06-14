use super::atlas::GlyphKey;
use super::pin_state::ScenePinBucketSignature;
use super::prepare::{
    build_glyph_scaler_from_face_bytes, glyph_render_at_bins, render_glyph_image,
};
use super::{TextPrepareScenePerf, TextSystem};
use fret_core::scene::Scene;
use std::time::Instant;

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

    #[cfg(test)]
    pub fn prepare_for_scene(&mut self, scene: &Scene, frame_index: u64) {
        let _ = self.prepare_for_scene_with_perf(scene, frame_index, false);
    }

    pub(crate) fn prepare_for_scene_with_perf(
        &mut self,
        scene: &Scene,
        frame_index: u64,
        perf_enabled: bool,
    ) -> TextPrepareScenePerf {
        let mut perf = TextPrepareScenePerf::default();
        let ring_len = self.pin_state.ring_len();
        if ring_len == 0 {
            return perf;
        }
        let bucket = (frame_index as usize) % ring_len;
        let epoch = frame_index;
        self.pin_state
            .clear_for_atlas_reset_generation(self.atlas_runtime.reset_generation());

        let collect_start = perf_enabled.then(Instant::now);
        if let Some(reuse) = self
            .pin_state
            .try_reuse_scene_bucket(bucket, scene, &self.blob_state)
        {
            perf.fast_scene_bucket_reused = true;
            perf.scene_text_blobs = usize_to_u64(reuse.scene_text_blobs);
            perf.pinned_glyph_keys = usize_to_u64(reuse.pinned_glyph_keys);
            perf.retained_glyph_keys = usize_to_u64(reuse.pinned_glyph_keys);
            if let Some(start) = collect_start {
                perf.collect_pin_keys += start.elapsed();
            }
            return perf;
        }

        let collection = self
            .pin_state
            .collect_scene_pinned_keys(scene, &self.blob_state);
        let pinned_keys = collection.buckets;
        perf.scene_text_blobs = usize_to_u64(collection.scene_text_blobs);
        perf.pinned_glyph_keys = usize_to_u64(pinned_keys.total_len());
        if let Some(start) = collect_start {
            perf.collect_pin_keys += start.elapsed();
        }

        let Some((old_mask, old_color, old_subpixel)) = self.pin_state.bucket(bucket) else {
            return perf;
        };

        let delta_start = perf_enabled.then(Instant::now);
        let delta = pinned_keys.retain_delta_from_existing(old_mask, old_color, old_subpixel);
        if let Some(start) = delta_start {
            perf.bucket_delta += start.elapsed();
        }
        let (retain_mask, retain_color, retain_subpixel) = delta.retained;
        let (mut add_mask, mut add_color, mut add_subpixel) = delta.added;
        let (remove_mask, remove_color, remove_subpixel) = delta.removed;
        perf.retained_glyph_keys =
            glyph_bucket_len_u64(&retain_mask, &retain_color, &retain_subpixel);
        perf.prewarm_glyph_keys = glyph_bucket_len_u64(&add_mask, &add_color, &add_subpixel);
        perf.removed_glyph_keys =
            glyph_bucket_len_u64(&remove_mask, &remove_color, &remove_subpixel);

        let pin_update_start = perf_enabled.then(Instant::now);
        self.atlas_runtime
            .dec_pin_bucket(&remove_mask, &remove_color, &remove_subpixel);
        if let Some(start) = pin_update_start {
            perf.pin_bucket_update += start.elapsed();
        }

        let prewarm_start = perf_enabled.then(Instant::now);
        self.prewarm_pin_bucket(&add_mask, &add_color, &add_subpixel, epoch);
        if let Some(start) = prewarm_start {
            perf.prewarm += start.elapsed();
        }

        add_mask.retain(|key| self.atlas_runtime.contains_key(*key));
        add_color.retain(|key| self.atlas_runtime.contains_key(*key));
        add_subpixel.retain(|key| self.atlas_runtime.contains_key(*key));
        perf.added_glyph_keys = glyph_bucket_len_u64(&add_mask, &add_color, &add_subpixel);
        let bucket_complete = perf.added_glyph_keys == perf.prewarm_glyph_keys;
        let next_signature = collection.signature.and_then(|signature| {
            bucket_complete
                .then(|| ScenePinBucketSignature::new(signature, perf.pinned_glyph_keys as usize))
        });

        let pin_update_start = perf_enabled.then(Instant::now);
        self.atlas_runtime
            .inc_pin_bucket(&add_mask, &add_color, &add_subpixel);

        self.pin_state.replace_bucket(
            bucket,
            append_pin_bucket(retain_mask, add_mask),
            append_pin_bucket(retain_color, add_color),
            append_pin_bucket(retain_subpixel, add_subpixel),
            next_signature,
        );
        if let Some(start) = pin_update_start {
            perf.pin_bucket_update += start.elapsed();
        }
        perf
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

fn glyph_bucket_len_u64(mask: &[GlyphKey], color: &[GlyphKey], subpixel: &[GlyphKey]) -> u64 {
    usize_to_u64(
        mask.len()
            .saturating_add(color.len())
            .saturating_add(subpixel.len()),
    )
}

fn usize_to_u64(value: usize) -> u64 {
    value.try_into().unwrap_or(u64::MAX)
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
