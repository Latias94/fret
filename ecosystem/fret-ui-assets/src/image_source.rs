//! Ecosystem-level `img(source)` helpers: decode/load + `ImageAssetCache` integration.
//!
//! This module is intentionally policy-layer code:
//! - It does not change the mechanism-layer `fret-ui` contract surface.
//! - It reuses `ImageAssetCache` (ADR 0004 flush-point resources) for GPU registration and budgets.
//! - Decoding runs off-thread via the runner-provided `DispatcherHandle` (ADR 0175).
//!
//! The primary goal is to avoid ad-hoc per-app crop/math and decode pipelines for common needs like
//! shadcn-style avatars and thumbnail grids.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use fret_assets::{AssetLoadError, AssetLocator, ResolvedAssetBytes};
use fret_core::{AppWindowId, ImageColorSpace, ImageId};
use fret_executor::{Inbox, InboxConfig, InboxDrainer};
use fret_runtime::{
    DispatchPriority, DispatcherHandle, EffectSink, GlobalsHost, InboxDrainHost,
    InboxDrainRegistry, TimeHost,
};
#[cfg(feature = "ui")]
use fret_runtime::{Model, ModelHost, ModelId};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast as _;

use crate::UiAssetsReloadEpoch;
use crate::image_asset_cache::{ImageAssetCacheHostExt, ImageAssetKey};
use crate::image_asset_state::{ImageLoadingStatus, image_state_from_asset_cache};

static WARNED_MISSING_DISPATCHER: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageSourceId(u64);

impl ImageSourceId {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone)]
enum ImageSourceKind {
    Bytes {
        bytes: Arc<[u8]>,
    },
    #[cfg(target_arch = "wasm32")]
    Url {
        url: Arc<str>,
    },
    Rgba8 {
        width: u32,
        height: u32,
        rgba: Arc<[u8]>,
        color_space: ImageColorSpace,
    },
    #[cfg(not(target_arch = "wasm32"))]
    Path {
        path: Arc<PathBuf>,
    },
}

/// A cheap-to-clone reference to an image that can be decoded/loaded and registered into the UI
/// render asset caches.
#[derive(Debug, Clone)]
pub struct ImageSource {
    id: ImageSourceId,
    kind: ImageSourceKind,
}

impl ImageSource {
    pub fn id(&self) -> ImageSourceId {
        self.id
    }

    #[cfg(feature = "ui")]
    pub(crate) fn rgba8_meta(&self) -> Option<(u32, u32, Arc<[u8]>, ImageColorSpace)> {
        match &self.kind {
            ImageSourceKind::Rgba8 {
                width,
                height,
                rgba,
                color_space,
            } => Some((*width, *height, rgba.clone(), *color_space)),
            ImageSourceKind::Bytes { .. } => None,
            #[cfg(target_arch = "wasm32")]
            ImageSourceKind::Url { .. } => None,
            #[cfg(not(target_arch = "wasm32"))]
            ImageSourceKind::Path { .. } => None,
        }
    }

    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        let bytes: Arc<[u8]> = bytes.into();
        let id = ImageSourceId(stable_hash(&(b"bytes.v1", bytes.as_ref())));
        Self {
            id,
            kind: ImageSourceKind::Bytes { bytes },
        }
    }

    pub fn from_resolved_asset_bytes(resolved: &ResolvedAssetBytes) -> Self {
        let id = ImageSourceId(stable_hash(&(
            b"asset-bytes.v1",
            &resolved.locator,
            resolved.revision,
        )));
        Self {
            id,
            kind: ImageSourceKind::Bytes {
                bytes: resolved.bytes.clone(),
            },
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn from_native_file_path(path: impl Into<Arc<PathBuf>>) -> Self {
        let path: Arc<PathBuf> = path.into();
        let id = ImageSourceId(stable_hash(&(
            b"path.v1",
            path.as_os_str().as_encoded_bytes(),
        )));
        Self {
            id,
            kind: ImageSourceKind::Path { path },
        }
    }

    pub fn from_asset_locator(locator: &AssetLocator) -> Result<Self, AssetLoadError> {
        match locator {
            #[cfg(target_arch = "wasm32")]
            AssetLocator::Url(url) => Ok(Self::from_url(url.as_str())),
            #[cfg(not(target_arch = "wasm32"))]
            AssetLocator::File(file) => Ok(Self::from_native_file_path(file.path.clone())),
            _ => Err(AssetLoadError::UnsupportedLocatorKind {
                kind: locator.kind(),
            }),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn from_url(url: impl Into<Arc<str>>) -> Self {
        let url: Arc<str> = url.into();
        let id = ImageSourceId(stable_hash(&(b"url.v1", url.as_bytes())));
        Self {
            id,
            kind: ImageSourceKind::Url { url },
        }
    }

    /// Construct an RGBA8 source whose ID is derived from the full byte payload.
    ///
    /// If you already have a stable key for the image, prefer [`ImageSource::rgba8_keyed`]
    /// to avoid hashing large buffers.
    pub fn rgba8(
        width: u32,
        height: u32,
        rgba: impl Into<Arc<[u8]>>,
        color_space: ImageColorSpace,
    ) -> Self {
        let rgba: Arc<[u8]> = rgba.into();
        let id = ImageSourceId(stable_hash(&(
            b"rgba8.v1",
            width,
            height,
            color_space,
            rgba.as_ref(),
        )));
        Self {
            id,
            kind: ImageSourceKind::Rgba8 {
                width,
                height,
                rgba,
                color_space,
            },
        }
    }

    /// Construct an RGBA8 source with an explicit stable ID (recommended for large buffers).
    pub fn rgba8_keyed(
        id: ImageSourceId,
        width: u32,
        height: u32,
        rgba: impl Into<Arc<[u8]>>,
        color_space: ImageColorSpace,
    ) -> Self {
        Self {
            id,
            kind: ImageSourceKind::Rgba8 {
                width,
                height,
                rgba: rgba.into(),
                color_space,
            },
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[deprecated(
        note = "prefer locator-first asset requests and UI helpers; direct file paths are a native/dev-only compatibility seam"
    )]
    pub fn from_file_path(path: impl Into<Arc<PathBuf>>) -> Self {
        Self::from_native_file_path(path)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[deprecated(
        note = "prefer locator-first asset requests and UI helpers; direct file paths are a native/dev-only compatibility seam"
    )]
    pub fn from_path(path: impl Into<Arc<PathBuf>>) -> Self {
        Self::from_native_file_path(path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ImageSourceRequestKey {
    source: ImageSourceId,
    color_space: ImageColorSpace,
    reload_epoch: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct ImageSourceOptions {
    /// Color space metadata attached to the registered image resource.
    ///
    /// Note: for `rgba8(...)` sources, the source-provided color space wins.
    pub color_space: ImageColorSpace,
}

impl Default for ImageSourceOptions {
    fn default() -> Self {
        Self {
            color_space: ImageColorSpace::Srgb,
        }
    }
}

#[cfg(feature = "ui")]
fn request_key_for_source_with_epoch(
    source: &ImageSource,
    options: ImageSourceOptions,
    reload_epoch: u64,
) -> ImageSourceRequestKey {
    let color_space = source
        .rgba8_meta()
        .map(|(_, _, _, cs)| cs)
        .unwrap_or(options.color_space);
    ImageSourceRequestKey {
        source: source.id,
        color_space,
        reload_epoch,
    }
}

fn reload_epoch_for_source<H: GlobalsHost>(host: &H, source: &ImageSource) -> u64 {
    match &source.kind {
        #[cfg(not(target_arch = "wasm32"))]
        ImageSourceKind::Path { .. } => host
            .global::<UiAssetsReloadEpoch>()
            .map(|v| v.0)
            .unwrap_or(0),
        _ => 0,
    }
}

#[derive(Debug, Clone)]
pub struct ImageSourceState {
    pub image: Option<ImageId>,
    pub status: ImageLoadingStatus,
    pub intrinsic_size_px: Option<(u32, u32)>,
    pub error: Option<Arc<str>>,
}

#[derive(Debug, Clone)]
struct DecodedRgba8 {
    width: u32,
    height: u32,
    rgba: Arc<[u8]>,
}

#[derive(Debug)]
enum ImageSourceEntryState {
    Idle,
    Loading {
        inflight_id: u64,
    },
    Decoded {
        inflight_id: u64,
        decoded: DecodedRgba8,
        asset_key: ImageAssetKey,
    },
    Ready {
        asset_key: ImageAssetKey,
        intrinsic_size_px: (u32, u32),
    },
    Failed {
        message: Arc<str>,
        last_attempt_frame: u64,
    },
}

#[derive(Debug)]
struct ImageSourceEntry {
    state: ImageSourceEntryState,
    last_used_frame: u64,
}

#[derive(Debug)]
struct ImageSourceMsg {
    request: ImageSourceRequestKey,
    window: AppWindowId,
    inflight_id: u64,
    attempt_frame: u64,
    result: Result<DecodedRgba8, String>,
}

/// A tiny per-request signal model used to make async decode completions observable by view-cached
/// subtrees.
///
/// The actual image data/state machine lives in `ImageSourceRuntime`; the UI only needs an observed
/// dependency that changes when decode finishes so `ViewCache` knows it must re-render.
#[cfg(feature = "ui")]
#[derive(Debug, Default)]
pub(crate) struct ImageSourceUiSignal {
    epoch: u64,
}

#[cfg(feature = "ui")]
#[derive(Debug, Clone)]
struct ImageSourceSignalHandle {
    model: Model<ImageSourceUiSignal>,
    last_used_frame: u64,
}

pub struct ImageSourceLoader {
    runtime: Arc<ImageSourceRuntime>,
    last_entries_gc_frame: Option<u64>,
    #[cfg(feature = "ui")]
    signal_handles: HashMap<ImageSourceRequestKey, ImageSourceSignalHandle>,
    #[cfg(feature = "ui")]
    last_signal_gc_frame: Option<u64>,
}

impl ImageSourceLoader {
    fn new(dispatcher: DispatcherHandle) -> Self {
        Self {
            runtime: Arc::new(ImageSourceRuntime::new(dispatcher)),
            last_entries_gc_frame: None,
            #[cfg(feature = "ui")]
            signal_handles: HashMap::new(),
            #[cfg(feature = "ui")]
            last_signal_gc_frame: None,
        }
    }

    fn gc_entries_if_needed(&mut self, frame: u64) {
        const GC_PERIOD_FRAMES: u64 = 300;
        const TTL_FRAMES: u64 = 1800;

        let should_gc = match self.last_entries_gc_frame {
            None => true,
            Some(last) => frame.saturating_sub(last) >= GC_PERIOD_FRAMES,
        };
        if !should_gc {
            return;
        }

        self.last_entries_gc_frame = Some(frame);

        let mut entries = self
            .runtime
            .entries
            .lock()
            .expect("poisoned ImageSourceRuntime mutex");
        entries.retain(|_key, entry| {
            // Never drop in-flight entries: if we do, we can lose the async completion and stall
            // without a new signal to re-render under ViewCache.
            match &entry.state {
                ImageSourceEntryState::Loading { .. } | ImageSourceEntryState::Decoded { .. } => {
                    true
                }
                ImageSourceEntryState::Idle
                | ImageSourceEntryState::Ready { .. }
                | ImageSourceEntryState::Failed { .. } => {
                    frame.saturating_sub(entry.last_used_frame) < TTL_FRAMES
                }
            }
        });
    }

    fn ensure_registered<H: GlobalsHost>(&mut self, host: &mut H) {
        if self.runtime.registered.swap(true, Ordering::SeqCst) {
            return;
        }
        let runtime = self.runtime.clone();
        host.with_global_mut_untracked(InboxDrainRegistry::default, |registry, _host| {
            registry.register(Arc::new(image_source_inbox_drainer(runtime)));
        });
    }

    fn start_decode_if_needed(
        &self,
        request: ImageSourceRequestKey,
        source: ImageSource,
        window: AppWindowId,
        frame: u64,
    ) {
        let mut entries = self
            .runtime
            .entries
            .lock()
            .expect("poisoned ImageSourceRuntime mutex");
        let entry = entries.entry(request).or_insert_with(|| ImageSourceEntry {
            state: ImageSourceEntryState::Idle,
            last_used_frame: frame,
        });
        entry.last_used_frame = frame;

        let should_start = match &entry.state {
            ImageSourceEntryState::Idle => true,
            ImageSourceEntryState::Failed {
                last_attempt_frame, ..
            } => frame.saturating_sub(*last_attempt_frame) >= self.runtime.retry_cooldown_frames,
            ImageSourceEntryState::Loading { .. }
            | ImageSourceEntryState::Decoded { .. }
            | ImageSourceEntryState::Ready { .. } => false,
        };

        if !should_start {
            return;
        }

        let inflight_id = self
            .runtime
            .next_inflight_id
            .fetch_add(1, Ordering::Relaxed);
        entry.state = ImageSourceEntryState::Loading { inflight_id };

        tracing::debug!(
            source = ?request.source,
            window = ?window,
            frame = frame,
            "image_source: start decode"
        );

        #[cfg(target_arch = "wasm32")]
        if let ImageSourceKind::Url { url } = &source.kind {
            let url: Arc<str> = url.clone();
            let sender = self.runtime.inbox.sender();
            let wake_dispatcher = self.runtime.dispatcher.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = fetch_and_decode_rgba8(url.as_ref()).await;
                let _ = sender.send(ImageSourceMsg {
                    request,
                    window,
                    inflight_id,
                    attempt_frame: frame,
                    result,
                });
                wake_dispatcher.wake(Some(window));
            });
            return;
        }

        let sender = self.runtime.inbox.sender();
        let dispatcher = self.runtime.dispatcher.clone();
        let wake_dispatcher = dispatcher.clone();
        dispatcher.dispatch_background(
            Box::new(move || {
                let result = decode_rgba8(&source);
                let _ = sender.send(ImageSourceMsg {
                    request,
                    window,
                    inflight_id,
                    attempt_frame: frame,
                    result,
                });
                wake_dispatcher.wake(Some(window));
            }),
            DispatchPriority::Normal,
        );
    }

    #[cfg(feature = "ui")]
    pub(crate) fn use_signal_model<H: ModelHost + TimeHost + GlobalsHost>(
        &mut self,
        app: &mut H,
        source: &ImageSource,
        options: ImageSourceOptions,
    ) -> Model<ImageSourceUiSignal> {
        const GC_PERIOD_FRAMES: u64 = 300;
        const TTL_FRAMES: u64 = 600;

        let frame = app.frame_id().0;
        let should_gc = match self.last_signal_gc_frame {
            None => true,
            Some(last) => frame.saturating_sub(last) >= GC_PERIOD_FRAMES,
        };
        if should_gc {
            self.last_signal_gc_frame = Some(frame);
            let mut expired = Vec::new();
            for (key, handle) in &self.signal_handles {
                if frame.saturating_sub(handle.last_used_frame) >= TTL_FRAMES {
                    expired.push(*key);
                }
            }
            if !expired.is_empty() {
                let mut signal_models = self
                    .runtime
                    .signal_models
                    .lock()
                    .expect("poisoned ImageSourceRuntime mutex");
                for key in expired {
                    self.signal_handles.remove(&key);
                    signal_models.remove(&key);
                }

                let live: std::collections::HashSet<ModelId> =
                    signal_models.values().copied().collect();
                drop(signal_models);

                let mut map = self
                    .runtime
                    .asset_key_to_signal_models
                    .lock()
                    .expect("poisoned ImageSourceRuntime mutex");
                map.retain(|_key, ids| {
                    ids.retain(|id| live.contains(id));
                    !ids.is_empty()
                });
            }
        }

        let reload_epoch = reload_epoch_for_source(app, source);
        let request = request_key_for_source_with_epoch(source, options, reload_epoch);

        let entry = self.signal_handles.entry(request).or_insert_with(|| {
            let model = app.models_mut().insert(ImageSourceUiSignal::default());
            self.runtime
                .signal_models
                .lock()
                .expect("poisoned ImageSourceRuntime mutex")
                .insert(request, model.id());
            ImageSourceSignalHandle {
                model,
                last_used_frame: frame,
            }
        });
        entry.last_used_frame = frame;
        entry.model.clone()
    }
}

struct ImageSourceRuntime {
    dispatcher: DispatcherHandle,
    inbox: Inbox<ImageSourceMsg>,
    registered: AtomicBool,
    next_inflight_id: AtomicU64,
    entries: Mutex<HashMap<ImageSourceRequestKey, ImageSourceEntry>>,
    #[cfg(feature = "ui")]
    signal_models: Mutex<HashMap<ImageSourceRequestKey, ModelId>>,
    #[cfg(feature = "ui")]
    asset_key_to_signal_models: Mutex<HashMap<ImageAssetKey, Vec<ModelId>>>,
    retry_cooldown_frames: u64,
}

impl ImageSourceRuntime {
    fn new(dispatcher: DispatcherHandle) -> Self {
        Self {
            dispatcher,
            inbox: Inbox::new(InboxConfig {
                capacity: 256,
                ..Default::default()
            }),
            registered: AtomicBool::new(false),
            next_inflight_id: AtomicU64::new(1),
            entries: Mutex::new(HashMap::new()),
            #[cfg(feature = "ui")]
            signal_models: Mutex::new(HashMap::new()),
            #[cfg(feature = "ui")]
            asset_key_to_signal_models: Mutex::new(HashMap::new()),
            retry_cooldown_frames: 60,
        }
    }

    fn apply_msg(&self, host: &mut dyn InboxDrainHost, msg: ImageSourceMsg) {
        {
            let mut entries = self
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex");
            let Some(entry) = entries.get_mut(&msg.request) else {
                return;
            };

            let inflight_matches = match &entry.state {
                ImageSourceEntryState::Loading { inflight_id, .. } => {
                    *inflight_id == msg.inflight_id
                }
                ImageSourceEntryState::Decoded { inflight_id, .. } => {
                    *inflight_id == msg.inflight_id
                }
                ImageSourceEntryState::Idle
                | ImageSourceEntryState::Ready { .. }
                | ImageSourceEntryState::Failed { .. } => false,
            };
            if !inflight_matches {
                return;
            }

            match msg.result {
                Ok(decoded) => {
                    tracing::debug!(
                        source = ?msg.request.source,
                        window = ?msg.window,
                        width = decoded.width,
                        height = decoded.height,
                        "image_source: decode ok"
                    );
                    let asset_key = ImageAssetKey::from_rgba8(
                        decoded.width,
                        decoded.height,
                        msg.request.color_space,
                        decoded.rgba.as_ref(),
                    );
                    entry.state = ImageSourceEntryState::Decoded {
                        inflight_id: msg.inflight_id,
                        decoded,
                        asset_key,
                    };

                    #[cfg(feature = "ui")]
                    self.register_asset_key_signal_mapping(msg.request, asset_key);
                }
                Err(err) => {
                    tracing::warn!(
                        source = ?msg.request.source,
                        window = ?msg.window,
                        error = %err,
                        "image_source: decode failed"
                    );
                    entry.state = ImageSourceEntryState::Failed {
                        message: Arc::<str>::from(err),
                        last_attempt_frame: msg.attempt_frame,
                    };
                }
            }
        }

        #[cfg(feature = "ui")]
        {
            // ViewCache-safe: bump the per-request signal model (if one is registered) so cached
            // subtrees re-render when decode finishes.
            if let Some(signal_model_id) = self
                .signal_models
                .lock()
                .expect("poisoned ImageSourceRuntime mutex")
                .get(&msg.request)
                .copied()
            {
                let updated = host
                    .models_mut()
                    .update_any(signal_model_id, |state_any| {
                        let state = state_any
                            .downcast_mut::<ImageSourceUiSignal>()
                            .expect("ImageSourceUiSignal model type mismatch");
                        state.epoch = state.epoch.wrapping_add(1);
                    })
                    .is_ok();
                if !updated {
                    let _ = self
                        .signal_models
                        .lock()
                        .expect("poisoned ImageSourceRuntime mutex")
                        .remove(&msg.request);
                }
            }
        }

        host.request_redraw(msg.window);
    }

    #[cfg(feature = "ui")]
    pub(crate) fn register_asset_key_signal_mapping(
        &self,
        request: ImageSourceRequestKey,
        asset_key: ImageAssetKey,
    ) {
        let Some(model_id) = self
            .signal_models
            .lock()
            .expect("poisoned ImageSourceRuntime mutex")
            .get(&request)
            .copied()
        else {
            return;
        };

        let mut map = self
            .asset_key_to_signal_models
            .lock()
            .expect("poisoned ImageSourceRuntime mutex");
        let list = map.entry(asset_key).or_default();
        if !list.contains(&model_id) {
            list.push(model_id);
        }
    }
}

#[cfg(feature = "ui")]
pub(crate) fn notify_image_asset_key<H: GlobalsHost + ModelHost>(app: &mut H, key: ImageAssetKey) {
    let Some(model_ids) = with_image_source_loader(app, |loader, _app| {
        loader
            .runtime
            .asset_key_to_signal_models
            .lock()
            .expect("poisoned ImageSourceRuntime mutex")
            .get(&key)
            .cloned()
            .unwrap_or_default()
    }) else {
        return;
    };

    if model_ids.is_empty() {
        return;
    }

    let original_len = model_ids.len();
    let mut alive = Vec::with_capacity(model_ids.len());
    for model_id in model_ids {
        if app
            .models_mut()
            .update_any(model_id, |state_any| {
                let state = state_any
                    .downcast_mut::<ImageSourceUiSignal>()
                    .expect("ImageSourceUiSignal model type mismatch");
                state.epoch = state.epoch.wrapping_add(1);
            })
            .is_err()
        {
            continue;
        }
        alive.push(model_id);
    }

    if alive.len() == original_len {
        return;
    }

    let _ = with_image_source_loader(app, |loader, _app| {
        let mut map = loader
            .runtime
            .asset_key_to_signal_models
            .lock()
            .expect("poisoned ImageSourceRuntime mutex");
        if alive.is_empty() {
            map.remove(&key);
        } else {
            map.insert(key, alive);
        }
    });
}

#[cfg(feature = "ui")]
pub(crate) fn register_asset_key_for_source<H: GlobalsHost>(
    app: &mut H,
    source: &ImageSource,
    options: ImageSourceOptions,
    asset_key: ImageAssetKey,
) {
    let reload_epoch = reload_epoch_for_source(app, source);
    let request = request_key_for_source_with_epoch(source, options, reload_epoch);

    let _ = with_image_source_loader(app, |loader, _app| {
        loader
            .runtime
            .register_asset_key_signal_mapping(request, asset_key);
    });
}

fn image_source_inbox_drainer(runtime: Arc<ImageSourceRuntime>) -> InboxDrainer<ImageSourceMsg> {
    InboxDrainer::new(runtime.inbox.clone(), move |host, _window, msg| {
        runtime.apply_msg(host, msg);
    })
}

/// Access the global [`ImageSourceLoader`], returning `None` when the runner did not install a
/// `DispatcherHandle` global.
pub(crate) fn with_image_source_loader<H: GlobalsHost, R>(
    host: &mut H,
    f: impl FnOnce(&mut ImageSourceLoader, &mut H) -> R,
) -> Option<R> {
    let dispatcher = host.global::<DispatcherHandle>()?.clone();
    Some(host.with_global_mut_untracked(
        || ImageSourceLoader::new(dispatcher),
        |loader, host| f(loader, host),
    ))
}

/// GPUI-style "use_image_source": calling this repeatedly is cheap; it schedules work only on miss.
pub fn use_image_source_state<H: GlobalsHost + TimeHost + EffectSink>(
    host: &mut H,
    window: AppWindowId,
    source: &ImageSource,
) -> ImageSourceState {
    use_image_source_state_with_options(host, window, source, ImageSourceOptions::default())
}

pub fn use_image_source_state_with_options<H: GlobalsHost + TimeHost + EffectSink>(
    host: &mut H,
    window: AppWindowId,
    source: &ImageSource,
    options: ImageSourceOptions,
) -> ImageSourceState {
    // Fast path: if the caller already has RGBA8 bytes, skip the async decode machinery.
    if let ImageSourceKind::Rgba8 {
        width,
        height,
        rgba,
        color_space,
    } = &source.kind
    {
        return host.with_image_asset_cache(|cache, host| {
            let (key, _image) =
                cache.use_rgba8(host, window, *width, *height, rgba.as_ref(), *color_space);
            let (image, status) = image_state_from_asset_cache(cache, key);
            if let Some(ready_image) = image {
                #[cfg(feature = "image-metadata")]
                {
                    record_intrinsic_metadata(host, ready_image, (*width, *height));
                }
                #[cfg(not(feature = "image-metadata"))]
                {
                    let _ = ready_image;
                }
            }
            ImageSourceState {
                image,
                status,
                intrinsic_size_px: Some((*width, *height)),
                error: cache.error(key).map(|s| Arc::<str>::from(s.to_string())),
            }
        });
    }

    let reload_epoch = reload_epoch_for_source(host, source);
    let request = ImageSourceRequestKey {
        source: source.id,
        color_space: options.color_space,
        reload_epoch,
    };
    let frame = host.frame_id().0;

    let Some(state) = with_image_source_loader(host, |loader, host| {
        loader.ensure_registered(host);
        loader.gc_entries_if_needed(frame);

        // Take a snapshot of the current entry state.
        let snapshot = {
            let entries = loader
                .runtime
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex");
            entries.get(&request).map(|e| match &e.state {
                ImageSourceEntryState::Idle => ImageSourceEntrySnapshot::Idle,
                ImageSourceEntryState::Loading { .. } => ImageSourceEntrySnapshot::Loading,
                ImageSourceEntryState::Decoded {
                    decoded, asset_key, ..
                } => ImageSourceEntrySnapshot::Decoded {
                    decoded: decoded.clone(),
                    asset_key: *asset_key,
                },
                ImageSourceEntryState::Ready {
                    asset_key,
                    intrinsic_size_px,
                } => ImageSourceEntrySnapshot::Ready {
                    asset_key: *asset_key,
                    intrinsic_size_px: *intrinsic_size_px,
                },
                ImageSourceEntryState::Failed { message, .. } => {
                    ImageSourceEntrySnapshot::Failed(message.clone())
                }
            })
        };

        // Touch last-used for GC after snapshot to avoid holding a mutable borrow across clones.
        if snapshot.is_some() {
            let mut entries = loader
                .runtime
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex");
            if let Some(entry) = entries.get_mut(&request) {
                entry.last_used_frame = frame;
            }
        }

        match snapshot {
            None | Some(ImageSourceEntrySnapshot::Idle) => {
                loader.start_decode_if_needed(request, source.clone(), window, frame);
                ImageSourceState {
                    image: None,
                    status: ImageLoadingStatus::Loading,
                    intrinsic_size_px: None,
                    error: None,
                }
            }
            Some(ImageSourceEntrySnapshot::Loading) => ImageSourceState {
                image: None,
                status: ImageLoadingStatus::Loading,
                intrinsic_size_px: None,
                error: None,
            },
            Some(ImageSourceEntrySnapshot::Failed(message)) => {
                loader.start_decode_if_needed(request, source.clone(), window, frame);

                // If we didn't start a retry (cooldown not reached), surface the error.
                let entries = loader
                    .runtime
                    .entries
                    .lock()
                    .expect("poisoned ImageSourceRuntime mutex");
                let retrying = matches!(
                    entries.get(&request).map(|e| &e.state),
                    Some(ImageSourceEntryState::Loading { .. })
                );

                if retrying {
                    ImageSourceState {
                        image: None,
                        status: ImageLoadingStatus::Loading,
                        intrinsic_size_px: None,
                        error: None,
                    }
                } else {
                    ImageSourceState {
                        image: None,
                        status: ImageLoadingStatus::Error,
                        intrinsic_size_px: None,
                        error: Some(message),
                    }
                }
            }
            Some(ImageSourceEntrySnapshot::Decoded { decoded, asset_key }) => {
                tracing::debug!(
                    source = ?request.source,
                    window = ?window,
                    width = decoded.width,
                    height = decoded.height,
                    asset_key = ?asset_key,
                    "image_source: feed decoded bytes into ImageAssetCache"
                );
                // Feed the decoded bytes into the `ImageAssetCache` state machine.
                host.with_image_asset_cache(|cache, host| {
                    let _image = cache.use_rgba8_keyed(
                        host,
                        window,
                        asset_key,
                        decoded.width,
                        decoded.height,
                        decoded.rgba.as_ref(),
                        request.color_space,
                    );
                    let (image, status) = image_state_from_asset_cache(cache, asset_key);

                    if let Some(ready_image) = image {
                        #[cfg(feature = "image-metadata")]
                        {
                            record_intrinsic_metadata(
                                host,
                                ready_image,
                                (decoded.width, decoded.height),
                            );
                        }
                        #[cfg(not(feature = "image-metadata"))]
                        {
                            let _ = ready_image;
                        }

                        // Once the GPU resource is ready, drop decoded bytes to avoid unbounded memory use.
                        let mut entries = loader
                            .runtime
                            .entries
                            .lock()
                            .expect("poisoned ImageSourceRuntime mutex");
                        if let Some(entry) = entries.get_mut(&request) {
                            entry.state = ImageSourceEntryState::Ready {
                                asset_key,
                                intrinsic_size_px: (decoded.width, decoded.height),
                            };
                        }
                    }

                    ImageSourceState {
                        image,
                        status,
                        intrinsic_size_px: Some((decoded.width, decoded.height)),
                        error: cache
                            .error(asset_key)
                            .map(|s| Arc::<str>::from(s.to_string())),
                    }
                })
            }
            Some(ImageSourceEntrySnapshot::Ready {
                asset_key,
                intrinsic_size_px,
            }) => {
                let state = host.with_image_asset_cache(|cache, _host| {
                    let (image, status) = image_state_from_asset_cache(cache, asset_key);
                    ImageSourceState {
                        image,
                        status,
                        intrinsic_size_px: Some(intrinsic_size_px),
                        error: cache
                            .error(asset_key)
                            .map(|s| Arc::<str>::from(s.to_string())),
                    }
                });

                if state.status == ImageLoadingStatus::Idle {
                    let mut entries = loader
                        .runtime
                        .entries
                        .lock()
                        .expect("poisoned ImageSourceRuntime mutex");
                    if let Some(entry) = entries.get_mut(&request) {
                        entry.state = ImageSourceEntryState::Idle;
                    }
                    drop(entries);
                    loader.start_decode_if_needed(request, source.clone(), window, frame);
                    return ImageSourceState {
                        image: None,
                        status: ImageLoadingStatus::Loading,
                        intrinsic_size_px: None,
                        error: None,
                    };
                }

                state
            }
        }
    }) else {
        if !WARNED_MISSING_DISPATCHER.swap(true, Ordering::Relaxed) {
            tracing::warn!("image_source: missing DispatcherHandle global (decoding disabled)");
        }
        return ImageSourceState {
            image: None,
            status: ImageLoadingStatus::Error,
            intrinsic_size_px: None,
            error: Some(Arc::<str>::from("missing DispatcherHandle global")),
        };
    };

    state
}

#[derive(Debug, Clone)]
enum ImageSourceEntrySnapshot {
    Idle,
    Loading,
    Decoded {
        decoded: DecodedRgba8,
        asset_key: ImageAssetKey,
    },
    Ready {
        asset_key: ImageAssetKey,
        intrinsic_size_px: (u32, u32),
    },
    Failed(Arc<str>),
}

fn decode_rgba8(source: &ImageSource) -> Result<DecodedRgba8, String> {
    match &source.kind {
        ImageSourceKind::Bytes { bytes } => decode_bytes_rgba8(bytes.as_ref()),
        #[cfg(target_arch = "wasm32")]
        ImageSourceKind::Url { .. } => {
            Err("url image sources must be decoded via fetch".to_string())
        }
        ImageSourceKind::Rgba8 {
            width,
            height,
            rgba,
            ..
        } => {
            let expected = (*width as u64)
                .checked_mul(*height as u64)
                .and_then(|px| px.checked_mul(4))
                .ok_or_else(|| "invalid rgba8 dimensions (overflow)".to_string())?;
            if rgba.len() as u64 != expected {
                return Err(format!(
                    "invalid rgba8 byte length: expected {expected} for {width}x{height}, got {}",
                    rgba.len()
                ));
            }
            Ok(DecodedRgba8 {
                width: *width,
                height: *height,
                rgba: rgba.clone(),
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        ImageSourceKind::Path { path } => {
            let bytes = std::fs::read(path.as_ref()).map_err(|e| e.to_string())?;
            decode_bytes_rgba8(&bytes)
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_and_decode_rgba8(url: &str) -> Result<DecodedRgba8, String> {
    let bytes = fetch_url_bytes(url).await?;
    decode_bytes_rgba8(&bytes)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen_futures::JsFuture;

    let Some(window) = web_sys::window() else {
        return Err("missing web_sys::window".to_string());
    };

    let resp_value = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("fetch failed: {e:?}"))?;
    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| "fetch did not return a Response".to_string())?;
    if !resp.ok() {
        return Err(format!(
            "fetch returned HTTP {} {}",
            resp.status(),
            resp.status_text()
        ));
    }

    let buf = JsFuture::from(
        resp.array_buffer()
            .map_err(|e| format!("array_buffer() failed: {e:?}"))?,
    )
    .await
    .map_err(|e| format!("await array_buffer failed: {e:?}"))?;
    let array = js_sys::Uint8Array::new(&buf);
    Ok(array.to_vec())
}

fn decode_bytes_rgba8(bytes: &[u8]) -> Result<DecodedRgba8, String> {
    #[cfg(feature = "image-decode")]
    {
        let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(DecodedRgba8 {
            width,
            height,
            rgba: Arc::from(rgba.into_raw()),
        })
    }
    #[cfg(not(feature = "image-decode"))]
    {
        let _ = bytes;
        Err("image decode disabled (enable fret-ui-assets feature `image-decode`)".to_string())
    }
}

#[derive(Default)]
struct Fnv1a64(u64);

impl Hasher for Fnv1a64 {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf29ce484222325
        } else {
            self.0
        };
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }
}

fn stable_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = Fnv1a64::default();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(feature = "image-metadata")]
fn record_intrinsic_metadata<H: GlobalsHost>(
    host: &mut H,
    image: ImageId,
    intrinsic_size_px: (u32, u32),
) {
    fret_ui_kit::with_image_metadata_store_mut(host, |store| {
        store.set_intrinsic_size_px(image, intrinsic_size_px);
    });
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::{HashMap, HashSet};
    use std::sync::Mutex;
    use std::time::Duration;

    use fret_assets::{AssetLocator, AssetRequest, AssetRevision, InMemoryAssetResolver};
    use fret_core::{
        AppWindowId, ClipboardToken, Event, FrameId, ImageColorSpace, ImageId, ImageUploadToken,
        TimerToken,
    };
    use fret_runtime::{
        DispatchPriority, Dispatcher, DispatcherHandle, Effect, ExecCapabilities, GlobalsHost,
        InboxDrainHost, ModelHost, ModelStore, Runnable, TickId, TimeHost,
    };

    use super::*;

    #[derive(Default)]
    struct QueuedDispatcher {
        background: Mutex<Vec<Runnable>>,
    }

    impl QueuedDispatcher {
        #[cfg(all(feature = "image-decode", not(target_arch = "wasm32")))]
        fn run_background_tasks(&self) {
            let tasks = {
                let mut tasks = self.background.lock().expect("poisoned background queue");
                std::mem::take(&mut *tasks)
            };
            for task in tasks {
                task();
            }
        }

        fn drop_background_tasks(&self) {
            let mut tasks = self.background.lock().expect("poisoned background queue");
            tasks.clear();
        }
    }

    impl Dispatcher for QueuedDispatcher {
        fn dispatch_on_main_thread(&self, _task: Runnable) {}

        fn dispatch_background(&self, task: Runnable, _priority: DispatchPriority) {
            let mut tasks = self.background.lock().expect("poisoned background queue");
            tasks.push(task);
        }

        fn dispatch_after(&self, _delay: Duration, _task: Runnable) {}

        fn wake(&self, _window: Option<AppWindowId>) {}

        fn exec_capabilities(&self) -> ExecCapabilities {
            ExecCapabilities::default()
        }
    }

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        effects: Vec<Effect>,
        redraws: HashSet<AppWindowId>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_image_upload_token: u64,
    }

    impl GlobalsHost for TestHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals.get(&TypeId::of::<T>())?.downcast_ref::<T>()
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let mut value = match self.globals.remove(&type_id) {
                None => init(),
                Some(v) => *v.downcast::<T>().expect("global type id must match"),
            };
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    impl TimeHost for TestHost {
        fn tick_id(&self) -> TickId {
            self.tick_id
        }

        fn frame_id(&self) -> FrameId {
            self.frame_id
        }

        fn next_share_sheet_token(&mut self) -> fret_core::ShareSheetToken {
            fret_core::ShareSheetToken(0)
        }

        fn next_timer_token(&mut self) -> TimerToken {
            let token = TimerToken(self.next_timer_token);
            self.next_timer_token += 1;
            token
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            let token = ClipboardToken(self.next_clipboard_token);
            self.next_clipboard_token += 1;
            token
        }

        fn next_image_upload_token(&mut self) -> ImageUploadToken {
            let token = ImageUploadToken(self.next_image_upload_token);
            self.next_image_upload_token += 1;
            token
        }
    }

    impl EffectSink for TestHost {
        fn request_redraw(&mut self, window: AppWindowId) {
            self.redraws.insert(window);
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }
    }

    impl ModelHost for TestHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl InboxDrainHost for TestHost {
        fn request_redraw(&mut self, window: AppWindowId) {
            <Self as EffectSink>::request_redraw(self, window);
        }

        fn push_effect(&mut self, effect: Effect) {
            <Self as EffectSink>::push_effect(self, effect);
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl TestHost {
        fn set_frame(&mut self, frame: u64) {
            self.frame_id = FrameId(frame);
        }
    }

    fn install_bundle_image_asset(
        host: &mut TestHost,
        revision: AssetRevision,
        bytes: impl Into<Arc<[u8]>>,
    ) {
        let mut resolver = InMemoryAssetResolver::new();
        resolver.insert_bundle("app", "images/logo.png", revision, bytes);
        fret_runtime::set_asset_resolver(host, Arc::new(resolver));
    }

    #[cfg(feature = "ui")]
    fn resolve_bundle_image_request_state(
        host: &mut TestHost,
        window: AppWindowId,
        request: &AssetRequest,
    ) -> (
        ImageSource,
        fret_runtime::Model<ImageSourceUiSignal>,
        ImageSourceRequestKey,
        ImageSourceState,
    ) {
        let source = crate::resolve_image_source_from_host(host, request)
            .expect("image source should resolve");
        let reload_epoch = reload_epoch_for_source(host, &source);
        let request_key =
            request_key_for_source_with_epoch(&source, ImageSourceOptions::default(), reload_epoch);
        let model = with_image_source_loader(host, |loader, host| {
            loader.use_signal_model(host, &source, ImageSourceOptions::default())
        })
        .expect("dispatcher installed");
        let state = use_image_source_state(host, window, &source);
        (source, model, request_key, state)
    }

    fn image_source_entry_count(host: &mut TestHost) -> usize {
        with_image_source_loader(host, |loader, _host| {
            loader
                .runtime
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex")
                .len()
        })
        .expect("dispatcher installed")
    }

    fn image_source_has_entry(host: &mut TestHost, request: ImageSourceRequestKey) -> bool {
        with_image_source_loader(host, |loader, _host| {
            loader
                .runtime
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex")
                .contains_key(&request)
        })
        .expect("dispatcher installed")
    }

    #[test]
    fn image_source_rgba8_fast_path_schedules_register_effect() {
        let mut host = TestHost {
            frame_id: FrameId(1),
            ..Default::default()
        };
        let window = AppWindowId::default();

        let src = ImageSource::rgba8(
            1,
            1,
            Arc::<[u8]>::from([0u8, 0, 0, 255]),
            ImageColorSpace::Srgb,
        );
        let state = use_image_source_state(&mut host, window, &src);
        assert_eq!(state.status, ImageLoadingStatus::Loading);

        let token = host
            .effects
            .iter()
            .find_map(|e| match e {
                Effect::ImageRegisterRgba8 { token, .. } => Some(*token),
                _ => None,
            })
            .expect("expected ImageRegisterRgba8 effect");

        // Simulate the runner delivering ImageRegistered, then ensure the cache reports Loaded.
        let image = ImageId::default();
        let event = Event::ImageRegistered {
            token,
            image,
            width: 1,
            height: 1,
        };
        host.with_image_asset_cache(|cache, host| {
            assert!(cache.handle_event(host, window, &event));
        });

        let state2 = use_image_source_state(&mut host, window, &src);
        assert_eq!(state2.status, ImageLoadingStatus::Loaded);
        assert_eq!(state2.image, Some(image));
    }

    #[cfg(all(feature = "ui", feature = "image-decode", not(target_arch = "wasm32")))]
    #[test]
    fn image_source_test_jpg_drives_decode_and_gpu_ready_bumps_signal_model() {
        let dispatcher = Arc::new(QueuedDispatcher::default());
        let mut host = TestHost {
            frame_id: FrameId(1),
            ..Default::default()
        };
        host.set_global::<DispatcherHandle>(dispatcher.clone());
        let window = AppWindowId::default();

        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/textures/test.jpg");
        let bytes: Arc<[u8]> = std::fs::read(path)
            .expect("expected assets/textures/test.jpg to exist")
            .into();
        let src = ImageSource::from_bytes(bytes);

        let model = with_image_source_loader(&mut host, |loader, host| {
            loader.use_signal_model(host, &src, ImageSourceOptions::default())
        })
        .expect("dispatcher installed");
        let rev0 = host.models().revision(&model).unwrap_or(0);

        // Schedule decode (background) for the bytes source.
        let _ = use_image_source_state(&mut host, window, &src);
        dispatcher.run_background_tasks();

        // Drain + apply decode completion on the main thread.
        let runtime = with_image_source_loader(&mut host, |loader, _host| loader.runtime.clone())
            .expect("dispatcher installed");
        for msg in runtime.inbox.drain() {
            runtime.apply_msg(&mut host, msg);
        }
        let rev1 = host.models().revision(&model).unwrap_or(0);
        assert!(
            rev1 > rev0,
            "expected decode completion to bump signal model"
        );

        let request = ImageSourceRequestKey {
            source: src.id,
            color_space: ImageColorSpace::Srgb,
            reload_epoch: 0,
        };
        let (decoded_width, decoded_height) =
            with_image_source_loader(&mut host, |loader, _host| {
                let entries = loader
                    .runtime
                    .entries
                    .lock()
                    .expect("poisoned ImageSourceRuntime mutex");
                let entry = entries.get(&request).expect("expected entry after decode");
                match &entry.state {
                    ImageSourceEntryState::Decoded { decoded, .. } => {
                        (decoded.width, decoded.height)
                    }
                    ImageSourceEntryState::Failed { message, .. } => {
                        panic!("decode failed: {message}");
                    }
                    other => panic!("expected Decoded state after inbox apply, got {other:?}"),
                }
            })
            .expect("dispatcher installed");

        // Now that we're decoded, the next use should schedule a GPU upload.
        let state = use_image_source_state(&mut host, window, &src);
        let (width, height) = state
            .intrinsic_size_px
            .unwrap_or((decoded_width, decoded_height));
        let token = host
            .effects
            .iter()
            .find_map(|e| match e {
                Effect::ImageRegisterRgba8 { token, .. } => Some(*token),
                _ => None,
            })
            .expect("expected ImageRegisterRgba8 after decode");

        // GPU-ready event should bump the same signal model (per-key).
        let image = ImageId::default();
        let event = Event::ImageRegistered {
            token,
            image,
            width,
            height,
        };
        let _ = crate::UiAssets::handle_event(&mut host, window, &event);
        let rev2 = host.models().revision(&model).unwrap_or(0);
        assert!(rev2 > rev1, "expected GPU-ready to bump signal model");
    }

    #[cfg(all(feature = "image-decode", not(target_arch = "wasm32")))]
    #[test]
    fn image_source_path_request_key_includes_reload_epoch() {
        let dispatcher = Arc::new(QueuedDispatcher::default());
        let mut host = TestHost {
            frame_id: FrameId(1),
            ..Default::default()
        };
        host.set_global::<DispatcherHandle>(dispatcher.clone());
        host.set_global(crate::UiAssetsReloadEpoch(0));
        let window = AppWindowId::default();

        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/textures/test.jpg");
        let src = ImageSource::from_native_file_path(path);

        let _ = use_image_source_state(&mut host, window, &src);
        let request0 = ImageSourceRequestKey {
            source: src.id,
            color_space: ImageColorSpace::Srgb,
            reload_epoch: 0,
        };
        let has0 = with_image_source_loader(&mut host, |loader, _host| {
            loader
                .runtime
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex")
                .contains_key(&request0)
        })
        .expect("dispatcher installed");
        assert!(has0, "expected entry for reload_epoch=0");

        crate::bump_ui_assets_reload_epoch(&mut host);
        let _ = use_image_source_state(&mut host, window, &src);
        let request1 = ImageSourceRequestKey {
            source: src.id,
            color_space: ImageColorSpace::Srgb,
            reload_epoch: 1,
        };
        let has1 = with_image_source_loader(&mut host, |loader, _host| {
            loader
                .runtime
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex")
                .contains_key(&request1)
        })
        .expect("dispatcher installed");
        assert!(has1, "expected entry for reload_epoch=1");
    }

    #[test]
    fn image_source_entries_gc_removes_stale_ready_entries() {
        let dispatcher = Arc::new(QueuedDispatcher::default());
        let mut host = TestHost {
            frame_id: FrameId(1),
            ..Default::default()
        };
        host.set_global::<DispatcherHandle>(dispatcher.clone());

        let window = AppWindowId::default();

        let src1: ImageSource = {
            let bytes: Arc<[u8]> = vec![0u8; 16].into();
            ImageSource::from_bytes(bytes)
        };
        let _ = use_image_source_state(&mut host, window, &src1);

        let request1 = ImageSourceRequestKey {
            source: src1.id,
            color_space: ImageColorSpace::Srgb,
            reload_epoch: 0,
        };

        // Make entry1 eligible for GC by forcing it into a stable state.
        with_image_source_loader(&mut host, |loader, _host| {
            let mut entries = loader
                .runtime
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex");
            let entry = entries.get_mut(&request1).expect("expected entry for src1");
            entry.state = ImageSourceEntryState::Ready {
                asset_key: ImageAssetKey::from_rgba8(1, 1, ImageColorSpace::Srgb, &[0, 0, 0, 0]),
                intrinsic_size_px: (1, 1),
            };
        })
        .expect("dispatcher installed");

        let len1 = with_image_source_loader(&mut host, |loader, _host| {
            loader
                .runtime
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex")
                .len()
        })
        .unwrap();
        assert_eq!(len1, 1);

        // Advance far enough that the entry becomes stale, then create a new request to trigger GC.
        host.set_frame(2200);
        let src2: ImageSource = {
            let bytes: Arc<[u8]> = vec![1u8; 16].into();
            ImageSource::from_bytes(bytes)
        };
        let _ = use_image_source_state(&mut host, window, &src2);

        let len2 = with_image_source_loader(&mut host, |loader, _host| {
            loader
                .runtime
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex")
                .len()
        })
        .unwrap();
        assert_eq!(len2, 1);

        let has_src1 = with_image_source_loader(&mut host, |loader, _host| {
            loader
                .runtime
                .entries
                .lock()
                .expect("poisoned ImageSourceRuntime mutex")
                .contains_key(&request1)
        })
        .unwrap();
        assert!(!has_src1);
    }

    #[test]
    fn image_source_does_not_retain_source_bytes_after_background_tasks_are_dropped() {
        let dispatcher = Arc::new(QueuedDispatcher::default());
        let mut host = TestHost {
            frame_id: FrameId(1),
            ..Default::default()
        };
        host.set_global::<DispatcherHandle>(dispatcher.clone());

        let window = AppWindowId::default();

        let bytes: Arc<[u8]> = vec![0u8; 1024].into();
        let weak = Arc::downgrade(&bytes);
        let src = ImageSource::from_bytes(bytes.clone());

        let _ = use_image_source_state(&mut host, window, &src);
        drop(src);
        drop(bytes);

        assert!(
            weak.upgrade().is_some(),
            "expected queued task to retain bytes"
        );

        dispatcher.drop_background_tasks();
        assert!(
            weak.upgrade().is_none(),
            "expected bytes to be released after dropping queued background tasks"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn image_source_can_bridge_from_file_asset_locator() {
        let locator = AssetLocator::file("assets/textures/test.jpg");
        let source = ImageSource::from_asset_locator(&locator).expect("file locator should bridge");
        let expected = ImageSource::from_native_file_path(std::path::PathBuf::from(
            "assets/textures/test.jpg",
        ));
        assert_eq!(source.id(), expected.id());
    }

    #[test]
    fn resolved_asset_bytes_image_source_id_tracks_locator_and_revision() {
        let locator = AssetLocator::bundle("app", "images/logo.png");
        let rev1 =
            ResolvedAssetBytes::new(locator.clone(), fret_assets::AssetRevision(1), [1u8, 2, 3]);
        let rev1_again =
            ResolvedAssetBytes::new(locator.clone(), fret_assets::AssetRevision(1), [9u8, 9, 9]);
        let rev2 = ResolvedAssetBytes::new(locator, fret_assets::AssetRevision(2), [1u8, 2, 3]);

        let src1 = ImageSource::from_resolved_asset_bytes(&rev1);
        let src1_again = ImageSource::from_resolved_asset_bytes(&rev1_again);
        let src2 = ImageSource::from_resolved_asset_bytes(&rev2);

        assert_eq!(src1.id(), src1_again.id());
        assert_ne!(src1.id(), src2.id());
    }

    #[cfg(feature = "ui")]
    #[test]
    fn bundle_asset_request_same_revision_reuses_signal_model_and_request_key() {
        let dispatcher = Arc::new(QueuedDispatcher::default());
        let mut host = TestHost {
            frame_id: FrameId(1),
            ..Default::default()
        };
        host.set_global::<DispatcherHandle>(dispatcher);
        let window = AppWindowId::default();
        let request = AssetRequest::new(AssetLocator::bundle("app", "images/logo.png"));

        install_bundle_image_asset(&mut host, AssetRevision(1), [1u8, 2, 3, 4]);
        let (source1, model1, request_key1, state1) =
            resolve_bundle_image_request_state(&mut host, window, &request);
        assert_eq!(state1.status, ImageLoadingStatus::Loading);
        assert!(image_source_has_entry(&mut host, request_key1));
        assert_eq!(image_source_entry_count(&mut host), 1);

        install_bundle_image_asset(&mut host, AssetRevision(1), [9u8, 9, 9, 9]);
        let (source2, model2, request_key2, state2) =
            resolve_bundle_image_request_state(&mut host, window, &request);

        assert_eq!(state2.status, ImageLoadingStatus::Loading);
        assert_eq!(source1.id(), source2.id());
        assert_eq!(model1.id(), model2.id());
        assert_eq!(request_key1, request_key2);
        assert!(image_source_has_entry(&mut host, request_key2));
        assert_eq!(image_source_entry_count(&mut host), 1);
    }

    #[cfg(feature = "ui")]
    #[test]
    fn bundle_asset_request_revision_change_creates_new_signal_model_and_request_key() {
        let dispatcher = Arc::new(QueuedDispatcher::default());
        let mut host = TestHost {
            frame_id: FrameId(1),
            ..Default::default()
        };
        host.set_global::<DispatcherHandle>(dispatcher);
        let window = AppWindowId::default();
        let request = AssetRequest::new(AssetLocator::bundle("app", "images/logo.png"));

        install_bundle_image_asset(&mut host, AssetRevision(1), [1u8, 2, 3, 4]);
        let (source1, model1, request_key1, state1) =
            resolve_bundle_image_request_state(&mut host, window, &request);
        assert_eq!(state1.status, ImageLoadingStatus::Loading);
        assert!(image_source_has_entry(&mut host, request_key1));

        install_bundle_image_asset(&mut host, AssetRevision(2), [9u8, 9, 9, 9]);
        let (source2, model2, request_key2, state2) =
            resolve_bundle_image_request_state(&mut host, window, &request);

        assert_eq!(state2.status, ImageLoadingStatus::Loading);
        assert_ne!(source1.id(), source2.id());
        assert_ne!(model1.id(), model2.id());
        assert_ne!(request_key1, request_key2);
        assert!(image_source_has_entry(&mut host, request_key1));
        assert!(image_source_has_entry(&mut host, request_key2));
        assert_eq!(image_source_entry_count(&mut host), 2);
    }
}
