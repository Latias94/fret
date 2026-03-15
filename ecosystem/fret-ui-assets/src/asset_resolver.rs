use fret_assets::{AssetLoadError, AssetRequest, AssetResolver, AssetRevision};

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
    locator: fret_assets::AssetLocator,
) -> Result<ImageSource, AssetLoadError> {
    resolve_image_source(resolver, &AssetRequest::new(locator))
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
    locator: fret_assets::AssetLocator,
) -> Result<fret_ui::SvgSource, AssetLoadError> {
    resolve_svg_source(resolver, &AssetRequest::new(locator))
}

pub fn resolved_revision(
    resolver: &dyn AssetResolver,
    request: &AssetRequest,
) -> Result<AssetRevision, AssetLoadError> {
    resolver
        .resolve_bytes(request)
        .map(|resolved| resolved.revision)
}

#[cfg(test)]
mod tests {
    use fret_assets::{AssetLocator, AssetRequest, AssetRevision, InMemoryAssetResolver};

    use super::*;

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
}
