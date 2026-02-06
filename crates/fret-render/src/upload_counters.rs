use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct UploadCounters {
    pub(crate) image_uploads: u64,
    pub(crate) image_upload_bytes: u64,
    pub(crate) svg_uploads: u64,
    pub(crate) svg_upload_bytes: u64,
}

static IMAGE_UPLOADS: AtomicU64 = AtomicU64::new(0);
static IMAGE_UPLOAD_BYTES: AtomicU64 = AtomicU64::new(0);
static SVG_UPLOADS: AtomicU64 = AtomicU64::new(0);
static SVG_UPLOAD_BYTES: AtomicU64 = AtomicU64::new(0);

pub(crate) fn record_image_upload(bytes: usize) {
    if bytes == 0 {
        return;
    }
    IMAGE_UPLOADS.fetch_add(1, Ordering::Relaxed);
    IMAGE_UPLOAD_BYTES.fetch_add(bytes as u64, Ordering::Relaxed);
}

pub(crate) fn record_svg_upload(bytes: usize) {
    if bytes == 0 {
        return;
    }
    SVG_UPLOADS.fetch_add(1, Ordering::Relaxed);
    SVG_UPLOAD_BYTES.fetch_add(bytes as u64, Ordering::Relaxed);
}

pub(crate) fn take_upload_counters() -> UploadCounters {
    UploadCounters {
        image_uploads: IMAGE_UPLOADS.swap(0, Ordering::Relaxed),
        image_upload_bytes: IMAGE_UPLOAD_BYTES.swap(0, Ordering::Relaxed),
        svg_uploads: SVG_UPLOADS.swap(0, Ordering::Relaxed),
        svg_upload_bytes: SVG_UPLOAD_BYTES.swap(0, Ordering::Relaxed),
    }
}
