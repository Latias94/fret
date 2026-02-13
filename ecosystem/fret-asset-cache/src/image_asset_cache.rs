use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use fret_core::{AppWindowId, Event, ImageColorSpace, ImageId, ImageUploadToken};
use fret_runtime::{Effect, EffectSink, GlobalsHost, TimeHost};

use crate::image_upload::{ImageMeta, ImageUploadService};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageAssetKey(u64);

impl ImageAssetKey {
    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn from_rgba8(width: u32, height: u32, color_space: ImageColorSpace, rgba: &[u8]) -> Self {
        rgba8_key(width, height, color_space, rgba)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageAssetStats {
    pub ready_count: usize,
    pub pending_count: usize,
    pub failed_count: usize,
    pub bytes_ready: u64,
    pub bytes_budget: u64,
}

#[derive(Debug)]
enum ImageAssetState {
    Pending {
        waiters: Vec<AppWindowId>,
        token: ImageUploadToken,
        meta: ImageMeta,
    },
    Ready {
        image: ImageId,
        meta: ImageMeta,
        bytes: u64,
        last_used_frame: u64,
    },
    Failed {
        waiters: Vec<AppWindowId>,
        message: String,
        last_attempt_frame: u64,
    },
}

#[derive(Debug)]
struct ImageAssetEntry {
    state: ImageAssetState,
}

#[derive(Debug)]
pub struct ImageAssetCache {
    entries: HashMap<ImageAssetKey, ImageAssetEntry>,
    token_to_key: HashMap<ImageUploadToken, ImageAssetKey>,
    bytes_budget: u64,
    bytes_ready: u64,
    max_ready_entries: usize,
    retry_cooldown_frames: u64,
}

impl Default for ImageAssetCache {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            token_to_key: HashMap::new(),
            bytes_budget: 64 * 1024 * 1024,
            bytes_ready: 0,
            max_ready_entries: 4096,
            retry_cooldown_frames: 60,
        }
    }
}

impl ImageAssetCache {
    pub fn set_budget_bytes(&mut self, bytes_budget: u64) {
        self.bytes_budget = bytes_budget;
    }

    pub fn set_max_ready_entries(&mut self, max_ready_entries: usize) {
        self.max_ready_entries = max_ready_entries;
    }

    pub fn key_for_token(&self, token: ImageUploadToken) -> Option<ImageAssetKey> {
        self.token_to_key.get(&token).copied()
    }

    pub fn stats(&self) -> ImageAssetStats {
        let mut ready_count = 0usize;
        let mut pending_count = 0usize;
        let mut failed_count = 0usize;
        for e in self.entries.values() {
            match e.state {
                ImageAssetState::Ready { .. } => ready_count += 1,
                ImageAssetState::Pending { .. } => pending_count += 1,
                ImageAssetState::Failed { .. } => failed_count += 1,
            }
        }
        ImageAssetStats {
            ready_count,
            pending_count,
            failed_count,
            bytes_ready: self.bytes_ready,
            bytes_budget: self.bytes_budget,
        }
    }

    /// GPUI-style "use_asset": calling this repeatedly is cheap; it schedules work only on miss.
    ///
    /// Returns `(key, image)` where `image` is `Some` only when ready.
    pub fn use_rgba8<H: GlobalsHost + TimeHost + EffectSink>(
        &mut self,
        host: &mut H,
        window: AppWindowId,
        width: u32,
        height: u32,
        rgba: &[u8],
        color_space: ImageColorSpace,
    ) -> (ImageAssetKey, Option<ImageId>) {
        let key = ImageAssetKey::from_rgba8(width, height, color_space, rgba);
        let image = self.use_rgba8_keyed(host, window, key, width, height, rgba, color_space);
        (key, image)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn use_rgba8_keyed<H: GlobalsHost + TimeHost + EffectSink>(
        &mut self,
        host: &mut H,
        window: AppWindowId,
        key: ImageAssetKey,
        width: u32,
        height: u32,
        rgba: &[u8],
        color_space: ImageColorSpace,
    ) -> Option<ImageId> {
        let frame = host.frame_id().0;
        self.use_rgba8_keyed_inner(host, window, key, frame, width, height, rgba, color_space);
        self.image(key)
    }

    pub fn image(&self, key: ImageAssetKey) -> Option<ImageId> {
        match self.entries.get(&key)?.state {
            ImageAssetState::Ready { image, .. } => Some(image),
            ImageAssetState::Pending { .. } | ImageAssetState::Failed { .. } => None,
        }
    }

    pub fn image_meta(&self, key: ImageAssetKey) -> Option<ImageMeta> {
        match self.entries.get(&key)?.state {
            ImageAssetState::Ready { meta, .. } => Some(meta),
            ImageAssetState::Pending { meta, .. } => Some(meta),
            ImageAssetState::Failed { .. } => None,
        }
    }

    pub fn error(&self, key: ImageAssetKey) -> Option<&str> {
        match &self.entries.get(&key)?.state {
            ImageAssetState::Failed { message, .. } => Some(message.as_str()),
            ImageAssetState::Pending { .. } | ImageAssetState::Ready { .. } => None,
        }
    }

    /// Drive the cache state machine using runner-delivered events.
    ///
    /// Returns `true` when an image transitions to ready/failed and a repaint is recommended.
    pub fn handle_event<H: GlobalsHost + TimeHost + EffectSink>(
        &mut self,
        host: &mut H,
        window: AppWindowId,
        event: &Event,
    ) -> bool {
        // Keep the underlying primitive service up to date (it is also used directly elsewhere).
        host.with_global_mut(ImageUploadService::default, |svc, _host| {
            svc.handle_event(event);
        });

        match event {
            Event::ImageRegistered {
                token,
                image,
                width,
                height,
            } => {
                let Some(key) = self.token_to_key.remove(token) else {
                    tracing::warn!(token = ?token, "image_asset_cache: ImageRegistered missing token mapping");
                    return false;
                };
                tracing::debug!(
                    token = ?token,
                    key = ?key,
                    image = ?image,
                    width = *width,
                    height = *height,
                    "image_asset_cache: image registered"
                );
                let bytes = rgba8_bytes_len(*width, *height).unwrap_or(0);
                let frame = host.frame_id().0;
                let meta = ImageMeta {
                    width: *width,
                    height: *height,
                };

                if let Some(entry) = self.entries.get_mut(&key) {
                    let prev_bytes = match &entry.state {
                        ImageAssetState::Ready { bytes, .. } => *bytes,
                        _ => 0,
                    };
                    self.bytes_ready = self
                        .bytes_ready
                        .saturating_sub(prev_bytes)
                        .saturating_add(bytes);

                    let waiters = match &mut entry.state {
                        ImageAssetState::Pending { waiters, .. } => std::mem::take(waiters),
                        ImageAssetState::Failed { waiters, .. } => std::mem::take(waiters),
                        ImageAssetState::Ready { .. } => Vec::new(),
                    };
                    entry.state = ImageAssetState::Ready {
                        image: *image,
                        meta,
                        bytes,
                        last_used_frame: frame,
                    };
                    self.prune(host);
                    for w in waiters {
                        host.request_redraw(w);
                    }
                    host.request_redraw(window);
                    return true;
                }
                false
            }
            Event::ImageRegisterFailed { token, message } => {
                let Some(key) = self.token_to_key.remove(token) else {
                    tracing::warn!(
                        token = ?token,
                        message = %message,
                        "image_asset_cache: ImageRegisterFailed missing token mapping"
                    );
                    return false;
                };
                let frame = host.frame_id().0;
                if let Some(entry) = self.entries.get_mut(&key) {
                    if let ImageAssetState::Ready { bytes, .. } = &entry.state {
                        self.bytes_ready = self.bytes_ready.saturating_sub(*bytes);
                    }

                    let waiters = match &mut entry.state {
                        ImageAssetState::Pending { waiters, .. } => std::mem::take(waiters),
                        ImageAssetState::Failed { waiters, .. } => std::mem::take(waiters),
                        ImageAssetState::Ready { .. } => Vec::new(),
                    };
                    entry.state = ImageAssetState::Failed {
                        waiters,
                        message: message.clone(),
                        last_attempt_frame: frame,
                    };
                    let ImageAssetState::Failed { waiters, .. } = &entry.state else {
                        unreachable!("failed state must be set above");
                    };
                    for w in waiters {
                        host.request_redraw(*w);
                    }
                    host.request_redraw(window);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    pub fn evict<H: EffectSink>(&mut self, host: &mut H, key: ImageAssetKey) -> bool {
        let Some(entry) = self.entries.remove(&key) else {
            return false;
        };

        match entry.state {
            ImageAssetState::Ready { image, bytes, .. } => {
                self.bytes_ready = self.bytes_ready.saturating_sub(bytes);
                host.push_effect(Effect::ImageUnregister { image });
                true
            }
            ImageAssetState::Pending { token, .. } => {
                self.token_to_key.remove(&token);
                true
            }
            ImageAssetState::Failed { .. } => true,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn use_rgba8_keyed_inner<H: GlobalsHost + TimeHost + EffectSink>(
        &mut self,
        host: &mut H,
        window: AppWindowId,
        key: ImageAssetKey,
        frame: u64,
        width: u32,
        height: u32,
        rgba: &[u8],
        color_space: ImageColorSpace,
    ) {
        let expected_len = rgba8_bytes_len(width, height);
        if expected_len.is_none() {
            self.entries.insert(
                key,
                ImageAssetEntry {
                    state: ImageAssetState::Failed {
                        waiters: vec![window],
                        message: "invalid rgba8 dimensions (overflow)".to_string(),
                        last_attempt_frame: frame,
                    },
                },
            );
            return;
        }
        let expected_len = expected_len.unwrap();
        if rgba.len() != expected_len as usize {
            self.entries.insert(
                key,
                ImageAssetEntry {
                    state: ImageAssetState::Failed {
                        waiters: vec![window],
                        message: format!(
                            "invalid rgba8 byte length: expected {expected_len} for {width}x{height}, got {}",
                            rgba.len()
                        ),
                        last_attempt_frame: frame,
                    },
                },
            );
            return;
        }

        let Some(entry) = self.entries.get_mut(&key) else {
            self.schedule_upload(host, window, key, width, height, rgba, color_space);
            return;
        };

        match &mut entry.state {
            ImageAssetState::Ready {
                last_used_frame, ..
            } => {
                *last_used_frame = frame;
            }
            ImageAssetState::Pending { waiters, .. } => {
                push_waiter(waiters, window);
            }
            ImageAssetState::Failed {
                waiters,
                last_attempt_frame,
                ..
            } => {
                push_waiter(waiters, window);
                if frame.saturating_sub(*last_attempt_frame) >= self.retry_cooldown_frames {
                    self.schedule_upload(host, window, key, width, height, rgba, color_space);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn schedule_upload<H: GlobalsHost + TimeHost + EffectSink>(
        &mut self,
        host: &mut H,
        window: AppWindowId,
        key: ImageAssetKey,
        width: u32,
        height: u32,
        rgba: &[u8],
        color_space: ImageColorSpace,
    ) {
        let bytes = rgba.to_vec();
        let meta = ImageMeta { width, height };

        let token = host.with_global_mut(ImageUploadService::default, |svc, host| {
            svc.request_rgba8(host, window, width, height, bytes, color_space)
        });

        tracing::debug!(
            window = ?window,
            key = ?key,
            token = ?token,
            width,
            height,
            "image_asset_cache: scheduled upload"
        );

        if let Some(existing) = self.entries.get(&key)
            && let ImageAssetState::Pending { token, .. } = &existing.state
        {
            self.token_to_key.remove(token);
        }

        self.entries.insert(
            key,
            ImageAssetEntry {
                state: ImageAssetState::Pending {
                    waiters: vec![window],
                    token,
                    meta,
                },
            },
        );
        self.token_to_key.insert(token, key);
    }

    fn prune<H: EffectSink>(&mut self, host: &mut H) {
        loop {
            let ready_count = self
                .entries
                .values()
                .filter(|e| matches!(e.state, ImageAssetState::Ready { .. }))
                .count();

            let over_budget = self.bytes_ready > self.bytes_budget;
            let over_entries = ready_count > self.max_ready_entries;
            if !over_budget && !over_entries {
                break;
            }

            let mut victim: Option<(ImageAssetKey, u64)> = None;
            for (key, entry) in &self.entries {
                let ImageAssetState::Ready {
                    last_used_frame, ..
                } = entry.state
                else {
                    continue;
                };
                let replace = match victim {
                    None => true,
                    Some((_, cur)) => last_used_frame < cur,
                };
                if replace {
                    victim = Some((*key, last_used_frame));
                }
            }

            let Some((key, _)) = victim else {
                break;
            };
            let _ = self.evict(host, key);
        }
    }
}

pub trait ImageAssetCacheHostExt: GlobalsHost {
    fn with_image_asset_cache<R>(
        &mut self,
        f: impl FnOnce(&mut ImageAssetCache, &mut Self) -> R,
    ) -> R {
        self.with_global_mut(ImageAssetCache::default, f)
    }
}

impl<H: GlobalsHost> ImageAssetCacheHostExt for H {}

fn rgba8_bytes_len(width: u32, height: u32) -> Option<u64> {
    let pixels = (width as u64).checked_mul(height as u64)?;
    pixels.checked_mul(4)
}

fn rgba8_key(width: u32, height: u32, color_space: ImageColorSpace, rgba: &[u8]) -> ImageAssetKey {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    width.hash(&mut h);
    height.hash(&mut h);
    color_space.hash(&mut h);
    rgba.hash(&mut h);
    ImageAssetKey(h.finish())
}

fn push_waiter(waiters: &mut Vec<AppWindowId>, window: AppWindowId) {
    if !waiters.contains(&window) {
        waiters.push(window);
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::{HashMap, HashSet};

    use fret_core::{ClipboardToken, FrameId, ShareSheetToken, TimerToken};
    use fret_runtime::{Effect, GlobalsHost, TickId, TimeHost};
    use slotmap::KeyData;

    use super::*;

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        effects: Vec<Effect>,
        redraws: Vec<AppWindowId>,
        redraws_set: HashSet<AppWindowId>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
        next_image_upload_token: u64,
    }

    impl TestHost {
        fn set_frame(&mut self, frame: u64) {
            self.frame_id = FrameId(frame);
        }

        fn take_redraws(&mut self) -> Vec<AppWindowId> {
            self.redraws_set.clear();
            std::mem::take(&mut self.redraws)
        }
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

    impl EffectSink for TestHost {
        fn request_redraw(&mut self, window: AppWindowId) {
            if self.redraws_set.insert(window) {
                self.redraws.push(window);
            }
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
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
            let out = TimerToken(self.next_timer_token);
            self.next_timer_token += 1;
            out
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            let out = ClipboardToken(self.next_clipboard_token);
            self.next_clipboard_token += 1;
            out
        }

        fn next_share_sheet_token(&mut self) -> ShareSheetToken {
            let out = ShareSheetToken(self.next_share_sheet_token);
            self.next_share_sheet_token += 1;
            out
        }

        fn next_image_upload_token(&mut self) -> ImageUploadToken {
            let out = ImageUploadToken(self.next_image_upload_token);
            self.next_image_upload_token += 1;
            out
        }
    }

    fn window(id: u64) -> AppWindowId {
        AppWindowId::from(KeyData::from_ffi(id))
    }

    fn image(id: u64) -> ImageId {
        ImageId::from(KeyData::from_ffi(id))
    }

    fn solid_rgba8(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
        let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&rgba);
        }
        out
    }

    fn last_image_register_token(effects: &[Effect]) -> ImageUploadToken {
        effects
            .iter()
            .rev()
            .find_map(|e| match e {
                Effect::ImageRegisterRgba8 { token, .. } => Some(*token),
                _ => None,
            })
            .expect("expected an ImageRegisterRgba8 effect")
    }

    #[test]
    fn keyed_use_does_not_duplicate_register_effects() {
        let mut host = TestHost::default();
        host.set_frame(1);
        let win = window(1);

        let mut cache = ImageAssetCache::default();
        let bytes = solid_rgba8(4, 4, [1, 2, 3, 255]);
        let key = ImageAssetKey::from_rgba8(4, 4, ImageColorSpace::Srgb, &bytes);

        assert!(
            cache
                .use_rgba8_keyed(&mut host, win, key, 4, 4, &bytes, ImageColorSpace::Srgb)
                .is_none()
        );
        assert_eq!(
            host.effects
                .iter()
                .filter(|e| matches!(e, Effect::ImageRegisterRgba8 { .. }))
                .count(),
            1
        );

        // Same key again should not schedule another upload.
        assert!(
            cache
                .use_rgba8_keyed(&mut host, win, key, 4, 4, &bytes, ImageColorSpace::Srgb)
                .is_none()
        );
        assert_eq!(
            host.effects
                .iter()
                .filter(|e| matches!(e, Effect::ImageRegisterRgba8 { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn waiters_redraw_all_windows_on_ready() {
        let mut host = TestHost::default();
        host.set_frame(1);
        let w1 = window(1);
        let w2 = window(2);

        let mut cache = ImageAssetCache::default();
        let bytes = solid_rgba8(4, 4, [8, 9, 10, 255]);
        let key = ImageAssetKey::from_rgba8(4, 4, ImageColorSpace::Srgb, &bytes);

        let _ = cache.use_rgba8_keyed(&mut host, w1, key, 4, 4, &bytes, ImageColorSpace::Srgb);
        let _ = cache.use_rgba8_keyed(&mut host, w2, key, 4, 4, &bytes, ImageColorSpace::Srgb);

        assert_eq!(
            host.effects
                .iter()
                .filter(|e| matches!(e, Effect::ImageRegisterRgba8 { .. }))
                .count(),
            1
        );

        let token = last_image_register_token(&host.effects);
        let event = Event::ImageRegistered {
            token,
            image: image(101),
            width: 4,
            height: 4,
        };

        let _ = host.take_redraws();
        assert!(cache.handle_event(&mut host, w1, &event));
        let redraws = host.take_redraws();
        assert!(redraws.contains(&w1));
        assert!(redraws.contains(&w2));
    }

    #[test]
    fn failed_cooldown_retries_after_threshold() {
        let mut host = TestHost::default();
        let win = window(1);

        let mut cache = ImageAssetCache::default();
        let bytes = solid_rgba8(4, 4, [4, 5, 6, 255]);
        let key = ImageAssetKey::from_rgba8(4, 4, ImageColorSpace::Srgb, &bytes);

        host.set_frame(1);
        let _ = cache.use_rgba8_keyed(&mut host, win, key, 4, 4, &bytes, ImageColorSpace::Srgb);
        let token1 = last_image_register_token(&host.effects);
        let fail = Event::ImageRegisterFailed {
            token: token1,
            message: "boom".to_string(),
        };
        assert!(cache.handle_event(&mut host, win, &fail));

        // Before cooldown: should not schedule a retry.
        host.set_frame(10);
        let _ = cache.use_rgba8_keyed(&mut host, win, key, 4, 4, &bytes, ImageColorSpace::Srgb);
        assert_eq!(
            host.effects
                .iter()
                .filter(|e| matches!(e, Effect::ImageRegisterRgba8 { .. }))
                .count(),
            1
        );

        // After cooldown: should schedule a retry.
        host.set_frame(61);
        let _ = cache.use_rgba8_keyed(&mut host, win, key, 4, 4, &bytes, ImageColorSpace::Srgb);
        assert_eq!(
            host.effects
                .iter()
                .filter(|e| matches!(e, Effect::ImageRegisterRgba8 { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn over_budget_prune_evicts_lru_and_emits_unregister() {
        let mut host = TestHost::default();
        let win = window(1);

        let mut cache = ImageAssetCache::default();
        // Each 4x4 rgba8 is 64 bytes. Budget forces one eviction after the second image becomes ready.
        cache.set_budget_bytes(80);

        let bytes1 = solid_rgba8(4, 4, [1, 0, 0, 255]);
        let bytes2 = solid_rgba8(4, 4, [0, 1, 0, 255]);
        let key1 = ImageAssetKey::from_rgba8(4, 4, ImageColorSpace::Srgb, &bytes1);
        let key2 = ImageAssetKey::from_rgba8(4, 4, ImageColorSpace::Srgb, &bytes2);

        host.set_frame(1);
        let _ = cache.use_rgba8_keyed(&mut host, win, key1, 4, 4, &bytes1, ImageColorSpace::Srgb);
        let token1 = last_image_register_token(&host.effects);
        let img1 = image(201);
        let ready1 = Event::ImageRegistered {
            token: token1,
            image: img1,
            width: 4,
            height: 4,
        };
        assert!(cache.handle_event(&mut host, win, &ready1));

        host.set_frame(2);
        let _ = cache.use_rgba8_keyed(&mut host, win, key2, 4, 4, &bytes2, ImageColorSpace::Srgb);
        let token2 = last_image_register_token(&host.effects);
        let img2 = image(202);
        let ready2 = Event::ImageRegistered {
            token: token2,
            image: img2,
            width: 4,
            height: 4,
        };
        assert!(cache.handle_event(&mut host, win, &ready2));

        assert!(
            host.effects
                .iter()
                .any(|e| matches!(e, Effect::ImageUnregister { image } if *image == img1)),
            "expected LRU eviction to unregister first image"
        );
        assert_eq!(cache.image(key1), None);
        assert_eq!(cache.image(key2), Some(img2));
    }
}
