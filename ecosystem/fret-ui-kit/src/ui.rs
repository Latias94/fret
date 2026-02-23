use std::marker::PhantomData;
use std::sync::Arc;

pub use crate::children;

use smallvec::SmallVec;

use fret_core::{
    AttributedText, Axis, Edges, FontId, FontWeight, Px, TextAlign, TextOverflow, TextSpan,
    TextWrap,
};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, InsetStyle, LayoutStyle, Length, Overflow,
    PositionStyle, ScrollAxis, ScrollProps, ScrollbarAxis, ScrollbarProps, ScrollbarStyle,
    SelectableTextProps, SizeStyle, StackProps, TextProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::style as decl_style;
use crate::declarative::text as decl_text;
use crate::{
    ChromeRefinement, Items, Justify, LayoutRefinement, MetricRef, Space, UiBuilder, UiIntoElement,
    UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout,
};

fn collect_ui_children<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    iter: I,
) -> SmallVec<[AnyElement; 8]>
where
    I: IntoIterator,
    I::Item: UiIntoElement,
{
    let mut out: SmallVec<[AnyElement; 8]> = SmallVec::new();
    for child in iter {
        out.push(crate::UiIntoElement::into_element(child, cx));
    }
    out
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
    pub(crate) gap: MetricRef,
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
    pub(crate) gap: MetricRef,
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
            gap: MetricRef::space(Space::N0),
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
            gap: MetricRef::space(Space::N0),
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
    I::Item: UiIntoElement,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app);

        let container = decl_style::container_props(theme, self.chrome, self.layout);

        let gap = self.gap.resolve(theme);
        let mut flex_props = FlexProps {
            direction: self.direction,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: self.justify.to_main_align(),
            align: self.items.to_cross_align(),
            wrap: self.wrap,
            ..Default::default()
        };
        flex_props.layout.size.width = Length::Fill;

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

        let container = decl_style::container_props(theme, self.chrome, self.layout);

        let gap = self.gap.resolve(theme);
        let mut flex_props = FlexProps {
            direction: self.direction,
            gap,
            padding: Edges::all(Px(0.0)),
            justify: self.justify.to_main_align(),
            align: self.items.to_cross_align(),
            wrap: self.wrap,
            ..Default::default()
        };
        flex_props.layout.size.width = Length::Fill;

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

/// Returns a patchable horizontal flex layout builder.
///
/// Usage:
/// - `ui::h_flex(cx, |cx| vec![...]).gap(Space::N2).px_2().into_element(cx)`
pub fn h_flex<H: UiHost, F, I>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: UiIntoElement,
{
    UiBuilder::new(FlexBox::new(Axis::Horizontal, children))
}

/// Variant of [`h_flex`] that avoids iterator borrow pitfalls by collecting into a sink.
///
/// Use this when the natural authoring form is an iterator that captures `&mut cx` (e.g.
/// `items.iter().map(|it| cx.keyed(...))`), which cannot be returned directly.
pub fn h_flex_build<H: UiHost, B>(
    _cx: &mut ElementContext<'_, H>,
    build: B,
) -> UiBuilder<FlexBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    UiBuilder::new(FlexBoxBuild::new(Axis::Horizontal, build))
}

/// Returns a patchable vertical flex layout builder.
pub fn v_flex<H: UiHost, F, I>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<FlexBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: UiIntoElement,
{
    UiBuilder::new(FlexBox::new(Axis::Vertical, children))
}

/// Variant of [`v_flex`] that avoids iterator borrow pitfalls by collecting into a sink.
///
/// Use this when the natural authoring form is an iterator that captures `&mut cx` (e.g.
/// `items.iter().map(|it| cx.keyed(...))`), which cannot be returned directly.
pub fn v_flex_build<H: UiHost, B>(
    _cx: &mut ElementContext<'_, H>,
    build: B,
) -> UiBuilder<FlexBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    UiBuilder::new(FlexBoxBuild::new(Axis::Vertical, build))
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
    I::Item: UiIntoElement,
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

/// Returns a patchable container builder.
///
/// Usage:
/// - `ui::container(cx, |cx| vec![...]).px_2().into_element(cx)`
pub fn container<H: UiHost, F, I>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<ContainerBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: UiIntoElement,
{
    UiBuilder::new(ContainerBox::new(children))
}

/// Variant of [`container`] that avoids iterator borrow pitfalls by collecting into a sink.
pub fn container_build<H: UiHost, B>(
    _cx: &mut ElementContext<'_, H>,
    build: B,
) -> UiBuilder<ContainerBoxBuild<H, B>>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    UiBuilder::new(ContainerBoxBuild::new(build))
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
    I::Item: UiIntoElement,
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (container, scrollbar_w, thumb, thumb_hover, corner_bg) = {
            let theme = Theme::global(&*cx.app);
            let container = decl_style::container_props(theme, self.chrome, self.layout);

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
        let children = self.children.expect("expected scroll children closure");

        cx.container(container, move |cx| {
            let handle = cx.with_state(ScrollHandle::default, |h| {
                if let Some(handle) = provided_handle.clone() {
                    *h = handle;
                }
                h.clone()
            });

            let mut scroll_layout = LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = Overflow::Clip;

            let scroll = cx.scroll(
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

            let scroll_id = scroll.id;
            let mut out = vec![scroll];

            if show_scrollbar_y {
                let scrollbar_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: Some(Px(0.0)),
                        right: Some(Px(0.0)),
                        bottom: Some(if show_scrollbar_x {
                            scrollbar_w
                        } else {
                            Px(0.0)
                        }),
                        left: None,
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
                        top: None,
                        right: Some(if show_scrollbar_y {
                            scrollbar_w
                        } else {
                            Px(0.0)
                        }),
                        bottom: Some(Px(0.0)),
                        left: Some(Px(0.0)),
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
                        top: None,
                        right: Some(Px(0.0)),
                        bottom: Some(Px(0.0)),
                        left: None,
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
        let (container, scrollbar_w, thumb, thumb_hover, corner_bg) = {
            let theme = Theme::global(&*cx.app);
            let container = decl_style::container_props(theme, self.chrome, self.layout);
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
        let build = self.build.expect("expected scroll area build closure");

        cx.container(container, move |cx| {
            let handle = cx.with_state(ScrollHandle::default, |h| {
                if let Some(handle) = provided_handle.clone() {
                    *h = handle;
                }
                h.clone()
            });

            let mut scroll_layout = LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = Overflow::Clip;

            let scroll = cx.scroll(
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

            let scroll_id = scroll.id;
            let mut out = vec![scroll];

            if show_scrollbar_y {
                let scrollbar_layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        top: Some(Px(0.0)),
                        right: Some(Px(0.0)),
                        bottom: Some(if show_scrollbar_x {
                            scrollbar_w
                        } else {
                            Px(0.0)
                        }),
                        left: None,
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
                        top: None,
                        right: Some(if show_scrollbar_y {
                            scrollbar_w
                        } else {
                            Px(0.0)
                        }),
                        bottom: Some(Px(0.0)),
                        left: Some(Px(0.0)),
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
                        top: None,
                        right: Some(Px(0.0)),
                        bottom: Some(Px(0.0)),
                        left: None,
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

/// Returns a patchable scroll area builder.
///
/// Defaults:
/// - axis: vertical
/// - scrollbar: Y on, X off
pub fn scroll_area<H: UiHost, F, I>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<ScrollAreaBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: UiIntoElement,
{
    UiBuilder::new(ScrollAreaBox::new(children))
}

/// Variant of [`scroll_area`] that avoids iterator borrow pitfalls by collecting into a sink.
pub fn scroll_area_build<H: UiHost, B>(
    _cx: &mut ElementContext<'_, H>,
    build: B,
) -> UiBuilder<ScrollAreaBoxBuild<H, B>>
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
    I::Item: UiIntoElement,
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

/// Returns a patchable stack layout builder.
///
/// Usage:
/// - `ui::stack(cx, |cx| vec![...]).inset(Space::N2).into_element(cx)`
pub fn stack<H: UiHost, F, I>(
    _cx: &mut ElementContext<'_, H>,
    children: F,
) -> UiBuilder<StackBox<H, F>>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator,
    I::Item: UiIntoElement,
{
    UiBuilder::new(StackBox::new(children))
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

impl UiIntoElement for TextBox {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let TextBox {
            layout: layout_refinement,
            text,
            preset,
            font_override,
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

            let resolved_color = color_override
                .as_ref()
                .map(|c| c.resolve(theme))
                .or_else(|| {
                    (preset == TextPreset::Label).then(|| {
                        theme
                            .color_by_key("foreground")
                            .unwrap_or_else(|| theme.color_token("foreground"))
                    })
                });

            (style, layout, label_line_height, resolved_color)
        };

        if let Some(font) = font_override {
            style.font = font;
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
/// - `ui::text(cx, "Hello").text_sm().font_medium().into_element(cx)`
pub fn text<H: UiHost>(
    _cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> UiBuilder<TextBox> {
    UiBuilder::new(TextBox::new(text, TextPreset::Sm))
}

/// Returns a patchable block text builder (full-width; shadcn-aligned defaults).
///
/// Use this for paragraph-like text that should wrap against the available inner width of its
/// containing block.
pub fn text_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    content: impl Into<Arc<str>>,
) -> UiBuilder<TextBox> {
    text(cx, content).w_full()
}

/// Returns a patchable selectable text builder (drag-to-select + `edit.copy`).
///
/// Prefer this for read-only values (paths/IDs/snippets) and documentation-like content.
/// Avoid using it inside pressable/clickable rows: it intentionally captures left-drag selection
/// gestures and stops propagation (use a dedicated copy button instead).
pub fn selectable_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> UiBuilder<TextBox> {
    crate::ui::text(cx, text).selectable_on()
}

/// Returns a patchable selectable block text builder (full-width; drag-to-select + `edit.copy`).
pub fn selectable_text_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    content: impl Into<Arc<str>>,
) -> UiBuilder<TextBox> {
    selectable_text(cx, content).w_full()
}

/// Returns a patchable label builder (single-line, medium weight).
pub fn label<H: UiHost>(
    _cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> UiBuilder<TextBox> {
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

impl UiPatchTarget for RawTextBox {
    fn apply_ui_patch(mut self, patch: UiPatch) -> Self {
        self.layout = self.layout.merge(patch.layout);
        self
    }
}

impl UiSupportsLayout for RawTextBox {}

impl UiIntoElement for RawTextBox {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let RawTextBox {
            layout: layout_refinement,
            text,
            color_override,
            wrap,
            overflow,
            align,
            ink_overflow_override,
        } = self;

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
pub fn raw_text<H: UiHost>(
    _cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> UiBuilder<RawTextBox> {
    UiBuilder::new(RawTextBox::new(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiExt;
    use crate::{LengthRefinement, MetricRef};
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::ElementKind;

    // Compile-only: ensure `ui::*` layout constructors accept `UiIntoElement` children
    // (e.g. `UiBuilder<TextBox>`) without requiring call-site `.into_element(cx)`.
    #[allow(dead_code)]
    fn h_flex_accepts_ui_builder_children<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
        h_flex(cx, |cx| [text(cx, "a"), text(cx, "b")])
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
            let el = text(cx, "hello").selectable_on().into_element(cx);
            assert!(
                matches!(el.kind, ElementKind::SelectableText(_)),
                "expected ui::text(...).selectable_on() to render a SelectableText element"
            );
        });
    }
}
