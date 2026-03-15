//! Development-oriented SVG file helpers.
//!
//! The goal is to make path-based SVGs usable in UI code without re-reading the file every frame,
//! while still allowing hot-reload via the shared `AssetReloadEpoch`.

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

use fret_assets::{AssetLoadError, AssetLocator};
use fret_runtime::AssetReloadEpoch;
use fret_runtime::GlobalsHost;
#[cfg(not(target_arch = "wasm32"))]
use fret_runtime::TimeHost;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct SvgFileSource {
    pub(crate) path: Arc<PathBuf>,
}

#[cfg(not(target_arch = "wasm32"))]
impl SvgFileSource {
    pub(crate) fn from_native_file_path(path: impl Into<Arc<PathBuf>>) -> Self {
        Self { path: path.into() }
    }

    pub fn from_asset_locator(locator: &AssetLocator) -> Result<Self, AssetLoadError> {
        match locator {
            AssetLocator::File(file) => Ok(Self::from_native_file_path(file.path.clone())),
            _ => Err(AssetLoadError::UnsupportedLocatorKind {
                kind: locator.kind(),
            }),
        }
    }

    #[deprecated(
        note = "prefer locator-first asset requests and UI helpers; direct file paths are a native/dev-only compatibility seam"
    )]
    pub fn from_file_path(path: impl Into<Arc<PathBuf>>) -> Self {
        Self::from_native_file_path(path)
    }

    #[deprecated(
        note = "prefer locator-first asset requests and UI helpers; direct file paths are a native/dev-only compatibility seam"
    )]
    pub fn from_path(path: impl Into<Arc<PathBuf>>) -> Self {
        Self::from_native_file_path(path)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Default)]
struct SvgFileCache {
    entries: std::collections::HashMap<PathBuf, SvgFileCacheEntry>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct SvgFileCacheEntry {
    epoch: u64,
    bytes: Option<Arc<[u8]>>,
    error: Option<Arc<str>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for SvgFileCacheEntry {
    fn default() -> Self {
        Self {
            epoch: u64::MAX,
            bytes: None,
            error: None,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct SvgFileState {
    pub bytes: Option<Arc<[u8]>>,
    pub error: Option<Arc<str>>,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_svg_file_cached<H: GlobalsHost + TimeHost>(
    host: &mut H,
    source: &SvgFileSource,
) -> SvgFileState {
    let epoch = host.global::<AssetReloadEpoch>().map(|v| v.0).unwrap_or(0);

    let path = source.path.as_ref();
    host.with_global_mut_untracked(SvgFileCache::default, |cache, _host| {
        let entry = cache
            .entries
            .entry(path.clone())
            .or_insert_with(SvgFileCacheEntry::default);
        if entry.epoch != epoch {
            match std::fs::read(path) {
                Ok(bytes) => {
                    entry.epoch = epoch;
                    entry.bytes = Some(Arc::<[u8]>::from(bytes));
                    entry.error = None;
                }
                Err(e) => {
                    entry.epoch = epoch;
                    entry.bytes = None;
                    entry.error = Some(Arc::<str>::from(e.to_string()));
                }
            }
        }
        SvgFileState {
            bytes: entry.bytes.clone(),
            error: entry.error.clone(),
        }
    })
}

#[cfg(all(feature = "ui", not(target_arch = "wasm32")))]
pub fn svg_source_from_file_cached<H: GlobalsHost + TimeHost>(
    host: &mut H,
    source: &SvgFileSource,
) -> Result<fret_ui::SvgSource, Arc<str>> {
    let state = read_svg_file_cached(host, source);
    if let Some(err) = state.error {
        return Err(err);
    }
    let Some(bytes) = state.bytes else {
        return Err(Arc::<str>::from("missing svg bytes"));
    };
    Ok(fret_ui::SvgSource::Bytes(bytes))
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_core::{ClipboardToken, FrameId, ImageUploadToken, TimerToken};
    use fret_runtime::TickId;

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

    #[test]
    fn svg_file_cache_respects_epoch_bumps() {
        let mut host = TestHost::default();
        host.set_global(AssetReloadEpoch(0));

        let mut tmp = std::env::temp_dir();
        let unique = format!(
            "fret_svg_file_cache_test_{}_{}.svg",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        tmp.push(unique);

        std::fs::write(&tmp, br#"<svg viewBox="0 0 1 1"></svg>"#).expect("write temp svg");
        let src = SvgFileSource::from_native_file_path(Arc::new(tmp.clone()));

        let s0 = read_svg_file_cached(&mut host, &src);
        assert!(s0.error.is_none());
        let b0 = s0.bytes.expect("bytes");

        std::fs::write(&tmp, br#"<svg viewBox="0 0 2 2"></svg>"#).expect("rewrite temp svg");
        // No epoch bump => cached bytes remain.
        let s_same = read_svg_file_cached(&mut host, &src);
        let b_same = s_same.bytes.expect("bytes");
        assert_eq!(b0.as_ref(), b_same.as_ref());

        // Bump epoch => re-read and observe new bytes.
        fret_runtime::bump_asset_reload_epoch(&mut host);
        let s1 = read_svg_file_cached(&mut host, &src);
        let b1 = s1.bytes.expect("bytes");
        assert_ne!(b0.as_ref(), b1.as_ref());
    }

    #[test]
    fn svg_file_source_can_bridge_from_file_asset_locator() {
        let locator = AssetLocator::file("assets/demo/icon-search.svg");
        let source =
            SvgFileSource::from_asset_locator(&locator).expect("file locator should bridge");
        let expected = SvgFileSource::from_native_file_path(std::path::PathBuf::from(
            "assets/demo/icon-search.svg",
        ));
        assert_eq!(source.path.as_ref(), expected.path.as_ref());
    }
}
