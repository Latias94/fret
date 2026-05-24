use std::sync::{Arc, Mutex};

use delinea::engine::window::DataWindow;
use delinea::{Action, AxisPosition, AxisRange, ChartEngine};
use fret_canvas::ui::{
    CanvasToolDownResult, CanvasToolEntry, CanvasToolHandlers, CanvasToolId,
    OnCanvasToolPointerDown, OnCanvasToolPointerMove, OnCanvasToolPointerUp, PanZoomCanvasPaintCx,
};
use fret_core::{Corners, DrawOrder, Edges, MouseButton, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::canvas::CanvasPainter;

use crate::ChartStyle;
use crate::slider_logic::{
    SliderDragKind, SliderDragPermissions, slider_drag_start_at_x, slider_drag_start_at_y,
    slider_drag_update_at_x, slider_drag_update_at_y, slider_norm,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataZoomAxisKind {
    X,
    Y,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DataZoomTrackState {
    pub(crate) axis_kind: DataZoomAxisKind,
    pub(crate) axis: delinea::AxisId,
    pub(crate) track: Rect,
    pub(crate) extent: DataWindow,
    pub(crate) current_window: DataWindow,
    pub(crate) permissions: SliderDragPermissions,
    pub(crate) interactive: bool,
}

#[derive(Debug, Clone, Copy)]
struct DataZoomSliderDrag {
    axis_kind: DataZoomAxisKind,
    axis: delinea::AxisId,
    kind: SliderDragKind,
    track: Rect,
    extent: DataWindow,
    start_pos: Point,
    start_window: DataWindow,
}

#[derive(Debug, Default)]
pub(crate) struct DataZoomOverlayState {
    tracks: Vec<DataZoomTrackState>,
    drag: Option<DataZoomSliderDrag>,
}

impl DataZoomOverlayState {
    pub(crate) fn sync_tracks(&mut self, tracks: Vec<DataZoomTrackState>) {
        let has_track = |axis: delinea::AxisId| tracks.iter().any(|track| track.axis == axis);

        if self.drag.is_some_and(|drag| !has_track(drag.axis)) {
            self.drag = None;
        }

        if let Some(drag) = self.drag.as_mut()
            && let Some(track) = tracks.iter().find(|track| track.axis == drag.axis)
        {
            drag.track = track.track;
            drag.extent = track.extent;
        }

        self.tracks = tracks;
    }

    fn track_at(&self, position: Point) -> Option<DataZoomTrackState> {
        self.tracks
            .iter()
            .copied()
            .find(|track| track.track.contains(position))
    }
}

#[derive(Debug, Clone, Copy)]
struct AxisBandLayout {
    axis: delinea::AxisId,
    position: AxisPosition,
    rect: Rect,
}

#[derive(Debug, Clone)]
struct ChartLayout {
    plot: Rect,
    x_axes: Vec<AxisBandLayout>,
    y_axes: Vec<AxisBandLayout>,
}

fn primary_axes(engine: &ChartEngine) -> Option<(delinea::AxisId, delinea::AxisId)> {
    let model = engine.model();
    for series_id in &model.series_order {
        let series = model.series.get(series_id)?;
        if series.visible {
            return Some((series.x_axis, series.y_axis));
        }
    }
    None
}

fn active_grid_for_chart(model: &delinea::engine::model::ChartModel) -> Option<delinea::GridId> {
    model
        .series_in_order()
        .find(|series| series.visible)
        .and_then(|series| model.axes.get(&series.x_axis).map(|axis| axis.grid))
}

fn chart_layout_for_engine(
    engine: &ChartEngine,
    bounds: Rect,
    style: ChartStyle,
) -> Option<ChartLayout> {
    let model = engine.model();
    let active_grid = active_grid_for_chart(model)?;

    let has_visual_map = model.series_in_order().any(|series| {
        series.visible
            && model
                .axes
                .get(&series.x_axis)
                .is_some_and(|axis| axis.grid == active_grid)
            && model.visual_map_by_series.contains_key(&series.id)
    });

    let axis_band_x = style.axis_band_x.0.max(0.0);
    let axis_band_y = style.axis_band_y.0.max(0.0);
    let visual_map_band_x = if has_visual_map {
        style.visual_map_band_x.0.max(0.0)
    } else {
        0.0
    };

    let mut x_top: Vec<delinea::AxisId> = Vec::new();
    let mut x_bottom: Vec<delinea::AxisId> = Vec::new();
    let mut y_left: Vec<delinea::AxisId> = Vec::new();
    let mut y_right: Vec<delinea::AxisId> = Vec::new();

    for (axis_id, axis) in &model.axes {
        if axis.grid != active_grid {
            continue;
        }

        match (axis.kind, axis.position) {
            (delinea::AxisKind::X, AxisPosition::Top) => x_top.push(*axis_id),
            (delinea::AxisKind::X, AxisPosition::Bottom) => x_bottom.push(*axis_id),
            (delinea::AxisKind::Y, AxisPosition::Left) => y_left.push(*axis_id),
            (delinea::AxisKind::Y, AxisPosition::Right) => y_right.push(*axis_id),
            _ => {}
        }
    }

    let mut inner = bounds;
    inner.origin.x.0 += style.padding.left.0;
    inner.origin.y.0 += style.padding.top.0;
    inner.size.width.0 =
        (inner.size.width.0 - style.padding.left.0 - style.padding.right.0).max(0.0);
    inner.size.height.0 =
        (inner.size.height.0 - style.padding.top.0 - style.padding.bottom.0).max(0.0);

    let left_total = axis_band_x * (y_left.len() as f32);
    let right_total = axis_band_x * (y_right.len() as f32);
    let top_total = axis_band_y * (x_top.len() as f32);
    let bottom_total = axis_band_y * (x_bottom.len() as f32);

    let plot_w = (inner.size.width.0 - left_total - right_total - visual_map_band_x).max(0.0);
    let plot_h = (inner.size.height.0 - top_total - bottom_total).max(0.0);
    let plot = Rect::new(
        Point::new(
            Px(inner.origin.x.0 + left_total),
            Px(inner.origin.y.0 + top_total),
        ),
        Size::new(Px(plot_w), Px(plot_h)),
    );

    let mut x_axes: Vec<AxisBandLayout> = Vec::with_capacity(x_top.len() + x_bottom.len());
    for (i, axis) in x_top.iter().copied().enumerate() {
        let rect = Rect::new(
            Point::new(
                plot.origin.x,
                Px(plot.origin.y.0 - axis_band_y * (i as f32 + 1.0)),
            ),
            Size::new(plot.size.width, Px(axis_band_y)),
        );
        x_axes.push(AxisBandLayout {
            axis,
            position: AxisPosition::Top,
            rect,
        });
    }
    for (i, axis) in x_bottom.iter().copied().enumerate() {
        let rect = Rect::new(
            Point::new(
                plot.origin.x,
                Px(plot.origin.y.0 + plot.size.height.0 + axis_band_y * (i as f32)),
            ),
            Size::new(plot.size.width, Px(axis_band_y)),
        );
        x_axes.push(AxisBandLayout {
            axis,
            position: AxisPosition::Bottom,
            rect,
        });
    }

    let mut y_axes: Vec<AxisBandLayout> = Vec::with_capacity(y_left.len() + y_right.len());
    for (i, axis) in y_left.iter().copied().enumerate() {
        let rect = Rect::new(
            Point::new(
                Px(plot.origin.x.0 - axis_band_x * (i as f32 + 1.0)),
                plot.origin.y,
            ),
            Size::new(Px(axis_band_x), plot.size.height),
        );
        y_axes.push(AxisBandLayout {
            axis,
            position: AxisPosition::Left,
            rect,
        });
    }
    for (i, axis) in y_right.iter().copied().enumerate() {
        let rect = Rect::new(
            Point::new(
                Px(plot.origin.x.0 + plot.size.width.0 + axis_band_x * (i as f32)),
                plot.origin.y,
            ),
            Size::new(Px(axis_band_x), plot.size.height),
        );
        y_axes.push(AxisBandLayout {
            axis,
            position: AxisPosition::Right,
            rect,
        });
    }

    Some(ChartLayout {
        plot,
        x_axes,
        y_axes,
    })
}

fn axis_range(engine: &ChartEngine, axis: delinea::AxisId) -> AxisRange {
    engine
        .model()
        .axes
        .get(&axis)
        .map(|axis| axis.range)
        .unwrap_or_default()
}

fn axis_is_fixed(engine: &ChartEngine, axis: delinea::AxisId) -> Option<DataWindow> {
    match axis_range(engine, axis) {
        AxisRange::Fixed { min, max } => {
            let mut window = DataWindow { min, max };
            window.clamp_non_degenerate();
            Some(window)
        }
        _ => None,
    }
}

fn axis_constraints(engine: &ChartEngine, axis: delinea::AxisId) -> (Option<f64>, Option<f64>) {
    match axis_range(engine, axis) {
        AxisRange::Auto => (None, None),
        AxisRange::LockMin { min } => (Some(min), None),
        AxisRange::LockMax { max } => (None, Some(max)),
        AxisRange::Fixed { min, max } => (Some(min), Some(max)),
    }
}

fn slider_permissions_for_axis(
    engine: &ChartEngine,
    axis: delinea::AxisId,
) -> SliderDragPermissions {
    let (locked_min, locked_max) = axis_constraints(engine, axis);
    SliderDragPermissions {
        pan: locked_min.is_none() && locked_max.is_none(),
        handle_min: locked_min.is_none(),
        handle_max: locked_max.is_none(),
    }
}

fn compute_axis_extent_from_data(
    engine: &mut ChartEngine,
    axis: delinea::AxisId,
    is_x: bool,
) -> DataWindow {
    let (axis_range, series_cols) = {
        let model = engine.model();
        let axis_range = model
            .axes
            .get(&axis)
            .map(|axis| axis.range)
            .unwrap_or_default();

        if let Some(axis_model) = model.axes.get(&axis)
            && let delinea::AxisScale::Category(scale) = &axis_model.scale
            && !scale.categories.is_empty()
        {
            let mut window = DataWindow {
                min: -0.5,
                max: scale.categories.len() as f64 - 0.5,
            };
            window = window.apply_constraints(axis_range.locked_min(), axis_range.locked_max());
            window.clamp_non_degenerate();
            return window;
        }

        let mut series_cols: Vec<(delinea::DatasetId, usize)> = Vec::new();
        for series_id in &model.series_order {
            let Some(series) = model.series.get(series_id) else {
                continue;
            };
            if !series.visible {
                continue;
            }

            let axis_id = if is_x { series.x_axis } else { series.y_axis };
            if axis_id != axis {
                continue;
            }

            let Some(dataset) = model.datasets.get(&series.dataset) else {
                continue;
            };
            let field = if is_x {
                series.encode.x
            } else {
                series.encode.y
            };
            let Some(col) = dataset.fields.get(&field).copied() else {
                continue;
            };
            series_cols.push((series.dataset, col));
        }

        (axis_range, series_cols)
    };

    let (min, max) = {
        let datasets = engine.datasets_mut();
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for (dataset_id, col) in &series_cols {
            let Some(table) = datasets.dataset_mut(*dataset_id) else {
                continue;
            };
            let Some(values) = table.column_f64(*col) else {
                continue;
            };

            for &value in values {
                if !value.is_finite() {
                    continue;
                }
                min = min.min(value);
                max = max.max(value);
            }
        }

        (min, max)
    };

    let mut window = if min.is_finite() && max.is_finite() && max > min {
        DataWindow { min, max }
    } else {
        DataWindow { min: 0.0, max: 1.0 }
    };
    window = window.apply_constraints(axis_range.locked_min(), axis_range.locked_max());
    window.clamp_non_degenerate();
    window
}

fn current_window_x_for_slider(
    engine: &ChartEngine,
    axis: delinea::AxisId,
    extent: DataWindow,
) -> DataWindow {
    if let Some(fixed) = axis_is_fixed(engine, axis) {
        return fixed;
    }

    let zoom_window = engine
        .state()
        .data_zoom_x
        .get(&axis)
        .copied()
        .and_then(|zoom| zoom.window);
    if let Some(window) = zoom_window {
        return window;
    }

    extent
}

fn current_window_y_for_slider(
    engine: &ChartEngine,
    axis: delinea::AxisId,
    extent: DataWindow,
) -> DataWindow {
    if let Some(fixed) = axis_is_fixed(engine, axis) {
        return fixed;
    }

    let window = engine.state().data_window_y.get(&axis).copied();
    if let Some(window) = window {
        return window;
    }

    extent
}

fn data_zoom_x_track_for_axis(layout: &ChartLayout, axis: delinea::AxisId) -> Option<Rect> {
    let plot = layout.plot;
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }

    let band = layout
        .x_axes
        .iter()
        .find(|band| band.axis == axis && band.position == AxisPosition::Bottom)?;

    let h = 9.0f32;
    let pad = 4.0f32;
    let y = band.rect.origin.y.0 + band.rect.size.height.0 - h - pad;
    Some(Rect::new(
        Point::new(plot.origin.x, Px(y)),
        Size::new(plot.size.width, Px(h)),
    ))
}

fn data_zoom_y_track_for_axis(layout: &ChartLayout, axis: delinea::AxisId) -> Option<Rect> {
    let plot = layout.plot;
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }

    let band = layout.y_axes.iter().find(|band| band.axis == axis)?;
    let w = 9.0f32;
    let pad = 4.0f32;
    let x = match band.position {
        AxisPosition::Right => band.rect.origin.x.0 + band.rect.size.width.0 - w - pad,
        _ => band.rect.origin.x.0 + pad,
    };

    Some(Rect::new(
        Point::new(Px(x), plot.origin.y),
        Size::new(Px(w), plot.size.height),
    ))
}

pub(crate) fn data_zoom_tracks_for_engine(
    engine: &mut ChartEngine,
    bounds: Rect,
    style: ChartStyle,
) -> Vec<DataZoomTrackState> {
    let Some(layout) = chart_layout_for_engine(engine, bounds, style) else {
        return Vec::new();
    };
    let Some((x_axis, y_axis)) = primary_axes(engine) else {
        return Vec::new();
    };

    let mut tracks = Vec::new();

    if let Some(track) = data_zoom_x_track_for_axis(&layout, x_axis) {
        let extent = compute_axis_extent_from_data(engine, x_axis, true);
        let mut current_window = current_window_x_for_slider(engine, x_axis, extent);
        current_window.clamp_non_degenerate();
        let interactive = axis_is_fixed(engine, x_axis).is_none()
            && !engine
                .state()
                .axis_locks
                .get(&x_axis)
                .copied()
                .unwrap_or_default()
                .zoom_locked;
        tracks.push(DataZoomTrackState {
            axis_kind: DataZoomAxisKind::X,
            axis: x_axis,
            track,
            extent,
            current_window,
            permissions: slider_permissions_for_axis(engine, x_axis),
            interactive,
        });
    }

    if let Some(track) = data_zoom_y_track_for_axis(&layout, y_axis) {
        let extent = compute_axis_extent_from_data(engine, y_axis, false);
        let mut current_window = current_window_y_for_slider(engine, y_axis, extent);
        current_window.clamp_non_degenerate();
        let interactive = axis_is_fixed(engine, y_axis).is_none()
            && !engine
                .state()
                .axis_locks
                .get(&y_axis)
                .copied()
                .unwrap_or_default()
                .zoom_locked;
        tracks.push(DataZoomTrackState {
            axis_kind: DataZoomAxisKind::Y,
            axis: y_axis,
            track,
            extent,
            current_window,
            permissions: slider_permissions_for_axis(engine, y_axis),
            interactive,
        });
    }

    tracks
}

pub(crate) fn data_zoom_overlay_tool(
    engine: Model<ChartEngine>,
    state: Arc<Mutex<DataZoomOverlayState>>,
    style: ChartStyle,
) -> CanvasToolEntry {
    let down_state = state.clone();
    let on_pointer_down: OnCanvasToolPointerDown =
        Arc::new(move |host, action_cx, _tool_cx, down| {
            let Ok(mut state) = down_state.lock() else {
                return CanvasToolDownResult::unhandled();
            };
            if state.drag.is_some() {
                return CanvasToolDownResult::unhandled();
            }
            if down.pointer_type != fret_core::PointerType::Mouse {
                return CanvasToolDownResult::unhandled();
            }

            let Some(track) = state.track_at(down.position) else {
                return CanvasToolDownResult::unhandled();
            };

            if !track.interactive {
                return CanvasToolDownResult::unhandled();
            }

            let drag_start = match track.axis_kind {
                DataZoomAxisKind::X => slider_drag_start_at_x(
                    track.track,
                    track.extent,
                    track.current_window,
                    down.position.x.0,
                    7.0,
                    track.permissions,
                ),
                DataZoomAxisKind::Y => slider_drag_start_at_y(
                    track.track,
                    track.extent,
                    track.current_window,
                    down.position.y.0,
                    7.0,
                    track.permissions,
                ),
            };
            let Some(start) = drag_start else {
                return CanvasToolDownResult::unhandled();
            };

            state.drag = Some(DataZoomSliderDrag {
                axis_kind: track.axis_kind,
                axis: track.axis,
                kind: start.kind,
                track: track.track,
                extent: track.extent,
                start_pos: down.position,
                start_window: start.start_window,
            });
            host.request_redraw(action_cx.window);
            CanvasToolDownResult {
                handled: true,
                activate: false,
                capture: true,
            }
        });

    let move_state = state.clone();
    let engine_move = engine.clone();
    let on_pointer_move: OnCanvasToolPointerMove =
        Arc::new(move |host, action_cx, _tool_cx, mv| {
            let Ok(mut state) = move_state.lock() else {
                return false;
            };
            let Some(drag) = state.drag else {
                return false;
            };
            if !state.tracks.iter().any(|track| track.axis == drag.axis) {
                state.drag = None;
                return false;
            }
            if !is_button_held(MouseButton::Left, mv.buttons) {
                return false;
            }

            let update = match drag.axis_kind {
                DataZoomAxisKind::X => slider_drag_update_at_x(
                    drag.track,
                    drag.extent,
                    drag.start_window,
                    drag.start_pos.x.0,
                    mv.position.x.0,
                    drag.kind,
                ),
                DataZoomAxisKind::Y => slider_drag_update_at_y(
                    drag.track,
                    drag.extent,
                    drag.start_window,
                    drag.start_pos.y.0,
                    mv.position.y.0,
                    drag.kind,
                ),
            };
            let Some(update) = update else {
                return false;
            };

            let _ = host
                .models_mut()
                .update(&engine_move, |engine| match drag.axis_kind {
                    DataZoomAxisKind::X => {
                        engine.apply_action(Action::SetDataWindowXFromZoom {
                            axis: drag.axis,
                            base: drag.start_window,
                            window: update.window,
                            anchor: update.anchor,
                        });
                    }
                    DataZoomAxisKind::Y => {
                        engine.apply_action(Action::SetDataWindowYFromZoom {
                            axis: drag.axis,
                            base: drag.start_window,
                            window: update.window,
                            anchor: update.anchor,
                        });
                    }
                });

            if let Some(track_state) = state
                .tracks
                .iter_mut()
                .find(|track| track.axis == drag.axis)
            {
                track_state.current_window = update.window;
            }
            state.drag = Some(DataZoomSliderDrag {
                start_pos: mv.position,
                start_window: update.window,
                ..drag
            });
            host.request_redraw(action_cx.window);
            true
        });

    let up_state = state.clone();
    let on_pointer_up: OnCanvasToolPointerUp = Arc::new(move |host, action_cx, _tool_cx, up| {
        let Ok(mut state) = up_state.lock() else {
            return false;
        };
        let Some(_drag) = state.drag else {
            return false;
        };
        if up.button != MouseButton::Left {
            return false;
        }

        state.drag = None;
        host.release_pointer_capture();
        host.request_redraw(action_cx.window);
        true
    });

    let paint_state = state.clone();
    let on_paint = Arc::new(
        move |painter: &mut CanvasPainter<'_>, _paint_cx: PanZoomCanvasPaintCx| {
            let Ok(state) = paint_state.lock() else {
                return;
            };

            if state.tracks.is_empty() {
                return;
            }

            let bounds = painter.bounds();
            if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
                return;
            }

            for track in &state.tracks {
                let order = DrawOrder(style.draw_order.0.saturating_add(8_650));
                let track_color = fret_core::Color {
                    a: 0.18,
                    ..style.axis_line_color
                };
                painter.scene().push(fret_core::SceneOp::Quad {
                    order,
                    rect: track.track,
                    background: fret_core::Paint::Solid(track_color).into(),
                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),
                    corner_radii: Corners::all(Px(4.0)),
                });

                match track.axis_kind {
                    DataZoomAxisKind::X => {
                        let t0 = slider_norm(track.extent, track.current_window.min);
                        let t1 = slider_norm(track.extent, track.current_window.max);
                        let left = track.track.origin.x.0 + t0 * track.track.size.width.0;
                        let right = track.track.origin.x.0 + t1 * track.track.size.width.0;

                        let win_rect = Rect::new(
                            Point::new(Px(left.min(right)), track.track.origin.y),
                            Size::new(Px((right - left).abs().max(1.0)), track.track.size.height),
                        );
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(order.0.saturating_add(1)),
                            rect: win_rect,
                            background: fret_core::Paint::Solid(style.selection_fill).into(),
                            border: Edges::all(style.selection_stroke_width),
                            border_paint: fret_core::Paint::Solid(style.selection_stroke).into(),
                            corner_radii: Corners::all(Px(4.0)),
                        });

                        let handle_w = 2.0f32.max(style.selection_stroke_width.0);
                        let handle_color = style.selection_stroke;
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(order.0.saturating_add(2)),
                            rect: Rect::new(
                                Point::new(Px(left - 0.5 * handle_w), track.track.origin.y),
                                Size::new(Px(handle_w), track.track.size.height),
                            ),
                            background: fret_core::Paint::Solid(handle_color).into(),
                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT.into(),
                            corner_radii: Corners::all(Px(0.0)),
                        });
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(order.0.saturating_add(3)),
                            rect: Rect::new(
                                Point::new(Px(right - 0.5 * handle_w), track.track.origin.y),
                                Size::new(Px(handle_w), track.track.size.height),
                            ),
                            background: fret_core::Paint::Solid(handle_color).into(),
                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT.into(),
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }
                    DataZoomAxisKind::Y => {
                        let t0 = slider_norm(track.extent, track.current_window.min);
                        let t1 = slider_norm(track.extent, track.current_window.max);

                        let height = track.track.size.height.0;
                        let bottom = track.track.origin.y.0 + height;
                        let y0 = bottom - t0 * height;
                        let y1 = bottom - t1 * height;

                        let top = y0.min(y1);
                        let bottom = y0.max(y1);
                        let win_rect = Rect::new(
                            Point::new(track.track.origin.x, Px(top)),
                            Size::new(track.track.size.width, Px((bottom - top).abs().max(1.0))),
                        );
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(order.0.saturating_add(1)),
                            rect: win_rect,
                            background: fret_core::Paint::Solid(style.selection_fill).into(),
                            border: Edges::all(style.selection_stroke_width),
                            border_paint: fret_core::Paint::Solid(style.selection_stroke).into(),
                            corner_radii: Corners::all(Px(4.0)),
                        });

                        let handle_h = 2.0f32.max(style.selection_stroke_width.0);
                        let handle_color = style.selection_stroke;
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(order.0.saturating_add(2)),
                            rect: Rect::new(
                                Point::new(track.track.origin.x, Px(y0 - 0.5 * handle_h)),
                                Size::new(track.track.size.width, Px(handle_h)),
                            ),
                            background: fret_core::Paint::Solid(handle_color).into(),
                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT.into(),
                            corner_radii: Corners::all(Px(0.0)),
                        });
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(order.0.saturating_add(3)),
                            rect: Rect::new(
                                Point::new(track.track.origin.x, Px(y1 - 0.5 * handle_h)),
                                Size::new(track.track.size.width, Px(handle_h)),
                            ),
                            background: fret_core::Paint::Solid(handle_color).into(),
                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT.into(),
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }
                }
            }
        },
    );

    CanvasToolEntry {
        id: CanvasToolId::new(13),
        priority: 170,
        handlers: CanvasToolHandlers {
            on_pointer_down: Some(on_pointer_down),
            on_pointer_move: Some(on_pointer_move),
            on_pointer_up: Some(on_pointer_up),
            on_paint: Some(on_paint),
            ..Default::default()
        },
    }
}

fn is_button_held(button: MouseButton, buttons: fret_core::MouseButtons) -> bool {
    match button {
        MouseButton::Left => buttons.left,
        MouseButton::Right => buttons.right,
        MouseButton::Middle => buttons.middle,
        _ => false,
    }
}
