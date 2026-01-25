use crate::Corners4;
use crate::{
    ChromeRefinement, ColorRef, Edges4, Items, Justify, LayoutRefinement, LengthRefinement,
    MarginEdge, MetricRef, Radius, SignedMetricRef, Space,
};
use fret_core::{FontWeight, Px, TextOverflow, TextWrap};
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

impl UiBuilder<crate::ui::TextBox> {
    pub fn text_sm(mut self) -> Self {
        self.inner.preset = crate::ui::TextPreset::Sm;
        self.inner.wrap = TextWrap::Word;
        self
    }

    pub fn text_base(mut self) -> Self {
        self.inner.preset = crate::ui::TextPreset::Base;
        self.inner.wrap = TextWrap::Word;
        self
    }

    pub fn text_prose(mut self) -> Self {
        self.inner.preset = crate::ui::TextPreset::Prose;
        self.inner.wrap = TextWrap::Word;
        self
    }

    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.inner.weight_override = Some(weight);
        self
    }

    pub fn font_normal(self) -> Self {
        self.font_weight(FontWeight::NORMAL)
    }

    pub fn font_medium(self) -> Self {
        self.font_weight(FontWeight::MEDIUM)
    }

    pub fn font_semibold(self) -> Self {
        self.font_weight(FontWeight::SEMIBOLD)
    }

    pub fn font_bold(self) -> Self {
        self.font_weight(FontWeight::BOLD)
    }

    pub fn text_color(mut self, color: ColorRef) -> Self {
        self.inner.color_override = Some(color);
        self
    }

    pub fn text_size_px(mut self, size: Px) -> Self {
        self.inner.size_override = Some(size);
        self
    }

    pub fn line_height_px(mut self, height: Px) -> Self {
        self.inner.line_height_override = Some(height);
        self
    }

    pub fn letter_spacing_em(mut self, letter_spacing_em: f32) -> Self {
        self.inner.letter_spacing_em_override = Some(letter_spacing_em);
        self
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.inner.wrap = wrap;
        self
    }

    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.inner.overflow = overflow;
        self
    }

    pub fn nowrap(self) -> Self {
        self.wrap(TextWrap::None).overflow(TextOverflow::Clip)
    }

    pub fn truncate(self) -> Self {
        self.wrap(TextWrap::None).overflow(TextOverflow::Ellipsis)
    }
}

impl UiBuilder<crate::ui::RawTextBox> {
    pub fn text_color(mut self, color: ColorRef) -> Self {
        self.inner.color_override = Some(color);
        self
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.inner.wrap = wrap;
        self
    }

    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.inner.overflow = overflow;
        self
    }

    pub fn nowrap(self) -> Self {
        self.wrap(TextWrap::None).overflow(TextOverflow::Clip)
    }

    pub fn truncate(self) -> Self {
        self.wrap(TextWrap::None).overflow(TextOverflow::Ellipsis)
    }
}

impl<T: UiSupportsChrome> UiBuilder<T> {
    pub fn paddings(self, paddings: impl Into<Edges4<MetricRef>>) -> Self {
        self.style_with(|mut c| {
            let Edges4 {
                top,
                right,
                bottom,
                left,
            } = paddings.into();
            let mut padding = c.padding.unwrap_or_default();
            padding.top = Some(top);
            padding.right = Some(right);
            padding.bottom = Some(bottom);
            padding.left = Some(left);
            c.padding = Some(padding);
            c
        })
    }

    pub fn padding(self, padding: impl Into<MetricRef>) -> Self {
        self.paddings(Edges4::all(padding.into()))
    }

    pub fn padding_px(self, px: Px) -> Self {
        self.padding(px)
    }

    pub fn padding_space(self, space: Space) -> Self {
        self.padding(space)
    }

    pub fn focused_border(self) -> Self {
        self.style_with(ChromeRefinement::focused_border)
    }

    pub fn corner_radii(self, radii: impl Into<Corners4<MetricRef>>) -> Self {
        self.style_with(|c| c.corner_radii(radii))
    }

    pub fn rounded_tl(self, radius: Radius) -> Self {
        self.style_with(|c| c.rounded_tl(radius))
    }

    pub fn rounded_tr(self, radius: Radius) -> Self {
        self.style_with(|c| c.rounded_tr(radius))
    }

    pub fn rounded_br(self, radius: Radius) -> Self {
        self.style_with(|c| c.rounded_br(radius))
    }

    pub fn rounded_bl(self, radius: Radius) -> Self {
        self.style_with(|c| c.rounded_bl(radius))
    }

    pub fn shadow_none(self) -> Self {
        self.style_with(ChromeRefinement::shadow_none)
    }

    pub fn shadow_xs(self) -> Self {
        self.style_with(ChromeRefinement::shadow_xs)
    }

    pub fn shadow_sm(self) -> Self {
        self.style_with(ChromeRefinement::shadow_sm)
    }

    pub fn shadow_md(self) -> Self {
        self.style_with(ChromeRefinement::shadow_md)
    }

    pub fn shadow_lg(self) -> Self {
        self.style_with(ChromeRefinement::shadow_lg)
    }

    pub fn debug_border(self, color: ColorRef) -> Self {
        self.style_with(|c| c.debug_border(color))
    }

    pub fn debug_border_primary(self) -> Self {
        self.style_with(ChromeRefinement::debug_border_primary)
    }

    pub fn debug_border_destructive(self) -> Self {
        self.style_with(ChromeRefinement::debug_border_destructive)
    }

    pub fn debug_border_ring(self) -> Self {
        self.style_with(ChromeRefinement::debug_border_ring)
    }

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
    pub fn insets(self, insets: impl Into<Edges4<SignedMetricRef>>) -> Self {
        self.layout_with(|mut l| {
            let Edges4 {
                top,
                right,
                bottom,
                left,
            } = insets.into();
            let mut inset = l.inset.unwrap_or_default();
            inset.top = Some(top);
            inset.right = Some(right);
            inset.bottom = Some(bottom);
            inset.left = Some(left);
            l.inset = Some(inset);
            l
        })
    }

    pub fn margins(self, margins: impl Into<Edges4<MarginEdge>>) -> Self {
        self.layout_with(|mut l| {
            let Edges4 {
                top,
                right,
                bottom,
                left,
            } = margins.into();
            let mut margin = l.margin.unwrap_or_default();
            margin.top = Some(top.into());
            margin.right = Some(right.into());
            margin.bottom = Some(bottom.into());
            margin.left = Some(left.into());
            l.margin = Some(margin);
            l
        })
    }

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

    pub fn min_w(self, width: impl Into<MetricRef>) -> Self {
        self.layout_with(|l| l.min_w(width))
    }

    pub fn min_w_space(self, width: Space) -> Self {
        self.layout_with(|l| l.min_w_space(width))
    }

    pub fn min_h(self, height: impl Into<MetricRef>) -> Self {
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

    pub fn w_px(self, width: impl Into<MetricRef>) -> Self {
        self.layout_with(|l| l.w_px(width))
    }

    pub fn w_space(self, width: Space) -> Self {
        self.layout_with(|l| l.w_space(width))
    }

    pub fn h_px(self, height: impl Into<MetricRef>) -> Self {
        self.layout_with(|l| l.h_px(height))
    }

    pub fn h_space(self, height: Space) -> Self {
        self.layout_with(|l| l.h_space(height))
    }

    pub fn max_w(self, width: impl Into<MetricRef>) -> Self {
        self.layout_with(|l| l.max_w(width))
    }

    pub fn max_w_space(self, width: Space) -> Self {
        self.layout_with(|l| l.max_w_space(width))
    }

    pub fn max_h(self, height: impl Into<MetricRef>) -> Self {
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

impl<H, F> UiBuilder<crate::ui::FlexBox<H, F>> {
    pub fn gap(mut self, gap: impl Into<MetricRef>) -> Self {
        self.inner.gap = gap.into();
        self
    }

    pub fn gap_px(self, gap: Px) -> Self {
        self.gap(gap)
    }

    pub fn gap_metric(self, gap: MetricRef) -> Self {
        self.gap(gap)
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.inner.justify = justify;
        self
    }

    pub fn justify_start(self) -> Self {
        self.justify(Justify::Start)
    }

    pub fn justify_center(self) -> Self {
        self.justify(Justify::Center)
    }

    pub fn justify_end(self) -> Self {
        self.justify(Justify::End)
    }

    pub fn justify_between(self) -> Self {
        self.justify(Justify::Between)
    }

    pub fn items(mut self, items: Items) -> Self {
        self.inner.items = items;
        self
    }

    pub fn items_start(self) -> Self {
        self.items(Items::Start)
    }

    pub fn items_center(self) -> Self {
        self.items(Items::Center)
    }

    pub fn items_end(self) -> Self {
        self.items(Items::End)
    }

    pub fn items_stretch(self) -> Self {
        self.items(Items::Stretch)
    }

    pub fn wrap(mut self) -> Self {
        self.inner.wrap = true;
        self
    }

    pub fn no_wrap(mut self) -> Self {
        self.inner.wrap = false;
        self
    }
}

impl<T: UiPatchTarget + UiIntoElement> UiBuilder<T> {
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.build().into_element(cx)
    }
}

impl<H: UiHost, F, I> UiBuilder<crate::ui::FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.build().into_element(cx)
    }
}

impl<H: UiHost, F, I> UiBuilder<crate::ui::ContainerBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.build().into_element(cx)
    }
}

impl<H: UiHost, F, I> UiBuilder<crate::ui::StackBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
    use fret_core::Axis;
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
    fn ui_builder_edges4_helpers_write_fields() {
        let dummy = Dummy::default()
            .ui()
            .paddings(Edges4::trbl(Space::N1, Space::N2, Space::N3, Space::N4))
            .margins(Edges4::trbl(
                MarginEdge::auto(),
                Space::N2.into(),
                Space::N3.into(),
                Space::N4.into(),
            ))
            .insets(Edges4::all(Space::N1).neg())
            .focused_border()
            .corner_radii(Corners4::tltrbrbl(
                Radius::Sm,
                Radius::Md,
                Radius::Lg,
                Radius::Full,
            ))
            .rounded_tl(Radius::Lg)
            .shadow_md()
            .debug_border_primary()
            .debug_border_destructive()
            .build();

        let padding = dummy.chrome.padding.expect("expected padding refinement");
        match padding.top {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N1.token_key()),
            other => panic!("expected top padding token, got {other:?}"),
        }
        match padding.right {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N2.token_key()),
            other => panic!("expected right padding token, got {other:?}"),
        }
        match padding.bottom {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N3.token_key()),
            other => panic!("expected bottom padding token, got {other:?}"),
        }
        match padding.left {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, Space::N4.token_key()),
            other => panic!("expected left padding token, got {other:?}"),
        }

        let margin = dummy.layout.margin.expect("expected margin refinement");
        assert!(matches!(
            margin.top,
            Some(crate::style::MarginEdgeRefinement::Auto)
        ));
        match margin.right {
            Some(crate::style::MarginEdgeRefinement::Px(SignedMetricRef::Pos(
                MetricRef::Token { key, .. },
            ))) => assert_eq!(key, Space::N2.token_key()),
            other => panic!("expected right margin token, got {other:?}"),
        }

        let inset = dummy.layout.inset.expect("expected inset refinement");
        match inset.left {
            Some(SignedMetricRef::Neg(MetricRef::Token { key, .. })) => {
                assert_eq!(key, Space::N1.token_key())
            }
            other => panic!("expected left inset negative token, got {other:?}"),
        }

        match dummy.chrome.border_color {
            Some(ColorRef::Token { key, .. }) => assert_eq!(key, "destructive"),
            other => panic!("expected debug_border_destructive to set border_color, got {other:?}"),
        }

        assert_eq!(dummy.chrome.shadow, Some(crate::style::ShadowPreset::Md));

        let radii = dummy
            .chrome
            .corner_radii
            .expect("expected corner radii refinement");
        match radii.top_left {
            Some(MetricRef::Token { key, .. }) => assert_eq!(key, "component.radius.lg"),
            other => panic!("expected top_left token radius, got {other:?}"),
        }
    }

    #[test]
    fn ui_builder_forwards_full_vocabulary_smoke() {
        let _ = Dummy::default()
            .ui()
            // ChromeRefinement
            .paddings(Edges4::all(Space::N1))
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
            .margins(Edges4::all(Space::N1))
            .insets(Edges4::all(Space::N1))
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

    #[test]
    fn ui_flex_box_builder_records_gap_and_alignment() {
        let flex = crate::ui::FlexBox::<(), ()>::new(Axis::Horizontal, ())
            .ui()
            .gap(Space::N2)
            .justify_between()
            .items_center()
            .wrap()
            .build();

        assert_eq!(flex.gap, Space::N2);
        assert_eq!(flex.justify, Justify::Between);
        assert_eq!(flex.items, Items::Center);
        assert!(flex.wrap);
    }
}
