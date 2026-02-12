use std::hash::{Hash, Hasher};
use std::sync::Arc;

use fret_canvas::cache::{SvgBytes, SvgCache};
use fret_core::{SvgId, UiServices};
use fret_runtime::{GlobalsHost, TimeHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SvgAssetKey(u64);

impl SvgAssetKey {
    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut h);
        Self(h.finish())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SvgAssetStats {
    pub ready_count: usize,
    pub bytes_ready: u64,
    pub bytes_budget: u64,
}

#[derive(Debug)]
pub struct SvgAssetCache {
    cache: SvgCache,
    bytes_budget: u64,
    max_ready_entries: usize,
    last_frame_id: u64,
}

impl Default for SvgAssetCache {
    fn default() -> Self {
        Self {
            cache: SvgCache::default(),
            bytes_budget: 16 * 1024 * 1024,
            max_ready_entries: 4096,
            last_frame_id: 0,
        }
    }
}

impl SvgAssetCache {
    pub fn set_budget_bytes(&mut self, bytes_budget: u64) {
        self.bytes_budget = bytes_budget;
    }

    pub fn set_max_ready_entries(&mut self, max_ready_entries: usize) {
        self.max_ready_entries = max_ready_entries;
    }

    pub fn stats(&self) -> SvgAssetStats {
        SvgAssetStats {
            ready_count: self.cache.len(),
            bytes_ready: self.cache.bytes_ready(),
            bytes_budget: self.bytes_budget,
        }
    }

    fn begin_frame_if_needed(&mut self, frame: u64) {
        if frame != self.last_frame_id {
            self.cache.begin_frame();
            self.last_frame_id = frame;
        }
    }

    /// GPUI-style "use_asset": calling this repeatedly is cheap; it registers only on miss.
    pub fn use_svg_bytes<H: TimeHost>(
        &mut self,
        host: &H,
        services: &mut dyn UiServices,
        bytes: &[u8],
    ) -> (SvgAssetKey, SvgId) {
        let key = SvgAssetKey::from_bytes(bytes);
        let svg = self.use_svg_bytes_keyed(host, services, key, bytes);
        (key, svg)
    }

    pub fn use_svg_bytes_keyed<H: TimeHost>(
        &mut self,
        host: &H,
        services: &mut dyn UiServices,
        key: SvgAssetKey,
        bytes: &[u8],
    ) -> SvgId {
        let frame = host.frame_id().0;
        self.begin_frame_if_needed(frame);

        if let Some(svg) = self.cache.get(key.as_u64()) {
            return svg;
        }

        let bytes: Arc<[u8]> = Arc::from(bytes);
        let svg = self
            .cache
            .prepare(services, key.as_u64(), SvgBytes::Bytes(bytes));
        self.cache.prune_with_budget(
            services,
            u64::MAX,
            self.max_ready_entries,
            self.bytes_budget,
        );
        svg
    }

    pub fn svg(&self, key: SvgAssetKey) -> Option<SvgId> {
        self.cache.peek(key.as_u64())
    }

    pub fn evict(&mut self, services: &mut dyn UiServices, key: SvgAssetKey) -> bool {
        self.cache.evict(services, key.as_u64())
    }
}

pub trait SvgAssetCacheHostExt: GlobalsHost {
    fn with_svg_asset_cache<R>(&mut self, f: impl FnOnce(&mut SvgAssetCache, &mut Self) -> R) -> R {
        self.with_global_mut(SvgAssetCache::default, f)
    }
}

impl<H: GlobalsHost> SvgAssetCacheHostExt for H {}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_core::{
        ClipboardToken, FrameId, PathId, SvgService, TextBlobId, TextMetrics, TextService,
        TimerToken,
    };
    use fret_runtime::TickId;
    use slotmap::KeyData;

    use super::*;

    #[derive(Default)]
    struct FakeUiServices {
        svg_next: u64,
        registered: Vec<Vec<u8>>,
        unregistered: Vec<SvgId>,
    }

    impl SvgService for FakeUiServices {
        fn register_svg(&mut self, bytes: &[u8]) -> SvgId {
            let id = SvgId::from(KeyData::from_ffi(self.svg_next));
            self.svg_next += 1;
            self.registered.push(bytes.to_vec());
            id
        }

        fn unregister_svg(&mut self, svg: SvgId) -> bool {
            self.unregistered.push(svg);
            true
        }
    }

    impl TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::default(),
                    baseline: fret_core::Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl fret_core::PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (PathId, fret_core::PathMetrics) {
            (PathId::default(), fret_core::PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl fret_core::MaterialService for FakeUiServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            false
        }
    }

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        frame_id: FrameId,
        tick_id: TickId,
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
            let out = TimerToken(self.next_timer_token);
            self.next_timer_token += 1;
            out
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            let out = ClipboardToken(self.next_clipboard_token);
            self.next_clipboard_token += 1;
            out
        }

        fn next_image_upload_token(&mut self) -> fret_core::ImageUploadToken {
            let out = fret_core::ImageUploadToken(self.next_image_upload_token);
            self.next_image_upload_token += 1;
            out
        }
    }

    #[test]
    fn use_svg_registers_only_once_per_key() {
        let mut host = TestHost::default();
        host.set_frame(1);
        let mut services = FakeUiServices::default();
        let mut cache = SvgAssetCache::default();

        let bytes = b"<svg></svg>";
        let key = SvgAssetKey::from_bytes(bytes);

        let svg1 = cache.use_svg_bytes_keyed(&host, &mut services, key, bytes);
        let svg2 = cache.use_svg_bytes_keyed(&host, &mut services, key, bytes);

        assert_eq!(svg1, svg2);
        assert_eq!(services.registered.len(), 1);
        assert_eq!(cache.svg(key), Some(svg1));
    }

    #[test]
    fn prune_evicts_lru_and_unregisters() {
        let mut host = TestHost::default();
        let mut services = FakeUiServices::default();
        let mut cache = SvgAssetCache::default();
        cache.set_budget_bytes(6);

        host.set_frame(1);
        let a = b"aaaa";
        let key_a = SvgAssetKey::from_bytes(a);
        let svg_a = cache.use_svg_bytes_keyed(&host, &mut services, key_a, a);

        host.set_frame(2);
        let b = b"bbbb";
        let key_b = SvgAssetKey::from_bytes(b);
        let svg_b = cache.use_svg_bytes_keyed(&host, &mut services, key_b, b);

        // Each entry counts its raw bytes; budget=6 means we must evict one after inserting the second.
        assert!(cache.svg(key_a).is_none() || cache.svg(key_b).is_none());
        assert!(
            services.unregistered.contains(&svg_a) || services.unregistered.contains(&svg_b),
            "expected one unregister on prune"
        );
    }
}
