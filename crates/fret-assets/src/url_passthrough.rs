use crate::{
    AssetCapabilities, AssetExternalReference, AssetLoadError, AssetLocator, AssetRequest,
    AssetResolver, AssetRevision, ResolvedAssetBytes, ResolvedAssetReference,
};

/// Resolver that turns `AssetLocator::Url(...)` into a direct external URL handoff.
///
/// This keeps URL loading explicit and reference-first: callers that need browser/native system
/// APIs can resolve a URL reference without pretending the framework already owns a byte-fetch
/// pipeline for that locator.
#[derive(Debug, Clone, Copy)]
pub struct UrlPassthroughAssetResolver {
    revision: AssetRevision,
}

impl UrlPassthroughAssetResolver {
    pub const fn new() -> Self {
        Self {
            revision: AssetRevision::ZERO,
        }
    }

    pub const fn with_revision(mut self, revision: AssetRevision) -> Self {
        self.revision = revision;
        self
    }
}

impl Default for UrlPassthroughAssetResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetResolver for UrlPassthroughAssetResolver {
    fn capabilities(&self) -> AssetCapabilities {
        AssetCapabilities {
            url: true,
            ..AssetCapabilities::default()
        }
    }

    fn resolve_bytes(&self, request: &AssetRequest) -> Result<ResolvedAssetBytes, AssetLoadError> {
        match &request.locator {
            AssetLocator::Url(_) => Err(AssetLoadError::ReferenceOnlyLocator {
                kind: request.locator.kind(),
            }),
            _ => Err(AssetLoadError::UnsupportedLocatorKind {
                kind: request.locator.kind(),
            }),
        }
    }

    fn resolve_reference(
        &self,
        request: &AssetRequest,
    ) -> Result<ResolvedAssetReference, AssetLoadError> {
        match &request.locator {
            AssetLocator::Url(url) => Ok(ResolvedAssetReference::new(
                request.locator.clone(),
                self.revision,
                AssetExternalReference::url(url.as_str()),
            )),
            _ => Err(AssetLoadError::UnsupportedLocatorKind {
                kind: request.locator.kind(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AssetLocatorKind;

    #[test]
    fn url_passthrough_capabilities_are_url_only() {
        let caps = UrlPassthroughAssetResolver::new().capabilities();

        assert_eq!(
            caps,
            AssetCapabilities {
                url: true,
                ..AssetCapabilities::default()
            }
        );
    }

    #[test]
    fn url_passthrough_resolves_reference_for_url_locators() {
        let resolver = UrlPassthroughAssetResolver::new().with_revision(AssetRevision(7));
        let request = AssetRequest::new(AssetLocator::url("https://example.com/logo.png"));

        let resolved = resolver
            .resolve_reference(&request)
            .expect("url locator should resolve to an external reference");

        assert_eq!(resolved.locator, request.locator);
        assert_eq!(resolved.revision, AssetRevision(7));
        assert_eq!(
            resolved.reference,
            AssetExternalReference::url("https://example.com/logo.png")
        );
    }

    #[test]
    fn url_passthrough_reports_reference_only_bytes_lane_for_url_locators() {
        let resolver = UrlPassthroughAssetResolver::new();
        let err = resolver
            .resolve_bytes(&AssetRequest::new(AssetLocator::url(
                "https://example.com/logo.png",
            )))
            .expect_err("url passthrough resolver should not fabricate bytes");

        assert_eq!(
            err,
            AssetLoadError::ReferenceOnlyLocator {
                kind: AssetLocatorKind::Url,
            }
        );
    }

    #[test]
    fn url_passthrough_rejects_non_url_locators() {
        let resolver = UrlPassthroughAssetResolver::new();
        let err = resolver
            .resolve_reference(&AssetRequest::new(AssetLocator::bundle(
                "app",
                "images/logo.png",
            )))
            .expect_err("bundle assets should stay outside the url passthrough lane");

        assert_eq!(
            err,
            AssetLoadError::UnsupportedLocatorKind {
                kind: AssetLocatorKind::BundleAsset,
            }
        );
    }
}
