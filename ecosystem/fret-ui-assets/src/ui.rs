//! UI sugar for `fret-ui-assets`.
//!
//! This module is intentionally optional (feature-flagged) to keep `fret-ui-assets` usable in
//! non-`fret-ui` contexts while still providing ViewCache-safe ergonomics for UI authors.

#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use fret_assets::AssetExternalReference;
use fret_assets::{AssetLoadError, AssetLocator, AssetRequest};
use fret_runtime::AssetReloadEpoch;
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::image_asset_cache::ImageAssetKey;
use crate::image_source::{
    ImageSource, ImageSourceOptions, ImageSourceState, register_asset_key_for_source,
    with_image_source_loader,
};
#[cfg(not(target_arch = "wasm32"))]
pub trait ImageSourceElementContextExt {
    fn use_image_source_state(&mut self, source: &ImageSource) -> ImageSourceState;

    fn use_image_source_state_with_options(
        &mut self,
        source: &ImageSource,
        options: ImageSourceOptions,
    ) -> ImageSourceState;

    /// Like [`Self::use_image_source_state_with_options`], but allows callers to choose the
    /// invalidation kind used for the underlying async decode/GPU-ready signal.
    ///
    /// Use `Invalidation::Layout` when the returned [`ImageSourceState`] influences layout (for
    /// example, when you explicitly size to `intrinsic_size_px`).
    fn use_image_source_state_with_options_and_invalidation(
        &mut self,
        source: &ImageSource,
        options: ImageSourceOptions,
        invalidation: Invalidation,
    ) -> ImageSourceState;

    fn use_image_source_state_from_asset_request(
        &mut self,
        request: &AssetRequest,
    ) -> ImageSourceState;

    fn use_image_source_state_from_asset_request_with_options(
        &mut self,
        request: &AssetRequest,
        options: ImageSourceOptions,
    ) -> ImageSourceState;

    fn use_image_source_state_from_asset_request_with_options_and_invalidation(
        &mut self,
        request: &AssetRequest,
        options: ImageSourceOptions,
        invalidation: Invalidation,
    ) -> ImageSourceState;

    fn use_image_source_state_from_asset_locator(
        &mut self,
        locator: AssetLocator,
    ) -> ImageSourceState {
        self.use_image_source_state_from_asset_request(&AssetRequest::new(locator))
    }
}

impl<H: UiHost> ImageSourceElementContextExt for ElementContext<'_, H> {
    fn use_image_source_state(&mut self, source: &ImageSource) -> ImageSourceState {
        self.use_image_source_state_with_options(source, ImageSourceOptions::default())
    }

    fn use_image_source_state_with_options(
        &mut self,
        source: &ImageSource,
        options: ImageSourceOptions,
    ) -> ImageSourceState {
        self.use_image_source_state_with_options_and_invalidation(
            source,
            options,
            Invalidation::Paint,
        )
    }

    fn use_image_source_state_with_options_and_invalidation(
        &mut self,
        source: &ImageSource,
        options: ImageSourceOptions,
        invalidation: Invalidation,
    ) -> ImageSourceState {
        // ViewCache correctness for dev reloads: observe a global epoch that can be bumped when
        // path-based assets should be reloaded (without restarting the app).
        self.observe_global::<AssetReloadEpoch>(invalidation);

        // ViewCache correctness:
        //
        // - observe a per-request model that is updated when async decode completes (inbox drainer),
        // - bump the same model on GPU-ready events (via `UiAssets::handle_event` integration).
        //
        // Without these observations, a view-cached subtree may never re-render even though the
        // app requests redraws.
        if let Some(model) = with_image_source_loader(self.app, |loader, app| {
            loader.use_signal_model(app, source, options)
        }) {
            self.observe_model(&model, invalidation);
        }

        // Fast path: RGBA8 sources skip async decode, so we register the key→signal mapping here.
        if let Some((width, height, rgba, color_space)) = source.rgba8_meta() {
            let key = ImageAssetKey::from_rgba8(width, height, color_space, rgba.as_ref());
            register_asset_key_for_source(self.app, source, options, key);
        }

        crate::use_image_source_state_with_options(self.app, self.window, source, options)
    }

    fn use_image_source_state_from_asset_request(
        &mut self,
        request: &AssetRequest,
    ) -> ImageSourceState {
        self.use_image_source_state_from_asset_request_with_options(
            request,
            ImageSourceOptions::default(),
        )
    }

    fn use_image_source_state_from_asset_request_with_options(
        &mut self,
        request: &AssetRequest,
        options: ImageSourceOptions,
    ) -> ImageSourceState {
        self.use_image_source_state_from_asset_request_with_options_and_invalidation(
            request,
            options,
            Invalidation::Paint,
        )
    }

    fn use_image_source_state_from_asset_request_with_options_and_invalidation(
        &mut self,
        request: &AssetRequest,
        options: ImageSourceOptions,
        invalidation: Invalidation,
    ) -> ImageSourceState {
        match crate::resolve_image_source_from_host(self.app, request) {
            Ok(source) => self.use_image_source_state_with_options_and_invalidation(
                &source,
                options,
                invalidation,
            ),
            Err(err) => {
                self.observe_global::<AssetReloadEpoch>(invalidation);
                ImageSourceState {
                    image: None,
                    status: crate::image_asset_state::ImageLoadingStatus::Error,
                    intrinsic_size_px: None,
                    error: Some(Arc::<str>::from(err.to_string())),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SvgAssetSourceState {
    pub source: Option<fret_ui::SvgSource>,
    pub error: Option<Arc<str>>,
}

impl SvgAssetSourceState {
    fn ready(source: fret_ui::SvgSource) -> Self {
        Self {
            source: Some(source),
            error: None,
        }
    }

    fn error(message: impl Into<Arc<str>>) -> Self {
        Self {
            source: None,
            error: Some(message.into()),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Default)]
struct SvgReferenceFileCache {
    entries: std::collections::HashMap<PathBuf, SvgReferenceFileCacheEntry>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct SvgReferenceFileCacheEntry {
    epoch: u64,
    bytes: Option<Arc<[u8]>>,
    error: Option<Arc<str>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for SvgReferenceFileCacheEntry {
    fn default() -> Self {
        Self {
            epoch: u64::MAX,
            bytes: None,
            error: None,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn svg_asset_source_state_from_native_file_cached<
    H: fret_runtime::GlobalsHost + fret_runtime::TimeHost,
>(
    host: &mut H,
    path: &Path,
) -> SvgAssetSourceState {
    let epoch = host
        .global::<AssetReloadEpoch>()
        .map(|value| value.0)
        .unwrap_or(0);
    let path = path.to_path_buf();

    host.with_global_mut_untracked(SvgReferenceFileCache::default, |cache, _host| {
        let entry = cache
            .entries
            .entry(path.clone())
            .or_insert_with(SvgReferenceFileCacheEntry::default);
        if entry.epoch != epoch {
            match std::fs::read(&path) {
                Ok(bytes) => {
                    entry.epoch = epoch;
                    entry.bytes = Some(Arc::<[u8]>::from(bytes));
                    entry.error = None;
                }
                Err(err) => {
                    entry.epoch = epoch;
                    entry.bytes = None;
                    entry.error = Some(Arc::<str>::from(err.to_string()));
                }
            }
        }

        if let Some(err) = entry.error.clone() {
            return SvgAssetSourceState::error(err);
        }

        let Some(bytes) = entry.bytes.clone() else {
            return SvgAssetSourceState::error("missing svg bytes");
        };

        SvgAssetSourceState::ready(fret_ui::SvgSource::Bytes(bytes))
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_svg_asset_source_state<H: fret_runtime::GlobalsHost + fret_runtime::TimeHost>(
    host: &mut H,
    request: &AssetRequest,
) -> SvgAssetSourceState {
    match fret_runtime::resolve_asset_reference(host, request) {
        Ok(resolved) => match resolved.reference {
            AssetExternalReference::FilePath(path) => {
                svg_asset_source_state_from_native_file_cached(host, &path)
            }
            AssetExternalReference::Url(_) => SvgAssetSourceState::error(
                AssetLoadError::ExternalReferenceUnavailable {
                    kind: request.locator.kind(),
                }
                .to_string(),
            ),
        },
        Err(AssetLoadError::ExternalReferenceUnavailable { .. }) => {
            match crate::resolve_svg_source_from_host(&*host, request) {
                Ok(source) => SvgAssetSourceState::ready(source),
                Err(err) => SvgAssetSourceState::error(err.to_string()),
            }
        }
        Err(err) => SvgAssetSourceState::error(err.to_string()),
    }
}

#[cfg(target_arch = "wasm32")]
fn resolve_svg_asset_source_state<H: fret_runtime::GlobalsHost + fret_runtime::TimeHost>(
    host: &mut H,
    request: &AssetRequest,
) -> SvgAssetSourceState {
    match crate::resolve_svg_source_from_host(&*host, request) {
        Ok(source) => SvgAssetSourceState::ready(source),
        Err(err) => SvgAssetSourceState::error(err.to_string()),
    }
}

pub trait SvgAssetElementContextExt {
    fn svg_source_state_from_asset_request(
        &mut self,
        request: &AssetRequest,
    ) -> SvgAssetSourceState;

    fn svg_source_state_from_asset_request_with_invalidation(
        &mut self,
        request: &AssetRequest,
        invalidation: Invalidation,
    ) -> SvgAssetSourceState;

    fn svg_source_state_from_asset_locator(
        &mut self,
        locator: AssetLocator,
    ) -> SvgAssetSourceState {
        self.svg_source_state_from_asset_request(&AssetRequest::new(locator))
    }
}

impl<H: UiHost> SvgAssetElementContextExt for ElementContext<'_, H> {
    fn svg_source_state_from_asset_request(
        &mut self,
        request: &AssetRequest,
    ) -> SvgAssetSourceState {
        self.svg_source_state_from_asset_request_with_invalidation(request, Invalidation::Paint)
    }

    fn svg_source_state_from_asset_request_with_invalidation(
        &mut self,
        request: &AssetRequest,
        invalidation: Invalidation,
    ) -> SvgAssetSourceState {
        self.observe_global::<AssetReloadEpoch>(invalidation);
        resolve_svg_asset_source_state(self.app, request)
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    #[cfg(not(target_arch = "wasm32"))]
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use fret_assets::{AssetLocator, AssetRevision, StaticAssetEntry};
    use fret_core::{ClipboardToken, FrameId, ImageUploadToken, ShareSheetToken, TimerToken};
    use fret_runtime::{GlobalsHost, TickId, TimeHost};

    use super::*;

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
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
                Some(value) => *value.downcast::<T>().expect("global type id must match"),
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

        fn next_share_sheet_token(&mut self) -> ShareSheetToken {
            let token = ShareSheetToken(self.next_share_sheet_token);
            self.next_share_sheet_token += 1;
            token
        }

        fn next_image_upload_token(&mut self) -> ImageUploadToken {
            let token = ImageUploadToken(self.next_image_upload_token);
            self.next_image_upload_token += 1;
            token
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    struct TempAssetDir {
        path: PathBuf,
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl TempAssetDir {
        fn new(test_name: &str, entries: &[(&str, &[u8])]) -> Self {
            let unique = format!(
                "fret_ui_assets_ui_{test_name}_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            let path = std::env::temp_dir().join(unique);
            std::fs::create_dir_all(&path).expect("temp asset root should be created");
            for (entry, bytes) in entries {
                let entry_path = path.join(entry);
                if let Some(parent) = entry_path.parent() {
                    std::fs::create_dir_all(parent)
                        .expect("temp asset parent dirs should be created");
                }
                std::fs::write(&entry_path, bytes).expect("temp asset file should be written");
            }
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl Drop for TempAssetDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn svg_asset_source_state_bridges_file_backed_bundle_locator() {
        let root = TempAssetDir::new(
            "svg_bridge",
            &[("icons/search.svg", br#"<svg viewBox="0 0 1 1"></svg>"#)],
        );
        let resolver = fret_assets::FileAssetManifestResolver::from_bundle_dir("app", root.path())
            .expect("bundle dir should scan");

        let mut host = TestHost::default();
        fret_runtime::set_asset_resolver(&mut host, Arc::new(resolver));

        let state = resolve_svg_asset_source_state(
            &mut host,
            &AssetRequest::new(AssetLocator::bundle("app", "icons/search.svg")),
        );

        assert!(state.error.is_none());
        match state.source {
            Some(fret_ui::SvgSource::Bytes(bytes)) => {
                assert_eq!(bytes.as_ref(), br#"<svg viewBox="0 0 1 1"></svg>"#);
            }
            other => panic!("expected bytes-backed svg source, got {other:?}"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn svg_asset_source_state_keeps_cached_file_bytes_until_reload_epoch_bumps() {
        let root = TempAssetDir::new(
            "svg_epoch_cache",
            &[("icons/search.svg", br#"<svg viewBox="0 0 1 1"></svg>"#)],
        );
        let resolver = fret_assets::FileAssetManifestResolver::from_bundle_dir("app", root.path())
            .expect("bundle dir should scan");

        let mut host = TestHost::default();
        host.set_global(AssetReloadEpoch(0));
        fret_runtime::set_asset_resolver(&mut host, Arc::new(resolver));

        let request = AssetRequest::new(AssetLocator::bundle("app", "icons/search.svg"));

        let first = resolve_svg_asset_source_state(&mut host, &request);
        let first_bytes = match first.source {
            Some(fret_ui::SvgSource::Bytes(bytes)) => bytes,
            other => panic!("expected bytes-backed svg source, got {other:?}"),
        };
        assert_eq!(first_bytes.as_ref(), br#"<svg viewBox="0 0 1 1"></svg>"#);

        std::fs::write(
            root.path().join("icons/search.svg"),
            br#"<svg viewBox="0 0 2 2"></svg>"#,
        )
        .expect("svg file should rewrite");

        let same_epoch = resolve_svg_asset_source_state(&mut host, &request);
        let same_epoch_bytes = match same_epoch.source {
            Some(fret_ui::SvgSource::Bytes(bytes)) => bytes,
            other => panic!("expected bytes-backed svg source, got {other:?}"),
        };
        assert_eq!(same_epoch_bytes.as_ref(), first_bytes.as_ref());

        fret_runtime::bump_asset_reload_epoch(&mut host);
        let bumped = resolve_svg_asset_source_state(&mut host, &request);
        let bumped_bytes = match bumped.source {
            Some(fret_ui::SvgSource::Bytes(bytes)) => bytes,
            other => panic!("expected bytes-backed svg source, got {other:?}"),
        };
        assert_eq!(bumped_bytes.as_ref(), br#"<svg viewBox="0 0 2 2"></svg>"#);
    }

    #[test]
    fn svg_asset_source_state_falls_back_to_bytes_when_reference_handoff_is_unavailable() {
        let mut host = TestHost::default();
        fret_runtime::register_bundle_asset_entries(
            &mut host,
            "app",
            [StaticAssetEntry::new(
                "icons/search.svg",
                AssetRevision(3),
                br#"<svg viewBox="0 0 1 1"></svg>"#,
            )],
        );

        let state = resolve_svg_asset_source_state(
            &mut host,
            &AssetRequest::new(AssetLocator::bundle("app", "icons/search.svg")),
        );

        assert!(state.error.is_none());
        match state.source {
            Some(fret_ui::SvgSource::Bytes(bytes)) => {
                assert_eq!(bytes.as_ref(), br#"<svg viewBox="0 0 1 1"></svg>"#);
            }
            other => panic!("expected bytes-backed svg source, got {other:?}"),
        }
    }
}
