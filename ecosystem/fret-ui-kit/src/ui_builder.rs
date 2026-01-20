use crate::{
    ChromeRefinement, ColorRef, LayoutRefinement, LengthRefinement, MetricRef, Radius, Space,
};
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
}

macro_rules! forward_style_noargs {
    ($($name:ident),+ $(,)?) => {
        $(
            pub fn $name(self) -> Self {
                self.style_with(|c| c.$name())
            }
        )+
    };
}

macro_rules! forward_layout_noargs {
    ($($name:ident),+ $(,)?) => {
        $(
            pub fn $name(self) -> Self {
                self.layout_with(|l| l.$name())
            }
        )+
    };
}

impl<T: UiSupportsChrome> UiBuilder<T> {
    pub fn px(self, space: Space) -> Self {
        self.style_with(|c| c.px(space))
    }

    pub fn py(self, space: Space) -> Self {
        self.style_with(|c| c.py(space))
    }

    pub fn p(self, space: Space) -> Self {
        self.style_with(|c| c.p(space))
    }

    pub fn pt(self, space: Space) -> Self {
        self.style_with(|c| c.pt(space))
    }

    pub fn pr(self, space: Space) -> Self {
        self.style_with(|c| c.pr(space))
    }

    pub fn pb(self, space: Space) -> Self {
        self.style_with(|c| c.pb(space))
    }

    pub fn pl(self, space: Space) -> Self {
        self.style_with(|c| c.pl(space))
    }

    pub fn rounded(self, radius: Radius) -> Self {
        self.style_with(|c| c.rounded(radius))
    }

    pub fn bg(self, color: ColorRef) -> Self {
        self.style_with(|c| c.bg(color))
    }

    pub fn border_color(self, color: ColorRef) -> Self {
        self.style_with(|c| c.border_color(color))
    }

    pub fn text_color(self, color: ColorRef) -> Self {
        self.style_with(|c| c.text_color(color))
    }

    forward_style_noargs!(
        px_0, px_1, px_0p5, px_1p5, px_2, px_2p5, px_3, px_4, py_0, py_1, py_0p5, py_1p5, py_2,
        py_2p5, py_3, py_4, p_0, p_1, p_0p5, p_1p5, p_2, p_2p5, p_3, p_4, rounded_md, border_1,
    );
}

impl<T: UiSupportsLayout> UiBuilder<T> {
    pub fn aspect_ratio(self, ratio: f32) -> Self {
        self.layout_with(|l| l.aspect_ratio(ratio))
    }

    pub fn inset(self, space: Space) -> Self {
        self.layout_with(|l| l.inset(space))
    }

    pub fn top(self, space: Space) -> Self {
        self.layout_with(|l| l.top(space))
    }

    pub fn top_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.top_neg(space))
    }

    pub fn right(self, space: Space) -> Self {
        self.layout_with(|l| l.right(space))
    }

    pub fn right_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.right_neg(space))
    }

    pub fn bottom(self, space: Space) -> Self {
        self.layout_with(|l| l.bottom(space))
    }

    pub fn bottom_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.bottom_neg(space))
    }

    pub fn left(self, space: Space) -> Self {
        self.layout_with(|l| l.left(space))
    }

    pub fn left_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.left_neg(space))
    }

    pub fn m(self, space: Space) -> Self {
        self.layout_with(|l| l.m(space))
    }

    pub fn m_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.m_neg(space))
    }

    pub fn mx(self, space: Space) -> Self {
        self.layout_with(|l| l.mx(space))
    }

    pub fn mx_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.mx_neg(space))
    }

    pub fn my(self, space: Space) -> Self {
        self.layout_with(|l| l.my(space))
    }

    pub fn my_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.my_neg(space))
    }

    pub fn mt(self, space: Space) -> Self {
        self.layout_with(|l| l.mt(space))
    }

    pub fn mt_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.mt_neg(space))
    }

    pub fn mr(self, space: Space) -> Self {
        self.layout_with(|l| l.mr(space))
    }

    pub fn mr_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.mr_neg(space))
    }

    pub fn mb(self, space: Space) -> Self {
        self.layout_with(|l| l.mb(space))
    }

    pub fn mb_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.mb_neg(space))
    }

    pub fn ml(self, space: Space) -> Self {
        self.layout_with(|l| l.ml(space))
    }

    pub fn ml_neg(self, space: Space) -> Self {
        self.layout_with(|l| l.ml_neg(space))
    }

    pub fn min_w(self, width: MetricRef) -> Self {
        self.layout_with(|l| l.min_w(width))
    }

    pub fn min_w_space(self, width: Space) -> Self {
        self.layout_with(|l| l.min_w_space(width))
    }

    pub fn min_h(self, height: MetricRef) -> Self {
        self.layout_with(|l| l.min_h(height))
    }

    pub fn min_h_space(self, height: Space) -> Self {
        self.layout_with(|l| l.min_h_space(height))
    }

    pub fn w(self, width: LengthRefinement) -> Self {
        self.layout_with(|l| l.w(width))
    }

    pub fn h(self, height: LengthRefinement) -> Self {
        self.layout_with(|l| l.h(height))
    }

    pub fn w_px(self, width: MetricRef) -> Self {
        self.layout_with(|l| l.w_px(width))
    }

    pub fn w_space(self, width: Space) -> Self {
        self.layout_with(|l| l.w_space(width))
    }

    pub fn h_px(self, height: MetricRef) -> Self {
        self.layout_with(|l| l.h_px(height))
    }

    pub fn h_space(self, height: Space) -> Self {
        self.layout_with(|l| l.h_space(height))
    }

    pub fn max_w(self, width: MetricRef) -> Self {
        self.layout_with(|l| l.max_w(width))
    }

    pub fn max_w_space(self, width: Space) -> Self {
        self.layout_with(|l| l.max_w_space(width))
    }

    pub fn max_h(self, height: MetricRef) -> Self {
        self.layout_with(|l| l.max_h(height))
    }

    pub fn max_h_space(self, height: Space) -> Self {
        self.layout_with(|l| l.max_h_space(height))
    }

    pub fn basis(self, basis: LengthRefinement) -> Self {
        self.layout_with(|l| l.basis(basis))
    }

    pub fn flex_grow(self, grow: f32) -> Self {
        self.layout_with(|l| l.flex_grow(grow))
    }

    pub fn flex_shrink(self, shrink: f32) -> Self {
        self.layout_with(|l| l.flex_shrink(shrink))
    }

    forward_layout_noargs!(
        relative,
        absolute,
        overflow_hidden,
        overflow_visible,
        overflow_x_hidden,
        overflow_y_hidden,
        m_auto,
        mx_auto,
        my_auto,
        mt_auto,
        mr_auto,
        mb_auto,
        ml_auto,
        min_w_0,
        w_full,
        h_full,
        size_full,
        basis_0,
        flex_shrink_0,
        flex_1,
        flex_none,
        w_0,
        h_0,
        min_h_0,
        max_w_0,
        max_h_0,
        w_0p5,
        h_0p5,
        min_w_0p5,
        min_h_0p5,
        max_w_0p5,
        max_h_0p5,
        w_1,
        h_1,
        min_w_1,
        min_h_1,
        max_w_1,
        max_h_1,
        w_1p5,
        h_1p5,
        min_w_1p5,
        min_h_1p5,
        max_w_1p5,
        max_h_1p5,
        w_2,
        h_2,
        min_w_2,
        min_h_2,
        max_w_2,
        max_h_2,
        w_2p5,
        h_2p5,
        min_w_2p5,
        min_h_2p5,
        max_w_2p5,
        max_h_2p5,
        w_3,
        h_3,
        min_w_3,
        min_h_3,
        max_w_3,
        max_h_3,
        w_3p5,
        h_3p5,
        min_w_3p5,
        min_h_3p5,
        max_w_3p5,
        max_h_3p5,
        w_4,
        h_4,
        min_w_4,
        min_h_4,
        max_w_4,
        max_h_4,
        w_5,
        h_5,
        min_w_5,
        min_h_5,
        max_w_5,
        max_h_5,
        w_6,
        h_6,
        min_w_6,
        min_h_6,
        max_w_6,
        max_h_6,
        w_8,
        h_8,
        min_w_8,
        min_h_8,
        max_w_8,
        max_h_8,
        w_10,
        h_10,
        min_w_10,
        min_h_10,
        max_w_10,
        max_h_10,
        w_11,
        h_11,
        min_w_11,
        min_h_11,
        max_w_11,
        max_h_11,
    );
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
    use fret_core::Color;
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

    #[test]
    fn ui_builder_forwards_full_vocabulary_smoke() {
        let _ = Dummy::default()
            .ui()
            // ChromeRefinement
            .px(Space::N1)
            .py(Space::N2)
            .p(Space::N3)
            .pt(Space::N0p5)
            .pr(Space::N1p5)
            .pb(Space::N2p5)
            .pl(Space::N3p5)
            .rounded(Radius::Full)
            .bg(ColorRef::Color(Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            }))
            .border_color(ColorRef::Color(Color {
                r: 0.3,
                g: 0.2,
                b: 0.1,
                a: 1.0,
            }))
            .text_color(ColorRef::Color(Color {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            }))
            .px_0()
            .px_1()
            .px_0p5()
            .px_1p5()
            .px_2()
            .px_2p5()
            .px_3()
            .px_4()
            .py_0()
            .py_1()
            .py_0p5()
            .py_1p5()
            .py_2()
            .py_2p5()
            .py_3()
            .py_4()
            .p_0()
            .p_1()
            .p_0p5()
            .p_1p5()
            .p_2()
            .p_2p5()
            .p_3()
            .p_4()
            .rounded_md()
            .border_1()
            // LayoutRefinement
            .aspect_ratio(1.0)
            .relative()
            .absolute()
            .overflow_hidden()
            .overflow_visible()
            .overflow_x_hidden()
            .overflow_y_hidden()
            .inset(Space::N2)
            .top(Space::N3)
            .top_neg(Space::N3)
            .right(Space::N3)
            .right_neg(Space::N3)
            .bottom(Space::N3)
            .bottom_neg(Space::N3)
            .left(Space::N3)
            .left_neg(Space::N3)
            .m(Space::N2)
            .m_neg(Space::N2)
            .m_auto()
            .mx(Space::N2)
            .mx_neg(Space::N2)
            .mx_auto()
            .my(Space::N2)
            .my_neg(Space::N2)
            .my_auto()
            .mt(Space::N2)
            .mt_neg(Space::N2)
            .mt_auto()
            .mr(Space::N2)
            .mr_neg(Space::N2)
            .mr_auto()
            .mb(Space::N2)
            .mb_neg(Space::N2)
            .mb_auto()
            .ml(Space::N2)
            .ml_neg(Space::N2)
            .ml_auto()
            .min_w(MetricRef::Px(Px(10.0)))
            .min_w_space(Space::N1)
            .min_h(MetricRef::Px(Px(10.0)))
            .min_h_space(Space::N1)
            .min_w_0()
            .w(LengthRefinement::Fill)
            .h(LengthRefinement::Auto)
            .w_px(MetricRef::Px(Px(10.0)))
            .w_space(Space::N10)
            .h_px(MetricRef::Px(Px(11.0)))
            .h_space(Space::N11)
            .w_full()
            .h_full()
            .size_full()
            .max_w(MetricRef::Px(Px(10.0)))
            .max_w_space(Space::N1)
            .max_h(MetricRef::Px(Px(10.0)))
            .max_h_space(Space::N1)
            .basis(LengthRefinement::Auto)
            .basis_0()
            .flex_grow(1.0)
            .flex_shrink(1.0)
            .flex_shrink_0()
            .flex_1()
            .flex_none()
            // LayoutRefinement shorthands
            .w_0()
            .h_0()
            .min_h_0()
            .max_w_0()
            .max_h_0()
            .w_0p5()
            .h_0p5()
            .min_w_0p5()
            .min_h_0p5()
            .max_w_0p5()
            .max_h_0p5()
            .w_1()
            .h_1()
            .min_w_1()
            .min_h_1()
            .max_w_1()
            .max_h_1()
            .w_1p5()
            .h_1p5()
            .min_w_1p5()
            .min_h_1p5()
            .max_w_1p5()
            .max_h_1p5()
            .w_2()
            .h_2()
            .min_w_2()
            .min_h_2()
            .max_w_2()
            .max_h_2()
            .w_2p5()
            .h_2p5()
            .min_w_2p5()
            .min_h_2p5()
            .max_w_2p5()
            .max_h_2p5()
            .w_3()
            .h_3()
            .min_w_3()
            .min_h_3()
            .max_w_3()
            .max_h_3()
            .w_3p5()
            .h_3p5()
            .min_w_3p5()
            .min_h_3p5()
            .max_w_3p5()
            .max_h_3p5()
            .w_4()
            .h_4()
            .min_w_4()
            .min_h_4()
            .max_w_4()
            .max_h_4()
            .w_5()
            .h_5()
            .min_w_5()
            .min_h_5()
            .max_w_5()
            .max_h_5()
            .w_6()
            .h_6()
            .min_w_6()
            .min_h_6()
            .max_w_6()
            .max_h_6()
            .w_8()
            .h_8()
            .min_w_8()
            .min_h_8()
            .max_w_8()
            .max_h_8()
            .w_10()
            .h_10()
            .min_w_10()
            .min_h_10()
            .max_w_10()
            .max_h_10()
            .w_11()
            .h_11()
            .min_w_11()
            .min_h_11()
            .max_w_11()
            .max_h_11()
            .build();
    }
}
