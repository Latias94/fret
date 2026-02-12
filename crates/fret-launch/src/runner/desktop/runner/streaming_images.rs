use fret_render::UploadedRgba8Image;

pub(super) struct UploadedImageEntry {
    pub(super) uploaded: UploadedRgba8Image,
    pub(super) stream_generation: u64,
    pub(super) alpha_mode: fret_core::AlphaMode,
    pub(super) nv12_planes: Option<crate::runner::yuv_gpu::Nv12Planes>,
}

pub(super) struct StreamingImageUpdateRgba8<'a> {
    pub(super) window: Option<fret_core::AppWindowId>,
    pub(super) token: fret_core::ImageUpdateToken,
    pub(super) image: fret_core::ImageId,
    pub(super) stream_generation: u64,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) update_rect_px: Option<fret_core::RectPx>,
    pub(super) bytes_per_row: u32,
    pub(super) bytes: &'a [u8],
    pub(super) color_info: fret_core::ImageColorInfo,
    pub(super) alpha_mode: fret_core::AlphaMode,
}

pub(super) struct StreamingImageUpdateNv12<'a> {
    pub(super) window: Option<fret_core::AppWindowId>,
    pub(super) token: fret_core::ImageUpdateToken,
    pub(super) image: fret_core::ImageId,
    pub(super) stream_generation: u64,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) update_rect_px: Option<fret_core::RectPx>,
    pub(super) y_bytes_per_row: u32,
    pub(super) y_plane: &'a [u8],
    pub(super) uv_bytes_per_row: u32,
    pub(super) uv_plane: &'a [u8],
    pub(super) color_info: fret_core::ImageColorInfo,
}
