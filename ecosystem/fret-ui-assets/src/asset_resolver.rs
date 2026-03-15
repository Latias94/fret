use fret_assets::{AssetLoadError, AssetLocator, AssetRequest, AssetResolver, AssetRevision};
use fret_runtime::GlobalsHost;

use crate::ImageSource;

pub fn resolve_image_source(
    resolver: &dyn AssetResolver,
    request: &AssetRequest,
) -> Result<ImageSource, AssetLoadError> {
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
    let resolved = fret_runtime::resolve_asset_bytes(host, request)?;
    Ok(ImageSource::from_resolved_asset_bytes(&resolved))
}

pub fn resolve_image_source_from_host_locator(
    host: &impl GlobalsHost,
    locator: AssetLocator,
) -> Result<ImageSource, AssetLoadError> {
    resolve_image_source_from_host(host, &AssetRequest::new(locator))
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
    use std::sync::Arc;

    use fret_assets::{AssetLocator, AssetRequest, AssetRevision, InMemoryAssetResolver};

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
}
