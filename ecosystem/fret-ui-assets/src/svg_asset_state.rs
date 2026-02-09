use fret_core::{SvgId, UiServices};
use fret_runtime::{GlobalsHost, TimeHost};

use crate::svg_asset_cache::{SvgAssetCache, SvgAssetKey, SvgAssetStats};

/// Convenience wrapper around `SvgAssetCache::use_svg_bytes`.
pub fn use_svg_bytes_cached<H: GlobalsHost + TimeHost>(
    host: &mut H,
    services: &mut dyn UiServices,
    bytes: &[u8],
) -> (SvgAssetKey, SvgId) {
    use crate::svg_asset_cache::SvgAssetCacheHostExt as _;

    host.with_svg_asset_cache(|cache, host| cache.use_svg_bytes(host, services, bytes))
}

/// Convenience wrapper around `SvgAssetCache::use_svg_bytes` that also reports `SvgAssetStats`.
pub fn use_svg_bytes_cached_with_stats<H: GlobalsHost + TimeHost>(
    host: &mut H,
    services: &mut dyn UiServices,
    bytes: &[u8],
) -> (SvgAssetKey, SvgId, SvgAssetStats) {
    use crate::svg_asset_cache::SvgAssetCacheHostExt as _;

    host.with_svg_asset_cache(|cache, host| {
        let (key, svg) = cache.use_svg_bytes(host, services, bytes);
        let stats = cache.stats();
        (key, svg, stats)
    })
}

/// Convenience wrapper around `SvgAssetCache::use_svg_bytes_keyed` that also reports `SvgAssetStats`.
pub fn use_svg_bytes_keyed_cached_with_stats<H: GlobalsHost + TimeHost>(
    host: &mut H,
    services: &mut dyn UiServices,
    key: SvgAssetKey,
    bytes: &[u8],
) -> (SvgId, SvgAssetStats) {
    use crate::svg_asset_cache::SvgAssetCacheHostExt as _;

    host.with_svg_asset_cache(|cache, host| {
        let svg = cache.use_svg_bytes_keyed(host, services, key, bytes);
        let stats = cache.stats();
        (svg, stats)
    })
}

/// Returns the currently cached svg id for the provided key, if present.
pub fn svg_from_asset_cache(cache: &SvgAssetCache, key: SvgAssetKey) -> Option<SvgId> {
    cache.svg(key)
}

/// Returns stats for the global `SvgAssetCache`.
pub fn svg_asset_cache_stats<H: GlobalsHost>(host: &mut H) -> SvgAssetStats {
    use crate::svg_asset_cache::SvgAssetCacheHostExt as _;

    host.with_svg_asset_cache(|cache, _host| cache.stats())
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_core::{
        ClipboardToken, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
        PathStyle, Px, Size, TextBlobId, TextConstraints, TextMetrics, TextService, TimerToken,
    };
    use fret_runtime::TickId;
    use slotmap::KeyData;

    use super::*;

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
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

        fn next_image_upload_token(&mut self) -> fret_core::ImageUploadToken {
            let token = fret_core::ImageUploadToken(self.next_image_upload_token);
            self.next_image_upload_token += 1;
            token
        }
    }

    #[derive(Default)]
    struct FakeUiServices {
        svg_next: u64,
    }

    impl fret_core::SvgService for FakeUiServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            let id = SvgId::from(KeyData::from_ffi(self.svg_next));
            self.svg_next += 1;
            id
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    #[test]
    fn use_svg_bytes_cached_is_stable_for_same_bytes() {
        let mut host = TestHost::default();
        host.set_frame(1);
        let mut services = FakeUiServices::default();

        let bytes = br#"<svg viewBox="0 0 1 1"></svg>"#;
        let (k1, id1) = use_svg_bytes_cached(&mut host, &mut services, bytes);
        let (k2, id2) = use_svg_bytes_cached(&mut host, &mut services, bytes);
        assert_eq!(k1.as_u64(), k2.as_u64());
        assert_eq!(id1, id2);

        let stats = svg_asset_cache_stats(&mut host);
        assert_eq!(stats.ready_count, 1);
    }

    #[test]
    fn use_svg_bytes_cached_with_stats_reports_stats() {
        let mut host = TestHost::default();
        host.set_frame(1);
        let mut services = FakeUiServices::default();

        let bytes = br#"<svg viewBox="0 0 1 1"></svg>"#;
        let (_k, _id, stats) = use_svg_bytes_cached_with_stats(&mut host, &mut services, bytes);
        assert_eq!(stats.ready_count, 1);
    }

    #[test]
    fn use_svg_bytes_keyed_cached_with_stats_reuses_cache_entry() {
        let mut host = TestHost::default();
        host.set_frame(1);
        let mut services = FakeUiServices::default();

        let bytes = br#"<svg viewBox="0 0 1 1"></svg>"#;
        let (key, id1) = use_svg_bytes_cached(&mut host, &mut services, bytes);
        let (id2, stats) =
            use_svg_bytes_keyed_cached_with_stats(&mut host, &mut services, key, bytes);
        assert_eq!(id1, id2);
        assert_eq!(stats.ready_count, 1);
    }
}
