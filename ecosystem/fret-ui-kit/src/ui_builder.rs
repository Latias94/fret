use crate::{ChromeRefinement, LayoutRefinement, Radius, Space};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

/// Aggregated authoring patch applied by `UiBuilder`.
///
/// This is an ecosystem-only authoring surface (see ADR 0175). It intentionally composes:
/// - control chrome patches (`ChromeRefinement`)
/// - layout-affecting patches (`LayoutRefinement`)
#[derive(Debug, Clone, Default)]
pub struct UiPatch {
    pub chrome: ChromeRefinement,
    pub layout: LayoutRefinement,
}

impl UiPatch {
    pub fn merge(mut self, other: UiPatch) -> Self {
        self.chrome = self.chrome.merge(other.chrome);
        self.layout = self.layout.merge(other.layout);
        self
    }
}

/// A type that opts into the `ui()` builder surface by accepting a `UiPatch`.
///
/// This is intentionally an ecosystem-only authoring surface (see ADR 0175).
pub trait UiPatchTarget: Sized {
    fn apply_ui_patch(self, patch: UiPatch) -> Self;
}

/// Marker trait enabling `UiBuilder` chrome/styling methods for a `UiPatchTarget`.
pub trait UiSupportsChrome {}

/// Marker trait enabling `UiBuilder` layout methods for a `UiPatchTarget`.
pub trait UiSupportsLayout {}

/// A type that can render itself into a declarative element.
///
/// This trait exists so `UiBuilder::into_element(cx)` can be implemented without relying on
/// inherent methods.
pub trait UiIntoElement: Sized {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement;
}

/// The main fluent authoring surface: `value.ui().px_2().w_full().into_element(cx)`.
#[derive(Debug, Clone)]
pub struct UiBuilder<T> {
    inner: T,
    patch: UiPatch,
}

impl<T> UiBuilder<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            patch: UiPatch::default(),
        }
    }

    pub fn style(mut self, style: ChromeRefinement) -> Self
    where
        T: UiSupportsChrome,
    {
        self.patch.chrome = self.patch.chrome.merge(style);
        self
    }

    pub fn layout(mut self, layout: LayoutRefinement) -> Self
    where
        T: UiSupportsLayout,
    {
        self.patch.layout = self.patch.layout.merge(layout);
        self
    }

    pub fn style_with(self, f: impl FnOnce(ChromeRefinement) -> ChromeRefinement) -> Self
    where
        T: UiSupportsChrome,
    {
        self.style(f(ChromeRefinement::default()))
    }

    pub fn layout_with(self, f: impl FnOnce(LayoutRefinement) -> LayoutRefinement) -> Self
    where
        T: UiSupportsLayout,
    {
        self.layout(f(LayoutRefinement::default()))
    }

    pub fn px(self, space: Space) -> Self
    where
        T: UiSupportsChrome,
    {
        self.style_with(|c| c.px(space))
    }

    pub fn py(self, space: Space) -> Self
    where
        T: UiSupportsChrome,
    {
        self.style_with(|c| c.py(space))
    }

    pub fn p(self, space: Space) -> Self
    where
        T: UiSupportsChrome,
    {
        self.style_with(|c| c.p(space))
    }

    pub fn px_2(self) -> Self
    where
        T: UiSupportsChrome,
    {
        self.px(Space::N2)
    }

    pub fn px_3(self) -> Self
    where
        T: UiSupportsChrome,
    {
        self.px(Space::N3)
    }

    pub fn py_2(self) -> Self
    where
        T: UiSupportsChrome,
    {
        self.py(Space::N2)
    }

    pub fn py_1p5(self) -> Self
    where
        T: UiSupportsChrome,
    {
        self.py(Space::N1p5)
    }

    pub fn p_4(self) -> Self
    where
        T: UiSupportsChrome,
    {
        self.p(Space::N4)
    }

    pub fn rounded(self, radius: Radius) -> Self
    where
        T: UiSupportsChrome,
    {
        self.style_with(|c| c.rounded(radius))
    }

    pub fn rounded_md(self) -> Self
    where
        T: UiSupportsChrome,
    {
        self.rounded(Radius::Md)
    }

    pub fn border_1(self) -> Self
    where
        T: UiSupportsChrome,
    {
        self.style_with(|c| c.border_1())
    }

    pub fn w_full(self) -> Self
    where
        T: UiSupportsLayout,
    {
        self.layout_with(|l| l.w_full())
    }

    pub fn h_full(self) -> Self
    where
        T: UiSupportsLayout,
    {
        self.layout_with(|l| l.h_full())
    }

    pub fn size_full(self) -> Self
    where
        T: UiSupportsLayout,
    {
        self.layout_with(|l| l.size_full())
    }
}

impl<T: UiPatchTarget> UiBuilder<T> {
    pub fn build(self) -> T {
        self.inner.apply_ui_patch(self.patch)
    }
}

impl<T: UiPatchTarget + UiIntoElement> UiBuilder<T> {
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.build().into_element(cx)
    }
}

/// Extension trait providing the `ui()` entrypoint for types that opt into `UiPatchTarget`.
pub trait UiExt: UiPatchTarget + Sized {
    fn ui(self) -> UiBuilder<Self> {
        UiBuilder::new(self)
    }
}

impl<T: UiPatchTarget> UiExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LengthRefinement, MetricRef};
    use fret_core::Px;

    #[derive(Debug, Default, Clone)]
    struct Dummy {
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
    }

    impl UiPatchTarget for Dummy {
        fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
            self.chrome = self.chrome.merge(patch.chrome);
            self.layout = self.layout.merge(patch.layout);
            self
        }
    }

    impl UiSupportsChrome for Dummy {}
    impl UiSupportsLayout for Dummy {}

    #[test]
    fn ui_builder_merges_chrome_and_layout() {
        let dummy = Dummy::default()
            .ui()
            .px_3()
            .py_2()
            .border_1()
            .rounded_md()
            .w_full()
            .build();

        let padding = dummy.chrome.padding.expect("expected padding refinement");
        match padding.left {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N3.token_key()),
            _ => panic!("expected left padding token"),
        }
        match padding.top {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N2.token_key()),
            _ => panic!("expected top padding token"),
        }

        assert!(dummy.chrome.border_width.is_some());
        assert!(dummy.chrome.radius.is_some());

        let size = dummy.layout.size.expect("expected size refinement");
        match size.width {
            Some(LengthRefinement::Fill) => {}
            other => panic!("expected width Fill, got {other:?}"),
        }
        assert!(size.min_width.is_none());
        assert!(size.min_height.is_none());
    }

    #[test]
    fn ui_builder_allows_px_and_space_mix() {
        let dummy = Dummy::default()
            .ui()
            .style_with(|mut c| {
                c.min_height = Some(MetricRef::Px(Px(40.0)));
                c
            })
            .build();
        assert!(dummy.chrome.min_height.is_some());
    }
}
