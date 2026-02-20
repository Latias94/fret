//! Apple AVFoundation helpers for EXTV2 native frame sources.
//!
//! This module is runner-facing and intentionally capability-gated. It is the staging area for
//! wiring a real AVFoundation-backed video source into `NativeExternalTextureFrame` without
//! leaking Metal/IOSurface handles into `fret-ui` or ecosystem code.
//!
//! Notes:
//! - The initial landing is intentionally conservative: a portable CPU upload path is acceptable
//!   as the first end-to-end adapter, as long as the deterministic fallback chain is respected.
//! - True low/zero-copy on Apple platforms typically requires a shared-allocation story
//!   (renderer-owned texture backed by IOSurface) or an explicit external-handle import API from
//!   the backend. Both remain capability-gated.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{
    EngineFrameKeepalive, NativeExternalImportError, NativeExternalImportedFrame,
    NativeExternalTextureFrame,
};
use fret_render::{
    RenderTargetIngestStrategy, RenderTargetMatrixCoefficients, RenderTargetMetadata,
    RendererCapabilities, WgpuContext,
};

#[derive(Clone)]
pub struct AvfVideoNativeExternalImporter {
    inner: Arc<Mutex<AvfVideoNativeExternalState>>,
}

struct AvfVideoNativeExternalState {
    path: String,
    #[cfg(target_os = "macos")]
    reader: Option<AvfVideoReader>,
    texture: Option<wgpu::Texture>,
    texture_size: (u32, u32),
    orientation: fret_render::RenderTargetOrientation,
}

struct AvfVideoNativeExternalFrame {
    inner: Arc<Mutex<AvfVideoNativeExternalState>>,
}

struct AvfVideoNativeExternalKeepalive {
    _inner: Arc<Mutex<AvfVideoNativeExternalState>>,
}

impl AvfVideoNativeExternalImporter {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AvfVideoNativeExternalState {
                path: path.into(),
                #[cfg(target_os = "macos")]
                reader: None,
                texture: None,
                texture_size: (0, 0),
                orientation: fret_render::RenderTargetOrientation::default(),
            })),
        }
    }

    pub fn path(&self) -> String {
        self.inner
            .lock()
            .ok()
            .map(|v| v.path.clone())
            .unwrap_or_default()
    }

    pub fn last_size(&self) -> Option<(u32, u32)> {
        self.inner.lock().ok().and_then(|v| {
            if v.texture.is_some() && v.texture_size.0 > 0 && v.texture_size.1 > 0 {
                Some(v.texture_size)
            } else {
                None
            }
        })
    }

    pub fn frame(&self) -> Box<dyn NativeExternalTextureFrame> {
        Box::new(AvfVideoNativeExternalFrame {
            inner: self.inner.clone(),
        })
    }
}

impl NativeExternalTextureFrame for AvfVideoNativeExternalFrame {
    #[cfg(target_os = "ios")]
    fn import(
        self: Box<Self>,
        _ctx: &WgpuContext,
        _caps: &RendererCapabilities,
    ) -> Result<NativeExternalImportedFrame, NativeExternalImportError> {
        Err(NativeExternalImportError::Unsupported)
    }

    #[cfg(target_os = "macos")]
    fn import(
        self: Box<Self>,
        ctx: &WgpuContext,
        _caps: &RendererCapabilities,
    ) -> Result<NativeExternalImportedFrame, NativeExternalImportError> {
        let keepalive = EngineFrameKeepalive::new(AvfVideoNativeExternalKeepalive {
            _inner: self.inner.clone(),
        });

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| NativeExternalImportError::Failed {
                reason: "avf_native_external_state_lock_poisoned",
            })?;

        let raw_path = guard.path.trim().to_string();
        if raw_path.is_empty() {
            return Err(NativeExternalImportError::Failed {
                reason: "avf_video_path_empty",
            });
        }

        if guard.reader.is_none() {
            guard.reader = Some(AvfVideoReader::new(&raw_path).map_err(|_| {
                NativeExternalImportError::Failed {
                    reason: "avf_reader_init_failed",
                }
            })?);
            guard.orientation = guard
                .reader
                .as_ref()
                .map(|r| r.orientation)
                .unwrap_or_default();
        }

        let frame = {
            let state = &mut *guard;
            let (reader_opt, texture_slot, texture_size) = (
                &mut state.reader,
                &mut state.texture,
                &mut state.texture_size,
            );
            let reader = reader_opt.as_mut().expect("avf reader exists");

            match reader
                .with_next_bgra32_frame(|size, bytes_per_row, bytes| {
                    if texture_slot.is_none() || *texture_size != size {
                        *texture_slot = Some(
                            AvfVideoNativeExternalState::ensure_bgra_shared_texture(ctx, size),
                        );
                        *texture_size = size;
                    }
                    let texture = texture_slot.as_ref().expect("texture allocated");
                    fret_render::write_rgba8_texture_region(
                        &ctx.queue,
                        texture,
                        (0, 0),
                        size,
                        bytes_per_row,
                        bytes,
                    );
                    Ok(())
                })
                .map_err(|_| NativeExternalImportError::Failed {
                    reason: "avf_reader_read_next_failed",
                })? {
                Some(v) => v,
                None => {
                    reader.reset().ok();
                    reader
                        .with_next_bgra32_frame(|size, bytes_per_row, bytes| {
                            if texture_slot.is_none() || *texture_size != size {
                                *texture_slot =
                                    Some(AvfVideoNativeExternalState::ensure_bgra_shared_texture(
                                        ctx, size,
                                    ));
                                *texture_size = size;
                            }
                            let texture = texture_slot.as_ref().expect("texture allocated");
                            fret_render::write_rgba8_texture_region(
                                &ctx.queue,
                                texture,
                                (0, 0),
                                size,
                                bytes_per_row,
                                bytes,
                            );
                            Ok(())
                        })
                        .map_err(|_| NativeExternalImportError::Failed {
                            reason: "avf_reader_read_next_failed_after_reset",
                        })?
                        .ok_or(NativeExternalImportError::Failed {
                            reason: "avf_reader_end_of_stream",
                        })?
                }
            }
        };

        let texture = guard.texture.as_ref().expect("texture allocated");
        let view = AvfVideoNativeExternalState::view_srgb(texture);
        let mut metadata = RenderTargetMetadata::default();
        metadata.requested_ingest_strategy = RenderTargetIngestStrategy::ExternalZeroCopy;
        metadata.ingest_strategy = RenderTargetIngestStrategy::CpuUpload;
        metadata.orientation = guard.orientation;
        metadata.color_encoding.matrix = RenderTargetMatrixCoefficients::Rgb;
        metadata.frame_timestamp_ns = frame.timestamp_ns;

        Ok(NativeExternalImportedFrame {
            view,
            size: frame.size,
            metadata,
            keepalive,
        })
    }
}

#[cfg(target_os = "macos")]
impl AvfVideoNativeExternalState {
    fn ensure_bgra_shared_texture(ctx: &WgpuContext, size: (u32, u32)) -> wgpu::Texture {
        let (w, h) = size;
        let view_formats = [wgpu::TextureFormat::Bgra8UnormSrgb];
        ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("avf video native external texture (bgra)"),
            size: wgpu::Extent3d {
                width: w.max(1),
                height: h.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &view_formats,
        })
    }

    fn view_srgb(texture: &wgpu::Texture) -> wgpu::TextureView {
        texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
            ..Default::default()
        })
    }
}

#[cfg(target_os = "macos")]
#[link(name = "AVFoundation", kind = "framework")]
unsafe extern "C" {}
#[cfg(target_os = "macos")]
#[link(name = "CoreVideo", kind = "framework")]
unsafe extern "C" {}
#[cfg(target_os = "macos")]
#[link(name = "CoreMedia", kind = "framework")]
unsafe extern "C" {}
#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {}
#[cfg(target_os = "macos")]
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {}

#[cfg(target_os = "macos")]
mod avf {
    use anyhow::Context as _;
    use std::ffi::CString;
    use std::os::raw::c_void;
    use std::path::Path;

    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    pub type CFTypeRef = *const c_void;
    pub type CMSampleBufferRef = *const c_void;
    pub type CVPixelBufferRef = *const c_void;

    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct CMTime {
        pub value: i64,
        pub timescale: i32,
        pub flags: u32,
        pub epoch: i64,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    pub struct CGAffineTransform {
        pub a: f64,
        pub b: f64,
        pub c: f64,
        pub d: f64,
        pub tx: f64,
        pub ty: f64,
    }

    unsafe extern "C" {
        pub static kCVPixelBufferPixelFormatTypeKey: *const c_void;

        pub fn CFRelease(obj: CFTypeRef);

        pub fn CMSampleBufferGetImageBuffer(sample: CMSampleBufferRef) -> CVPixelBufferRef;
        pub fn CMSampleBufferGetPresentationTimeStamp(sample: CMSampleBufferRef) -> CMTime;

        pub fn CVPixelBufferLockBaseAddress(pixel_buffer: CVPixelBufferRef, lock_flags: u64)
        -> i32;
        pub fn CVPixelBufferUnlockBaseAddress(
            pixel_buffer: CVPixelBufferRef,
            lock_flags: u64,
        ) -> i32;
        pub fn CVPixelBufferGetWidth(pixel_buffer: CVPixelBufferRef) -> usize;
        pub fn CVPixelBufferGetHeight(pixel_buffer: CVPixelBufferRef) -> usize;
        pub fn CVPixelBufferGetBytesPerRow(pixel_buffer: CVPixelBufferRef) -> usize;
        pub fn CVPixelBufferGetBaseAddress(pixel_buffer: CVPixelBufferRef) -> *mut c_void;
        pub fn CVPixelBufferGetPixelFormatType(pixel_buffer: CVPixelBufferRef) -> u32;
    }

    #[derive(Debug)]
    pub struct VideoFrameMeta {
        pub size: (u32, u32),
        pub timestamp_ns: Option<u64>,
    }

    struct CfReleaseGuard(CFTypeRef);

    impl Drop for CfReleaseGuard {
        fn drop(&mut self) {
            unsafe {
                if !self.0.is_null() {
                    CFRelease(self.0);
                }
            }
        }
    }

    struct PixelBufferLockGuard {
        pixel_buffer: CVPixelBufferRef,
        flags: u64,
    }

    impl Drop for PixelBufferLockGuard {
        fn drop(&mut self) {
            unsafe {
                if !self.pixel_buffer.is_null() {
                    let _ = CVPixelBufferUnlockBaseAddress(self.pixel_buffer, self.flags);
                }
            }
        }
    }

    fn nsstring_from_str(s: &str) -> Option<*mut Object> {
        unsafe {
            let class = Class::get("NSString")?;
            let cstr = CString::new(s).ok()?;
            let obj: *mut Object = msg_send![class, alloc];
            if obj.is_null() {
                return None;
            }
            let obj: *mut Object = msg_send![obj, initWithUTF8String: cstr.as_ptr()];
            if obj.is_null() {
                return None;
            }
            let obj: *mut Object = msg_send![obj, autorelease];
            Some(obj)
        }
    }

    fn nsurl_from_path(path: &Path) -> Option<*mut Object> {
        unsafe {
            let url_class = Class::get("NSURL")?;
            let p = path.to_string_lossy();
            let ns_path = nsstring_from_str(&p)?;
            let url: *mut Object = msg_send![url_class, fileURLWithPath: ns_path];
            (!url.is_null()).then_some(url)
        }
    }

    fn dict_pixel_format_bgra32() -> Option<*mut Object> {
        unsafe {
            let dict_class = Class::get("NSDictionary")?;
            let number_class = Class::get("NSNumber")?;

            const BGRA: u32 = u32::from_be_bytes(*b"BGRA");
            let n: *mut Object = msg_send![number_class, numberWithUnsignedInt: BGRA];
            if n.is_null() {
                return None;
            }
            let key = kCVPixelBufferPixelFormatTypeKey as *mut Object;
            if key.is_null() {
                return None;
            }
            let dict: *mut Object = msg_send![dict_class, dictionaryWithObject: n forKey: key];
            (!dict.is_null()).then_some(dict)
        }
    }

    fn map_preferred_transform(
        transform: CGAffineTransform,
    ) -> fret_render::RenderTargetOrientation {
        let eps = 1e-3;
        let approx = |a: f64, b: f64| (a - b).abs() <= eps;

        let rotation = if approx(transform.a, 1.0)
            && approx(transform.b, 0.0)
            && approx(transform.c, 0.0)
            && approx(transform.d, 1.0)
        {
            fret_render::RenderTargetRotation::R0
        } else if approx(transform.a, 0.0)
            && approx(transform.b, 1.0)
            && approx(transform.c, -1.0)
            && approx(transform.d, 0.0)
        {
            fret_render::RenderTargetRotation::R90
        } else if approx(transform.a, -1.0)
            && approx(transform.b, 0.0)
            && approx(transform.c, 0.0)
            && approx(transform.d, -1.0)
        {
            fret_render::RenderTargetRotation::R180
        } else if approx(transform.a, 0.0)
            && approx(transform.b, -1.0)
            && approx(transform.c, 1.0)
            && approx(transform.d, 0.0)
        {
            fret_render::RenderTargetRotation::R270
        } else {
            fret_render::RenderTargetRotation::R0
        };

        let det = transform.a * transform.d - transform.b * transform.c;
        let mirror_x = det < 0.0;

        fret_render::RenderTargetOrientation { rotation, mirror_x }
    }

    pub struct AvfVideoReader {
        source_raw: String,
        asset: *mut Object,
        reader: *mut Object,
        output: *mut Object,
        pub orientation: fret_render::RenderTargetOrientation,
    }

    impl Drop for AvfVideoReader {
        fn drop(&mut self) {
            unsafe {
                if !self.output.is_null() {
                    let _: () = msg_send![self.output, release];
                }
                if !self.reader.is_null() {
                    let _: () = msg_send![self.reader, release];
                }
                if !self.asset.is_null() {
                    let _: () = msg_send![self.asset, release];
                }
            }
        }
    }

    impl AvfVideoReader {
        pub fn new(raw: &str) -> anyhow::Result<Self> {
            let path = super::resolve_video_source_path(raw)?;
            let url = nsurl_from_path(&path).context("file URL")?;

            unsafe {
                let asset_class = Class::get("AVAsset").context("AVAsset class")?;
                let asset: *mut Object = msg_send![asset_class, assetWithURL: url];
                anyhow::ensure!(!asset.is_null(), "assetWithURL returned null");
                let asset: *mut Object = msg_send![asset, retain];

                let video_type = nsstring_from_str("vide").context("media type string")?;
                let tracks: *mut Object = msg_send![asset, tracksWithMediaType: video_type];
                anyhow::ensure!(!tracks.is_null(), "tracksWithMediaType returned null");
                let count: usize = msg_send![tracks, count];
                anyhow::ensure!(count > 0, "no video tracks");
                let track: *mut Object = msg_send![tracks, objectAtIndex: 0usize];
                anyhow::ensure!(!track.is_null(), "video track is null");

                let transform: CGAffineTransform = msg_send![track, preferredTransform];
                let orientation = map_preferred_transform(transform);

                let reader_class = Class::get("AVAssetReader").context("AVAssetReader class")?;
                let reader: *mut Object = msg_send![reader_class, alloc];
                anyhow::ensure!(!reader.is_null(), "AVAssetReader alloc returned null");
                let mut err: *mut Object = std::ptr::null_mut();
                let reader: *mut Object = msg_send![reader, initWithAsset: asset error: &mut err];
                anyhow::ensure!(!reader.is_null(), "AVAssetReader init returned null");

                let settings = dict_pixel_format_bgra32().context("output settings dict")?;
                let output_class =
                    Class::get("AVAssetReaderTrackOutput").context("AVAssetReaderTrackOutput")?;
                let output: *mut Object = msg_send![output_class, alloc];
                anyhow::ensure!(!output.is_null(), "output alloc returned null");
                let output: *mut Object =
                    msg_send![output, initWithTrack: track outputSettings: settings];
                anyhow::ensure!(!output.is_null(), "output init returned null");

                let _: () = msg_send![output, setAlwaysCopiesSampleData: false];

                let added: bool = msg_send![reader, addOutput: output];
                anyhow::ensure!(added, "addOutput returned false");

                let started: bool = msg_send![reader, startReading];
                if !started {
                    let err_obj: *mut Object = msg_send![reader, error];
                    if !err_obj.is_null() {
                        let desc: *mut Object = msg_send![err_obj, localizedDescription];
                        if !desc.is_null() {
                            let c_str: *const std::os::raw::c_char = msg_send![desc, UTF8String];
                            if !c_str.is_null() {
                                let s = std::ffi::CStr::from_ptr(c_str).to_string_lossy();
                                anyhow::bail!("startReading failed: {s}");
                            }
                        }
                    }
                    anyhow::bail!("startReading failed");
                }

                Ok(Self {
                    source_raw: raw.to_string(),
                    asset,
                    reader,
                    output,
                    orientation,
                })
            }
        }

        pub fn reset(&mut self) -> anyhow::Result<()> {
            // Deterministic reset: recreate the reader/output graph.
            let raw = self.source_raw.clone();
            *self = Self::new(&raw)?;
            Ok(())
        }

        pub fn with_next_bgra32_frame(
            &mut self,
            mut on_frame: impl FnMut((u32, u32), u32, &[u8]) -> anyhow::Result<()>,
        ) -> anyhow::Result<Option<VideoFrameMeta>> {
            unsafe {
                let sample: CMSampleBufferRef = msg_send![self.output, copyNextSampleBuffer];
                if sample.is_null() {
                    return Ok(None);
                }
                let _sample_guard = CfReleaseGuard(sample as CFTypeRef);

                let pb = CMSampleBufferGetImageBuffer(sample);
                if pb.is_null() {
                    anyhow::bail!("sample missing image buffer");
                }

                const BGRA: u32 = u32::from_be_bytes(*b"BGRA");
                let fmt = CVPixelBufferGetPixelFormatType(pb);
                anyhow::ensure!(fmt == BGRA, "unexpected pixel format: {fmt:#X}");

                let lock = CVPixelBufferLockBaseAddress(pb, 0);
                anyhow::ensure!(lock == 0, "CVPixelBufferLockBaseAddress failed: {lock}");
                let _pb_guard = PixelBufferLockGuard {
                    pixel_buffer: pb,
                    flags: 0,
                };

                let width = CVPixelBufferGetWidth(pb);
                let height = CVPixelBufferGetHeight(pb);
                let bytes_per_row = CVPixelBufferGetBytesPerRow(pb);
                let base = CVPixelBufferGetBaseAddress(pb) as *const u8;
                anyhow::ensure!(!base.is_null(), "pixel buffer base address is null");

                let len = bytes_per_row.saturating_mul(height);
                let bytes = std::slice::from_raw_parts(base, len);
                on_frame((width as u32, height as u32), bytes_per_row as u32, bytes)?;

                let ts = CMSampleBufferGetPresentationTimeStamp(sample);
                let timestamp_ns = if ts.timescale > 0 && ts.value >= 0 {
                    let ns = (ts.value as i128)
                        .saturating_mul(1_000_000_000i128)
                        .saturating_div(ts.timescale as i128);
                    u64::try_from(ns).ok()
                } else {
                    None
                };

                Ok(Some(VideoFrameMeta {
                    size: (width as u32, height as u32),
                    timestamp_ns,
                }))
            }
        }
    }
}

#[cfg(target_os = "macos")]
use avf::AvfVideoReader;

fn resolve_video_source_path(raw: &str) -> anyhow::Result<PathBuf> {
    let p = Path::new(raw);
    if p.is_file() {
        return Ok(p.to_path_buf());
    }
    if p.is_dir() {
        let mut candidates: Vec<PathBuf> = Vec::new();
        for entry in std::fs::read_dir(p)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if matches!(ext.as_str(), "mp4" | "mov" | "m4v") {
                candidates.push(path);
            }
        }
        candidates.sort();
        return candidates
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no supported video files in directory"));
    }

    Ok(p.to_path_buf())
}
