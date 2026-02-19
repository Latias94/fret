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

use std::sync::{Arc, Mutex};

use super::{NativeExternalImportError, NativeExternalImportedFrame, NativeExternalTextureFrame};
use fret_render::{RendererCapabilities, WgpuContext};

#[derive(Clone, Debug)]
pub struct AvfVideoNativeExternalImporter {
    inner: Arc<Mutex<AvfVideoNativeExternalState>>,
}

#[derive(Debug)]
struct AvfVideoNativeExternalState {
    path: String,
}

#[derive(Debug)]
struct AvfVideoNativeExternalFrame {
    _inner: Arc<Mutex<AvfVideoNativeExternalState>>,
}

impl AvfVideoNativeExternalImporter {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AvfVideoNativeExternalState {
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
        Box::new(AvfVideoNativeExternalFrame {
            _inner: self.inner.clone(),
        })
    }
}

impl NativeExternalTextureFrame for AvfVideoNativeExternalFrame {
    fn import(
        self: Box<Self>,
        _ctx: &WgpuContext,
        _caps: &RendererCapabilities,
    ) -> Result<NativeExternalImportedFrame, NativeExternalImportError> {
        // TODO(EXTV2-native-120): Implement AVFoundation-backed video decode:
        // - decode into CVPixelBuffer (BGRA/NV12),
        // - apply orientation + color encoding hints into RenderTargetMetadata,
        // - stage into a renderer-owned texture via deterministic ingest strategy selection.
        Err(NativeExternalImportError::Unsupported)
    }
}
