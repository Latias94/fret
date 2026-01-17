use fret_ui::element::{AnyElement, ViewCacheProps};
use fret_ui::{ElementContext, UiHost};

/// Component-layer helper for authoring explicit cached subtree boundaries.
///
/// This intentionally lives in the ecosystem layer (ADR 0066): it is sugar on top of the
/// mechanism-only `ElementContext::view_cache(...)` API in `fret-ui`.
pub trait CachedSubtreeExt {
    /// Build an explicit cached subtree boundary using default cache-root behavior.
    fn cached_subtree(&mut self, f: impl FnOnce(&mut Self) -> Vec<AnyElement>) -> AnyElement {
        self.cached_subtree_with(CachedSubtreeProps::default(), f)
    }

    /// Build an explicit cached subtree boundary with additional cache-root hints.
    fn cached_subtree_with(
        &mut self,
        props: CachedSubtreeProps,
        f: impl FnOnce(&mut Self) -> Vec<AnyElement>,
    ) -> AnyElement;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CachedSubtreeProps {
    pub contained_layout: bool,
}

impl CachedSubtreeProps {
    pub fn contained_layout(mut self, contained_layout: bool) -> Self {
        self.contained_layout = contained_layout;
        self
    }
}

impl<'a, H: UiHost> CachedSubtreeExt for ElementContext<'a, H> {
    fn cached_subtree_with(
        &mut self,
        props: CachedSubtreeProps,
        f: impl FnOnce(&mut Self) -> Vec<AnyElement>,
    ) -> AnyElement {
        let mut view_cache = ViewCacheProps::default();
        view_cache.contained_layout = props.contained_layout;
        self.view_cache(view_cache, f)
    }
}
