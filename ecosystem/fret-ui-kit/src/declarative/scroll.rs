use fret_core::{Color, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
    ScrollAxis, ScrollProps, ScrollbarAxis, ScrollbarProps, ScrollbarStyle, SizeStyle, StackProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};

use crate::LayoutRefinement;
use crate::declarative::stack;
use crate::declarative::style;

/// Component-layer scroll helper (typed, declarative).
///
/// Fret treats scrolling as an explicit element (not a boolean overflow flag). This wrapper exists
/// to match gpui/tailwind ergonomics while keeping the runtime contract explicit.
pub fn overflow_scroll<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar: bool,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let (layout, scrollbar_w, thumb, thumb_hover) = {
        let theme = Theme::global(&*cx.app);
        let layout = style::layout_style(theme, layout);

        let scrollbar_w = theme.metric_required("metric.scrollbar.width");

        let thumb = theme.color_required("scrollbar.thumb.background");
        let thumb_hover = theme.color_required("scrollbar.thumb.hover.background");

        (layout, scrollbar_w, thumb, thumb_hover)
    };

    cx.stack_props(StackProps { layout }, move |cx| {
        let handle = cx.with_state(ScrollHandle::default, |h| h.clone());
        let mut scroll_layout = LayoutStyle::default();
        scroll_layout.size.width = Length::Fill;
        scroll_layout.size.height = Length::Fill;
        scroll_layout.overflow = Overflow::Clip;

        let scroll = cx.scroll(
            ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(handle.clone()),
                ..Default::default()
            },
            f,
        );

        let scroll_id = scroll.id;
        let mut children = vec![scroll];
        if show_scrollbar {
            let scrollbar_layout = LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    top: Some(Px(0.0)),
                    right: Some(Px(0.0)),
                    bottom: Some(Px(0.0)),
                    left: None,
                },
                size: SizeStyle {
                    width: Length::Px(scrollbar_w),
                    ..Default::default()
                },
                ..Default::default()
            };

            children.push(cx.scrollbar(ScrollbarProps {
                layout: scrollbar_layout,
                axis: ScrollbarAxis::Vertical,
                scroll_target: Some(scroll_id),
                scroll_handle: handle,
                style: ScrollbarStyle {
                    thumb,
                    thumb_hover,
                    ..Default::default()
                },
            }));
        }

        children
    })
}

pub fn overflow_scroll_with_handle<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar: bool,
    handle: ScrollHandle,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let (layout, scrollbar_w, thumb, thumb_hover) = {
        let theme = Theme::global(&*cx.app);
        let layout = style::layout_style(theme, layout);

        let scrollbar_w = theme.metric_required("metric.scrollbar.width");

        let thumb = theme.color_required("scrollbar.thumb.background");
        let thumb_hover = theme.color_required("scrollbar.thumb.hover.background");

        (layout, scrollbar_w, thumb, thumb_hover)
    };

    cx.stack_props(StackProps { layout }, move |cx| {
        let mut scroll_layout = LayoutStyle::default();
        scroll_layout.size.width = Length::Fill;
        scroll_layout.size.height = Length::Fill;
        scroll_layout.overflow = Overflow::Clip;

        let scroll = cx.scroll(
            ScrollProps {
                layout: scroll_layout,
                scroll_handle: Some(handle.clone()),
                ..Default::default()
            },
            f,
        );

        let scroll_id = scroll.id;
        let mut children = vec![scroll];
        if show_scrollbar {
            let scrollbar_layout = LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    top: Some(Px(0.0)),
                    right: Some(Px(0.0)),
                    bottom: Some(Px(0.0)),
                    left: None,
                },
                size: SizeStyle {
                    width: Length::Px(scrollbar_w),
                    ..Default::default()
                },
                ..Default::default()
            };

            children.push(cx.scrollbar(ScrollbarProps {
                layout: scrollbar_layout,
                axis: ScrollbarAxis::Vertical,
                scroll_target: Some(scroll_id),
                scroll_handle: handle,
                style: ScrollbarStyle {
                    thumb,
                    thumb_hover,
                    ..Default::default()
                },
            }));
        }

        children
    })
}

pub fn overflow_scroll_with_handle_xy<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar_x: bool,
    show_scrollbar_y: bool,
    handle: ScrollHandle,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let (layout, scrollbar_w, thumb, thumb_hover, corner_bg) = {
        let theme = Theme::global(&*cx.app);
        let layout = style::layout_style(theme, layout);

        let scrollbar_w = theme.metric_required("metric.scrollbar.width");

        let thumb = theme.color_required("scrollbar.thumb.background");
        let thumb_hover = theme.color_required("scrollbar.thumb.hover.background");

        let corner_bg = theme
            .color_by_key("scrollbar.corner.background")
            .or_else(|| theme.color_by_key("scrollbar.track.background"))
            .unwrap_or(Color::TRANSPARENT);

        (layout, scrollbar_w, thumb, thumb_hover, corner_bg)
    };

    cx.stack_props(StackProps { layout }, move |cx| {
        let mut scroll_layout = LayoutStyle::default();
        scroll_layout.size.width = Length::Fill;
        scroll_layout.size.height = Length::Fill;
        scroll_layout.overflow = Overflow::Clip;

        let scroll = cx.scroll(
            ScrollProps {
                layout: scroll_layout,
                axis: ScrollAxis::Both,
                scroll_handle: Some(handle.clone()),
                ..Default::default()
            },
            f,
        );

        let scroll_id = scroll.id;
        let mut children = vec![scroll];

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

            children.push(cx.scrollbar(ScrollbarProps {
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

            children.push(cx.scrollbar(ScrollbarProps {
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
                    right: Some(Px(0.0)),
                    bottom: Some(Px(0.0)),
                    ..Default::default()
                },
                size: SizeStyle {
                    width: Length::Px(scrollbar_w),
                    height: Length::Px(scrollbar_w),
                    ..Default::default()
                },
                ..Default::default()
            };

            children.push(cx.container(
                ContainerProps {
                    layout: corner_layout,
                    background: Some(corner_bg),
                    ..Default::default()
                },
                |_cx| vec![],
            ));
        }

        children
    })
}

pub fn overflow_scrollbar<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    overflow_scroll(cx, layout, true, f)
}

/// Like `overflow_scroll`, but enforces a single content root.
///
/// Note: `Scroll` does not lay out multiple children; if you pass a `Vec` of siblings they will
/// overlap. Prefer this helper (or `*_vstack`) to make the intended structure explicit.
pub fn overflow_scroll_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar: bool,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    overflow_scroll(cx, layout, show_scrollbar, |cx| vec![content(cx)])
}

/// Vertical scrolling with a `vstack` content root.
pub fn overflow_scroll_vstack<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar: bool,
    vstack: stack::VStackProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    overflow_scroll_content(cx, layout, show_scrollbar, |cx| {
        stack::vstack_iter(cx, vstack, f)
    })
}

/// Horizontal scrolling with a single content root.
pub fn overflow_scroll_x_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar_x: bool,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    let (layout, scrollbar_w, thumb, thumb_hover) = {
        let theme = Theme::global(&*cx.app);
        let layout = style::layout_style(theme, layout);

        let scrollbar_w = theme.metric_required("metric.scrollbar.width");

        let thumb = theme.color_required("scrollbar.thumb.background");
        let thumb_hover = theme.color_required("scrollbar.thumb.hover.background");

        (layout, scrollbar_w, thumb, thumb_hover)
    };

    cx.stack_props(StackProps { layout }, move |cx| {
        let handle = cx.with_state(ScrollHandle::default, |h| h.clone());
        let mut scroll_layout = LayoutStyle::default();
        scroll_layout.size.width = Length::Fill;
        // For X-only scrolling, the common expectation is "height = content height" (e.g. code
        // blocks) while width fills the viewport.
        scroll_layout.size.height = Length::Auto;
        scroll_layout.overflow = Overflow::Clip;

        let scroll = cx.scroll(
            ScrollProps {
                layout: scroll_layout,
                axis: ScrollAxis::X,
                scroll_handle: Some(handle.clone()),
                ..Default::default()
            },
            |cx| vec![content(cx)],
        );

        let scroll_id = scroll.id;
        let mut children = vec![scroll];

        if show_scrollbar_x {
            let scrollbar_layout = LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    top: None,
                    right: Some(Px(0.0)),
                    bottom: Some(Px(0.0)),
                    left: Some(Px(0.0)),
                },
                size: SizeStyle {
                    height: Length::Px(scrollbar_w),
                    ..Default::default()
                },
                ..Default::default()
            };

            children.push(cx.scrollbar(ScrollbarProps {
                layout: scrollbar_layout,
                axis: ScrollbarAxis::Horizontal,
                scroll_target: Some(scroll_id),
                scroll_handle: handle,
                style: ScrollbarStyle {
                    thumb,
                    thumb_hover,
                    ..Default::default()
                },
            }));
        }

        children
    })
}

/// Horizontal scrolling with a `vstack` content root.
pub fn overflow_scroll_x_vstack<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar_x: bool,
    vstack: stack::VStackProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    overflow_scroll_x_content(cx, layout, show_scrollbar_x, |cx| {
        stack::vstack_iter(cx, vstack, f)
    })
}

/// Like `overflow_scroll_with_handle_xy`, but enforces a single content root.
pub fn overflow_scroll_with_handle_xy_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar_x: bool,
    show_scrollbar_y: bool,
    handle: ScrollHandle,
    content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) -> AnyElement {
    overflow_scroll_with_handle_xy(
        cx,
        layout,
        show_scrollbar_x,
        show_scrollbar_y,
        handle,
        |cx| vec![content(cx)],
    )
}
