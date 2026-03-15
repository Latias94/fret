use fret_assets::{
    AssetExternalReference, AssetLoadError, AssetLocator, AssetRequest, AssetResolver,
    AssetRevision, ResolvedAssetReference,
};
use fret_runtime::GlobalsHost;

use crate::ImageSource;
#[cfg(not(target_arch = "wasm32"))]
use crate::SvgFileSource;

fn image_source_from_reference(resolved: &ResolvedAssetReference) -> Option<ImageSource> {
    match &resolved.reference {
        #[cfg(not(target_arch = "wasm32"))]
        AssetExternalReference::FilePath(path) => {
            Some(ImageSource::from_native_file_path(path.clone()))
        }
        #[cfg(target_arch = "wasm32")]
        AssetExternalReference::Url(url) => Some(ImageSource::from_url(url.as_str())),
        _ => None,
    }
}

pub fn resolve_image_source(
    resolver: &dyn AssetResolver,
    request: &AssetRequest,
) -> Result<ImageSource, AssetLoadError> {
    match resolver.resolve_reference(request) {
        Ok(resolved) => {
            if let Some(source) = image_source_from_reference(&resolved) {
                return Ok(source);
            }
        }
        Err(AssetLoadError::ExternalReferenceUnavailable { .. }) => {}
        Err(err) => return Err(err),
    }
    let resolved = resolver.resolve_bytes(request)?;
    Ok(ImageSource::from_resolved_asset_bytes(&resolved))
}

pub fn resolve_image_source_from_locator(
    resolver: &dyn AssetResolver,
    locator: AssetLocator,
) -> Result<ImageSource, AssetLoadError> {
    resolve_image_source(resolver, &AssetRequest::new(locator))
}

pub fn resolve_image_source_from_host(
    host: &impl GlobalsHost,
    request: &AssetRequest,
) -> Result<ImageSource, AssetLoadError> {
    match fret_runtime::resolve_asset_reference(host, request) {
        Ok(resolved) => {
            if let Some(source) = image_source_from_reference(&resolved) {
                return Ok(source);
            }
        }
        Err(AssetLoadError::ExternalReferenceUnavailable { .. }) => {}
        Err(err) => return Err(err),
    }
    let resolved = fret_runtime::resolve_asset_bytes(host, request)?;
    Ok(ImageSource::from_resolved_asset_bytes(&resolved))
}

pub fn resolve_image_source_from_host_locator(
    host: &impl GlobalsHost,
    locator: AssetLocator,
) -> Result<ImageSource, AssetLoadError> {
    resolve_image_source_from_host(host, &AssetRequest::new(locator))
}

#[cfg(not(target_arch = "wasm32"))]
fn svg_file_source_from_reference(resolved: &ResolvedAssetReference) -> Option<SvgFileSource> {
    match &resolved.reference {
        AssetExternalReference::FilePath(path) => {
            Some(SvgFileSource::from_native_file_path(path.clone()))
        }
        AssetExternalReference::Url(_) => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_svg_file_source(
    resolver: &dyn AssetResolver,
    request: &AssetRequest,
) -> Result<SvgFileSource, AssetLoadError> {
    let resolved = resolver.resolve_reference(request)?;
    svg_file_source_from_reference(&resolved).ok_or(AssetLoadError::ExternalReferenceUnavailable {
        kind: request.locator.kind(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_svg_file_source_from_locator(
    resolver: &dyn AssetResolver,
    locator: AssetLocator,
) -> Result<SvgFileSource, AssetLoadError> {
    resolve_svg_file_source(resolver, &AssetRequest::new(locator))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_svg_file_source_from_host(
    host: &impl GlobalsHost,
    request: &AssetRequest,
) -> Result<SvgFileSource, AssetLoadError> {
    let resolved = fret_runtime::resolve_asset_reference(host, request)?;
    svg_file_source_from_reference(&resolved).ok_or(AssetLoadError::ExternalReferenceUnavailable {
        kind: request.locator.kind(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_svg_file_source_from_host_locator(
    host: &impl GlobalsHost,
    locator: AssetLocator,
) -> Result<SvgFileSource, AssetLoadError> {
    resolve_svg_file_source_from_host(host, &AssetRequest::new(locator))
}

#[cfg(feature = "ui")]
pub fn resolve_svg_source(
    resolver: &dyn AssetResolver,
    request: &AssetRequest,
) -> Result<fret_ui::SvgSource, AssetLoadError> {
    let resolved = resolver.resolve_bytes(request)?;
    Ok(fret_ui::SvgSource::Bytes(resolved.bytes))
}

#[cfg(feature = "ui")]
pub fn resolve_svg_source_from_locator(
    resolver: &dyn AssetResolver,
    locator: AssetLocator,
) -> Result<fret_ui::SvgSource, AssetLoadError> {
    resolve_svg_source(resolver, &AssetRequest::new(locator))
}

#[cfg(feature = "ui")]
pub fn resolve_svg_source_from_host(
    host: &impl GlobalsHost,
    request: &AssetRequest,
) -> Result<fret_ui::SvgSource, AssetLoadError> {
    let resolved = fret_runtime::resolve_asset_bytes(host, request)?;
    Ok(fret_ui::SvgSource::Bytes(resolved.bytes))
}

#[cfg(feature = "ui")]
pub fn resolve_svg_source_from_host_locator(
    host: &impl GlobalsHost,
    locator: AssetLocator,
) -> Result<fret_ui::SvgSource, AssetLoadError> {
    resolve_svg_source_from_host(host, &AssetRequest::new(locator))
}

pub fn resolved_revision(
    resolver: &dyn AssetResolver,
    request: &AssetRequest,
) -> Result<AssetRevision, AssetLoadError> {
    resolver
        .resolve_bytes(request)
        .map(|resolved| resolved.revision)
}

pub fn resolved_revision_from_host(
    host: &impl GlobalsHost,
    request: &AssetRequest,
) -> Result<AssetRevision, AssetLoadError> {
    fret_runtime::resolve_asset_bytes(host, request).map(|resolved| resolved.revision)
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    #[cfg(not(target_arch = "wasm32"))]
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use fret_assets::{
        AssetCapabilities, AssetLocator, AssetRequest, AssetResolver, AssetRevision,
        InMemoryAssetResolver, ResolvedAssetBytes, ResolvedAssetReference,
    };

    use super::*;

    #[derive(Default)]
    struct TestHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
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

    #[cfg(not(target_arch = "wasm32"))]
    struct TempAssetDir {
        path: PathBuf,
    }

    #[cfg(not(target_arch = "wasm32"))]
    impl TempAssetDir {
        fn new(test_name: &str, entries: &[(&str, &[u8])]) -> Self {
            let unique = format!(
                "fret_ui_assets_{test_name}_{}_{}",
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

    #[test]
    fn resolve_image_source_supports_bundle_assets_via_resolver() {
        let mut resolver = InMemoryAssetResolver::new();
        resolver.insert_bundle("app", "images/logo.png", AssetRevision(3), [1u8, 2, 3]);

        let request = AssetRequest::new(AssetLocator::bundle("app", "images/logo.png"));
        let source =
            resolve_image_source(&resolver, &request).expect("bundle asset should resolve");
        let revision = resolved_revision(&resolver, &request).expect("revision should resolve");

        assert_eq!(revision, AssetRevision(3));
        assert_eq!(
            source.id(),
            resolve_image_source(&resolver, &request)
                .expect("bundle asset should resolve twice")
                .id()
        );
    }

    #[test]
    fn resolve_image_source_supports_embedded_assets_via_resolver() {
        let mut resolver = InMemoryAssetResolver::new();
        resolver.insert_embedded(
            "fret-ui-shadcn",
            "icons/search.svg",
            AssetRevision(4),
            [9u8, 8, 7],
        );

        let request =
            AssetRequest::new(AssetLocator::embedded("fret-ui-shadcn", "icons/search.svg"));
        let source =
            resolve_image_source(&resolver, &request).expect("embedded asset should resolve");

        assert_eq!(
            source.id(),
            resolve_image_source(&resolver, &request)
                .expect("embedded asset should resolve twice")
                .id()
        );
    }

    #[test]
    fn resolve_image_source_supports_bundle_assets_via_host_service() {
        let mut host = TestHost::default();
        let mut resolver = InMemoryAssetResolver::new();
        resolver.insert_bundle("app", "images/logo.png", AssetRevision(6), [1u8, 2, 3]);
        fret_runtime::set_asset_resolver(&mut host, Arc::new(resolver));

        let request = AssetRequest::new(AssetLocator::bundle("app", "images/logo.png"));
        let source =
            resolve_image_source_from_host(&host, &request).expect("bundle asset should resolve");
        let revision =
            resolved_revision_from_host(&host, &request).expect("revision should resolve");

        assert_eq!(revision, AssetRevision(6));
        assert_eq!(
            source.id(),
            resolve_image_source_from_host(&host, &request)
                .expect("bundle asset should resolve twice")
                .id()
        );
    }

    #[test]
    fn resolve_image_source_supports_embedded_assets_via_host_service() {
        let mut host = TestHost::default();
        let mut resolver = InMemoryAssetResolver::new();
        resolver.insert_embedded(
            "fret-ui-shadcn",
            "icons/search.svg",
            AssetRevision(8),
            [9u8, 8, 7],
        );
        fret_runtime::set_asset_resolver(&mut host, Arc::new(resolver));

        let request =
            AssetRequest::new(AssetLocator::embedded("fret-ui-shadcn", "icons/search.svg"));
        let source =
            resolve_image_source_from_host(&host, &request).expect("embedded asset should resolve");

        assert_eq!(
            source.id(),
            resolve_image_source_from_host(&host, &request)
                .expect("embedded asset should resolve twice")
                .id()
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resolve_image_source_from_host_prefers_file_reference_for_file_backed_bundle_assets() {
        let root = TempAssetDir::new("image_file_reference", &[("images/logo.png", b"png")]);
        let resolver = fret_assets::FileAssetManifestResolver::from_bundle_dir("app", root.path())
            .expect("bundle dir should scan");

        let mut host = TestHost::default();
        fret_runtime::set_asset_resolver(&mut host, Arc::new(resolver));

        let source = resolve_image_source_from_host_locator(
            &host,
            AssetLocator::bundle("app", "images/logo.png"),
        )
        .expect("file-backed bundle asset should bridge to a native path");

        let expected = ImageSource::from_native_file_path(root.path().join("images/logo.png"));
        assert_eq!(source.id(), expected.id());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resolve_svg_file_source_from_host_locator_bridges_file_backed_bundle_locator() {
        let root = TempAssetDir::new(
            "svg_file_reference",
            &[("icons/search.svg", br#"<svg viewBox="0 0 1 1"></svg>"#)],
        );
        let resolver = fret_assets::FileAssetManifestResolver::from_bundle_dir("app", root.path())
            .expect("bundle dir should scan");

        let mut host = TestHost::default();
        fret_runtime::set_asset_resolver(&mut host, Arc::new(resolver));

        let source = resolve_svg_file_source_from_host_locator(
            &host,
            AssetLocator::bundle("app", "icons/search.svg"),
        )
        .expect("file-backed bundle locator should bridge to SvgFileSource");
        let expected = root.path().join("icons/search.svg");

        assert_eq!(source.path.as_ref(), expected.as_path());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn resolve_image_source_falls_back_to_bytes_when_later_layer_blocks_reference_handoff() {
        struct FileReferenceResolver;

        impl AssetResolver for FileReferenceResolver {
            fn capabilities(&self) -> AssetCapabilities {
                AssetCapabilities {
                    memory: false,
                    embedded: false,
                    bundle_asset: true,
                    file: false,
                    url: false,
                    file_watch: false,
                    system_font_scan: false,
                }
            }

            fn resolve_bytes(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetBytes, AssetLoadError> {
                Ok(ResolvedAssetBytes::new(
                    request.locator.clone(),
                    AssetRevision(1),
                    b"earlier".as_slice(),
                ))
            }

            fn resolve_reference(
                &self,
                request: &AssetRequest,
            ) -> Result<ResolvedAssetReference, AssetLoadError> {
                Ok(ResolvedAssetReference::new(
                    request.locator.clone(),
                    AssetRevision(1),
                    AssetExternalReference::file_path("assets/earlier.png"),
                ))
            }
        }

        let mut host = TestHost::default();
        fret_runtime::register_asset_resolver(&mut host, Arc::new(FileReferenceResolver));
        fret_runtime::register_bundle_asset_entries(
            &mut host,
            "app",
            [fret_assets::StaticAssetEntry::new(
                "images/logo.png",
                AssetRevision(9),
                b"override",
            )],
        );

        let source = resolve_image_source_from_host_locator(
            &host,
            AssetLocator::bundle("app", "images/logo.png"),
        )
        .expect("later bytes layer should fall back to bytes");
        let resolved = fret_runtime::resolve_asset_locator_bytes(
            &host,
            AssetLocator::bundle("app", "images/logo.png"),
        )
        .expect("later bytes layer should still own byte resolution");

        assert_eq!(
            source.id(),
            ImageSource::from_resolved_asset_bytes(&resolved).id()
        );
        assert_ne!(
            source.id(),
            ImageSource::from_native_file_path(PathBuf::from("assets/earlier.png")).id()
        );
    }
}
