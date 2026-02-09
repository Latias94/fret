//! Ecosystem-level `img(source)` helpers: decode/load + `ImageAssetCache` integration.
//!
//! This module is intentionally policy-layer code:
//! - It does not change the mechanism-layer `fret-ui` contract surface.
//! - It reuses `ImageAssetCache` (ADR 0004 flush-point resources) for GPU registration and budgets.
//! - Decoding runs off-thread via the runner-provided `DispatcherHandle` (ADR 0190).
//!
//! The primary goal is to avoid ad-hoc per-app crop/math and decode pipelines for common needs like
//! shadcn-style avatars and thumbnail grids.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use fret_core::{AppWindowId, ImageColorSpace, ImageId};
use fret_executor::{Inbox, InboxConfig, InboxDrainer};
use fret_runtime::{
    DispatchPriority, DispatcherHandle, EffectSink, GlobalsHost, InboxDrainHost,
    InboxDrainRegistry, TimeHost,
};

use crate::image_asset_cache::{ImageAssetCacheHostExt, ImageAssetKey};
use crate::image_asset_state::{ImageLoadingStatus, image_state_from_asset_cache};

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

    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        let bytes: Arc<[u8]> = bytes.into();
        let id = ImageSourceId(stable_hash(&(b"bytes.v1", bytes.as_ref())));
        Self {
            id,
            kind: ImageSourceKind::Bytes { bytes },
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
    pub fn from_path(path: impl Into<Arc<PathBuf>>) -> Self {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ImageSourceRequestKey {
    source: ImageSourceId,
    color_space: ImageColorSpace,
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
    source: ImageSource,
    state: ImageSourceEntryState,
}

#[derive(Debug)]
struct ImageSourceMsg {
    request: ImageSourceRequestKey,
    window: AppWindowId,
    inflight_id: u64,
    attempt_frame: u64,
    result: Result<DecodedRgba8, String>,
}

pub struct ImageSourceLoader {
    runtime: Arc<ImageSourceRuntime>,
}

impl ImageSourceLoader {
    fn new(dispatcher: DispatcherHandle) -> Self {
        Self {
            runtime: Arc::new(ImageSourceRuntime::new(dispatcher)),
        }
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
            source: source.clone(),
            state: ImageSourceEntryState::Idle,
        });

        // Keep the latest source clone around (cheap; `Arc`-backed).
        entry.source = source.clone();

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

    fn apply_msg(&self, host: &mut dyn InboxDrainHost, msg: ImageSourceMsg) {
        let mut entries = self
            .runtime
            .entries
            .lock()
            .expect("poisoned ImageSourceRuntime mutex");
        let Some(entry) = entries.get_mut(&msg.request) else {
            return;
        };

        let inflight_matches = match &entry.state {
            ImageSourceEntryState::Loading { inflight_id, .. } => *inflight_id == msg.inflight_id,
            ImageSourceEntryState::Decoded { inflight_id, .. } => *inflight_id == msg.inflight_id,
            ImageSourceEntryState::Idle
            | ImageSourceEntryState::Ready { .. }
            | ImageSourceEntryState::Failed { .. } => false,
        };
        if !inflight_matches {
            return;
        }

        match msg.result {
            Ok(decoded) => {
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
            }
            Err(err) => {
                entry.state = ImageSourceEntryState::Failed {
                    message: Arc::<str>::from(err),
                    last_attempt_frame: msg.attempt_frame,
                };
            }
        }

        host.request_redraw(msg.window);
    }
}

struct ImageSourceRuntime {
    dispatcher: DispatcherHandle,
    inbox: Inbox<ImageSourceMsg>,
    registered: AtomicBool,
    next_inflight_id: AtomicU64,
    entries: Mutex<HashMap<ImageSourceRequestKey, ImageSourceEntry>>,
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
            retry_cooldown_frames: 60,
        }
    }
}

fn image_source_inbox_drainer(runtime: Arc<ImageSourceRuntime>) -> InboxDrainer<ImageSourceMsg> {
    InboxDrainer::new(runtime.inbox.clone(), move |host, _window, msg| {
        // Apply via the global loader to keep all mutation main-thread-only.
        //
        // The loader itself is stored as a global, but the runtime is an `Arc` that we can mutate
        // safely from this drain boundary without needing `GlobalsHost`.
        let loader = ImageSourceLoader {
            runtime: runtime.clone(),
        };
        loader.apply_msg(host, msg);
    })
}

/// Access the global [`ImageSourceLoader`], returning `None` when the runner did not install a
/// `DispatcherHandle` global.
fn with_image_source_loader<H: GlobalsHost, R>(
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

    let request = ImageSourceRequestKey {
        source: source.id,
        color_space: options.color_space,
    };
    let frame = host.frame_id().0;

    let Some(state) = with_image_source_loader(host, |loader, host| {
        loader.ensure_registered(host);

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

    use fret_core::{
        AppWindowId, ClipboardToken, Event, FrameId, ImageColorSpace, ImageId, ImageUploadToken,
        TimerToken,
    };
    use fret_runtime::{Effect, GlobalsHost, TickId, TimeHost};

    use super::*;

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
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
}
