use super::super::atlas::GlyphKey;
use fret_render_text::FontFaceKey;

pub(super) struct PreparedGlyphRaster {
    glyph_key: GlyphKey,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    bytes_per_pixel: u32,
    data: Vec<u8>,
}

struct PreparedGlyphRasterPayload {
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    bytes_per_pixel: u32,
    data: Vec<u8>,
}

struct PreparedGlyphRasterPlacement {
    width: u32,
    height: u32,
    left: i32,
    top: i32,
}

struct PreparedGlyphRasterMetadata {
    glyph_key: GlyphKey,
    bytes_per_pixel: u32,
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

    pub(super) fn into_atlas_insert(self) -> (GlyphKey, u32, u32, i32, i32, u32, Vec<u8>) {
        (
            self.glyph_key,
            self.width,
            self.height,
            self.left,
            self.top,
            self.bytes_per_pixel,
            self.data,
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

pub(super) fn prepared_glyph_lookup_keys(
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
) -> [GlyphKey; 3] {
    GlyphKey::lookup_keys(face_key, glyph_id, size_bits, x_bin, y_bin)
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
    let (glyph_key, bytes_per_pixel) =
        GlyphKey::from_image_content(face_key, glyph_id, size_bits, x_bin, y_bin, content);
    PreparedGlyphRasterMetadata {
        glyph_key,
        bytes_per_pixel,
    }
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
    prepared_glyph_raster_from_image_parts_with_metadata(image, placement, metadata)
}

fn prepared_glyph_raster_from_image_parts_with_metadata(
    image: parley::swash::scale::image::Image,
    placement: PreparedGlyphRasterPlacement,
    metadata: PreparedGlyphRasterMetadata,
) -> PreparedGlyphRaster {
    let PreparedGlyphRasterPlacement {
        width,
        height,
        left,
        top,
    } = placement;
    let PreparedGlyphRasterMetadata {
        glyph_key,
        bytes_per_pixel,
    } = metadata;
    prepared_glyph_raster_from_image_parts_with_payload(
        glyph_key,
        width,
        height,
        left,
        top,
        bytes_per_pixel,
        image.data,
    )
}

fn prepared_glyph_raster_from_image_parts_with_payload(
    glyph_key: GlyphKey,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    bytes_per_pixel: u32,
    data: Vec<u8>,
) -> PreparedGlyphRaster {
    prepared_glyph_raster(glyph_key, width, height, left, top, bytes_per_pixel, data)
}

fn prepared_glyph_raster(
    glyph_key: GlyphKey,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    bytes_per_pixel: u32,
    data: Vec<u8>,
) -> PreparedGlyphRaster {
    let payload = prepared_glyph_raster_payload(width, height, left, top, bytes_per_pixel, data);
    prepared_glyph_raster_with_key(glyph_key, payload)
}

fn prepared_glyph_raster_with_key(
    glyph_key: GlyphKey,
    payload: PreparedGlyphRasterPayload,
) -> PreparedGlyphRaster {
    let PreparedGlyphRasterPayload {
        width,
        height,
        left,
        top,
        bytes_per_pixel,
        data,
    } = payload;
    PreparedGlyphRaster {
        glyph_key,
        width,
        height,
        left,
        top,
        bytes_per_pixel,
        data,
    }
}

fn prepared_glyph_raster_payload(
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    bytes_per_pixel: u32,
    data: Vec<u8>,
) -> PreparedGlyphRasterPayload {
    PreparedGlyphRasterPayload {
        width,
        height,
        left,
        top,
        bytes_per_pixel,
        data,
    }
}
