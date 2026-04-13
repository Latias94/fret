use std::hash::Hash;
use std::marker::PhantomData;
use std::panic::Location;
use std::sync::Arc;

pub use crate::children;

use smallvec::SmallVec;

use fret_core::{
    AttributedText, Axis, Edges, EffectChain, EffectMode, FontId, FontWeight, Px, TextAlign,
    TextOverflow, TextSpan, TextStyle, TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, EffectLayerProps, Elements, FlexProps, HoverRegionProps,
    InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, ScrollAxis, ScrollProps,
    ScrollbarAxis, ScrollbarProps, ScrollbarStyle, SelectableTextProps, SizeStyle, StackProps,
    StyledTextProps, TextProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, ElementContextAccess, Theme, UiHost};

use crate::declarative::style as decl_style;
use crate::declarative::text as decl_text;
use crate::{
    ChromeRefinement, IntoUiElement, Items, Justify, LayoutRefinement, LengthRefinement, MetricRef,
    Space, UiBuilder, UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout,
};

fn collect_ui_children<'a, H: UiHost + 'a, Cx, I>(cx: &mut Cx, iter: I) -> SmallVec<[AnyElement; 8]>
where
    Cx: ElementContextAccess<'a, H>,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    let mut out: SmallVec<[AnyElement; 8]> = SmallVec::new();
    for child in iter {
        out.push(crate::land_child(cx, child));
    }
    out
}

fn metric_ref_is_zero(metric: &MetricRef) -> bool {
    match metric {
        MetricRef::Px(px) => px.0.abs() <= f32::EPSILON,
        MetricRef::Token { key, fallback } => {
            *key == crate::Space::N0.token_key()
                || matches!(fallback, crate::style::MetricFallback::Px(px) if px.0.abs() <= f32::EPSILON)
        }
    }
}

fn min_max_axis_refinement_requests_fill(length: &LengthRefinement) -> bool {
    match length {
        LengthRefinement::Auto => false,
        LengthRefinement::Fill | LengthRefinement::Fraction(_) => true,
        // `min_h_0()` / `max_h_0()` are escape hatches for shrink/clamp behavior; they should
        // not force wrapped scroll/flex roots to fill the main axis when the outer box is still
        // auto-sized.
        LengthRefinement::Px(metric) => !metric_ref_is_zero(metric),
    }
}

fn layout_requests_fill_on_axis(axis: Axis, layout: &LayoutRefinement) -> bool {
    let Some(size) = layout.size.as_ref() else {
        return false;
    };

    let (main, min, max) = match axis {
        Axis::Horizontal => (&size.width, &size.min_width, &size.max_width),
        Axis::Vertical => (&size.height, &size.min_height, &size.max_height),
    };

    matches!(
        main,
        Some(LengthRefinement::Px(_) | LengthRefinement::Fraction(_) | LengthRefinement::Fill)
    ) || min
        .as_ref()
        .is_some_and(min_max_axis_refinement_requests_fill)
        || max
            .as_ref()
            .is_some_and(min_max_axis_refinement_requests_fill)
}

fn scroll_root_needs_fill_on_axis(
    scroll_axis: ScrollAxis,
    axis: Axis,
    layout: &LayoutRefinement,
) -> bool {
    let scrolls_on_axis = match axis {
        Axis::Horizontal => scroll_axis.scroll_x(),
        Axis::Vertical => scroll_axis.scroll_y(),
    };

    !scrolls_on_axis || layout_requests_fill_on_axis(axis, layout)
}

fn flex_root_needs_fill_height(direction: Axis, layout: &LayoutRefinement) -> bool {
    let has_height_constraint = layout_requests_fill_on_axis(Axis::Vertical, layout);
    if has_height_constraint {
        return true;
    }

    matches!(direction, Axis::Vertical)
        && layout.flex_item.as_ref().is_some_and(|flex| {
            flex.grow.is_some_and(|grow| grow > 0.0)
                || matches!(
                    flex.basis,
                    Some(
                        LengthRefinement::Px(_)
                            | LengthRefinement::Fraction(_)
                            | LengthRefinement::Fill
                    )
                )
        })
}

fn apply_inner_flex_root_width_constraints(
    theme: &Theme,
    layout: &LayoutRefinement,
    force_width_fill: bool,
    flex_props: &mut FlexProps,
) {
    let resolved_layout = decl_style::layout_style(theme, layout.clone());
    let size = layout.size.as_ref();

    if size.and_then(|size| size.width.as_ref()).is_some() {
        flex_props.layout.size.width = resolved_layout.size.width;
    } else if force_width_fill {
        flex_props.layout.size.width = Length::Fill;
    }

    if size.and_then(|size| size.min_width.as_ref()).is_some() {
        flex_props.layout.size.min_width = resolved_layout.size.min_width;
    }

    if size.and_then(|size| size.max_width.as_ref()).is_some() {
        flex_props.layout.size.max_width = resolved_layout.size.max_width;
    }
}

/// Late-lands a single typed child into `Ui` / `Elements`.
///
/// This is the narrow default-path helper for render roots or wrapper closures that only need to
/// return one already-typed child without spelling `ui::children![cx; child].into()`.
#[track_caller]
pub fn single<'a, H: UiHost + 'a, Cx, T>(cx: &mut Cx, child: T) -> Elements
where
    Cx: ElementContextAccess<'a, H>,
    T: IntoUiElement<H>,
{
    Elements::from(crate::land_child(cx, child))
}

/// Extension helpers for `*_build` child sinks.
///
/// These helpers make builder-first layout code read more like direct composition without falling
/// back to `ui::children!` only to convert a heterogeneous list into `AnyElement` values.
pub trait UiElementSinkExt {
    fn push_ui<'a, H: UiHost + 'a, Cx, T>(&mut self, cx: &mut Cx, child: T)
    where
        Cx: ElementContextAccess<'a, H>,
        T: IntoUiElement<H>;

    fn extend_ui<'a, H: UiHost + 'a, Cx, I>(&mut self, cx: &mut Cx, children: I)
    where
        Cx: ElementContextAccess<'a, H>,
        I: IntoIterator,
        I::Item: IntoUiElement<H>;
}

impl UiElementSinkExt for Vec<AnyElement> {
    fn push_ui<'a, H: UiHost + 'a, Cx, T>(&mut self, cx: &mut Cx, child: T)
    where
        Cx: ElementContextAccess<'a, H>,
        T: IntoUiElement<H>,
    {
        self.push(crate::land_child(cx, child));
    }

    fn extend_ui<'a, H: UiHost + 'a, Cx, I>(&mut self, cx: &mut Cx, children: I)
    where
        Cx: ElementContextAccess<'a, H>,
        I: IntoIterator,
        I::Item: IntoUiElement<H>,
    {
        for child in children {
            self.push_ui(cx, child);
        }
    }
}

fn resolve_text_align_for_direction(
    align: TextAlign,
    direction: crate::primitives::direction::LayoutDirection,
) -> TextAlign {
    use crate::primitives::direction::LayoutDirection;

    match (align, direction) {
        (TextAlign::Start, LayoutDirection::Rtl) => TextAlign::End,
        (TextAlign::End, LayoutDirection::Rtl) => TextAlign::Start,
        _ => align,
    }
}

/// A patchable flex layout constructor for authoring ergonomics.
///
/// This is an ecosystem-only helper intended to reduce runtime-props boilerplate in layout-only
/// code while keeping layering rules intact (no policy in `crates/fret-ui`).
#[derive(Debug, Clone)]
pub struct FlexBox<H, F> {
    pub(crate) chrome: ChromeRefinement,
    pub(crate) layout: LayoutRefinement,
    pub(crate) direction: Axis,
    pub(crate) force_width_fill: bool,
    pub(crate) gap: MetricRef,
    pub(crate) gap_length: Option<LengthRefinement>,
    pub(crate) justify: Justify,
    pub(crate) items: Items,
    pub(crate) wrap: bool,
    pub(crate) children: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

/// Variant of [`FlexBox`] that collects children into a sink to avoid iterator borrow pitfalls.
#[derive(Debug)]
pub struct FlexBoxBuild<H, B> {
    pub(crate) chrome: ChromeRefinement,
    pub(crate) layout: LayoutRefinement,
    pub(crate) direction: Axis,
    pub(crate) force_width_fill: bool,
    pub(crate) gap: MetricRef,
    pub(crate) gap_length: Option<LengthRefinement>,
    pub(crate) justify: Justify,
    pub(crate) items: Items,
    pub(crate) wrap: bool,
    pub(crate) build: Option<B>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

impl<H, F> FlexBox<H, F> {
    pub fn new(direction: Axis, children: F) -> Self {
        let items = match direction {
            Axis::Horizontal => Items::Center,
            Axis::Vertical => Items::Stretch,
        };
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            direction,
            force_width_fill: true,
            gap: MetricRef::space(Space::N0),
            gap_length: None,
            justify: Justify::Start,
            items,
            wrap: false,
            children: Some(children),
            _phantom: PhantomData,
        }
    }
}

impl<H, B> FlexBoxBuild<H, B> {
    pub fn new(direction: Axis, build: B) -> Self {
        let items = match direction {
            Axis::Horizontal => Items::Center,
            Axis::Vertical => Items::Stretch,
        };
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            direction,
            force_width_fill: true,
            gap: MetricRef::space(Space::N0),
            gap_length: None,
            justify: Justify::Start,
            items,
            wrap: false,
            build: Some(build),
            _phantom: PhantomData,
        }
    }
}

impl<H, F> UiPatchTarget for FlexBox<H, F> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, F> UiSupportsChrome for FlexBox<H, F> {}
impl<H, F> UiSupportsLayout for FlexBox<H, F> {}

impl<H, B> UiPatchTarget for FlexBoxBuild<H, B> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, B> UiSupportsChrome for FlexBoxBuild<H, B> {}
impl<H, B> UiSupportsLayout for FlexBoxBuild<H, B> {}

impl<H: UiHost, F, I> FlexBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let needs_fill_height = flex_root_needs_fill_height(self.direction, &self.layout);
        let layout = self.layout;
        let container = decl_style::container_props(theme, self.chrome, layout.clone());

        let gap = self.gap_length.as_ref().and_then(|l| match l {
            LengthRefinement::Auto => None,
            LengthRefinement::Px(m) => Some(fret_ui::element::SpacingLength::Px(m.resolve(theme))),
            LengthRefinement::Fraction(f) => Some(fret_ui::element::SpacingLength::Fraction(*f)),
            LengthRefinement::Fill => Some(fret_ui::element::SpacingLength::Fill),
        });
        let gap =
            gap.unwrap_or_else(|| fret_ui::element::SpacingLength::Px(self.gap.resolve(theme)));
        let mut flex_props = FlexProps {
            direction: self.direction,
            gap,
            padding: Edges::all(Px(0.0)).into(),
            justify: self.justify.to_main_align(),
            align: self.items.to_cross_align(),
            wrap: self.wrap,
            ..Default::default()
        };
        apply_inner_flex_root_width_constraints(
            theme,
            &layout,
            self.force_width_fill,
            &mut flex_props,
        );
        if needs_fill_height {
            flex_props.layout.size.height = Length::Fill;
        }

        let children = self.children.expect("expected flex children closure");
        cx.container(container, move |cx| {
            vec![cx.flex(flex_props, move |cx| {
                let children = children(cx);
                collect_ui_children(cx, children)
            })]
        })
    }
}

impl<H: UiHost, B> FlexBoxBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let needs_fill_height = flex_root_needs_fill_height(self.direction, &self.layout);
        let layout = self.layout;
        let container = decl_style::container_props(theme, self.chrome, layout.clone());

        let gap = self.gap_length.as_ref().and_then(|l| match l {
            LengthRefinement::Auto => None,
            LengthRefinement::Px(m) => Some(fret_ui::element::SpacingLength::Px(m.resolve(theme))),
            LengthRefinement::Fraction(f) => Some(fret_ui::element::SpacingLength::Fraction(*f)),
            LengthRefinement::Fill => Some(fret_ui::element::SpacingLength::Fill),
        });
        let gap =
            gap.unwrap_or_else(|| fret_ui::element::SpacingLength::Px(self.gap.resolve(theme)));
        let mut flex_props = FlexProps {
            direction: self.direction,
            gap,
            padding: Edges::all(Px(0.0)).into(),
            justify: self.justify.to_main_align(),
            align: self.items.to_cross_align(),
            wrap: self.wrap,
            ..Default::default()
        };
        apply_inner_flex_root_width_constraints(
            theme,
            &layout,
            self.force_width_fill,
            &mut flex_props,
        );
        if needs_fill_height {
            flex_props.layout.size.height = Length::Fill;
        }

        let build = self.build.expect("expected flex build closure");
        cx.container(container, move |cx| {
            vec![cx.flex(flex_props, move |cx| {
                let mut out = Vec::new();
                build(cx, &mut out);
                out
            })]
        })
    }
}

impl<H: UiHost, F, I> IntoUiElement<H> for FlexBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        FlexBox::<H, F>::into_element(self, cx)
    }
}

impl<H: UiHost, B> IntoUiElement<H> for FlexBoxBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        FlexBoxBuild::<H, B>::into_element(self, cx)
    }
}

/// Returns a patchable horizontal flex layout builder.
///
/// Usage:
/// - `ui::h_flex(|cx| vec![...]).gap(Space::N2).px_2().into_element(cx)`
pub fn h_flex<H: UiHost, F, I>(children: F) -> UiBuilder<FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    UiBuilder::new(FlexBox::new(Axis::Horizontal, children))
}

/// Returns a patchable horizontal flex layout builder that does **not** force `width: fill`.
///
/// Use this when you want the row to shrink-wrap its contents (or to avoid inflating child hit
/// boxes due to a fill-width flex root).
pub fn h_row<H: UiHost, F, I>(children: F) -> UiBuilder<FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    let mut flex = FlexBox::new(Axis::Horizontal, children);
    flex.force_width_fill = false;
    UiBuilder::new(flex)
}

/// Variant of [`h_flex`] that avoids iterator borrow pitfalls by collecting into a sink.
///
/// Use this when the natural authoring form is an iterator that captures `&mut cx` (e.g.
/// `items.iter().map(|it| cx.keyed(...))`), which cannot be returned directly.
pub fn h_flex_build<H: UiHost, B>(build: B) -> UiBuilder<FlexBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    UiBuilder::new(FlexBoxBuild::new(Axis::Horizontal, build))
}

/// Variant of [`h_row`] that avoids iterator borrow pitfalls by collecting into a sink.
pub fn h_row_build<H: UiHost, B>(build: B) -> UiBuilder<FlexBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    let mut flex = FlexBoxBuild::new(Axis::Horizontal, build);
    flex.force_width_fill = false;
    UiBuilder::new(flex)
}

/// Returns a patchable vertical flex layout builder.
pub fn v_flex<H: UiHost, F, I>(children: F) -> UiBuilder<FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    UiBuilder::new(FlexBox::new(Axis::Vertical, children))
}

/// Returns a patchable vertical flex layout builder that does **not** force `width: fill`.
///
/// Use this when you want the column to shrink-wrap its contents (or to avoid inflating child hit
/// boxes due to a fill-width flex root).
pub fn v_stack<H: UiHost, F, I>(children: F) -> UiBuilder<FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    let mut flex = FlexBox::new(Axis::Vertical, children);
    flex.force_width_fill = false;
    UiBuilder::new(flex)
}

/// Variant of [`v_flex`] that avoids iterator borrow pitfalls by collecting into a sink.
///
/// Use this when the natural authoring form is an iterator that captures `&mut cx` (e.g.
/// `items.iter().map(|it| cx.keyed(...))`), which cannot be returned directly.
pub fn v_flex_build<H: UiHost, B>(build: B) -> UiBuilder<FlexBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    UiBuilder::new(FlexBoxBuild::new(Axis::Vertical, build))
}

/// Variant of [`v_stack`] that avoids iterator borrow pitfalls by collecting into a sink.
pub fn v_stack_build<H: UiHost, B>(build: B) -> UiBuilder<FlexBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    let mut flex = FlexBoxBuild::new(Axis::Vertical, build);
    flex.force_width_fill = false;
    UiBuilder::new(flex)
}

/// A patchable container constructor for authoring ergonomics.
///
/// This is intended to be the default “box” layout node in the fluent authoring surface.
#[derive(Debug, Clone)]
pub struct ContainerBox<H, F> {
    pub(crate) chrome: ChromeRefinement,
    pub(crate) layout: LayoutRefinement,
    pub(crate) children: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

/// Variant of [`ContainerBox`] that collects children into a sink to avoid iterator borrow pitfalls.
#[derive(Debug)]
pub struct ContainerBoxBuild<H, B> {
    pub(crate) chrome: ChromeRefinement,
    pub(crate) layout: LayoutRefinement,
    pub(crate) build: Option<B>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

/// A raw-container variant that preserves caller-provided [`ContainerProps`] while still allowing
/// builder-first child authoring to land at the last possible moment.
#[derive(Debug, Clone)]
pub struct ContainerPropsBox<H, F> {
    pub(crate) props: ContainerProps,
    pub(crate) children: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

/// Sink-based variant of [`ContainerPropsBox`] for iterator-heavy or borrow-sensitive child flows.
#[derive(Debug)]
pub struct ContainerPropsBoxBuild<H, B> {
    pub(crate) props: ContainerProps,
    pub(crate) build: Option<B>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

impl<H, F> ContainerBox<H, F> {
    pub fn new(children: F) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children: Some(children),
            _phantom: PhantomData,
        }
    }
}

impl<H, B> ContainerBoxBuild<H, B> {
    pub fn new(build: B) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            build: Some(build),
            _phantom: PhantomData,
        }
    }
}

impl<H, F> ContainerPropsBox<H, F> {
    pub fn new(props: ContainerProps, children: F) -> Self {
        Self {
            props,
            children: Some(children),
            _phantom: PhantomData,
        }
    }
}

impl<H, B> ContainerPropsBoxBuild<H, B> {
    pub fn new(props: ContainerProps, build: B) -> Self {
        Self {
            props,
            build: Some(build),
            _phantom: PhantomData,
        }
    }
}

impl<H, F> UiPatchTarget for ContainerBox<H, F> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, F> UiSupportsChrome for ContainerBox<H, F> {}
impl<H, F> UiSupportsLayout for ContainerBox<H, F> {}

impl<H, B> UiPatchTarget for ContainerBoxBuild<H, B> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, B> UiSupportsChrome for ContainerBoxBuild<H, B> {}
impl<H, B> UiSupportsLayout for ContainerBoxBuild<H, B> {}

impl<H: UiHost, F, I> ContainerBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let container = decl_style::container_props(theme, self.chrome, self.layout);
        let children = self.children.expect("expected container children closure");
        cx.container(container, move |cx| {
            let children = children(cx);
            collect_ui_children(cx, children)
        })
    }
}

impl<H: UiHost, B> ContainerBoxBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let container = decl_style::container_props(theme, self.chrome, self.layout);
        let build = self.build.expect("expected container build closure");
        cx.container(container, move |cx| {
            let mut out = Vec::new();
            build(cx, &mut out);
            out
        })
    }
}

impl<H: UiHost, F, I> ContainerPropsBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = self.props;
        let children = self
            .children
            .expect("expected container-props children closure");
        cx.container(props, move |cx| {
            let children = children(cx);
            collect_ui_children(cx, children)
        })
    }
}

impl<H: UiHost, B> ContainerPropsBoxBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = self.props;
        let build = self.build.expect("expected container-props build closure");
        cx.container(props, move |cx| {
            let mut out = Vec::new();
            build(cx, &mut out);
            out
        })
    }
}

impl<H: UiHost, F, I> IntoUiElement<H> for ContainerBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ContainerBox::<H, F>::into_element(self, cx)
    }
}

impl<H: UiHost, B> IntoUiElement<H> for ContainerBoxBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ContainerBoxBuild::<H, B>::into_element(self, cx)
    }
}

impl<H: UiHost, F, I> IntoUiElement<H> for ContainerPropsBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ContainerPropsBox::<H, F>::into_element(self, cx)
    }
}

impl<H: UiHost, B> IntoUiElement<H> for ContainerPropsBoxBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ContainerPropsBoxBuild::<H, B>::into_element(self, cx)
    }
}

/// Returns a patchable container builder.
///
/// Usage:
/// - `ui::container(|cx| vec![...]).px_2().into_element(cx)`
pub fn container<H: UiHost, F, I>(children: F) -> UiBuilder<ContainerBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    UiBuilder::new(ContainerBox::new(children))
}

/// Variant of [`container`] that avoids iterator borrow pitfalls by collecting into a sink.
pub fn container_build<H: UiHost, B>(build: B) -> UiBuilder<ContainerBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    UiBuilder::new(ContainerBoxBuild::new(build))
}

/// Returns a raw `ContainerProps` root that still keeps child authoring on the late-landing path.
pub fn container_props<H: UiHost, F, I>(
    props: ContainerProps,
    children: F,
) -> UiBuilder<ContainerPropsBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    UiBuilder::new(ContainerPropsBox::new(props, children))
}

/// Sink-based variant of [`container_props`] for iterator-heavy or borrow-sensitive child flows.
pub fn container_props_build<H: UiHost, B>(
    props: ContainerProps,
    build: B,
) -> UiBuilder<ContainerPropsBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    UiBuilder::new(ContainerPropsBoxBuild::new(props, build))
}

/// A patchable scroll area constructor for authoring ergonomics.
///
/// This is a thin wrapper over the runtime `Scroll` + `Scrollbar` elements with sensible defaults.
#[derive(Debug, Clone)]
pub struct ScrollAreaBox<H, F> {
    pub(crate) chrome: ChromeRefinement,
    pub(crate) layout: LayoutRefinement,
    pub(crate) axis: ScrollAxis,
    pub(crate) show_scrollbar_x: bool,
    pub(crate) show_scrollbar_y: bool,
    pub(crate) handle: Option<ScrollHandle>,
    pub(crate) viewport_test_id: Option<Arc<str>>,
    pub(crate) children: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

/// Variant of [`ScrollAreaBox`] that collects children into a sink to avoid iterator borrow pitfalls.
#[derive(Debug)]
pub struct ScrollAreaBoxBuild<H, B> {
    pub(crate) chrome: ChromeRefinement,
    pub(crate) layout: LayoutRefinement,
    pub(crate) axis: ScrollAxis,
    pub(crate) show_scrollbar_x: bool,
    pub(crate) show_scrollbar_y: bool,
    pub(crate) handle: Option<ScrollHandle>,
    pub(crate) viewport_test_id: Option<Arc<str>>,
    pub(crate) build: Option<B>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

impl<H, F> ScrollAreaBox<H, F> {
    pub fn new(children: F) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            axis: ScrollAxis::Y,
            show_scrollbar_x: false,
            show_scrollbar_y: true,
            handle: None,
            viewport_test_id: None,
            children: Some(children),
            _phantom: PhantomData,
        }
    }
}

impl<H, B> ScrollAreaBoxBuild<H, B> {
    pub fn new(build: B) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            axis: ScrollAxis::Y,
            show_scrollbar_x: false,
            show_scrollbar_y: true,
            handle: None,
            viewport_test_id: None,
            build: Some(build),
            _phantom: PhantomData,
        }
    }
}

impl<H, F> UiPatchTarget for ScrollAreaBox<H, F> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, F> UiSupportsChrome for ScrollAreaBox<H, F> {}
impl<H, F> UiSupportsLayout for ScrollAreaBox<H, F> {}

impl<H, B> UiPatchTarget for ScrollAreaBoxBuild<H, B> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, B> UiSupportsChrome for ScrollAreaBoxBuild<H, B> {}
impl<H, B> UiSupportsLayout for ScrollAreaBoxBuild<H, B> {}

impl<H: UiHost, F, I> ScrollAreaBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let layout = self.layout;
        let (container, scrollbar_w, thumb, thumb_hover, corner_bg) = {
            let theme = Theme::global(&*cx.app);
            let container = decl_style::container_props(theme, self.chrome, layout.clone());

            let scrollbar_w = theme.metric_token("metric.scrollbar.width");
            let thumb = theme.color_token("scrollbar.thumb.background");
            let thumb_hover = theme.color_token("scrollbar.thumb.hover.background");
            let corner_bg = theme
                .color_by_key("scrollbar.corner.background")
                .or_else(|| theme.color_by_key("scrollbar.track.background"))
                .unwrap_or(fret_core::Color::TRANSPARENT);
            (container, scrollbar_w, thumb, thumb_hover, corner_bg)
        };

        let axis = self.axis;
        let show_scrollbar_x = self.show_scrollbar_x;
        let show_scrollbar_y = self.show_scrollbar_y;
        let provided_handle = self.handle;
        let viewport_test_id = self.viewport_test_id;
        let children = self.children.expect("expected scroll children closure");
        let fill_width = scroll_root_needs_fill_on_axis(axis, Axis::Horizontal, &layout);
        let fill_height = scroll_root_needs_fill_on_axis(axis, Axis::Vertical, &layout);

        cx.container(container, move |cx| {
            let handle = cx.slot_state(ScrollHandle::default, |h| {
                if let Some(handle) = provided_handle.clone() {
                    *h = handle;
                }
                h.clone()
            });

            let mut scroll_layout = LayoutStyle::default();
            scroll_layout.size.width = if fill_width {
                Length::Fill
            } else {
                Length::Auto
            };
            scroll_layout.size.height = if fill_height {
                Length::Fill
            } else {
                Length::Auto
            };
            scroll_layout.overflow = Overflow::Clip;

            let mut scroll = cx.scroll(
                ScrollProps {
                    layout: scroll_layout,
                    axis,
                    scroll_handle: Some(handle.clone()),
                    ..Default::default()
                },
                move |cx| {
                    let children = children(cx);
                    collect_ui_children(cx, children)
                },
            );
            if let Some(test_id) = viewport_test_id.clone() {
                scroll = scroll.test_id(test_id);
            }

            let scroll_id = scroll.id;
            let mut out = vec![scroll];

            if show_scrollbar_y {
                let scrollbar_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: Some(Px(0.0)).into(),
                        right: Some(Px(0.0)).into(),
                        bottom: Some(if show_scrollbar_x {
                            scrollbar_w
                        } else {
                            Px(0.0)
                        })
                        .into(),
                        left: None.into(),
                    },
                    size: SizeStyle {
                        width: Length::Px(scrollbar_w),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                out.push(cx.scrollbar(ScrollbarProps {
                    layout: scrollbar_layout,
                    axis: ScrollbarAxis::Vertical,
                    scroll_target: Some(scroll_id),
                    scroll_handle: handle.clone(),
                    style: ScrollbarStyle {
                        thumb,
                        thumb_hover,
                        ..Default::default()
                    },
                }));
            }

            if show_scrollbar_x {
                let scrollbar_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: None.into(),
                        right: Some(if show_scrollbar_y {
                            scrollbar_w
                        } else {
                            Px(0.0)
                        })
                        .into(),
                        bottom: Some(Px(0.0)).into(),
                        left: Some(Px(0.0)).into(),
                    },
                    size: SizeStyle {
                        height: Length::Px(scrollbar_w),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                out.push(cx.scrollbar(ScrollbarProps {
                    layout: scrollbar_layout,
                    axis: ScrollbarAxis::Horizontal,
                    scroll_target: Some(scroll_id),
                    scroll_handle: handle.clone(),
                    style: ScrollbarStyle {
                        thumb,
                        thumb_hover,
                        ..Default::default()
                    },
                }));
            }

            if show_scrollbar_x && show_scrollbar_y {
                let corner_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: None.into(),
                        right: Some(Px(0.0)).into(),
                        bottom: Some(Px(0.0)).into(),
                        left: None.into(),
                    },
                    size: SizeStyle {
                        width: Length::Px(scrollbar_w),
                        height: Length::Px(scrollbar_w),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                out.push(cx.container(
                    ContainerProps {
                        layout: corner_layout,
                        background: Some(corner_bg),
                        ..Default::default()
                    },
                    |_cx| [],
                ));
            }

            out
        })
    }
}

impl<H: UiHost, B> ScrollAreaBoxBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let layout = self.layout;
        let (container, scrollbar_w, thumb, thumb_hover, corner_bg) = {
            let theme = Theme::global(&*cx.app);
            let container = decl_style::container_props(theme, self.chrome, layout.clone());
            let scrollbar_w = theme.metric_token("metric.scrollbar.width");
            let thumb = theme.color_token("scrollbar.thumb.background");
            let thumb_hover = theme.color_token("scrollbar.thumb.hover.background");
            let corner_bg = theme.color_token("scrollbar.track.background");
            (container, scrollbar_w, thumb, thumb_hover, corner_bg)
        };

        let axis = self.axis;
        let show_scrollbar_x = self.show_scrollbar_x;
        let show_scrollbar_y = self.show_scrollbar_y;
        let provided_handle = self.handle;
        let viewport_test_id = self.viewport_test_id;
        let build = self.build.expect("expected scroll area build closure");
        let fill_width = scroll_root_needs_fill_on_axis(axis, Axis::Horizontal, &layout);
        let fill_height = scroll_root_needs_fill_on_axis(axis, Axis::Vertical, &layout);

        cx.container(container, move |cx| {
            let handle = cx.slot_state(ScrollHandle::default, |h| {
                if let Some(handle) = provided_handle.clone() {
                    *h = handle;
                }
                h.clone()
            });

            let mut scroll_layout = LayoutStyle::default();
            scroll_layout.size.width = if fill_width {
                Length::Fill
            } else {
                Length::Auto
            };
            scroll_layout.size.height = if fill_height {
                Length::Fill
            } else {
                Length::Auto
            };
            scroll_layout.overflow = Overflow::Clip;

            let mut scroll = cx.scroll(
                ScrollProps {
                    layout: scroll_layout,
                    axis,
                    scroll_handle: Some(handle.clone()),
                    ..Default::default()
                },
                move |cx| {
                    let mut out = Vec::new();
                    build(cx, &mut out);
                    out
                },
            );
            if let Some(test_id) = viewport_test_id.clone() {
                scroll = scroll.test_id(test_id);
            }

            let scroll_id = scroll.id;
            let mut out = vec![scroll];

            if show_scrollbar_y {
                let scrollbar_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: Some(Px(0.0)).into(),
                        right: Some(Px(0.0)).into(),
                        bottom: Some(if show_scrollbar_x {
                            scrollbar_w
                        } else {
                            Px(0.0)
                        })
                        .into(),
                        left: None.into(),
                    },
                    size: SizeStyle {
                        width: Length::Px(scrollbar_w),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                out.push(cx.scrollbar(ScrollbarProps {
                    layout: scrollbar_layout,
                    axis: ScrollbarAxis::Vertical,
                    scroll_target: Some(scroll_id),
                    scroll_handle: handle.clone(),
                    style: ScrollbarStyle {
                        thumb,
                        thumb_hover,
                        ..Default::default()
                    },
                }));
            }

            if show_scrollbar_x {
                let scrollbar_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: None.into(),
                        right: Some(if show_scrollbar_y {
                            scrollbar_w
                        } else {
                            Px(0.0)
                        })
                        .into(),
                        bottom: Some(Px(0.0)).into(),
                        left: Some(Px(0.0)).into(),
                    },
                    size: SizeStyle {
                        height: Length::Px(scrollbar_w),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                out.push(cx.scrollbar(ScrollbarProps {
                    layout: scrollbar_layout,
                    axis: ScrollbarAxis::Horizontal,
                    scroll_target: Some(scroll_id),
                    scroll_handle: handle.clone(),
                    style: ScrollbarStyle {
                        thumb,
                        thumb_hover,
                        ..Default::default()
                    },
                }));
            }

            if show_scrollbar_x && show_scrollbar_y {
                let corner_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: None.into(),
                        right: Some(Px(0.0)).into(),
                        bottom: Some(Px(0.0)).into(),
                        left: None.into(),
                    },
                    size: SizeStyle {
                        width: Length::Px(scrollbar_w),
                        height: Length::Px(scrollbar_w),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                out.push(cx.container(
                    ContainerProps {
                        layout: corner_layout,
                        background: Some(corner_bg),
                        ..Default::default()
                    },
                    |_cx| [],
                ));
            }

            out
        })
    }
}

impl<H: UiHost, F, I> IntoUiElement<H> for ScrollAreaBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ScrollAreaBox::<H, F>::into_element(self, cx)
    }
}

impl<H: UiHost, B> IntoUiElement<H> for ScrollAreaBoxBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        ScrollAreaBoxBuild::<H, B>::into_element(self, cx)
    }
}

/// Returns a patchable scroll area builder.
///
/// Defaults:
/// - axis: vertical
/// - scrollbar: Y on, X off
pub fn scroll_area<H: UiHost, F, I>(children: F) -> UiBuilder<ScrollAreaBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    UiBuilder::new(ScrollAreaBox::new(children))
}

/// Variant of [`scroll_area`] that avoids iterator borrow pitfalls by collecting into a sink.
pub fn scroll_area_build<H: UiHost, B>(build: B) -> UiBuilder<ScrollAreaBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    UiBuilder::new(ScrollAreaBoxBuild::new(build))
}

/// A patchable stack layout constructor for authoring ergonomics.
///
/// The runtime `Stack` element is a positioned-container style layout: children can be absolutely
/// positioned, and non-absolute children are laid out against the same bounds.
#[derive(Debug, Clone)]
pub struct StackBox<H, F> {
    pub(crate) chrome: ChromeRefinement,
    pub(crate) layout: LayoutRefinement,
    pub(crate) children: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

impl<H, F> StackBox<H, F> {
    pub fn new(children: F) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children: Some(children),
            _phantom: PhantomData,
        }
    }
}

impl<H, F> UiPatchTarget for StackBox<H, F> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.chrome = self.chrome.merge(patch.chrome);
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, F> UiSupportsChrome for StackBox<H, F> {}
impl<H, F> UiSupportsLayout for StackBox<H, F> {}

impl<H: UiHost, F, I> StackBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let container = decl_style::container_props(theme, self.chrome, self.layout);
        let children = self.children.expect("expected stack children closure");

        cx.container(container, move |cx| {
            vec![cx.stack_props(StackProps::default(), move |cx| {
                let children = children(cx);
                collect_ui_children(cx, children)
            })]
        })
    }
}

impl<H: UiHost, F, I> IntoUiElement<H> for StackBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        StackBox::<H, F>::into_element(self, cx)
    }
}

/// Returns a patchable stack layout builder.
///
/// Usage:
/// - `ui::stack(|cx| vec![...]).inset(Space::N2).into_element(cx)`
pub fn stack<H: UiHost, F, I>(children: F) -> UiBuilder<StackBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    UiBuilder::new(StackBox::new(children))
}

/// A keyed identity wrapper that keeps the original `cx.keyed(...)` callsite stable across
/// builder-first / late-landing authoring paths.
#[derive(Debug, Clone)]
pub struct KeyedBox<H, K, F> {
    pub(crate) callsite: &'static Location<'static>,
    pub(crate) key: Option<K>,
    pub(crate) child: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

impl<H, K, F> KeyedBox<H, K, F> {
    fn new(callsite: &'static Location<'static>, key: K, child: F) -> Self {
        Self {
            callsite,
            key: Some(key),
            child: Some(child),
            _phantom: PhantomData,
        }
    }
}

impl<H, K, F> UiPatchTarget for KeyedBox<H, K, F> {
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost, K: Hash, F, T> KeyedBox<H, K, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> T,
    T: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Self {
            callsite,
            key,
            child,
            _phantom: _,
        } = self;
        let key = key.expect("keyed box key already taken");
        let child = child.expect("keyed box child already taken");
        cx.keyed_at(callsite, key, |cx| child(cx).into_element(cx))
    }
}

impl<H: UiHost, K: Hash, F, T> IntoUiElement<H> for KeyedBox<H, K, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> T,
    T: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        KeyedBox::<H, K, F>::into_element(self, cx)
    }
}

/// Returns an identity-preserving keyed wrapper for a single child subtree.
///
/// Prefer this over raw `cx.keyed(...)` inside `*_build(|cx, out| ...)` sinks when you want to
/// stay on the builder-first path and avoid materializing `AnyElement` early.
#[track_caller]
pub fn keyed<H: UiHost, K: Hash, F, T>(key: K, child: F) -> UiBuilder<KeyedBox<H, K, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> T,
    T: IntoUiElement<H>,
{
    UiBuilder::new(KeyedBox::new(Location::caller(), key, child))
}

/// Collects a keyed dynamic child list without forcing callers onto `*_build(|cx, out| ...)`.
///
/// This is the preferred authoring helper when a layout closure naturally wants to return
/// a conditional `Vec<AnyElement>`: empty-state content can still use `ui::children![cx; ...]`,
/// while the non-empty branch keeps stable keyed identity without open-coding
/// `for item in ... { out.push_ui(cx, ui::keyed(...)) }`.
///
/// Use [`for_each_keyed_with_cx`] when the per-row builder itself needs the keyed child scope.
#[track_caller]
pub fn for_each_keyed<H: UiHost, I, KF, BF, K, T>(
    cx: &mut ElementContext<'_, H>,
    items: I,
    key_of: KF,
    mut build: BF,
) -> Vec<AnyElement>
where
    I: IntoIterator,
    KF: FnMut(&I::Item) -> K,
    BF: FnMut(I::Item) -> T,
    K: Hash,
    T: IntoUiElement<H>,
{
    for_each_keyed_with_cx(cx, items, key_of, |_cx, item| build(item))
}

/// Collects a keyed dynamic child list while exposing the keyed child scope to each row builder.
///
/// Prefer this when each row needs its own keyed `cx` for row-local state, `cx.text(...)`,
/// nested local models, or other child-scope work that should happen inside the keyed boundary.
#[track_caller]
pub fn for_each_keyed_with_cx<H: UiHost, I, KF, BF, K, T>(
    cx: &mut ElementContext<'_, H>,
    items: I,
    mut key_of: KF,
    mut build: BF,
) -> Vec<AnyElement>
where
    I: IntoIterator,
    KF: FnMut(&I::Item) -> K,
    BF: FnMut(&mut ElementContext<'_, H>, I::Item) -> T,
    K: Hash,
    T: IntoUiElement<H>,
{
    let callsite = Location::caller();
    let mut out = Vec::new();

    for item in items {
        let key = key_of(&item);
        let build = &mut build;
        out.push(cx.keyed_at(callsite, key, |cx| build(cx, item).into_element(cx)));
    }

    out
}

#[derive(Debug, Clone)]
pub struct EffectLayerBox<H, F> {
    pub(crate) props: EffectLayerProps,
    pub(crate) layout: LayoutRefinement,
    pub(crate) children: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

#[derive(Debug)]
pub struct EffectLayerBoxBuild<H, B> {
    pub(crate) props: EffectLayerProps,
    pub(crate) layout: LayoutRefinement,
    pub(crate) build: Option<B>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

impl<H, F> EffectLayerBox<H, F> {
    pub fn new(props: EffectLayerProps, children: F) -> Self {
        Self {
            props,
            layout: LayoutRefinement::default(),
            children: Some(children),
            _phantom: PhantomData,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }
}

impl<H, B> EffectLayerBoxBuild<H, B> {
    pub fn new(props: EffectLayerProps, build: B) -> Self {
        Self {
            props,
            layout: LayoutRefinement::default(),
            build: Some(build),
            _phantom: PhantomData,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }
}

impl<H, F> UiPatchTarget for EffectLayerBox<H, F> {
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_layout(patch.layout)
    }
}

impl<H, B> UiPatchTarget for EffectLayerBoxBuild<H, B> {
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_layout(patch.layout)
    }
}

impl<H, F> UiSupportsLayout for EffectLayerBox<H, F> {}
impl<H, B> UiSupportsLayout for EffectLayerBoxBuild<H, B> {}

impl<H: UiHost, F, I> EffectLayerBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let mut props = self.props;
        decl_style::apply_layout_refinement(theme, self.layout, &mut props.layout);
        let children = self
            .children
            .expect("expected effect layer children closure");
        cx.effect_layer_props(props, move |cx| {
            let children = children(cx);
            collect_ui_children(cx, children)
        })
    }
}

impl<H: UiHost, B> EffectLayerBoxBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);
        let mut props = self.props;
        decl_style::apply_layout_refinement(theme, self.layout, &mut props.layout);
        let build = self.build.expect("expected effect layer build closure");
        cx.effect_layer_props(props, move |cx| {
            let mut out = Vec::new();
            build(cx, &mut out);
            out
        })
    }
}

impl<H: UiHost, F, I> IntoUiElement<H> for EffectLayerBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        EffectLayerBox::<H, F>::into_element(self, cx)
    }
}

impl<H: UiHost, B> IntoUiElement<H> for EffectLayerBoxBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        EffectLayerBoxBuild::<H, B>::into_element(self, cx)
    }
}

/// Returns a patchable effect-layer builder.
///
/// Usage:
/// - `ui::effect_layer(EffectMode::FilterContent, chain, |_cx| [child]).w_full().into_element(cx)`
pub fn effect_layer<H: UiHost, F, I>(
    mode: EffectMode,
    chain: EffectChain,
    children: F,
) -> UiBuilder<EffectLayerBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    effect_layer_props(
        EffectLayerProps {
            mode,
            chain,
            ..Default::default()
        },
        children,
    )
}

/// Returns a patchable effect-layer builder with explicit props.
pub fn effect_layer_props<H: UiHost, F, I>(
    props: EffectLayerProps,
    children: F,
) -> UiBuilder<EffectLayerBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    UiBuilder::new(EffectLayerBox::new(props, children))
}

/// Variant of [`effect_layer`] that avoids iterator borrow pitfalls by collecting into a sink.
pub fn effect_layer_build<H: UiHost, B>(
    mode: EffectMode,
    chain: EffectChain,
    build: B,
) -> UiBuilder<EffectLayerBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    effect_layer_props_build(
        EffectLayerProps {
            mode,
            chain,
            ..Default::default()
        },
        build,
    )
}

/// Variant of [`effect_layer_props`] that collects children into a sink.
pub fn effect_layer_props_build<H: UiHost, B>(
    props: EffectLayerProps,
    build: B,
) -> UiBuilder<EffectLayerBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    UiBuilder::new(EffectLayerBoxBuild::new(props, build))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextPreset {
    Xs,
    Sm,
    Base,
    Prose,
    Label,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextLineHeightPreset {
    Compact,
    Standard,
    Comfortable,
    Custom(f32),
}

impl TextLineHeightPreset {
    pub fn em(self) -> f32 {
        match self {
            Self::Compact => 1.2,
            Self::Standard => 1.3,
            Self::Comfortable => 1.618,
            Self::Custom(v) => v,
        }
    }
}

/// A patchable text constructor for authoring ergonomics.
///
/// This is intentionally small: it supports layout patching and a minimal text refinement surface
/// (size preset, weight, color, wrap/overflow).
#[derive(Debug, Clone)]
pub struct TextBox {
    pub(crate) layout: LayoutRefinement,
    pub(crate) text: Arc<str>,
    pub(crate) preset: TextPreset,
    pub(crate) selectable: bool,
    pub(crate) font_override: Option<FontId>,
    pub(crate) features_override: Vec<fret_core::TextFontFeatureSetting>,
    pub(crate) axes_override: Vec<fret_core::TextFontAxisSetting>,
    pub(crate) size_override: Option<Px>,
    pub(crate) line_height_override: Option<Px>,
    pub(crate) line_height_em_override: Option<f32>,
    pub(crate) line_height_policy_override: Option<fret_core::TextLineHeightPolicy>,
    pub(crate) ink_overflow_override: Option<fret_ui::element::TextInkOverflow>,
    pub(crate) weight_override: Option<FontWeight>,
    pub(crate) letter_spacing_em_override: Option<f32>,
    pub(crate) color_override: Option<crate::ColorRef>,
    pub(crate) wrap: TextWrap,
    pub(crate) overflow: TextOverflow,
    pub(crate) align: TextAlign,
    pub(crate) vertical_placement_override: Option<fret_core::TextVerticalPlacement>,
}

impl TextBox {
    pub fn new(text: impl Into<Arc<str>>, preset: TextPreset) -> Self {
        let wrap = match preset {
            TextPreset::Label => TextWrap::None,
            TextPreset::Xs | TextPreset::Sm | TextPreset::Base | TextPreset::Prose => {
                TextWrap::Word
            }
        };

        Self {
            layout: LayoutRefinement::default(),
            text: text.into(),
            preset,
            selectable: false,
            font_override: None,
            features_override: Vec::new(),
            axes_override: Vec::new(),
            size_override: None,
            line_height_override: None,
            line_height_em_override: None,
            line_height_policy_override: None,
            ink_overflow_override: None,
            weight_override: None,
            letter_spacing_em_override: None,
            color_override: None,
            wrap,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
            vertical_placement_override: None,
        }
    }
}

impl UiPatchTarget for TextBox {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl UiSupportsLayout for TextBox {}

impl<H: UiHost> IntoUiElement<H> for TextBox {
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let TextBox {
            layout: layout_refinement,
            text,
            preset,
            font_override,
            features_override,
            axes_override,
            size_override,
            line_height_override,
            line_height_em_override,
            line_height_policy_override,
            ink_overflow_override,
            weight_override,
            letter_spacing_em_override,
            color_override,
            wrap,
            overflow,
            align,
            vertical_placement_override,
            selectable,
        } = self;

        let direction = crate::primitives::direction::use_direction_in_scope(cx, None);
        let align = resolve_text_align_for_direction(align, direction);

        let (mut style, mut layout, default_label_line_height, resolved_color) = {
            let theme = Theme::global(&*cx.app);

            let (style, label_line_height) = match preset {
                TextPreset::Xs => (decl_text::text_xs_style(theme), None),
                TextPreset::Sm => (decl_text::text_sm_style(theme), None),
                TextPreset::Base => (decl_text::text_base_style(theme), None),
                TextPreset::Prose => (decl_text::text_prose_style(theme), None),
                TextPreset::Label => {
                    let (style, line_height) = decl_text::label_style(theme);
                    (style, Some(line_height))
                }
            };

            let layout = decl_style::layout_style(theme, layout_refinement);

            let resolved_color = color_override.as_ref().map(|c| c.resolve(theme));

            (style, layout, label_line_height, resolved_color)
        };

        if let Some(font) = font_override {
            style.font = font;
        }
        if !features_override.is_empty() {
            style.features.extend(features_override);
        }
        if !axes_override.is_empty() {
            style.axes.extend(axes_override);
        }
        if let Some(size) = size_override {
            style.size = size;
        }
        if let Some(height) = line_height_override {
            style.line_height = Some(height);
        }
        if let Some(line_height_em) = line_height_em_override {
            style.line_height_em = Some(line_height_em);
        }
        if let Some(weight) = weight_override {
            style.weight = weight;
        }
        if let Some(letter_spacing_em) = letter_spacing_em_override {
            style.letter_spacing_em = Some(letter_spacing_em);
        }
        if let Some(line_height_policy) = line_height_policy_override {
            style.line_height_policy = line_height_policy;
        }
        if let Some(vertical_placement) = vertical_placement_override {
            style.vertical_placement = vertical_placement;
        }

        // `TextPreset::Label` defaults to single-line text (Tailwind/shadcn `leading-none` label),
        // so we fix the line box height by default. If the caller explicitly enables wrapping,
        // keep the height auto so multi-line labels can expand without overlap.
        if preset == TextPreset::Label
            && wrap == TextWrap::None
            && matches!(layout.size.height, Length::Auto)
        {
            let line_height = line_height_override
                .or(default_label_line_height)
                .unwrap_or(Px(0.0));
            layout.size.height = Length::Px(line_height);
        }

        if selectable {
            let spans: Arc<[TextSpan]> = Arc::from([TextSpan::new(text.len())]);
            let rich = AttributedText::new(text, spans);
            cx.selectable_text_props(SelectableTextProps {
                layout,
                rich,
                style: Some(style),
                color: resolved_color,
                wrap,
                overflow,
                align,
                ink_overflow: ink_overflow_override.unwrap_or_default(),
                interactive_spans: Arc::from([]),
            })
        } else {
            cx.text_props(TextProps {
                layout,
                text,
                style: Some(style),
                color: resolved_color,
                wrap,
                overflow,
                align,
                ink_overflow: ink_overflow_override.unwrap_or_default(),
            })
        }
    }
}

/// Returns a patchable text builder (shadcn-aligned defaults).
///
/// Usage:
/// - `ui::text("Hello").text_sm().font_medium().into_element(cx)`
pub fn text(text: impl Into<Arc<str>>) -> UiBuilder<TextBox> {
    UiBuilder::new(TextBox::new(text, TextPreset::Sm))
}

/// Returns a patchable block text builder (full-width; shadcn-aligned defaults).
///
/// Use this for paragraph-like text that should wrap against the available inner width of its
/// containing block.
pub fn text_block(content: impl Into<Arc<str>>) -> UiBuilder<TextBox> {
    text(content).w_full()
}

/// Returns a patchable selectable text builder (drag-to-select + `edit.copy`).
///
/// Prefer this for read-only values (paths/IDs/snippets) and documentation-like content.
/// Avoid using it inside pressable/clickable rows: it intentionally captures left-drag selection
/// gestures and stops propagation (use a dedicated copy button instead).
pub fn selectable_text(text: impl Into<Arc<str>>) -> UiBuilder<TextBox> {
    crate::ui::text(text).selectable_on()
}

/// Returns a patchable selectable block text builder (full-width; drag-to-select + `edit.copy`).
pub fn selectable_text_block(content: impl Into<Arc<str>>) -> UiBuilder<TextBox> {
    selectable_text(content).w_full()
}

/// Returns a patchable label builder (single-line, medium weight).
pub fn label(text: impl Into<Arc<str>>) -> UiBuilder<TextBox> {
    UiBuilder::new(TextBox::new(text, TextPreset::Label))
}

/// A patchable unstyled text builder matching `TextProps::new(...)` defaults.
#[derive(Debug, Clone)]
pub struct RawTextBox {
    pub(crate) layout: LayoutRefinement,
    pub(crate) text: Arc<str>,
    pub(crate) color_override: Option<crate::ColorRef>,
    pub(crate) wrap: TextWrap,
    pub(crate) overflow: TextOverflow,
    pub(crate) align: TextAlign,
    pub(crate) ink_overflow_override: Option<fret_ui::element::TextInkOverflow>,
}

impl RawTextBox {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            layout: LayoutRefinement::default(),
            text: text.into(),
            color_override: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
            ink_overflow_override: None,
        }
    }
}

/// A patchable attributed-text builder matching `StyledTextProps::new(...)` defaults.
#[derive(Debug, Clone)]
pub struct RichTextBox {
    pub(crate) layout: LayoutRefinement,
    pub(crate) rich: AttributedText,
    pub(crate) style_override: Option<TextStyle>,
    pub(crate) color_override: Option<crate::ColorRef>,
    pub(crate) wrap: TextWrap,
    pub(crate) overflow: TextOverflow,
    pub(crate) align: TextAlign,
    pub(crate) ink_overflow_override: Option<fret_ui::element::TextInkOverflow>,
}

impl RichTextBox {
    pub fn new(rich: AttributedText) -> Self {
        Self {
            layout: LayoutRefinement::default(),
            rich,
            style_override: None,
            color_override: None,
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: TextAlign::Start,
            ink_overflow_override: None,
        }
    }
}

impl UiPatchTarget for RichTextBox {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl UiSupportsLayout for RichTextBox {}

impl<H: UiHost> IntoUiElement<H> for RichTextBox {
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let RichTextBox {
            layout: layout_refinement,
            rich,
            style_override,
            color_override,
            wrap,
            overflow,
            align,
            ink_overflow_override,
        } = self;

        let direction = crate::primitives::direction::use_direction_in_scope(cx, None);
        let align = resolve_text_align_for_direction(align, direction);

        let (layout, color) = {
            let theme = Theme::global(&*cx.app);
            let layout = decl_style::layout_style(theme, layout_refinement);
            let color = color_override.as_ref().map(|c| c.resolve(theme));
            (layout, color)
        };

        cx.styled_text_props(StyledTextProps {
            layout,
            rich,
            style: style_override,
            color,
            wrap,
            overflow,
            align,
            ink_overflow: ink_overflow_override.unwrap_or_default(),
        })
    }
}

/// Returns a patchable attributed-text builder matching `StyledTextProps::new(...)` defaults.
pub fn rich_text(rich: AttributedText) -> UiBuilder<RichTextBox> {
    UiBuilder::new(RichTextBox::new(rich))
}

/// A patchable hover-region builder for app-facing interaction shells.
#[derive(Debug)]
pub struct HoverRegionBox<H, F> {
    pub(crate) layout: LayoutRefinement,
    pub(crate) children: Option<F>,
    pub(crate) _phantom: PhantomData<fn() -> H>,
}

impl<H, F> HoverRegionBox<H, F> {
    pub fn new(children: F) -> Self {
        Self {
            layout: LayoutRefinement::default(),
            children: Some(children),
            _phantom: PhantomData,
        }
    }
}

impl<H, F> UiPatchTarget for HoverRegionBox<H, F> {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl<H, F> UiSupportsLayout for HoverRegionBox<H, F> {}

impl<H: UiHost, F, I> IntoUiElement<H> for HoverRegionBox<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>, bool) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let layout = {
            let theme = Theme::global(&*cx.app);
            decl_style::layout_style(theme, self.layout)
        };
        let children = self
            .children
            .expect("HoverRegionBox::into_element called more than once");

        cx.hover_region(HoverRegionProps { layout }, move |cx, hovered| {
            let built = children(cx, hovered);
            let mut out: SmallVec<[AnyElement; 8]> = SmallVec::new();
            for child in built {
                out.push(crate::land_child(cx, child));
            }
            out
        })
    }
}

/// Returns a patchable hover-region builder.
pub fn hover_region<H: UiHost, F, I>(children: F) -> UiBuilder<HoverRegionBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>, bool) -> I,
    I: IntoIterator,
    I::Item: IntoUiElement<H>,
{
    UiBuilder::new(HoverRegionBox::new(children))
}

impl UiPatchTarget for RawTextBox {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl UiSupportsLayout for RawTextBox {}

impl<H: UiHost> IntoUiElement<H> for RawTextBox {
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let RawTextBox {
            layout: layout_refinement,
            text,
            color_override,
            wrap,
            overflow,
            align,
            ink_overflow_override,
        } = self;

        let direction = crate::primitives::direction::use_direction_in_scope(cx, None);
        let align = resolve_text_align_for_direction(align, direction);

        let (layout, color) = {
            let theme = Theme::global(&*cx.app);
            let layout = decl_style::layout_style(theme, layout_refinement);
            let color = color_override.as_ref().map(|c| c.resolve(theme));
            (layout, color)
        };

        cx.text_props(TextProps {
            layout,
            text,
            style: None,
            color,
            wrap,
            overflow,
            align,
            ink_overflow: ink_overflow_override.unwrap_or_default(),
        })
    }
}

/// Returns a patchable unstyled text builder matching `TextProps::new(...)` defaults.
pub fn raw_text(text: impl Into<Arc<str>>) -> UiBuilder<RawTextBox> {
    UiBuilder::new(RawTextBox::new(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiExt;
    use crate::{LengthRefinement, MetricRef};
    use fret_app::App;
    use fret_core::SemanticsRole;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{ElementKind, Length};

    #[test]
    fn text_align_start_end_flip_under_rtl() {
        use crate::primitives::direction::LayoutDirection;

        assert_eq!(
            resolve_text_align_for_direction(TextAlign::Start, LayoutDirection::Ltr),
            TextAlign::Start
        );
        assert_eq!(
            resolve_text_align_for_direction(TextAlign::Start, LayoutDirection::Rtl),
            TextAlign::End
        );
        assert_eq!(
            resolve_text_align_for_direction(TextAlign::End, LayoutDirection::Rtl),
            TextAlign::Start
        );
        assert_eq!(
            resolve_text_align_for_direction(TextAlign::Center, LayoutDirection::Rtl),
            TextAlign::Center
        );
    }

    // Compile-only: ensure `ui::*` layout constructors accept `IntoUiElement<H>` children
    // (e.g. `UiBuilder<TextBox>`) without requiring call-site `.into_element(cx)`.
    #[allow(dead_code)]
    fn h_flex_accepts_ui_builder_children<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        h_flex(|_cx| [text("a"), text("b")])
            .gap(Space::N2)
            .into_element(cx)
    }

    #[allow(dead_code)]
    fn hover_region_accepts_ui_builder_children<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        hover_region(|_cx, hovered| [text(if hovered { "hovered" } else { "idle" }).truncate()])
            .w_full()
            .into_element(cx)
    }

    #[allow(dead_code)]
    fn rich_text_builder_lands_without_raw_styled_text_props<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        rich: AttributedText,
    ) -> AnyElement {
        rich_text(rich).truncate().w_full().into_element(cx)
    }

    // Compile-only: ensure `ui::children!` accepts nested layout builders without requiring an
    // explicit `.into_element(cx)` cliff at heterogeneous child boundaries.
    #[allow(dead_code)]
    fn children_macro_accepts_nested_layout_builders<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        h_flex(|cx| {
            children![cx;
                v_flex(|cx| children![cx; text("a"), text("b")]).gap(Space::N1),
                container(|cx| children![cx; text("c")]).p_1(),
            ]
        })
        .gap(Space::N2)
        .into_element(cx)
    }

    // Compile-only: ensure effect-layer roots accept late builder children without forcing the
    // child subtree to materialize before the effect boundary.
    #[allow(dead_code)]
    fn effect_layer_accepts_ui_builder_children<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        effect_layer(EffectMode::FilterContent, EffectChain::EMPTY, |_cx| {
            [container(|_cx| [text("effect child")]).w_full().h_full()]
        })
        .w_full()
        .into_element(cx)
    }

    // Compile-only: ensure keyed late-landing helpers preserve builder-first child authoring
    // without falling back to raw `cx.keyed(...)` + eager `AnyElement` materialization.
    #[allow(dead_code)]
    fn keyed_accepts_ui_builder_children<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        v_flex_build(|cx, out| {
            out.push_ui(cx, keyed("row-1", |_cx| text("row").test_id("row")));
        })
        .test_id("rows")
        .into_element(cx)
    }

    // Compile-only: ensure keyed list helpers can stay on the ordinary `v_flex(|cx| ..)` lane
    // without falling back to sink-based `v_flex_build(...)` authoring.
    #[allow(dead_code)]
    fn for_each_keyed_accepts_ui_builder_children<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let rows = [("row-1", "Alpha"), ("row-2", "Beta")];

        v_flex(|cx| for_each_keyed(cx, rows, |(id, _label)| *id, |(_id, label)| text(label)))
            .test_id("rows")
            .into_element(cx)
    }

    // Compile-only: ensure keyed list helpers can also hand row builders the inner keyed scope
    // when the row content needs to be assembled inside that boundary.
    #[allow(dead_code)]
    fn for_each_keyed_with_cx_accepts_row_local_scope<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let rows = [("row-1", "Alpha"), ("row-2", "Beta")];

        v_flex(|cx| {
            for_each_keyed_with_cx(
                cx,
                rows,
                |(id, _label)| *id,
                |_cx, (_id, label)| container(move |cx| [cx.text(label)]).test_id(label),
            )
        })
        .test_id("rows")
        .into_element(cx)
    }

    // Compile-only: ensure a single typed child can be late-landed into `Ui` / `Elements`
    // without spelling `ui::children![cx; child].into()` at the call site.
    #[allow(dead_code)]
    fn single_accepts_typed_child_roots<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
    ) -> fret_ui::element::Elements {
        single(
            cx,
            container(|_cx| [text("root child")])
                .w_full()
                .test_id("single-root"),
        )
    }

    // Compile-only: ensure low-level raw `ContainerProps` roots can still keep children on the
    // builder-first path without forcing eager landing before the host container boundary.
    #[allow(dead_code)]
    fn container_props_accepts_ui_builder_children<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        container_props(
            ContainerProps {
                layout,
                ..Default::default()
            },
            |_cx| {
                [h_flex(|_cx| [text("row"), text("meta")])
                    .gap(Space::N2)
                    .w_full()]
            },
        )
        .test_id("container-props")
        .into_element(cx)
    }

    // Compile-only: ensure the sink-based raw-container variant stays on the same child pipeline.
    #[allow(dead_code)]
    fn container_props_build_accepts_ui_builder_children<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        container_props_build(
            ContainerProps {
                layout,
                ..Default::default()
            },
            |cx, out| {
                out.push_ui(cx, text("row"));
            },
        )
        .test_id("container-props-build")
        .into_element(cx)
    }

    // Compile-only: ensure layout constructor roots can be decorated on the builder path without
    // early landing, mirroring common cookbook usage (`test_id`, role).
    #[allow(dead_code)]
    fn h_flex_root_accepts_semantics_decorators<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        h_flex(|_cx| [text("a"), text("b")])
            .test_id("root")
            .a11y_role(SemanticsRole::Group)
            .into_element(cx)
    }

    // Compile-only: ensure public-trait semantics decorators can be applied before
    // `into_element(cx)` (so callsites can avoid "decorate-only" early landing).
    #[allow(dead_code)]
    fn h_flex_accepts_decorated_children<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        h_flex(|_cx| [text("a").test_id("a"), text("b").test_id("b")])
            .gap(Space::N2)
            .into_element(cx)
    }

    #[test]
    fn container_box_accepts_ui_patches() {
        let container = ContainerBox::<(), ()>::new(())
            .ui()
            .p_1()
            .w(LengthRefinement::Fill)
            .build();

        let padding = container
            .chrome
            .padding
            .expect("expected padding refinement");
        assert!(matches!(padding.left, Some(MetricRef::Token { .. })));
        assert!(container.layout.size.is_some());
    }

    #[test]
    fn text_box_supports_layout_and_text_refinements() {
        let text = TextBox::new("hello", TextPreset::Sm)
            .ui()
            .w(LengthRefinement::Fill)
            .font_bold()
            .build();

        assert!(text.layout.size.is_some());
        assert_eq!(text.weight_override, Some(FontWeight::BOLD));
    }

    #[test]
    fn stack_box_accepts_ui_patches() {
        let stack = StackBox::<(), ()>::new(())
            .ui()
            .p_1()
            .w(LengthRefinement::Fill)
            .build();

        let padding = stack.chrome.padding.expect("expected padding refinement");
        assert!(matches!(padding.left, Some(MetricRef::Token { .. })));
        assert!(stack.layout.size.is_some());
    }

    #[test]
    fn text_box_selectable_renders_selectable_text_element() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = text("hello").selectable_on().into_element(cx);
            assert!(
                matches!(el.kind, ElementKind::SelectableText(_)),
                "expected ui::text(...).selectable_on() to render a SelectableText element"
            );
        });
    }

    #[test]
    fn text_inherits_current_color_when_available() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        let expected = fret_core::Color {
            r: 0.25,
            g: 0.5,
            b: 0.75,
            a: 1.0,
        };

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let mut els = crate::declarative::current_color::scope_children(
                cx,
                crate::ColorRef::Color(expected),
                |cx| [text("hello").into_element(cx)],
            );

            let child = els.pop().expect("expected a child element");
            assert_eq!(
                child.inherited_foreground,
                Some(expected),
                "expected current_color::scope_children(...) to stamp inherited foreground on the existing root"
            );
            let ElementKind::Text(props) = child.kind else {
                panic!("expected Text element");
            };
            assert_eq!(
                props.color, None,
                "expected text to keep color late-bound for inherited foreground paint resolution"
            );
        });
    }

    #[test]
    fn rich_text_builder_renders_styled_text_element() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        let rich = AttributedText::new(
            Arc::<str>::from("hello"),
            [TextSpan {
                len: 5,
                shaping: Default::default(),
                paint: Default::default(),
            }],
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = rich_text(rich.clone()).truncate().w_full().into_element(cx);
            let ElementKind::StyledText(props) = el.kind else {
                panic!("expected ui::rich_text(...) to render a StyledText element");
            };
            assert_eq!(props.wrap, TextWrap::None);
            assert_eq!(props.overflow, TextOverflow::Ellipsis);
        });
    }

    #[test]
    fn hover_region_builder_renders_hover_region_element() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = hover_region(|_cx, hovered| [text(if hovered { "hovered" } else { "idle" })])
                .w_full()
                .into_element(cx);
            assert!(
                matches!(el.kind, ElementKind::HoverRegion(_)),
                "expected ui::hover_region(...) to render a HoverRegion element"
            );
            assert_eq!(el.children.len(), 1, "expected a single hover-region child");
            assert!(
                matches!(el.children[0].kind, ElementKind::Text(_)),
                "expected ui::hover_region(...) child to late-land text"
            );
        });
    }

    #[test]
    fn flex_box_height_constraints_propagate_fill_height_to_inner_flex_root() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = v_flex(|_cx| [text("hello")])
                .min_h(Px(100.0))
                .max_h(Px(100.0))
                .into_element(cx);

            let inner = match &el.kind {
                ElementKind::Container(props) => {
                    assert_eq!(props.layout.size.min_height, Some(Length::Px(Px(100.0))));
                    assert_eq!(props.layout.size.max_height, Some(Length::Px(Px(100.0))));
                    el.children
                        .first()
                        .expect("flex box container should wrap an inner flex root")
                }
                other => panic!("expected outer container wrapper, got {other:?}"),
            };

            match &inner.kind {
                ElementKind::Flex(props) => {
                    assert!(
                        matches!(props.layout.size.height, Length::Fill),
                        "inner flex root should fill the constrained outer wrapper height"
                    );
                }
                other => panic!("expected inner flex root, got {other:?}"),
            }
        });
    }

    #[test]
    fn stack_box_min_h_0_keeps_inner_flex_root_auto_height() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = v_stack(|_cx| [text("hello")]).min_h_0().into_element(cx);

            let inner = match &el.kind {
                ElementKind::Container(props) => {
                    assert_eq!(props.layout.size.min_height, Some(Length::Px(Px(0.0))));
                    el.children
                        .first()
                        .expect("stack box container should wrap an inner flex root")
                }
                other => panic!("expected outer container wrapper, got {other:?}"),
            };

            match &inner.kind {
                ElementKind::Flex(props) => {
                    assert_eq!(
                        props.layout.size.height,
                        Length::Auto,
                        "min_h_0 should not force the inner stack root to fill available height"
                    );
                }
                other => panic!("expected inner flex root, got {other:?}"),
            }
        });
    }

    #[test]
    fn h_row_width_constraints_propagate_to_inner_flex_root() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = h_row(|_cx| [text("hello")])
                .w_full()
                .min_w_0()
                .max_w(Px(280.0))
                .into_element(cx);

            let inner = match &el.kind {
                ElementKind::Container(props) => {
                    assert_eq!(props.layout.size.width, Length::Fill);
                    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
                    assert_eq!(props.layout.size.max_width, Some(Length::Px(Px(280.0))));
                    el.children
                        .first()
                        .expect("flex box container should wrap an inner flex root")
                }
                other => panic!("expected outer container wrapper, got {other:?}"),
            };

            match &inner.kind {
                ElementKind::Flex(props) => {
                    assert_eq!(
                        props.layout.size.width,
                        Length::Fill,
                        "expected explicit width constraints on h_row(...) to land on the inner row root"
                    );
                    assert_eq!(
                        props.layout.size.min_width,
                        Some(Length::Px(Px(0.0))),
                        "expected min_w_0 on h_row(...) to land on the inner row root"
                    );
                    assert_eq!(
                        props.layout.size.max_width,
                        Some(Length::Px(Px(280.0))),
                        "expected max_w on h_row(...) to land on the inner row root"
                    );
                }
                other => panic!("expected inner flex root, got {other:?}"),
            }
        });
    }

    #[test]
    fn h_flex_explicit_width_overrides_default_fill_on_inner_flex_root() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = h_flex(|_cx| [text("hello")])
                .layout(LayoutRefinement::default().w_auto().min_w_0())
                .into_element(cx);

            let inner = match &el.kind {
                ElementKind::Container(props) => {
                    assert_eq!(props.layout.size.width, Length::Auto);
                    assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
                    el.children
                        .first()
                        .expect("flex box container should wrap an inner flex root")
                }
                other => panic!("expected outer container wrapper, got {other:?}"),
            };

            match &inner.kind {
                ElementKind::Flex(props) => {
                    assert_eq!(
                        props.layout.size.width,
                        Length::Auto,
                        "expected explicit w_auto() to override the default fill-width inner flex root"
                    );
                    assert_eq!(
                        props.layout.size.min_width,
                        Some(Length::Px(Px(0.0))),
                        "expected min_w_0 to land on the inner flex root even when explicit width overrides the default fill contract"
                    );
                }
                other => panic!("expected inner flex root, got {other:?}"),
            }
        });
    }

    #[test]
    fn scroll_area_explicit_height_propagates_fill_height_to_inner_scroll_root() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = scroll_area(|_cx| [text("hello")])
                .h_px(Px(120.0))
                .into_element(cx);

            let inner = match &el.kind {
                ElementKind::Container(props) => {
                    assert_eq!(props.layout.size.height, Length::Px(Px(120.0)));
                    el.children
                        .first()
                        .expect("scroll area container should wrap an inner scroll root")
                }
                other => panic!("expected outer container wrapper, got {other:?}"),
            };

            match &inner.kind {
                ElementKind::Scroll(props) => {
                    assert_eq!(
                        props.layout.size.height,
                        Length::Fill,
                        "explicit scroll-area height should keep the inner scroll root fill-sized on its scroll axis"
                    );
                }
                other => panic!("expected inner scroll root, got {other:?}"),
            }
        });
    }

    #[test]
    fn scroll_area_build_without_height_constraints_keeps_inner_scroll_root_auto_height() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = scroll_area_build(|cx, out| out.push(text("hello").into_element(cx)))
                .into_element(cx);

            let inner = match &el.kind {
                ElementKind::Container(props) => {
                    assert_eq!(props.layout.size.height, Length::Auto);
                    el.children
                        .first()
                        .expect("scroll area container should wrap an inner scroll root")
                }
                other => panic!("expected outer container wrapper, got {other:?}"),
            };

            match &inner.kind {
                ElementKind::Scroll(props) => {
                    assert_eq!(
                        props.layout.size.height,
                        Length::Auto,
                        "auto-height scroll-area wrappers should keep the inner scroll root auto-sized on the scroll axis"
                    );
                }
                other => panic!("expected inner scroll root, got {other:?}"),
            }
        });
    }

    #[test]
    fn scroll_area_viewport_test_id_lands_on_inner_scroll_root() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = scroll_area(|_cx| [text("hello")])
                .viewport_test_id("scroll.viewport")
                .into_element(cx);

            let inner = match &el.kind {
                ElementKind::Container(_) => el
                    .children
                    .first()
                    .expect("scroll area container should wrap an inner scroll root"),
                other => panic!("expected outer container wrapper, got {other:?}"),
            };

            match &inner.kind {
                ElementKind::Scroll(_) => {
                    assert_eq!(
                        inner
                            .semantics_decoration
                            .as_ref()
                            .and_then(|decoration| decoration.test_id.as_deref()),
                        Some("scroll.viewport")
                    );
                }
                other => panic!("expected inner scroll root, got {other:?}"),
            }
        });
    }
}
