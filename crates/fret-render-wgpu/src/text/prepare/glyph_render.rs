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
            glyph.id, image, face_key, size_bits, x_bin, y_bin,
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
        let scaler_builder = prepared_glyph_scaler_builder_with_normalized_coords(
            &mut self.parley_scale,
            glyph,
            font_ref,
        );
        scaler_builder.build()
    }
}

fn prepared_glyph_font_ref<'a>(glyph: &'a ParleyGlyph) -> Option<parley::swash::FontRef<'a>> {
    parley::swash::FontRef::from_index(glyph.font.data.data(), glyph.font.index as usize)
}

fn prepared_glyph_scaler_size(glyph: &ParleyGlyph) -> f32 {
    glyph.font_size.max(1.0)
}

pub(super) fn prepared_glyph_has_normalized_coords(glyph: &ParleyGlyph) -> bool {
    !glyph.normalized_coords.is_empty()
}

fn prepared_glyph_scaler_builder<'a>(
    parley_scale: &'a mut parley::swash::scale::ScaleContext,
    glyph: &'a ParleyGlyph,
    font_ref: parley::swash::FontRef<'a>,
) -> parley::swash::scale::ScalerBuilder<'a> {
    parley_scale
        .builder(font_ref)
        .size(prepared_glyph_scaler_size(glyph))
        .hint(false)
}

fn prepared_glyph_scaler_builder_with_normalized_coords<'a>(
    parley_scale: &'a mut parley::swash::scale::ScaleContext,
    glyph: &'a ParleyGlyph,
    font_ref: parley::swash::FontRef<'a>,
) -> parley::swash::scale::ScalerBuilder<'a> {
    let scaler_builder = prepared_glyph_scaler_builder(parley_scale, glyph, font_ref);
    apply_prepared_glyph_normalized_coords(scaler_builder, glyph)
}

fn render_prepared_glyph_image_at_bins(
    scaler: &mut parley::swash::scale::Scaler<'_>,
    glyph_id: u16,
    x_bin: u8,
    y_bin: u8,
) -> Option<parley::swash::scale::image::Image> {
    let offset_px = super::prepared_glyph_offset_px(x_bin, y_bin);
    render_prepared_glyph_image_with_scaler(scaler, glyph_id, offset_px)
}

fn render_prepared_glyph_image_from_scaler(
    scaler: &mut parley::swash::scale::Scaler<'_>,
    glyph_id: u16,
    x_bin: u8,
    y_bin: u8,
) -> Option<parley::swash::scale::image::Image> {
    render_prepared_glyph_image_at_bins(scaler, glyph_id, x_bin, y_bin)
}

fn apply_prepared_glyph_normalized_coords<'a>(
    scaler_builder: parley::swash::scale::ScalerBuilder<'a>,
    glyph: &'a ParleyGlyph,
) -> parley::swash::scale::ScalerBuilder<'a> {
    if !prepared_glyph_has_normalized_coords(glyph) {
        return scaler_builder;
    }
    apply_prepared_glyph_normalized_coords_values(scaler_builder, glyph)
}

fn apply_prepared_glyph_normalized_coords_values<'a>(
    scaler_builder: parley::swash::scale::ScalerBuilder<'a>,
    glyph: &'a ParleyGlyph,
) -> parley::swash::scale::ScalerBuilder<'a> {
    scaler_builder.normalized_coords(glyph.normalized_coords.iter())
}

fn render_prepared_glyph_image_with_scaler(
    scaler: &mut parley::swash::scale::Scaler<'_>,
    glyph_id: u16,
    offset_px: parley::swash::zeno::Vector,
) -> Option<parley::swash::scale::image::Image> {
    parley::swash::scale::Render::new(&prepared_glyph_render_sources())
        .offset(offset_px)
        .render(scaler, glyph_id)
}

fn prepared_glyph_render_sources() -> [parley::swash::scale::Source; 3] {
    [
        parley::swash::scale::Source::ColorOutline(0),
        parley::swash::scale::Source::ColorBitmap(parley::swash::scale::StrikeWith::BestFit),
        parley::swash::scale::Source::Outline,
    ]
}
