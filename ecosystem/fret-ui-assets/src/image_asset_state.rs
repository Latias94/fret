use fret_core::{AppWindowId, ImageColorSpace, ImageId};
use fret_runtime::{EffectSink, GlobalsHost, TimeHost};

use crate::image_asset_cache::{ImageAssetCache, ImageAssetKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageLoadingStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

/// Maps an `ImageAssetCache` entry to a generic image loading status.
///
/// Mapping rules:
/// - `Ready` => `Loaded` with `Some(ImageId)`
/// - `Failed` => `Error` with `None`
/// - `Pending` => `Loading` with `None`
/// - unknown key => `Idle` with `None`
pub fn image_state_from_asset_cache(
    cache: &ImageAssetCache,
    key: ImageAssetKey,
) -> (Option<ImageId>, ImageLoadingStatus) {
    if let Some(image) = cache.image(key) {
        return (Some(image), ImageLoadingStatus::Loaded);
    }
    if cache.error(key).is_some() {
        return (None, ImageLoadingStatus::Error);
    }
    if cache.image_meta(key).is_some() {
        return (None, ImageLoadingStatus::Loading);
    }
    (None, ImageLoadingStatus::Idle)
}

/// Convenience wrapper around `ImageAssetCache::use_rgba8` that also reports a generic
/// `ImageLoadingStatus`.
pub fn use_rgba8_image_state<H: GlobalsHost + TimeHost + EffectSink>(
    host: &mut H,
    window: AppWindowId,
    width: u32,
    height: u32,
    rgba: &[u8],
    color_space: ImageColorSpace,
) -> (ImageAssetKey, Option<ImageId>, ImageLoadingStatus) {
    use crate::image_asset_cache::ImageAssetCacheHostExt as _;

    host.with_image_asset_cache(|cache, host| {
        let (key, image) = cache.use_rgba8(host, window, width, height, rgba, color_space);
        let (_, status) = image_state_from_asset_cache(cache, key);
        (key, image, status)
    })
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::{HashMap, HashSet};

    use fret_core::{ClipboardToken, Event, FrameId, ImageUploadToken, TimerToken};
    use fret_runtime::{Effect, TickId};

    use super::*;

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        effects: Vec<Effect>,
        redraws_set: HashSet<AppWindowId>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_image_upload_token: u64,
    }

    impl TestHost {
        fn set_frame(&mut self, frame: u64) {
            self.frame_id = FrameId(frame);
        }

        fn take_effects(&mut self) -> Vec<Effect> {
            std::mem::take(&mut self.effects)
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
        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }

        fn request_redraw(&mut self, window: AppWindowId) {
            self.redraws_set.insert(window);
        }
    }

    fn extract_upload_token(effects: &[Effect]) -> ImageUploadToken {
        effects
            .iter()
            .find_map(|e| match e {
                Effect::ImageRegisterRgba8 { token, .. } => Some(*token),
                _ => None,
            })
            .expect("expected ImageRegisterRgba8 effect")
    }

    #[test]
    fn image_state_maps_pending_ready_failed_and_missing() {
        let mut host = TestHost::default();
        host.set_frame(1);
        let window = AppWindowId::default();
        let mut cache = ImageAssetCache::default();

        let unknown = ImageAssetKey::from_rgba8(1, 1, ImageColorSpace::Srgb, &[0, 0, 0, 0]);
        assert_eq!(
            image_state_from_asset_cache(&cache, unknown),
            (None, ImageLoadingStatus::Idle)
        );

        let rgba = [0u8; 4];
        let (key, _image) = cache.use_rgba8(&mut host, window, 1, 1, &rgba, ImageColorSpace::Srgb);
        assert_eq!(
            image_state_from_asset_cache(&cache, key),
            (None, ImageLoadingStatus::Loading)
        );

        let token = extract_upload_token(&host.take_effects());
        let image = ImageId::default();
        let event = Event::ImageRegistered {
            token,
            image,
            width: 1,
            height: 1,
        };
        assert!(cache.handle_event(&mut host, window, &event));
        assert_eq!(
            image_state_from_asset_cache(&cache, key),
            (Some(image), ImageLoadingStatus::Loaded)
        );

        let bad_rgba = [0u8; 3];
        let (bad_key, _bad_image) =
            cache.use_rgba8(&mut host, window, 1, 1, &bad_rgba, ImageColorSpace::Srgb);
        assert_eq!(
            image_state_from_asset_cache(&cache, bad_key),
            (None, ImageLoadingStatus::Error)
        );
    }

    #[test]
    fn use_rgba8_image_state_reports_loading_status() {
        let mut host = TestHost::default();
        host.set_frame(1);
        let window = AppWindowId::default();

        let rgba = [0u8; 4];
        let (_key, image, status) =
            use_rgba8_image_state(&mut host, window, 1, 1, &rgba, ImageColorSpace::Srgb);
        assert_eq!(image, None);
        assert_eq!(status, ImageLoadingStatus::Loading);
    }
}
