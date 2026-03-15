use super::super::GlyphQuadKind;
use super::super::atlas::{GlyphAtlas, GlyphKey};
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
    kind: GlyphQuadKind,
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

    pub(super) fn kind(&self) -> GlyphQuadKind {
        self.glyph_key.kind
    }
}

pub(super) fn insert_prepared_glyph_raster_into_atlas(
    atlas: &mut GlyphAtlas,
    raster: PreparedGlyphRaster,
    epoch: u64,
) {
    let PreparedGlyphRaster {
        glyph_key,
        width,
        height,
        left,
        top,
        bytes_per_pixel,
        data,
    } = raster;
    let _ = atlas.get_or_insert(
        glyph_key,
        width,
        height,
        left,
        top,
        bytes_per_pixel,
        data,
        epoch,
    );
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

pub(super) fn prepared_glyph_lookup_key(
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    kind: GlyphQuadKind,
) -> GlyphKey {
    prepared_glyph_key(face_key, glyph_id, size_bits, x_bin, y_bin, kind)
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
    content: parley::swash::scale::image::Content,
) -> PreparedGlyphRasterMetadata {
    match content {
        parley::swash::scale::image::Content::Mask => PreparedGlyphRasterMetadata {
            kind: GlyphQuadKind::Mask,
            bytes_per_pixel: 1,
        },
        parley::swash::scale::image::Content::Color => PreparedGlyphRasterMetadata {
            kind: GlyphQuadKind::Color,
            bytes_per_pixel: 4,
        },
        parley::swash::scale::image::Content::SubpixelMask => PreparedGlyphRasterMetadata {
            kind: GlyphQuadKind::Subpixel,
            bytes_per_pixel: 4,
        },
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
    let metadata = prepared_glyph_raster_metadata(image.content);
    prepared_glyph_raster_from_image_parts_with_metadata(
        face_key, glyph_id, size_bits, x_bin, y_bin, image, placement, metadata,
    )
}

fn prepared_glyph_raster_from_image_parts_with_metadata(
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
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
        kind,
        bytes_per_pixel,
    } = metadata;
    prepared_glyph_raster_from_image_parts_with_payload(
        face_key,
        glyph_id,
        size_bits,
        x_bin,
        y_bin,
        width,
        height,
        left,
        top,
        kind,
        bytes_per_pixel,
        image.data,
    )
}

fn prepared_glyph_raster_from_image_parts_with_payload(
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    kind: GlyphQuadKind,
    bytes_per_pixel: u32,
    data: Vec<u8>,
) -> PreparedGlyphRaster {
    prepared_glyph_raster(
        face_key,
        glyph_id,
        size_bits,
        x_bin,
        y_bin,
        kind,
        width,
        height,
        left,
        top,
        bytes_per_pixel,
        data,
    )
}

fn prepared_glyph_raster(
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    kind: GlyphQuadKind,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    bytes_per_pixel: u32,
    data: Vec<u8>,
) -> PreparedGlyphRaster {
    let glyph_key = prepared_glyph_raster_key(face_key, glyph_id, size_bits, x_bin, y_bin, kind);
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

fn prepared_glyph_raster_key(
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    kind: GlyphQuadKind,
) -> GlyphKey {
    prepared_glyph_key(face_key, glyph_id, size_bits, x_bin, y_bin, kind)
}

fn prepared_glyph_key(
    face_key: FontFaceKey,
    glyph_id: u32,
    size_bits: u32,
    x_bin: u8,
    y_bin: u8,
    kind: GlyphQuadKind,
) -> GlyphKey {
    GlyphKey {
        font: face_key,
        glyph_id,
        size_bits,
        x_bin,
        y_bin,
        kind,
    }
}
