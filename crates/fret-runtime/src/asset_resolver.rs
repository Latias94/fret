use std::fmt;
use std::sync::Arc;

use fret_assets::{
    AssetCapabilities, AssetLoadError, AssetLocator, AssetRequest, AssetResolver,
    ResolvedAssetBytes,
};

use crate::GlobalsHost;

#[derive(Clone)]
pub struct AssetResolverService {
    resolver: Arc<dyn AssetResolver>,
}

impl AssetResolverService {
    pub fn new(resolver: Arc<dyn AssetResolver>) -> Self {
        Self { resolver }
    }

    pub fn resolver(&self) -> &(dyn AssetResolver + 'static) {
        self.resolver.as_ref()
    }

    pub fn shared(&self) -> Arc<dyn AssetResolver> {
        self.resolver.clone()
    }

    pub fn capabilities(&self) -> AssetCapabilities {
        self.resolver.capabilities()
    }

    pub fn supports(&self, locator: &AssetLocator) -> bool {
        self.resolver.supports(locator)
    }

    pub fn resolve_bytes(
        &self,
        request: &AssetRequest,
    ) -> Result<ResolvedAssetBytes, AssetLoadError> {
        self.resolver.resolve_bytes(request)
    }

    pub fn resolve_locator_bytes(
        &self,
        locator: AssetLocator,
    ) -> Result<ResolvedAssetBytes, AssetLoadError> {
        self.resolver.resolve_locator_bytes(locator)
    }
}

impl fmt::Debug for AssetResolverService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetResolverService")
            .field("capabilities", &self.capabilities())
            .finish_non_exhaustive()
    }
}

impl From<Arc<dyn AssetResolver>> for AssetResolverService {
    fn from(resolver: Arc<dyn AssetResolver>) -> Self {
        Self::new(resolver)
    }
}

pub fn set_asset_resolver(host: &mut impl GlobalsHost, resolver: Arc<dyn AssetResolver>) {
    host.set_global(AssetResolverService::new(resolver));
}

pub fn asset_resolver(host: &impl GlobalsHost) -> Option<&AssetResolverService> {
    host.global::<AssetResolverService>()
}

pub fn asset_capabilities(host: &impl GlobalsHost) -> Option<AssetCapabilities> {
    asset_resolver(host).map(AssetResolverService::capabilities)
}

pub fn resolve_asset_bytes(
    host: &impl GlobalsHost,
    request: &AssetRequest,
) -> Result<ResolvedAssetBytes, AssetLoadError> {
    asset_resolver(host)
        .ok_or(AssetLoadError::ResolverUnavailable)?
        .resolve_bytes(request)
}

pub fn resolve_asset_locator_bytes(
    host: &impl GlobalsHost,
    locator: AssetLocator,
) -> Result<ResolvedAssetBytes, AssetLoadError> {
    resolve_asset_bytes(host, &AssetRequest::new(locator))
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_assets::{AssetLocator, AssetRevision, InMemoryAssetResolver};

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
    fn resolve_asset_bytes_requires_installed_service() {
        let host = TestHost::default();
        let err =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect_err("missing service should fail");

        assert_eq!(err, AssetLoadError::ResolverUnavailable);
    }

    #[test]
    fn installed_service_resolves_bundle_assets() {
        let mut host = TestHost::default();
        let mut resolver = InMemoryAssetResolver::new();
        resolver.insert_bundle("app", "images/logo.png", AssetRevision(7), [1u8, 2, 3]);
        set_asset_resolver(&mut host, Arc::new(resolver));

        let caps = asset_capabilities(&host).expect("resolver caps should exist");
        let resolved =
            resolve_asset_locator_bytes(&host, AssetLocator::bundle("app", "images/logo.png"))
                .expect("bundle asset should resolve");

        assert!(caps.bundle_asset);
        assert_eq!(resolved.revision, AssetRevision(7));
        assert_eq!(resolved.bytes.as_ref(), &[1, 2, 3]);
    }
}
