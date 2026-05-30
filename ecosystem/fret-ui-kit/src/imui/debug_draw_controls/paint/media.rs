use fret_core::{DrawOrder, ImageId, Rect, UvRect};

mod dispatch;
mod raster;
mod rounded;
mod svg;

pub(super) use dispatch::paint_debug_draw_media_command;

#[derive(Debug, Clone, Copy)]
pub(super) struct MediaPaintKey {
    pub(super) key: u64,
    pub(super) order: DrawOrder,
    pub(super) rect: Rect,
}

pub(super) type RasterImage = ImageId;
pub(super) type RasterUvRect = UvRect;
