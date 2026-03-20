use super::super::TextSystem;
use super::glyph_raster::{PreparedGlyphRaster, prepared_glyph_raster_from_image};
use fret_render_text::{FontFaceKey, ParleyGlyph};

impl TextSystem {
    pub(super) fn render_prepared_glyph_raster(
        &mut self,
        glyph: &ParleyGlyph,
        glyph_id: u16,
        face_key: FontFaceKey,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
    ) -> Option<PreparedGlyphRaster> {
        let image = self.render_prepared_glyph_image(glyph, glyph_id, x_bin, y_bin)?;
        self.render_prepared_glyph_raster_from_image(
            glyph.id(),
            image,
            face_key,
            size_bits,
            x_bin,
            y_bin,
        )
    }

    fn render_prepared_glyph_raster_from_image(
        &mut self,
        glyph_id: u32,
        image: parley::swash::scale::image::Image,
        face_key: FontFaceKey,
        size_bits: u32,
        x_bin: u8,
        y_bin: u8,
    ) -> Option<PreparedGlyphRaster> {
        prepared_glyph_raster_from_image(image, face_key, glyph_id, size_bits, x_bin, y_bin)
    }

    fn render_prepared_glyph_image(
        &mut self,
        glyph: &ParleyGlyph,
        glyph_id: u16,
        x_bin: u8,
        y_bin: u8,
    ) -> Option<parley::swash::scale::image::Image> {
        let font_ref = prepared_glyph_font_ref(glyph)?;
        self.render_prepared_glyph_image_with_font_ref(glyph, font_ref, glyph_id, x_bin, y_bin)
    }

    fn render_prepared_glyph_image_with_font_ref(
        &mut self,
        glyph: &ParleyGlyph,
        font_ref: parley::swash::FontRef<'_>,
        glyph_id: u16,
        x_bin: u8,
        y_bin: u8,
    ) -> Option<parley::swash::scale::image::Image> {
        let mut scaler = self.build_prepared_glyph_scaler(glyph, font_ref);
        render_prepared_glyph_image_from_scaler(&mut scaler, glyph_id, x_bin, y_bin)
    }

    fn build_prepared_glyph_scaler<'a>(
        &'a mut self,
        glyph: &'a ParleyGlyph,
        font_ref: parley::swash::FontRef<'a>,
    ) -> parley::swash::scale::Scaler<'a> {
        super::build_glyph_scaler(
            &mut self.parley_scale,
            font_ref,
            glyph.font_size(),
            prepared_glyph_normalized_coords(glyph),
        )
    }
}

fn prepared_glyph_font_ref<'a>(glyph: &'a ParleyGlyph) -> Option<parley::swash::FontRef<'a>> {
    parley::swash::FontRef::from_index(glyph.font().bytes(), glyph.font().face_index() as usize)
}

pub(super) fn prepared_glyph_has_normalized_coords(glyph: &ParleyGlyph) -> bool {
    !glyph.normalized_coords().is_empty()
}

fn prepared_glyph_normalized_coords(glyph: &ParleyGlyph) -> Option<&[i16]> {
    prepared_glyph_has_normalized_coords(glyph).then_some(glyph.normalized_coords())
}

fn render_prepared_glyph_image_at_bins(
    scaler: &mut parley::swash::scale::Scaler<'_>,
    glyph_id: u16,
    x_bin: u8,
    y_bin: u8,
) -> Option<parley::swash::scale::image::Image> {
    super::glyph_render_at_bins(x_bin, y_bin).render(scaler, glyph_id)
}

fn render_prepared_glyph_image_from_scaler(
    scaler: &mut parley::swash::scale::Scaler<'_>,
    glyph_id: u16,
    x_bin: u8,
    y_bin: u8,
) -> Option<parley::swash::scale::image::Image> {
    render_prepared_glyph_image_at_bins(scaler, glyph_id, x_bin, y_bin)
}
