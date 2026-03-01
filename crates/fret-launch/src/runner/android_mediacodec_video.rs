//! Android MediaCodec helpers for EXTV2 native frame sources.
//!
//! This module is runner-facing and intentionally capability-gated. It is the staging area for
//! wiring a real MediaCodec-backed video source into `NativeExternalTextureFrame` without
//! leaking `AHardwareBuffer` / EGL / Vulkan handles into `fret-ui` or ecosystem code.
//!
//! Notes:
//! - The first end-to-end landing can be CPU upload (portable, deterministic), as long as
//!   the fallback chain is explicit and observable.
//! - True low/zero-copy on Android typically requires an `AHardwareBuffer` shared allocation
//!   or explicit external-handle import support from the backend. Both remain capability-gated.

use std::sync::{Arc, Mutex};

use super::{NativeExternalImportError, NativeExternalImportedFrame, NativeExternalTextureFrame};
use fret_render::{RendererCapabilities, WgpuContext};

#[derive(Clone, Debug)]
pub struct MediaCodecVideoNativeExternalImporter {
    inner: Arc<Mutex<MediaCodecVideoNativeExternalState>>,
}

#[derive(Debug)]
struct MediaCodecVideoNativeExternalState {
    path: String,
}

#[derive(Debug)]
struct MediaCodecVideoNativeExternalFrame {
    _inner: Arc<Mutex<MediaCodecVideoNativeExternalState>>,
}

impl MediaCodecVideoNativeExternalImporter {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MediaCodecVideoNativeExternalState {
                path: path.into(),
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

    pub fn frame(&self) -> Box<dyn NativeExternalTextureFrame> {
        Box::new(MediaCodecVideoNativeExternalFrame {
            _inner: self.inner.clone(),
        })
    }
}

impl NativeExternalTextureFrame for MediaCodecVideoNativeExternalFrame {
    fn import(
        self: Box<Self>,
        _ctx: &WgpuContext,
        _caps: &RendererCapabilities,
    ) -> Result<NativeExternalImportedFrame, NativeExternalImportError> {
        // TODO(EXTV2-native-130): Implement MediaCodec-backed video decode:
        // - decode into an AHardwareBuffer / ImageReader surface (NV12-like),
        // - normalize orientation + color encoding metadata,
        // - stage into a renderer-owned texture via deterministic ingest strategy selection.
        Err(NativeExternalImportError::Unsupported)
    }
}
