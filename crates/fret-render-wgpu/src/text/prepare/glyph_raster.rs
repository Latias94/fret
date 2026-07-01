use super::super::atlas::GlyphKey;
use fret_render_text::FontFaceKey;

pub(super) struct PreparedGlyphRaster {
    glyph_key: GlyphKey,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
}

struct PreparedGlyphRasterPlacement {
    width: u32,
    height: u32,
    left: i32,
    top: i32,
}

struct PreparedGlyphRasterMetadata {
    glyph_key: GlyphKey,
}

impl PreparedGlyphRaster {
    pub(super) fn bounds(&self, x: i32, y: i32) -> (GlyphKey, f32, f32, f32, f32) {
        (
            self.glyph_key,
            x as f32 + self.left as f32,
            y as f32 - self.top as f32,
            self.width as f32,
            self.height as f32,
        )
    }
}

pub(super) fn prepared_glyph_raster_from_image(
    image: parley::swash::scale::image::Image,
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
) -> Option<PreparedGlyphRaster> {
    let placement = prepared_glyph_raster_placement(&image)?;
    Some(prepared_glyph_raster_from_image_with_placement(
        face_key, glyph_id, size_bits, x_bin, y_bin, image, placement,
    ))
}

fn prepared_glyph_raster_from_image_with_placement(
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    image: parley::swash::scale::image::Image,
    placement: PreparedGlyphRasterPlacement,
) -> PreparedGlyphRaster {
    prepared_glyph_raster_from_image_parts(
        face_key, glyph_id, size_bits, x_bin, y_bin, image, placement,
    )
}

fn prepared_glyph_raster_placement(
    image: &parley::swash::scale::image::Image,
) -> Option<PreparedGlyphRasterPlacement> {
    let placement = image.placement;
    if placement.width == 0 || placement.height == 0 {
        return None;
    }
    Some(PreparedGlyphRasterPlacement {
        width: placement.width,
        height: placement.height,
        left: placement.left,
        top: placement.top,
    })
}

fn prepared_glyph_raster_metadata(
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    content: parley::swash::scale::image::Content,
) -> PreparedGlyphRasterMetadata {
    let (glyph_key, _bytes_per_pixel) =
        GlyphKey::from_image_content(face_key, glyph_id, size_bits, x_bin, y_bin, content);
    PreparedGlyphRasterMetadata { glyph_key }
}

fn prepared_glyph_raster_from_image_parts(
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    image: parley::swash::scale::image::Image,
    placement: PreparedGlyphRasterPlacement,
) -> PreparedGlyphRaster {
    let metadata =
        prepared_glyph_raster_metadata(face_key, glyph_id, size_bits, x_bin, y_bin, image.content);
    prepared_glyph_raster_from_image_parts_with_metadata(placement, metadata)
}

fn prepared_glyph_raster_from_image_parts_with_metadata(
    placement: PreparedGlyphRasterPlacement,
    metadata: PreparedGlyphRasterMetadata,
) -> PreparedGlyphRaster {
    let PreparedGlyphRasterPlacement {
        width,
        height,
        left,
        top,
    } = placement;
    let PreparedGlyphRasterMetadata { glyph_key } = metadata;
    PreparedGlyphRaster {
        glyph_key,
        width,
        height,
        left,
        top,
    }
}
