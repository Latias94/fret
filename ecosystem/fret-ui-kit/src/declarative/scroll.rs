use fret_core::Px;
use fret_ui::element::{
    AnyElement, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, ScrollProps,
    ScrollbarProps, ScrollbarStyle, SizeStyle, StackProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};

use crate::LayoutRefinement;
use crate::declarative::style;

/// Component-layer scroll helper (typed, declarative).
///
/// Fret treats scrolling as an explicit element (not a boolean overflow flag). This wrapper exists
/// to match gpui/tailwind ergonomics while keeping the runtime contract explicit.
pub fn overflow_scroll<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar: bool,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let (layout, scrollbar_w, thumb, thumb_hover) = {
        let theme = Theme::global(&*cx.app);
        let layout = style::layout_style(theme, layout);

        let scrollbar_w = theme
            .metric_by_key("metric.scrollbar.width")
            .unwrap_or(theme.metrics.scrollbar_width);

        let thumb = theme
            .color_by_key("scrollbar.thumb.background")
            .unwrap_or(theme.colors.scrollbar_thumb);
        let thumb_hover = theme
            .color_by_key("scrollbar.thumb.hover.background")
            .unwrap_or(
                theme
                    .color_by_key("scrollbar.thumb.background")
                    .unwrap_or(theme.colors.scrollbar_thumb_hover),
            );

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
                scroll_target: Some(scroll_id),
                scroll_handle: handle,
                style: ScrollbarStyle {
                    thumb,
                    thumb_hover,
                    thumb_idle_alpha: 0.65,
                },
            }));
        }

        children
    })
}

pub fn overflow_scroll_with_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    show_scrollbar: bool,
    handle: ScrollHandle,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let (layout, scrollbar_w, thumb, thumb_hover) = {
        let theme = Theme::global(&*cx.app);
        let layout = style::layout_style(theme, layout);

        let scrollbar_w = theme
            .metric_by_key("metric.scrollbar.width")
            .unwrap_or(theme.metrics.scrollbar_width);

        let thumb = theme
            .color_by_key("scrollbar.thumb.background")
            .unwrap_or(theme.colors.scrollbar_thumb);
        let thumb_hover = theme
            .color_by_key("scrollbar.thumb.hover.background")
            .unwrap_or(
                theme
                    .color_by_key("scrollbar.thumb.background")
                    .unwrap_or(theme.colors.scrollbar_thumb_hover),
            );

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
                scroll_target: Some(scroll_id),
                scroll_handle: handle,
                style: ScrollbarStyle {
                    thumb,
                    thumb_hover,
                    thumb_idle_alpha: 0.65,
                },
            }));
        }

        children
    })
}

pub fn overflow_scrollbar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    overflow_scroll(cx, layout, true, f)
}
