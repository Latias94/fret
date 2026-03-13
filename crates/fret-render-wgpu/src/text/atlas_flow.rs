use super::atlas::{GlyphKey, subpixel_bin_as_float};
use super::{GlyphQuadKind, TextSystem};
use fret_core::scene::{Scene, SceneOp};
use std::collections::HashSet;

impl TextSystem {
    pub fn atlas_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.atlas_runtime.atlas_bind_group_layout
    }

    pub fn mask_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.atlas_runtime.mask_atlas.bind_group(page)
    }

    pub fn color_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.atlas_runtime.color_atlas.bind_group(page)
    }

    pub fn subpixel_atlas_bind_group(&self, page: u16) -> &wgpu::BindGroup {
        self.atlas_runtime.subpixel_atlas.bind_group(page)
    }

    pub fn flush_uploads(&mut self, queue: &wgpu::Queue) {
        self.atlas_runtime.mask_atlas.flush_uploads(queue);
        self.atlas_runtime.color_atlas.flush_uploads(queue);
        self.atlas_runtime.subpixel_atlas.flush_uploads(queue);
    }

    pub fn prepare_for_scene(&mut self, scene: &Scene, frame_index: u64) {
        let ring_len = self.pin_state.ring_len();
        if ring_len == 0 {
            return;
        }
        let bucket = (frame_index as usize) % ring_len;

        let (old_mask, old_color, old_subpixel) = self.pin_state.take_bucket(bucket);
        self.atlas_runtime.mask_atlas.dec_live_refs(&old_mask);
        self.atlas_runtime.color_atlas.dec_live_refs(&old_color);
        self.atlas_runtime
            .subpixel_atlas
            .dec_live_refs(&old_subpixel);

        let mut mask_keys: HashSet<GlyphKey> = HashSet::new();
        let mut color_keys: HashSet<GlyphKey> = HashSet::new();
        let mut subpixel_keys: HashSet<GlyphKey> = HashSet::new();

        for op in scene.ops() {
            let SceneOp::Text { text, .. } = *op else {
                continue;
            };
            let Some(blob) = self.blob_state.blobs.get(text) else {
                continue;
            };
            for glyph in blob.shape.glyphs.as_ref() {
                match glyph.kind() {
                    GlyphQuadKind::Mask => {
                        mask_keys.insert(glyph.key);
                    }
                    GlyphQuadKind::Color => {
                        color_keys.insert(glyph.key);
                    }
                    GlyphQuadKind::Subpixel => {
                        subpixel_keys.insert(glyph.key);
                    }
                }
            }
        }

        let epoch = frame_index;
        let new_mask: Vec<GlyphKey> = mask_keys.into_iter().collect();
        let new_color: Vec<GlyphKey> = color_keys.into_iter().collect();
        let new_subpixel: Vec<GlyphKey> = subpixel_keys.into_iter().collect();

        for &key in &new_mask {
            self.ensure_glyph_in_atlas(key, epoch);
        }
        for &key in &new_color {
            self.ensure_glyph_in_atlas(key, epoch);
        }
        for &key in &new_subpixel {
            self.ensure_glyph_in_atlas(key, epoch);
        }

        self.atlas_runtime.mask_atlas.inc_live_refs(&new_mask);
        self.atlas_runtime.color_atlas.inc_live_refs(&new_color);
        self.atlas_runtime
            .subpixel_atlas
            .inc_live_refs(&new_subpixel);

        self.pin_state
            .append_bucket(bucket, new_mask, new_color, new_subpixel);
    }

    pub(super) fn ensure_glyph_in_atlas(&mut self, key: GlyphKey, epoch: u64) {
        let already_present = match key.kind {
            GlyphQuadKind::Mask => self.atlas_runtime.mask_atlas.get(key, epoch).is_some(),
            GlyphQuadKind::Color => self.atlas_runtime.color_atlas.get(key, epoch).is_some(),
            GlyphQuadKind::Subpixel => self.atlas_runtime.subpixel_atlas.get(key, epoch).is_some(),
        };
        if already_present {
            return;
        }

        self.ensure_parley_glyph(key, epoch);
    }

    fn ensure_parley_glyph(&mut self, key: GlyphKey, epoch: u64) {
        let Some(font_data) = self
            .face_cache
            .font_data_by_face
            .get(&(key.font.font_data_id, key.font.face_index))
        else {
            return;
        };

        let Some(font_ref) =
            parley::swash::FontRef::from_index(font_data.data.data(), key.font.face_index as usize)
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

        if key.font.synthesis_embolden {
            let strength = (font_size / 48.0).clamp(0.25, 1.0);
            render.embolden(strength);
        }

        if key.font.synthesis_skew_degrees != 0 {
            let angle =
                parley::swash::zeno::Angle::from_degrees(key.font.synthesis_skew_degrees as f32);
            let t = parley::swash::zeno::Transform::skew(angle, parley::swash::zeno::Angle::ZERO);
            render.transform(Some(t));
        }

        let Some(image) = render.render(&mut scaler, glyph_id) else {
            return;
        };
        if image.placement.width == 0 || image.placement.height == 0 {
            return;
        }

        let (image_kind, bytes_per_pixel) = match image.content {
            parley::swash::scale::image::Content::Mask => (GlyphQuadKind::Mask, 1),
            parley::swash::scale::image::Content::Color => (GlyphQuadKind::Color, 4),
            parley::swash::scale::image::Content::SubpixelMask => (GlyphQuadKind::Subpixel, 4),
        };
        if image_kind != key.kind {
            return;
        }

        let data = image.data;

        match key.kind {
            GlyphQuadKind::Mask => {
                let _ = self.atlas_runtime.mask_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    image.placement.left,
                    image.placement.top,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
            GlyphQuadKind::Color => {
                let _ = self.atlas_runtime.color_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    image.placement.left,
                    image.placement.top,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
            GlyphQuadKind::Subpixel => {
                let _ = self.atlas_runtime.subpixel_atlas.get_or_insert(
                    key,
                    image.placement.width,
                    image.placement.height,
                    image.placement.left,
                    image.placement.top,
                    bytes_per_pixel,
                    data,
                    epoch,
                );
            }
        }
    }
}
