use fret_core::{Corners, Rect, TextStyle};
use fret_ui::element::{AnyElement, LayoutStyle, ViewCacheProps};
use fret_ui::{ElementContext, UiHost};

use crate::{IntoUiElement, collect_children};

/// Component-layer helper for authoring explicit cached subtree boundaries.
///
/// This intentionally lives in the ecosystem layer (ADR 0066): it is sugar on top of the
/// mechanism-only `ElementContext::view_cache(...)` API in `fret-ui`.
pub trait CachedSubtreeExt {
    type Host: UiHost;

    /// Build an explicit cached subtree boundary using default cache-root behavior.
    fn cached_subtree<I, T>(&mut self, f: impl FnOnce(&mut Self) -> I) -> AnyElement
    where
        I: IntoIterator<Item = T>,
        T: IntoUiElement<Self::Host>,
    {
        self.cached_subtree_with(CachedSubtreeProps::default(), f)
    }

    /// Build an explicit cached subtree boundary with additional cache-root hints.
    fn cached_subtree_with<I, T>(
        &mut self,
        props: CachedSubtreeProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = T>,
        T: IntoUiElement<Self::Host>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CachedSubtreeProps {
    pub layout: LayoutStyle,
    pub contained_layout: bool,
    pub cache_key: u64,
}

impl CachedSubtreeProps {
    pub fn layout(mut self, layout: LayoutStyle) -> Self {
        self.layout = layout;
        self
    }

    pub fn contained_layout(mut self, contained_layout: bool) -> Self {
        self.contained_layout = contained_layout;
        self
    }

    pub fn cache_key(mut self, cache_key: u64) -> Self {
        self.cache_key = cache_key;
        self
    }

    pub fn cache_key_text_style(mut self, style: &TextStyle) -> Self {
        self.cache_key =
            fret_ui::cache_key::mix(self.cache_key, fret_ui::cache_key::text_style_key(style));
        self
    }

    pub fn cache_key_clip_rect(mut self, rect: Rect) -> Self {
        self.cache_key =
            fret_ui::cache_key::mix(self.cache_key, fret_ui::cache_key::rect_key(rect));
        self
    }

    pub fn cache_key_clip_rrect(mut self, rect: Rect, corners: Corners) -> Self {
        self.cache_key =
            fret_ui::cache_key::mix(self.cache_key, fret_ui::cache_key::rect_key(rect));
        self.cache_key =
            fret_ui::cache_key::mix(self.cache_key, fret_ui::cache_key::corners_key(corners));
        self
    }
}

impl<'a, H: UiHost> CachedSubtreeExt for ElementContext<'a, H> {
    type Host = H;

    fn cached_subtree_with<I, T>(
        &mut self,
        props: CachedSubtreeProps,
        f: impl FnOnce(&mut Self) -> I,
    ) -> AnyElement
    where
        I: IntoIterator<Item = T>,
        T: IntoUiElement<Self::Host>,
    {
        let view_cache = ViewCacheProps {
            layout: props.layout,
            contained_layout: props.contained_layout,
            cache_key: props.cache_key,
        };
        self.view_cache(view_cache, move |cx| {
            let items = f(cx);
            collect_children(cx, items)
        })
    }
}
