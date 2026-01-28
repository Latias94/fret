//! Windowed row surface helpers.
//!
//! This module provides an ecosystem-level building block for “prepaint-windowed virtual
//! surfaces” (ADR 0190) in the subset of cases where:
//!
//! - the surface can be modeled as a single scrollable region, and
//! - per-row UI does not need to be represented as a fully composable declarative subtree.
//!
//! The core idea is to keep the element tree structurally stable (a `Scroll` + leaf `Canvas`)
//! while drawing only the visible rows in the canvas paint handler. This avoids cache-root
//! rerenders for scroll-only deltas and provides a reusable pattern for:
//!
//! - huge inspectors/log panes,
//! - simple search/command result lists,
//! - table “body” surfaces that handle hit-testing internally.
//!
//! If you need fully composable rows with per-item semantics/focus, prefer `VirtualList`-based
//! helpers (e.g. `list_virtualized`) and keep the “window jump” cost low via overscan.

use fret_core::{Point, Px, Rect, Size};
use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx, UiPointerActionHost};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{
    AnyElement, CanvasProps, Length, PointerRegionProps, ScrollAxis, ScrollProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::virtual_list::VirtualListMetrics;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone, Copy)]
pub struct WindowedRowsPaintFrame {
    pub viewport_height: Px,
    pub offset_y: Px,
    pub visible_start: usize,
    pub visible_end: usize,
}

pub type OnWindowedRowsPaintFrame =
    std::sync::Arc<dyn for<'p> Fn(&mut CanvasPainter<'p>, WindowedRowsPaintFrame) + 'static>;

/// Props for [`windowed_rows_surface`].
///
/// Note: this helper is intentionally fixed-row-height for v1. Variable-height virtualization
/// needs a measurement pipeline and is tracked separately in the workstream TODOs.
#[derive(Clone)]
pub struct WindowedRowsSurfaceProps {
    pub scroll: ScrollProps,
    pub canvas: CanvasProps,
    pub len: usize,
    pub row_height: Px,
    pub overscan: usize,
    pub gap: Px,
    pub scroll_margin: Px,
    pub scroll_handle: ScrollHandle,
    pub on_paint_frame: Option<OnWindowedRowsPaintFrame>,
}

impl Default for WindowedRowsSurfaceProps {
    fn default() -> Self {
        let mut scroll = ScrollProps::default();
        scroll.axis = ScrollAxis::Y;
        scroll.layout.size.width = Length::Fill;
        scroll.layout.size.height = Length::Fill;

        let mut canvas = CanvasProps::default();
        canvas.layout.size.width = Length::Fill;

        Self {
            scroll,
            canvas,
            len: 0,
            row_height: Px(0.0),
            overscan: 0,
            gap: Px(0.0),
            scroll_margin: Px(0.0),
            scroll_handle: ScrollHandle::default(),
            on_paint_frame: None,
        }
    }
}

/// Build a fixed-row-height scroll surface that paints only the visible row window.
///
/// `paint_row` is called for each visible row (including overscan) with the row bounds in
/// **content space**. The scroll container applies the scroll transform to descendants, so the
/// painted rows appear in viewport space automatically.
#[track_caller]
pub fn windowed_rows_surface<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: WindowedRowsSurfaceProps,
    paint_row: impl for<'p> Fn(&mut CanvasPainter<'p>, usize, Rect) + 'static,
) -> AnyElement {
    let WindowedRowsSurfaceProps {
        mut scroll,
        mut canvas,
        len,
        row_height,
        overscan,
        gap,
        scroll_margin,
        scroll_handle,
        on_paint_frame,
    } = props;

    let mut metrics = VirtualListMetrics::default();
    metrics.ensure_with_mode(
        fret_ui::element::VirtualListMeasureMode::Fixed,
        len,
        row_height,
        gap,
        scroll_margin,
    );
    let content_h = metrics.total_height();

    scroll.axis = ScrollAxis::Y;
    scroll.scroll_handle = Some(scroll_handle.clone());

    canvas.layout.size.width = Length::Fill;
    canvas.layout.size.height = Length::Px(content_h);

    cx.scroll(scroll, move |cx| {
        let scroll_handle = scroll_handle.clone();
        let metrics = metrics.clone();
        let paint_row = std::sync::Arc::new(paint_row);
        let on_paint_frame = on_paint_frame.clone();

        vec![cx.canvas(canvas, move |painter| {
            let viewport_h = Px(scroll_handle.viewport_size().height.0.max(0.0));
            let offset_y = Px(scroll_handle.offset().y.0.max(0.0));
            let offset_y = metrics.clamp_offset(offset_y, viewport_h);
            let Some(visible) = metrics.visible_range(offset_y, viewport_h, overscan) else {
                return;
            };

            let width = Px(painter.bounds().size.width.0.max(0.0));
            let count = visible.count;
            if count == 0 {
                return;
            }

            let start = visible.start_index.saturating_sub(visible.overscan);
            let end = (visible.end_index + visible.overscan).min(count.saturating_sub(1));

            if let Some(on_paint_frame) = &on_paint_frame {
                on_paint_frame(
                    painter,
                    WindowedRowsPaintFrame {
                        viewport_height: viewport_h,
                        offset_y,
                        visible_start: start,
                        visible_end: end,
                    },
                );
            }

            for index in start..=end {
                let y = metrics.offset_for_index(index);
                let h = metrics.height_at(index);
                let rect = Rect::new(Point::new(Px(0.0), y), Size::new(width, h));
                paint_row(painter, index, rect);
            }
        })]
    })
}

pub type OnWindowedRowsPointerDown = std::sync::Arc<
    dyn Fn(&mut dyn UiPointerActionHost, ActionCx, usize, PointerDownCx) -> bool + 'static,
>;

pub type OnWindowedRowsPointerMove = std::sync::Arc<
    dyn Fn(&mut dyn UiPointerActionHost, ActionCx, Option<usize>, PointerMoveCx) -> bool + 'static,
>;

pub type OnWindowedRowsPointerUp = std::sync::Arc<
    dyn Fn(
            &mut dyn UiPointerActionHost,
            ActionCx,
            Option<usize>,
            fret_ui::action::PointerUpCx,
        ) -> bool
        + 'static,
>;

pub type OnWindowedRowsPointerCancel = std::sync::Arc<
    dyn Fn(&mut dyn UiPointerActionHost, ActionCx, fret_ui::action::PointerCancelCx) -> bool
        + 'static,
>;

#[derive(Default, Clone)]
pub struct WindowedRowsSurfacePointerHandlers {
    pub on_pointer_down: Option<OnWindowedRowsPointerDown>,
    pub on_pointer_move: Option<OnWindowedRowsPointerMove>,
    pub on_pointer_up: Option<OnWindowedRowsPointerUp>,
    pub on_pointer_cancel: Option<OnWindowedRowsPointerCancel>,
}

fn row_index_for_pointer(
    metrics: &VirtualListMetrics,
    scroll_handle: &ScrollHandle,
    bounds: Rect,
    position: Point,
    len: usize,
) -> Option<usize> {
    if len == 0 {
        return None;
    }

    let viewport_h = Px(scroll_handle.viewport_size().height.0.max(0.0));
    if viewport_h.0 <= 0.0 {
        return None;
    }

    let offset_y = Px(scroll_handle.offset().y.0.max(0.0));
    let offset_y = metrics.clamp_offset(offset_y, viewport_h);

    let local_y = Px(position.y.0 - bounds.origin.y.0);
    if local_y.0 < 0.0 {
        return None;
    }

    let content_y = Px(offset_y.0 + local_y.0);
    let idx = metrics.index_for_offset(content_y);
    (idx < len).then_some(idx)
}

/// Like [`windowed_rows_surface`], but wraps the canvas in a `PointerRegion` that performs row
/// hit-testing and forwards pointer events to the provided handlers.
#[track_caller]
pub fn windowed_rows_surface_with_pointer_region<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: WindowedRowsSurfaceProps,
    pointer: PointerRegionProps,
    handlers: WindowedRowsSurfacePointerHandlers,
    content_semantics: Option<fret_ui::element::SemanticsProps>,
    paint_row: impl for<'p> Fn(&mut CanvasPainter<'p>, usize, Rect) + 'static,
) -> AnyElement {
    let WindowedRowsSurfacePointerHandlers {
        on_pointer_down,
        on_pointer_move,
        on_pointer_up,
        on_pointer_cancel,
    } = handlers;

    let WindowedRowsSurfaceProps {
        mut scroll,
        mut canvas,
        len,
        row_height,
        overscan,
        gap,
        scroll_margin,
        scroll_handle,
        on_paint_frame,
    } = props;

    let mut metrics = VirtualListMetrics::default();
    metrics.ensure_with_mode(
        fret_ui::element::VirtualListMeasureMode::Fixed,
        len,
        row_height,
        gap,
        scroll_margin,
    );
    let content_h = metrics.total_height();

    scroll.axis = ScrollAxis::Y;
    scroll.scroll_handle = Some(scroll_handle.clone());

    canvas.layout.size.width = Length::Fill;
    canvas.layout.size.height = Length::Px(content_h);

    cx.scroll(scroll, move |cx| {
        let scroll_handle = scroll_handle.clone();
        let metrics = metrics.clone();
        let paint_row = std::sync::Arc::new(paint_row);
        let on_pointer_down = on_pointer_down.clone();
        let on_pointer_move = on_pointer_move.clone();
        let on_pointer_up = on_pointer_up.clone();
        let on_pointer_cancel = on_pointer_cancel.clone();
        let content_semantics = content_semantics.clone();
        let on_paint_frame = on_paint_frame.clone();

        vec![cx.pointer_region(pointer, move |cx| {
            if let Some(on_pointer_down) = on_pointer_down.clone() {
                let scroll_handle = scroll_handle.clone();
                let metrics = metrics.clone();
                cx.pointer_region_on_pointer_down(std::sync::Arc::new(
                    move |host, action_cx, down| {
                        let bounds = host.bounds();
                        let idx = row_index_for_pointer(
                            &metrics,
                            &scroll_handle,
                            bounds,
                            down.position,
                            len,
                        );
                        let Some(idx) = idx else {
                            return false;
                        };
                        on_pointer_down(host, action_cx, idx, down)
                    },
                ));
            }

            if let Some(on_pointer_move) = on_pointer_move.clone() {
                let scroll_handle = scroll_handle.clone();
                let metrics = metrics.clone();
                cx.pointer_region_on_pointer_move(std::sync::Arc::new(
                    move |host, action_cx, mv| {
                        let bounds = host.bounds();
                        let idx = row_index_for_pointer(
                            &metrics,
                            &scroll_handle,
                            bounds,
                            mv.position,
                            len,
                        );
                        on_pointer_move(host, action_cx, idx, mv)
                    },
                ));
            }

            if let Some(on_pointer_up) = on_pointer_up.clone() {
                let scroll_handle = scroll_handle.clone();
                let metrics = metrics.clone();
                cx.pointer_region_on_pointer_up(std::sync::Arc::new(move |host, action_cx, up| {
                    let bounds = host.bounds();
                    let idx =
                        row_index_for_pointer(&metrics, &scroll_handle, bounds, up.position, len);
                    on_pointer_up(host, action_cx, idx, up)
                }));
            }

            if let Some(on_pointer_cancel) = on_pointer_cancel.clone() {
                cx.pointer_region_on_pointer_cancel(on_pointer_cancel);
            }

            let canvas_children = vec![cx.canvas(canvas, move |painter| {
                let viewport_h = Px(scroll_handle.viewport_size().height.0.max(0.0));
                let offset_y = Px(scroll_handle.offset().y.0.max(0.0));
                let offset_y = metrics.clamp_offset(offset_y, viewport_h);
                let Some(visible) = metrics.visible_range(offset_y, viewport_h, overscan) else {
                    return;
                };

                let width = Px(painter.bounds().size.width.0.max(0.0));
                let count = visible.count;
                if count == 0 {
                    return;
                }

                let start = visible.start_index.saturating_sub(visible.overscan);
                let end = (visible.end_index + visible.overscan).min(count.saturating_sub(1));

                if let Some(on_paint_frame) = &on_paint_frame {
                    on_paint_frame(
                        painter,
                        WindowedRowsPaintFrame {
                            viewport_height: viewport_h,
                            offset_y,
                            visible_start: start,
                            visible_end: end,
                        },
                    );
                }

                for index in start..=end {
                    let y = metrics.offset_for_index(index);
                    let h = metrics.height_at(index);
                    let rect = Rect::new(Point::new(Px(0.0), y), Size::new(width, h));
                    paint_row(painter, index, rect);
                }
            })];

            if let Some(semantics) = content_semantics.clone() {
                vec![cx.semantics(semantics, |_cx| canvas_children)]
            } else {
                canvas_children
            }
        })]
    })
}
