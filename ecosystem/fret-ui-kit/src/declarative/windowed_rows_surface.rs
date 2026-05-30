//! Windowed row surface helpers.
//!
//! This module provides an ecosystem-level building block for “prepaint-windowed virtual
//! surfaces” (ADR 0175) in the subset of cases where:
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

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::panic::Location;

use fret_core::time::Instant;
use fret_core::{Point, Px, Rect, Size};
use fret_runtime::FrameId;
use fret_ui::action::{ActionCx, OnTimer, PointerDownCx, PointerMoveCx, UiPointerActionHost};
use fret_ui::canvas::{CanvasPainter, CanvasPrepaintCx};
use fret_ui::element::{
    AnyElement, CanvasProps, Length, PointerRegionProps, ScrollAxis, ScrollProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::virtual_list::VirtualListMetrics;
use fret_ui::{ElementContext, UiHost};
use tracing::info;

#[derive(Debug, Clone, Copy)]
pub struct WindowedRowsPaintFrame {
    pub viewport_height: Px,
    pub offset_y: Px,
    pub row_height: Px,
    pub row_stride: Px,
    pub gap: Px,
    pub scroll_margin: Px,
    pub visible_start: usize,
    pub visible_end: usize,
}

impl WindowedRowsPaintFrame {
    pub fn row_offset_y(&self, index: usize) -> Px {
        Px(self.scroll_margin.0.max(0.0) + self.row_stride.0.max(0.0) * index as f32)
    }

    pub fn row_rect(&self, content_bounds: Rect, index: usize) -> Option<Rect> {
        if index < self.visible_start || index > self.visible_end {
            return None;
        }

        Some(self.row_rect_for_visible_index(content_bounds, index))
    }

    pub fn row_rects(&self, content_bounds: Rect) -> WindowedRowsRectIter {
        WindowedRowsRectIter {
            next_index: self.visible_start,
            end_index: self.visible_end,
            next_offset_y: self.row_offset_y(self.visible_start).0,
            row_stride: self.row_stride.0.max(0.0),
            origin_x: content_bounds.origin.x,
            origin_y: content_bounds.origin.y,
            width: Px(content_bounds.size.width.0.max(0.0)),
            height: Px(self.row_height.0.max(0.0)),
            finished: self.visible_start > self.visible_end,
        }
    }

    fn row_rect_for_visible_index(&self, content_bounds: Rect, index: usize) -> Rect {
        let offset_y = self.row_offset_y(index);
        Rect::new(
            Point::new(
                content_bounds.origin.x,
                Px(content_bounds.origin.y.0 + offset_y.0),
            ),
            Size::new(
                Px(content_bounds.size.width.0.max(0.0)),
                Px(self.row_height.0.max(0.0)),
            ),
        )
    }
}

#[derive(Debug, Clone)]
pub struct WindowedRowsRectIter {
    next_index: usize,
    end_index: usize,
    next_offset_y: f32,
    row_stride: f32,
    origin_x: Px,
    origin_y: Px,
    width: Px,
    height: Px,
    finished: bool,
}

impl Iterator for WindowedRowsRectIter {
    type Item = (usize, Rect);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let index = self.next_index;
        if index > self.end_index {
            self.finished = true;
            return None;
        }

        let rect = Rect::new(
            Point::new(self.origin_x, Px(self.origin_y.0 + self.next_offset_y)),
            Size::new(self.width, self.height),
        );

        if index == self.end_index {
            self.finished = true;
        } else {
            self.next_index = self.next_index.saturating_add(1);
            self.next_offset_y += self.row_stride;
        }

        Some((index, rect))
    }
}

pub type OnWindowedRowsPaintFrame =
    std::sync::Arc<dyn for<'p> Fn(&mut CanvasPainter<'p>, WindowedRowsPaintFrame) + 'static>;
pub type OnWindowedRowsPrepaintFrame =
    std::sync::Arc<dyn for<'p> Fn(&mut CanvasPrepaintCx<'p>, WindowedRowsPaintFrame) + 'static>;
pub type OnWindowedRowsPaintDiagnostics =
    std::sync::Arc<dyn Fn(WindowedRowsPaintDiagnostics) + 'static>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WindowedRowsPaintDiagnostics {
    pub visible_start: u64,
    pub visible_end: u64,
    pub visible_rows: u64,
    pub rows_iterated: u64,
    pub rows_with_rect: u64,

    pub us_paint_callback: u64,
    pub us_frame_lookup: u64,
    pub us_on_paint_frame: u64,
    pub us_row_loop: u64,
    pub us_row_rect: u64,
    pub us_row_paint: u64,
    pub us_non_row: u64,

    pub ns_paint_callback: u64,
    pub ns_frame_lookup: u64,
    pub ns_on_paint_frame: u64,
    pub ns_row_loop: u64,
    pub ns_row_rect: u64,
    pub ns_row_paint: u64,
    pub ns_non_row: u64,
}

impl WindowedRowsPaintDiagnostics {
    fn for_frame(frame: WindowedRowsPaintFrame) -> Self {
        Self {
            visible_start: frame.visible_start as u64,
            visible_end: frame.visible_end as u64,
            visible_rows: frame
                .visible_end
                .saturating_sub(frame.visible_start)
                .saturating_add(1) as u64,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowedRowsSurfaceWindowTelemetry {
    pub callsite_id: u64,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,

    pub len: u64,
    pub row_height: Px,
    pub overscan: u64,
    pub gap: Px,
    pub scroll_margin: Px,

    pub viewport_height: Px,
    pub offset_y: Px,
    pub content_height: Px,

    pub visible_start: Option<u64>,
    pub visible_end: Option<u64>,
    pub visible_count: u64,
}

#[derive(Default)]
pub struct WindowedRowsSurfaceDiagnosticsStore {
    per_window: HashMap<fret_core::AppWindowId, WindowedRowsSurfaceDiagnosticsFrame>,
}

#[derive(Default)]
struct WindowedRowsSurfaceDiagnosticsFrame {
    frame_id: FrameId,
    windows: Vec<WindowedRowsSurfaceWindowTelemetry>,
}

impl WindowedRowsSurfaceDiagnosticsStore {
    pub fn begin_frame(&mut self, window: fret_core::AppWindowId, frame_id: FrameId) {
        let w = self.per_window.entry(window).or_default();
        if w.frame_id != frame_id {
            w.frame_id = frame_id;
            w.windows.clear();
        }
    }

    pub fn record_window(
        &mut self,
        window: fret_core::AppWindowId,
        frame_id: FrameId,
        telemetry: WindowedRowsSurfaceWindowTelemetry,
    ) {
        self.begin_frame(window, frame_id);
        let w = self.per_window.entry(window).or_default();
        w.windows.push(telemetry);
    }

    #[allow(dead_code)]
    pub fn windows_for_window(
        &self,
        window: fret_core::AppWindowId,
        frame_id: FrameId,
    ) -> Option<&[WindowedRowsSurfaceWindowTelemetry]> {
        let w = self.per_window.get(&window)?;
        (w.frame_id == frame_id).then_some(w.windows.as_slice())
    }
}

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
    pub on_prepaint_frame: Option<OnWindowedRowsPrepaintFrame>,
    pub on_paint_frame: Option<OnWindowedRowsPaintFrame>,
    pub on_paint_diagnostics: Option<OnWindowedRowsPaintDiagnostics>,
}

impl Default for WindowedRowsSurfaceProps {
    fn default() -> Self {
        let scroll = ScrollProps {
            axis: ScrollAxis::Y,
            layout: fret_ui::element::LayoutStyle {
                size: fret_ui::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            // This surface's paint output depends on the scroll offset (visible window changes), so
            // scroll-handle updates must repaint even when the view-cache render subtree is reused.
            windowed_paint: true,
            ..Default::default()
        };

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
            on_prepaint_frame: None,
            on_paint_frame: None,
            on_paint_diagnostics: None,
        }
    }
}

fn elapsed_us_ns(started: Instant) -> (u64, u64) {
    let elapsed = started.elapsed();
    (
        elapsed.as_micros().min(u128::from(u64::MAX)) as u64,
        elapsed.as_nanos().min(u128::from(u64::MAX)) as u64,
    )
}

fn current_windowed_rows_frame(
    metrics: &VirtualListMetrics,
    scroll_handle: &ScrollHandle,
    overscan: usize,
) -> Option<WindowedRowsPaintFrame> {
    let viewport_h = Px(scroll_handle.viewport_size().height.0.max(0.0));
    let offset_y = Px(scroll_handle.offset().y.0.max(0.0));
    let offset_y = metrics.clamp_offset(offset_y, viewport_h);
    let visible = metrics.visible_range(offset_y, viewport_h, overscan)?;
    if visible.count == 0 {
        return None;
    }

    let start = visible.start_index.saturating_sub(visible.overscan);
    let end = (visible.end_index + visible.overscan).min(visible.count.saturating_sub(1));
    let row_height = metrics.height_at(0);
    let scroll_margin = metrics.offset_for_index(0);
    let row_stride = if visible.count > 1 {
        Px((metrics.offset_for_index(1).0 - scroll_margin.0).max(0.0))
    } else {
        Px((row_height.0 + metrics.gap().0).max(0.0))
    };

    Some(WindowedRowsPaintFrame {
        viewport_height: viewport_h,
        offset_y,
        row_height,
        row_stride,
        gap: metrics.gap(),
        scroll_margin,
        visible_start: start,
        visible_end: end,
    })
}

fn paint_windowed_rows<P>(
    metrics: &VirtualListMetrics,
    scroll_handle: &ScrollHandle,
    overscan: usize,
    painter: &mut CanvasPainter<'_>,
    on_paint_frame: Option<&OnWindowedRowsPaintFrame>,
    on_paint_diagnostics: Option<&OnWindowedRowsPaintDiagnostics>,
    paint_row: &P,
) where
    P: for<'p> Fn(&mut CanvasPainter<'p>, usize, Rect) + ?Sized,
{
    let Some(on_paint_diagnostics) = on_paint_diagnostics else {
        let Some(frame) = current_windowed_rows_frame(metrics, scroll_handle, overscan) else {
            return;
        };

        let bounds = painter.bounds();

        if let Some(on_paint_frame) = on_paint_frame {
            on_paint_frame(painter, frame);
        }

        for (index, rect) in frame.row_rects(bounds) {
            paint_row(painter, index, rect);
        }
        return;
    };

    let paint_callback_started = Instant::now();
    let frame_lookup_started = Instant::now();
    let Some(frame) = current_windowed_rows_frame(metrics, scroll_handle, overscan) else {
        return;
    };
    let (us_frame_lookup, ns_frame_lookup) = elapsed_us_ns(frame_lookup_started);
    let mut diagnostics = WindowedRowsPaintDiagnostics::for_frame(frame);
    diagnostics.us_frame_lookup = us_frame_lookup;
    diagnostics.ns_frame_lookup = ns_frame_lookup;

    let bounds = painter.bounds();

    if let Some(on_paint_frame) = on_paint_frame {
        let started = Instant::now();
        on_paint_frame(painter, frame);
        let (us, ns) = elapsed_us_ns(started);
        diagnostics.us_on_paint_frame = us;
        diagnostics.ns_on_paint_frame = ns;
    }

    let row_loop_started = Instant::now();
    let mut row_rects = frame.row_rects(bounds);
    loop {
        let row_rect_started = Instant::now();
        let next = row_rects.next();
        let (us, ns) = elapsed_us_ns(row_rect_started);
        diagnostics.us_row_rect = diagnostics.us_row_rect.saturating_add(us);
        diagnostics.ns_row_rect = diagnostics.ns_row_rect.saturating_add(ns);

        let Some((index, rect)) = next else {
            break;
        };

        diagnostics.rows_iterated = diagnostics.rows_iterated.saturating_add(1);
        diagnostics.rows_with_rect = diagnostics.rows_with_rect.saturating_add(1);
        let row_paint_started = Instant::now();
        paint_row(painter, index, rect);
        let (us, ns) = elapsed_us_ns(row_paint_started);
        diagnostics.us_row_paint = diagnostics.us_row_paint.saturating_add(us);
        diagnostics.ns_row_paint = diagnostics.ns_row_paint.saturating_add(ns);
    }
    let (us_row_loop, ns_row_loop) = elapsed_us_ns(row_loop_started);
    diagnostics.us_row_loop = us_row_loop;
    diagnostics.ns_row_loop = ns_row_loop;

    let (us_paint_callback, ns_paint_callback) = elapsed_us_ns(paint_callback_started);
    diagnostics.us_paint_callback = us_paint_callback;
    diagnostics.ns_paint_callback = ns_paint_callback;
    diagnostics.us_non_row = diagnostics
        .us_paint_callback
        .saturating_sub(diagnostics.us_row_paint);
    diagnostics.ns_non_row = diagnostics
        .ns_paint_callback
        .saturating_sub(diagnostics.ns_row_paint);

    on_paint_diagnostics(diagnostics);
}

/// Build a fixed-row-height scroll surface that paints only the visible row window.
///
/// `paint_row` is called for each visible row (including overscan).
///
/// Coordinate space: the provided `Rect` is expressed in the same "content space" that the
/// scroll container uses for its child subtree. Concretely, it is anchored at the canvas node's
/// layout bounds (not `0,0`).
///
/// This matches how other `CanvasPainter` consumers treat `Rect` coordinates (absolute in the
/// current transform space) and avoids callers accidentally painting at the window origin.
#[track_caller]
pub fn windowed_rows_surface<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: WindowedRowsSurfaceProps,
    paint_row: impl for<'p> Fn(&mut CanvasPainter<'p>, usize, Rect) + 'static,
) -> AnyElement {
    let caller = Location::caller();
    let WindowedRowsSurfaceProps {
        mut scroll,
        mut canvas,
        len,
        row_height,
        overscan,
        gap,
        scroll_margin,
        scroll_handle,
        on_prepaint_frame,
        on_paint_frame,
        on_paint_diagnostics,
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

    let viewport_h = Px(scroll_handle.viewport_size().height.0.max(0.0));
    let offset_y = Px(scroll_handle.offset().y.0.max(0.0));
    let offset_y = metrics.clamp_offset(offset_y, viewport_h);
    let visible = metrics.visible_range(offset_y, viewport_h, overscan);

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    caller.file().hash(&mut hasher);
    caller.line().hash(&mut hasher);
    caller.column().hash(&mut hasher);
    let callsite_id = hasher.finish();

    cx.app.with_global_mut_untracked(
        WindowedRowsSurfaceDiagnosticsStore::default,
        |store, _app| {
            let (visible_start, visible_end, visible_count) = visible
                .map(|visible| {
                    let count = visible.count;
                    if count == 0 {
                        return (None, None, 0u64);
                    }
                    let start = visible.start_index.saturating_sub(visible.overscan);
                    let end = (visible.end_index + visible.overscan).min(count.saturating_sub(1));
                    (
                        Some(start as u64),
                        Some(end as u64),
                        (end.saturating_sub(start) as u64).saturating_add(1),
                    )
                })
                .unwrap_or((None, None, 0));
            store.record_window(
                cx.window,
                cx.frame_id,
                WindowedRowsSurfaceWindowTelemetry {
                    callsite_id,
                    file: caller.file(),
                    line: caller.line(),
                    column: caller.column(),
                    len: len as u64,
                    row_height,
                    overscan: overscan as u64,
                    gap,
                    scroll_margin,
                    viewport_height: viewport_h,
                    offset_y,
                    content_height: content_h,
                    visible_start,
                    visible_end,
                    visible_count,
                },
            );
        },
    );

    scroll.axis = ScrollAxis::Y;
    scroll.scroll_handle = Some(scroll_handle.clone());
    // This surface's paint output depends on the scroll offset (visible window changes), so
    // scroll-handle updates must repaint even when the view-cache render subtree is reused.
    scroll.windowed_paint = true;

    canvas.layout.size.width = Length::Fill;
    canvas.layout.size.height = Length::Px(content_h);

    cx.scroll(scroll, move |cx| {
        let scroll_handle = scroll_handle.clone();
        let metrics = metrics.clone();
        let paint_row = std::sync::Arc::new(paint_row);
        let on_prepaint_frame = on_prepaint_frame.clone();
        let on_paint_frame = on_paint_frame.clone();
        let on_paint_diagnostics = on_paint_diagnostics.clone();
        let prepaint_scroll_handle = scroll_handle.clone();
        let prepaint_metrics = metrics.clone();

        let paint = move |painter: &mut CanvasPainter<'_>| {
            paint_windowed_rows(
                &metrics,
                &scroll_handle,
                overscan,
                painter,
                on_paint_frame.as_ref(),
                on_paint_diagnostics.as_ref(),
                paint_row.as_ref(),
            );
        };

        let canvas = if let Some(on_prepaint_frame) = on_prepaint_frame {
            cx.canvas_with_prepaint(
                canvas,
                move |cx| {
                    if let Some(frame) = current_windowed_rows_frame(
                        &prepaint_metrics,
                        &prepaint_scroll_handle,
                        overscan,
                    ) {
                        on_prepaint_frame(cx, frame);
                    }
                },
                paint,
            )
        } else {
            cx.canvas(canvas, paint)
        };

        vec![canvas]
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
    pub on_timer: Option<OnTimer>,
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

    if std::env::var_os("FRET_WINDOWED_ROWS_POINTER_DEBUG")
        .is_some_and(|v| !v.is_empty() && v != "0")
    {
        info!(
            "windowed_rows_pointer bounds_y={} pos_y={} local_y={} offset_y={} viewport_h={}",
            bounds.origin.y.0, position.y.0, local_y.0, offset_y.0, viewport_h.0
        );
    }

    // Pointer event positions are mapped through the UI tree's transforms. Scroll containers apply
    // their offset via `children_render_transform`, so descendants typically receive positions in
    // stable "content space" already.
    //
    // For robustness (and to avoid double-counting the scroll offset), compute candidate indices
    // for both:
    // - viewport-space events: content_y = offset + local
    // - content-space events:  content_y = local
    let idx_viewport = metrics.index_for_offset(Px(offset_y.0 + local_y.0));
    let idx_content = metrics.index_for_offset(local_y);

    let idx = if let Some(visible) = metrics.visible_range(offset_y, viewport_h, 0) {
        let in_visible = |idx: usize| idx >= visible.start_index && idx <= visible.end_index;
        match (in_visible(idx_viewport), in_visible(idx_content)) {
            (true, false) => idx_viewport,
            (false, true) => idx_content,
            // Prefer content-space indices by default (matches runtime event mapping).
            _ => idx_content,
        }
    } else {
        idx_content
    };

    Some(idx.min(len.saturating_sub(1)))
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
    let caller = Location::caller();
    let WindowedRowsSurfacePointerHandlers {
        on_pointer_down,
        on_pointer_move,
        on_pointer_up,
        on_pointer_cancel,
        on_timer,
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
        on_prepaint_frame,
        on_paint_frame,
        on_paint_diagnostics,
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

    let viewport_h = Px(scroll_handle.viewport_size().height.0.max(0.0));
    let offset_y = Px(scroll_handle.offset().y.0.max(0.0));
    let offset_y = metrics.clamp_offset(offset_y, viewport_h);
    let visible = metrics.visible_range(offset_y, viewport_h, overscan);

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    caller.file().hash(&mut hasher);
    caller.line().hash(&mut hasher);
    caller.column().hash(&mut hasher);
    let callsite_id = hasher.finish();

    cx.app.with_global_mut_untracked(
        WindowedRowsSurfaceDiagnosticsStore::default,
        |store, _app| {
            let (visible_start, visible_end, visible_count) = visible
                .map(|visible| {
                    let count = visible.count;
                    if count == 0 {
                        return (None, None, 0u64);
                    }
                    let start = visible.start_index.saturating_sub(visible.overscan);
                    let end = (visible.end_index + visible.overscan).min(count.saturating_sub(1));
                    (
                        Some(start as u64),
                        Some(end as u64),
                        (end.saturating_sub(start) as u64).saturating_add(1),
                    )
                })
                .unwrap_or((None, None, 0));
            store.record_window(
                cx.window,
                cx.frame_id,
                WindowedRowsSurfaceWindowTelemetry {
                    callsite_id,
                    file: caller.file(),
                    line: caller.line(),
                    column: caller.column(),
                    len: len as u64,
                    row_height,
                    overscan: overscan as u64,
                    gap,
                    scroll_margin,
                    viewport_height: viewport_h,
                    offset_y,
                    content_height: content_h,
                    visible_start,
                    visible_end,
                    visible_count,
                },
            );
        },
    );

    scroll.axis = ScrollAxis::Y;
    scroll.scroll_handle = Some(scroll_handle.clone());
    // This surface's paint output depends on the scroll offset (visible window changes), so
    // scroll-handle updates must repaint even when the view-cache render subtree is reused.
    scroll.windowed_paint = true;

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
        let on_prepaint_frame = on_prepaint_frame.clone();
        let on_paint_frame = on_paint_frame.clone();
        let on_paint_diagnostics = on_paint_diagnostics.clone();

        vec![cx.pointer_region(pointer, move |cx| {
            if let Some(on_timer) = on_timer.clone() {
                // Surface hooks may share the pointer region root with other helper-installed
                // timers, so compose rather than replacing an existing handler.
                cx.timer_add_on_timer_for(cx.root_id(), on_timer);
            }

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

            let prepaint_scroll_handle = scroll_handle.clone();
            let prepaint_metrics = metrics.clone();
            let paint = move |painter: &mut CanvasPainter<'_>| {
                paint_windowed_rows(
                    &metrics,
                    &scroll_handle,
                    overscan,
                    painter,
                    on_paint_frame.as_ref(),
                    on_paint_diagnostics.as_ref(),
                    paint_row.as_ref(),
                );
            };

            let canvas = if let Some(on_prepaint_frame) = on_prepaint_frame.clone() {
                cx.canvas_with_prepaint(
                    canvas,
                    move |cx| {
                        if let Some(frame) = current_windowed_rows_frame(
                            &prepaint_metrics,
                            &prepaint_scroll_handle,
                            overscan,
                        ) {
                            on_prepaint_frame(cx, frame);
                        }
                    },
                    paint,
                )
            } else {
                cx.canvas(canvas, paint)
            };

            let canvas_children = vec![canvas];

            if let Some(semantics) = content_semantics.clone() {
                vec![cx.semantics(semantics, |_cx| canvas_children)]
            } else {
                canvas_children
            }
        })]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Scene, Size};
    use fret_ui::{UiTree, declarative::render_root};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct FakeServices;

    impl fret_core::TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: Size::new(Px(10.0), Px(16.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    #[test]
    fn default_props_enable_windowed_paint() {
        let props = WindowedRowsSurfaceProps::default();
        assert_eq!(props.scroll.axis, ScrollAxis::Y);
        assert!(props.scroll.windowed_paint);
        assert!(props.on_paint_diagnostics.is_none());
    }

    #[test]
    fn windowed_rows_frame_row_rect_uses_surface_geometry() {
        let frame = WindowedRowsPaintFrame {
            viewport_height: Px(48.0),
            offset_y: Px(0.0),
            row_height: Px(20.0),
            row_stride: Px(24.0),
            gap: Px(4.0),
            scroll_margin: Px(6.0),
            visible_start: 2,
            visible_end: 4,
        };
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(30.0)),
            Size::new(Px(120.0), Px(96.0)),
        );

        assert_eq!(frame.row_rect(bounds, 1), None);
        assert_eq!(
            frame.row_rect(bounds, 3),
            Some(Rect::new(
                Point::new(Px(10.0), Px(108.0)),
                Size::new(Px(120.0), Px(20.0))
            ))
        );
    }

    #[test]
    fn windowed_rows_frame_row_rects_iterates_visible_rows() {
        let frame = WindowedRowsPaintFrame {
            viewport_height: Px(48.0),
            offset_y: Px(0.0),
            row_height: Px(20.0),
            row_stride: Px(24.0),
            gap: Px(4.0),
            scroll_margin: Px(6.0),
            visible_start: 2,
            visible_end: 4,
        };
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(30.0)),
            Size::new(Px(120.0), Px(96.0)),
        );

        let rows = frame.row_rects(bounds).collect::<Vec<_>>();
        assert_eq!(
            rows,
            vec![
                (
                    2,
                    Rect::new(
                        Point::new(Px(10.0), Px(84.0)),
                        Size::new(Px(120.0), Px(20.0))
                    )
                ),
                (
                    3,
                    Rect::new(
                        Point::new(Px(10.0), Px(108.0)),
                        Size::new(Px(120.0), Px(20.0))
                    )
                ),
                (
                    4,
                    Rect::new(
                        Point::new(Px(10.0), Px(132.0)),
                        Size::new(Px(120.0), Px(20.0))
                    )
                ),
            ]
        );
    }

    #[test]
    fn on_prepaint_frame_runs_before_on_paint_frame_for_windowed_rows_surface() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(48.0)));
        let scroll_handle = fret_ui::scroll::ScrollHandle::default();
        scroll_handle.set_viewport_size(Size::new(Px(160.0), Px(48.0)));
        scroll_handle.set_content_size(Size::new(Px(160.0), Px(96.0)));

        let prepaint_calls = Arc::new(AtomicUsize::new(0));
        let paint_calls = Arc::new(AtomicUsize::new(0));
        let paint_diagnostics = Arc::new(Mutex::new(None));

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "windowed-rows",
            |cx| {
                let prepaint_calls = Arc::clone(&prepaint_calls);
                let prepaint_calls_for_paint = Arc::clone(&prepaint_calls);
                let paint_calls = Arc::clone(&paint_calls);
                let paint_diagnostics = Arc::clone(&paint_diagnostics);

                let mut props = WindowedRowsSurfaceProps::default();
                props.len = 4;
                props.row_height = Px(24.0);
                props.overscan = 0;
                props.scroll_handle = scroll_handle.clone();
                props.on_prepaint_frame = Some(Arc::new(move |cx, frame| {
                    assert_eq!(frame.visible_start, 0);
                    assert!(frame.visible_end >= frame.visible_start);
                    let _ = cx;
                    prepaint_calls.fetch_add(1, Ordering::SeqCst);
                }));
                props.on_paint_frame = Some(Arc::new(move |_painter, _frame| {
                    assert_eq!(prepaint_calls_for_paint.load(Ordering::SeqCst), 1);
                    paint_calls.fetch_add(1, Ordering::SeqCst);
                }));
                props.on_paint_diagnostics = Some(Arc::new(move |diagnostics| {
                    *paint_diagnostics.lock().expect("paint diagnostics lock") = Some(diagnostics);
                }));

                vec![windowed_rows_surface(
                    cx,
                    props,
                    move |_painter, _index, _rect| {},
                )]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        assert_eq!(prepaint_calls.load(Ordering::SeqCst), 1);
        assert_eq!(paint_calls.load(Ordering::SeqCst), 1);
        let diagnostics = paint_diagnostics
            .lock()
            .expect("paint diagnostics lock")
            .expect("paint diagnostics");
        assert_eq!(diagnostics.visible_start, 0);
        assert!(diagnostics.visible_rows > 0);
        assert_eq!(diagnostics.rows_iterated, diagnostics.visible_rows);
        assert_eq!(diagnostics.rows_with_rect, diagnostics.visible_rows);
    }
}
