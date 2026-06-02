use std::cell::{Cell, RefCell};
use std::rc::Rc;

use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, MouseButton, Paint, PathStyle, Point, Px, Rect, Size,
    StrokeStyle,
};
use fret_runtime::Model;
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, CanvasProps, Length, ManagedSurfaceProps};
use fret_ui::{ElementContext, UiHost};

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform, polyline_commands};
use crate::input_map::{ModifierKey, ModifiersMask, PlotInputMap};
use crate::models::{StepMode, YAxis};
use crate::plot::axis::{
    AxisLabelFormatter, AxisTicks, axis_ticks_scaled, log10_tick_label_or_empty,
};
use crate::plot::view::{
    clamp_view_to_data_scaled, clamp_zoom_factors, data_rect_from_plot_points_scaled,
    local_from_absolute, sanitize_data_rect_scaled, zoom_view_at_px_scaled,
};
use crate::series::SeriesId;
use crate::state::{
    PlotDragOutput, PlotDragPhase, PlotImageLayer, PlotOutput, PlotOutputSnapshot, PlotOverlays,
    PlotState,
};
use crate::style::LinePlotStyle;

mod axis_labels;
mod commands;
mod heatmap;
mod legend;
mod model;
mod overlays;
mod panels;
mod props;
mod readout;
mod selection;

use axis_labels::{paint_line_plot_axis_tick_labels, paint_line_plot_right_axis_tick_labels};
use commands::{
    area_fill_commands_from_polyline, bars_commands_from_series, candlestick_commands_from_series,
    error_bars_commands_from_series, histogram_commands_from_series, line_plot_area_fill_path_key,
    line_plot_candlestick_down_path_key, line_plot_series_path_key,
    line_plot_shaded_lower_path_key, shaded_band_commands_from_series, stems_commands_from_points,
    step_commands_from_polyline,
};
use heatmap::{paint_line_plot_heatmap, paint_line_plot_heatmap_colorbar};
use legend::{LinePlotLegendHit, line_plot_legend_hit, paint_line_plot_legend};
use model::PlotPanelModel;
use overlays::{
    paint_line_plot_draggable_overlay_labels, paint_line_plot_draggable_shapes,
    paint_line_plot_images, paint_line_plot_reference_lines, paint_line_plot_tag_overlays,
    paint_line_plot_text_overlays,
};
use readout::{paint_line_plot_cursor_readout, paint_line_plot_linked_cursor_readout};
use selection::{
    paint_line_plot_active_selection, paint_line_plot_query_selection,
    paint_line_plot_selection_tooltip,
};

pub use panels::{
    area_plot_panel, area_plot_panel_in, bars_plot_panel, bars_plot_panel_in,
    candlestick_plot_panel, candlestick_plot_panel_in, error_bars_plot_panel,
    error_bars_plot_panel_in, heatmap_plot_panel, heatmap_plot_panel_in, histogram_plot_panel,
    histogram_plot_panel_in, histogram2d_plot_panel, histogram2d_plot_panel_in, line_plot_panel,
    line_plot_panel_in, shaded_plot_panel, shaded_plot_panel_in, stems_plot_panel,
    stems_plot_panel_in,
};
pub use props::{
    AreaPlotPanelProps, BarsPlotPanelProps, CandlestickPlotPanelProps, ErrorBarsPlotPanelProps,
    HeatmapPlotPanelProps, Histogram2DPlotPanelProps, HistogramPlotPanelProps, LinePlotPanelProps,
    ShadedPlotPanelProps, StemsPlotPanelProps,
};

#[derive(Clone)]
struct PlotPanelProps {
    canvas: CanvasProps,
    model: PlotPanelModel,
    state: Option<Model<PlotState>>,
    output: Option<Model<PlotOutput>>,
    style: LinePlotStyle,
    x_axis_labels: Option<AxisLabelFormatter>,
    y_axis_labels: Option<AxisLabelFormatter>,
    y2_axis_labels: Option<AxisLabelFormatter>,
    y3_axis_labels: Option<AxisLabelFormatter>,
    y4_axis_labels: Option<AxisLabelFormatter>,
    x_scale: AxisScale,
    y_scale: AxisScale,
    step_mode: Option<StepMode>,
}

#[track_caller]
fn plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    mut props: PlotPanelProps,
) -> AnyElement {
    props.canvas.layout.size.width = Length::Fill;
    props.canvas.layout.size.height = Length::Fill;
    if let Some(state) = &props.state {
        cx.observe_model(state, fret_ui::Invalidation::Paint);
    }
    let model = props.model;
    let step_mode = props.step_mode;
    let output_snapshot = props.output.as_ref().and_then(|output| {
        cx.read_model_ref(output, fret_ui::Invalidation::Paint, |output| {
            output.snapshot
        })
        .ok()
    });
    let output_snapshot = Rc::new(Cell::new(output_snapshot));
    let linked_cursor_x = Rc::new(Cell::new(None::<f64>));
    let pinned_series = Rc::new(Cell::new(None::<SeriesId>));
    let legend_hover = Rc::new(Cell::new(None::<SeriesId>));
    let hidden_series = Rc::new(RefCell::new(Vec::<SeriesId>::new()));
    let pan_session = Rc::new(RefCell::new(None::<LinePlotPanSession>));
    let box_zoom_session = Rc::new(RefCell::new(None::<LinePlotBoxZoomSession>));
    let query_drag_session = Rc::new(RefCell::new(None::<LinePlotQueryDragSession>));
    let drag_session = Rc::new(RefCell::new(None::<LinePlotDragSession>));
    let active_selection = Rc::new(Cell::new(None::<LinePlotSelectionOverlay>));
    let query_selection = Rc::new(Cell::new(None::<DataRect>));
    let overlays = Rc::new(RefCell::new(PlotOverlays::default()));
    let style = props.style;
    let x_axis_labels = props.x_axis_labels.unwrap_or_default();
    let y_axis_labels = props.y_axis_labels.unwrap_or_default();
    let y2_axis_labels = props.y2_axis_labels.unwrap_or_default();
    let y3_axis_labels = props.y3_axis_labels.unwrap_or_default();
    let y4_axis_labels = props.y4_axis_labels.unwrap_or_default();
    let x_scale = props.x_scale;
    let y_scale = props.y_scale;
    let view_bounds = Rc::new(Cell::new(line_plot_view_bounds_from_state(
        &model, None, style, x_scale, y_scale,
    )));
    let state = props.state.clone();
    let event_state = props.state.clone();
    let output = props.output.clone();
    let event_model = model.clone();
    let event_output = output.clone();
    let event_output_snapshot = output_snapshot.clone();
    let event_style = style;
    let event_x_scale = x_scale;
    let event_y_scale = y_scale;
    let event_legend_hover = legend_hover.clone();
    let event_view_bounds = view_bounds.clone();
    let event_pan_session = pan_session.clone();
    let event_box_zoom_session = box_zoom_session.clone();
    let event_query_drag_session = query_drag_session.clone();
    let event_drag_session = drag_session.clone();
    let event_active_selection = active_selection.clone();

    let mut surface = ManagedSurfaceProps::default();
    surface.layout = props.canvas.layout;
    let canvas = props.canvas;
    let element = cx.managed_surface(
        surface,
        |cx| {
            cx.layout_unplaced_children(cx.bounds());
            cx.set_hit_test_rects([cx.bounds()]);
        },
        {
            let linked_cursor_x = linked_cursor_x.clone();
            let pinned_series = pinned_series.clone();
            let view_bounds = view_bounds.clone();
            let hidden_series = hidden_series.clone();
            let query_selection = query_selection.clone();
            let overlays = overlays.clone();
            let state = state.clone();
            let model = model.clone();
            move |cx| {
                if let Some(state) = state.as_ref() {
                    let (linked_x, pinned, hidden, query, next_overlays, next_view_bounds) = state
                        .read_ref(cx.app(), |state| {
                            (
                                state.linked_cursor_x.filter(|x| x.is_finite()),
                                state
                                    .pinned_series
                                    .filter(|id| !state.hidden_series.contains(id)),
                                state.hidden_series.iter().copied().collect::<Vec<_>>(),
                                state.query,
                                state.overlays.clone(),
                                line_plot_view_bounds_from_state(
                                    &model,
                                    Some(state),
                                    style,
                                    x_scale,
                                    y_scale,
                                ),
                            )
                        })
                        .unwrap_or_else(|_| {
                            (
                                None,
                                None,
                                Vec::new(),
                                None,
                                PlotOverlays::default(),
                                line_plot_view_bounds_from_state(
                                    &model, None, style, x_scale, y_scale,
                                ),
                            )
                        });
                    linked_cursor_x.set(linked_x);
                    pinned_series.set(pinned);
                    query_selection.set(query);
                    view_bounds.set(next_view_bounds);
                    hidden_series.replace(hidden);
                    overlays.replace(next_overlays);
                } else {
                    linked_cursor_x.set(None);
                    pinned_series.set(None);
                    query_selection.set(None);
                    view_bounds.set(line_plot_view_bounds_from_state(
                        &model, None, style, x_scale, y_scale,
                    ));
                    hidden_series.replace(Vec::new());
                    overlays.replace(PlotOverlays::default());
                }

                let bounds = cx.bounds();
                for child in cx.children().to_vec() {
                    cx.paint_child(child, bounds);
                }
            }
        },
        move |cx| {
            let model = model.clone();
            let output_snapshot = output_snapshot.clone();
            let linked_cursor_x = linked_cursor_x.clone();
            let pinned_series = pinned_series.clone();
            let legend_hover = legend_hover.clone();
            let view_bounds = view_bounds.clone();
            let hidden_series = hidden_series.clone();
            let active_selection = active_selection.clone();
            let query_selection = query_selection.clone();
            let overlays = overlays.clone();
            vec![cx.canvas(canvas, move |painter| {
                let hidden_series = hidden_series.borrow();
                let overlays = overlays.borrow();
                paint_line_plot_panel(
                    painter,
                    &model,
                    output_snapshot.get(),
                    linked_cursor_x.get(),
                    pinned_series.get(),
                    legend_hover.get(),
                    view_bounds.get(),
                    query_selection.get(),
                    active_selection.get(),
                    &overlays,
                    &hidden_series,
                    step_mode,
                    style,
                    &x_axis_labels,
                    &y_axis_labels,
                    &y2_axis_labels,
                    &y3_axis_labels,
                    &y4_axis_labels,
                    x_scale,
                    y_scale,
                );
            })]
        },
    );
    let surface_id = element.id;
    cx.managed_surface_on_event_for(surface_id, move |cx, event| {
        let bounds = cx.bounds();
        if let Some(state) = event_state.as_ref()
            && handle_line_plot_wheel_zoom_event(
                cx.app(),
                state,
                event,
                bounds,
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            )
        {
            let current_view_bounds = line_plot_current_view_bounds_for_event(
                cx.app(),
                event_state.as_ref(),
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            );
            event_view_bounds.set(current_view_bounds);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if let Some(state) = event_state.as_ref()
            && let Some(drag_output) = handle_line_plot_draggable_overlay_event(
                cx.app(),
                state,
                &event_drag_session,
                event,
                bounds,
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            )
        {
            let current_view_bounds = line_plot_current_view_bounds_for_event(
                cx.app(),
                event_state.as_ref(),
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            );
            event_view_bounds.set(current_view_bounds);
            let snapshot = line_plot_output_snapshot_with_drag(
                current_view_bounds,
                None,
                line_plot_query_from_state(cx.app(), event_state.as_ref()),
                Some(drag_output),
            );
            let visual_changed = event_output_snapshot.get() != Some(snapshot);
            event_output_snapshot.set(Some(snapshot));
            let output_changed =
                publish_line_plot_panel_output(cx.app(), event_output.as_ref(), snapshot);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            if visual_changed || output_changed {
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }

        if let Some(state) = event_state.as_ref()
            && handle_line_plot_query_drag_event(
                cx.app(),
                state,
                &event_query_drag_session,
                &event_active_selection,
                event,
                bounds,
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            )
        {
            let current_view_bounds = line_plot_current_view_bounds_for_event(
                cx.app(),
                event_state.as_ref(),
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            );
            event_view_bounds.set(current_view_bounds);
            let snapshot = line_plot_output_snapshot(
                current_view_bounds,
                None,
                line_plot_query_from_state(cx.app(), event_state.as_ref()),
            );
            let visual_changed = event_output_snapshot.get() != Some(snapshot);
            event_output_snapshot.set(Some(snapshot));
            let output_changed =
                publish_line_plot_panel_output(cx.app(), event_output.as_ref(), snapshot);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            if visual_changed || output_changed {
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }

        if let Some(state) = event_state.as_ref()
            && handle_line_plot_box_zoom_event(
                cx.app(),
                state,
                &event_box_zoom_session,
                &event_active_selection,
                event,
                bounds,
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            )
        {
            let current_view_bounds = line_plot_current_view_bounds_for_event(
                cx.app(),
                event_state.as_ref(),
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            );
            event_view_bounds.set(current_view_bounds);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if let Some(state) = event_state.as_ref()
            && handle_line_plot_pan_event(
                cx.app(),
                state,
                &event_pan_session,
                event,
                bounds,
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            )
        {
            let current_view_bounds = line_plot_current_view_bounds_for_event(
                cx.app(),
                event_state.as_ref(),
                &event_model,
                event_style,
                event_x_scale,
                event_y_scale,
            );
            event_view_bounds.set(current_view_bounds);
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if let Some(state) = event_state.as_ref()
            && handle_line_plot_legend_pointer_event(
                cx.app(),
                state,
                event,
                bounds,
                &event_model,
                event_style,
            )
        {
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if let Some(hovered) =
            line_plot_legend_hover_from_event(event, bounds, &event_model, event_style)
        {
            let changed = event_legend_hover.get() != hovered;
            event_legend_hover.set(hovered);
            if changed {
                cx.invalidate_self(fret_ui::Invalidation::Paint);
                cx.request_redraw();
                cx.notify();
            }
            if hovered.is_some() {
                cx.stop_propagation();
                return;
            }
        }

        let current_view_bounds = line_plot_current_view_bounds_for_event(
            cx.app(),
            event_state.as_ref(),
            &event_model,
            event_style,
            event_x_scale,
            event_y_scale,
        );
        event_view_bounds.set(current_view_bounds);

        let Some(snapshot) = line_plot_panel_event_snapshot(
            event,
            bounds,
            &event_model,
            event_style,
            event_x_scale,
            event_y_scale,
            current_view_bounds,
            line_plot_query_from_state(cx.app(), event_state.as_ref()),
        ) else {
            return;
        };
        let visual_changed = event_output_snapshot.get() != Some(snapshot);
        event_output_snapshot.set(Some(snapshot));
        let output_changed =
            publish_line_plot_panel_output(cx.app(), event_output.as_ref(), snapshot);
        if visual_changed || output_changed {
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
        }
    });
    element
}

#[derive(Debug, Clone, Copy)]
struct LinePlotPanSession {
    last_position: Point,
}

#[derive(Debug, Clone, Copy)]
struct LinePlotBoxZoomSession {
    start: Point,
    current: Point,
    button: MouseButton,
    required_mods: ModifiersMask,
}

#[derive(Debug, Clone, Copy)]
struct LinePlotQueryDragSession {
    start: Point,
    current: Point,
    button: MouseButton,
}

#[derive(Debug, Clone, Copy)]
enum LinePlotDragSession {
    LineX {
        id: u64,
        button: MouseButton,
        offset_x: f64,
        current_x: f64,
    },
    LineY {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        offset_y: f64,
        current_y: f64,
    },
    Point {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        offset: DataPoint,
        current: DataPoint,
    },
    Rect {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        handle: LinePlotDragRectHandle,
        offset: DataPoint,
        start: DataRect,
        current: DataRect,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinePlotDragRectHandle {
    Inside,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinePlotSelectionKind {
    Query,
    BoxZoom,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct LinePlotSelectionOverlay {
    start: Point,
    current: Point,
    kind: LinePlotSelectionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinePlotWheelRegion {
    Plot,
    XAxis,
    YAxis,
}

fn paint_line_plot_panel(
    painter: &mut CanvasPainter<'_>,
    model: &PlotPanelModel,
    output: Option<PlotOutputSnapshot>,
    linked_cursor_x: Option<f64>,
    pinned_series: Option<SeriesId>,
    legend_hover: Option<SeriesId>,
    view_bounds: DataRect,
    query_selection: Option<DataRect>,
    active_selection: Option<LinePlotSelectionOverlay>,
    overlays: &PlotOverlays,
    hidden_series: &[SeriesId],
    step_mode: Option<StepMode>,
    style: LinePlotStyle,
    x_axis_labels: &AxisLabelFormatter,
    y_axis_labels: &AxisLabelFormatter,
    y2_axis_labels: &AxisLabelFormatter,
    y3_axis_labels: &AxisLabelFormatter,
    y4_axis_labels: &AxisLabelFormatter,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    let bounds = painter.bounds();
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let background = style
        .background
        .unwrap_or_else(|| painter.theme().snapshot().color_required("surface"));
    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: Paint::Solid(background).into(),
        border: if style.border.is_some() {
            Edges::all(style.border_width)
        } else {
            Edges::default()
        },
        border_paint: Paint::Solid(style.border.unwrap_or(Color::TRANSPARENT)).into(),
        corner_radii: Corners::default(),
    });

    let transform = PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    };
    paint_line_plot_images(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        PlotImageLayer::BelowGrid,
        x_scale,
        y_scale,
    );
    paint_line_plot_grid_and_axes(painter, transform, style, &x_axis_labels, &y_axis_labels);
    if let Some(heatmap) = &model.heatmap {
        paint_line_plot_heatmap(painter, transform, heatmap, style);
        paint_line_plot_heatmap_colorbar(painter, plot, heatmap, style);
    }
    paint_line_plot_right_axis_tick_labels(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        style,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        x_scale,
        y_scale,
    );
    paint_line_plot_images(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        PlotImageLayer::AboveGrid,
        x_scale,
        y_scale,
    );
    paint_line_plot_reference_lines(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_draggable_shapes(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_draggable_overlay_labels(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_tag_overlays(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_text_overlays(
        painter,
        plot,
        view_bounds,
        model.data_bounds_y2,
        model.data_bounds_y3,
        model.data_bounds_y4,
        overlays,
        style,
        x_scale,
        y_scale,
    );

    let series_count = model.series.len();
    let raster_scale_factor = painter.scale_factor();
    let emphasized_series = if style.emphasize_hovered_series {
        pinned_series.or(legend_hover)
    } else {
        None
    };
    let right_transform = model.data_bounds_y2.map(|axis_bounds| PlotTransform {
        viewport: plot,
        data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
        x_scale,
        y_scale,
    });
    let right2_transform = model.data_bounds_y3.map(|axis_bounds| PlotTransform {
        viewport: plot,
        data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
        x_scale,
        y_scale,
    });
    let right3_transform = model.data_bounds_y4.map(|axis_bounds| PlotTransform {
        viewport: plot,
        data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
        x_scale,
        y_scale,
    });

    for (index, series) in model.series.iter().enumerate() {
        if hidden_series.contains(&series.id) {
            continue;
        }
        let series_transform = match series.y_axis {
            YAxis::Left => transform,
            YAxis::Right => right_transform.unwrap_or(transform),
            YAxis::Right2 => right2_transform.unwrap_or(transform),
            YAxis::Right3 => right3_transform.unwrap_or(transform),
        };
        if let Some(candlestick) = &series.candlestick {
            let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
            let (wick_commands, up_body_commands, down_body_commands) =
                candlestick_commands_from_series(
                    series_transform,
                    candlestick,
                    stroke_width,
                    raster_scale_factor,
                );
            if wick_commands.is_empty()
                && up_body_commands.is_empty()
                && down_body_commands.is_empty()
            {
                continue;
            }

            let mut wick_color = candlestick
                .wick_color
                .or(series.stroke_color)
                .unwrap_or_else(|| series_color(style, index, series_count));
            let mut up_fill = candlestick.up_fill.unwrap_or(Color {
                r: 0.25,
                g: 0.80,
                b: 0.45,
                a: 0.85,
            });
            let mut down_fill = candlestick.down_fill.unwrap_or(Color {
                r: 0.90,
                g: 0.35,
                b: 0.45,
                a: 0.85,
            });
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                let dim = style.dimmed_series_alpha.clamp(0.0, 1.0);
                wick_color.a *= dim;
                up_fill.a *= dim;
                down_fill.a *= dim;
            }

            if !up_body_commands.is_empty() {
                painter.path(
                    line_plot_area_fill_path_key(series.id.0),
                    DrawOrder(19),
                    Point::new(Px(0.0), Px(0.0)),
                    &up_body_commands,
                    PathStyle::Fill(fret_core::FillStyle::default()),
                    up_fill,
                    raster_scale_factor,
                );
            }
            if !down_body_commands.is_empty() {
                painter.path(
                    line_plot_candlestick_down_path_key(series.id.0),
                    DrawOrder(19),
                    Point::new(Px(0.0), Px(0.0)),
                    &down_body_commands,
                    PathStyle::Fill(fret_core::FillStyle::default()),
                    down_fill,
                    raster_scale_factor,
                );
            }
            if wick_commands.len() >= 2 {
                painter.path(
                    line_plot_series_path_key(series.id.0),
                    DrawOrder(20),
                    Point::new(Px(0.0), Px(0.0)),
                    &wick_commands,
                    PathStyle::Stroke(StrokeStyle {
                        width: stroke_width,
                    }),
                    wick_color,
                    raster_scale_factor,
                );
            }
            continue;
        }
        if let Some(bars) = &series.bars {
            let commands = bars_commands_from_series(series_transform, &*series.data, bars);
            if commands.is_empty() {
                continue;
            }

            let mut fill_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
            }
            painter.path(
                line_plot_area_fill_path_key(series.id.0),
                DrawOrder(19),
                Point::new(Px(0.0), Px(0.0)),
                &commands,
                PathStyle::Fill(fret_core::FillStyle::default()),
                fill_color,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(histogram) = &series.histogram {
            let commands =
                histogram_commands_from_series(series_transform, &*series.data, histogram);
            if commands.is_empty() {
                continue;
            }

            let mut fill_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
            }
            painter.path(
                line_plot_area_fill_path_key(series.id.0),
                DrawOrder(19),
                Point::new(Px(0.0), Px(0.0)),
                &commands,
                PathStyle::Fill(fret_core::FillStyle::default()),
                fill_color,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(error_bars) = &series.error_bars {
            let commands =
                error_bars_commands_from_series(series_transform, &*series.data, error_bars);
            if commands.len() < 2 {
                continue;
            }

            let mut stroke_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                stroke_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
            }
            let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
            painter.path(
                line_plot_series_path_key(series.id.0),
                DrawOrder(20),
                Point::new(Px(0.0), Px(0.0)),
                &commands,
                PathStyle::Stroke(StrokeStyle {
                    width: stroke_width,
                }),
                stroke_color,
                raster_scale_factor,
            );
            continue;
        }
        if let Some(lower_data) = &series.lower_data {
            let (fill_commands, upper_commands, lower_commands) =
                shaded_band_commands_from_series(series_transform, &*series.data, &**lower_data);

            let mut stroke_color = series
                .stroke_color
                .unwrap_or_else(|| series_color(style, index, series_count));
            if let Some(emphasized) = emphasized_series
                && series.id != emphasized
            {
                stroke_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
            }

            if let Some(fill) = series.fill
                && !fill_commands.is_empty()
            {
                let mut fill_color = fill.color.unwrap_or_else(|| {
                    series
                        .stroke_color
                        .unwrap_or_else(|| series_color(style, index, series_count))
                });
                fill_color.a = (fill_color.a * fill.alpha.clamp(0.0, 1.0)).clamp(0.0, 1.0);
                if let Some(emphasized) = emphasized_series
                    && series.id != emphasized
                {
                    fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
                }
                painter.path(
                    line_plot_area_fill_path_key(series.id.0),
                    DrawOrder(19),
                    Point::new(Px(0.0), Px(0.0)),
                    &fill_commands,
                    PathStyle::Fill(fret_core::FillStyle::default()),
                    fill_color,
                    raster_scale_factor,
                );
            }

            let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
            if upper_commands.len() >= 2 {
                painter.path(
                    line_plot_series_path_key(series.id.0),
                    DrawOrder(20),
                    Point::new(Px(0.0), Px(0.0)),
                    &upper_commands,
                    PathStyle::Stroke(StrokeStyle {
                        width: stroke_width,
                    }),
                    stroke_color,
                    raster_scale_factor,
                );
            }
            if lower_commands.len() >= 2 {
                painter.path(
                    line_plot_shaded_lower_path_key(series.id.0),
                    DrawOrder(20),
                    Point::new(Px(0.0), Px(0.0)),
                    &lower_commands,
                    PathStyle::Stroke(StrokeStyle {
                        width: stroke_width,
                    }),
                    stroke_color,
                    raster_scale_factor,
                );
            }
            continue;
        }
        let Some(points) = series.data.as_slice() else {
            continue;
        };
        let commands = if let Some(baseline) = series.stem_baseline {
            stems_commands_from_points(series_transform, points, baseline)
        } else {
            let commands = polyline_commands(series_transform, points);
            if let Some(step_mode) = step_mode {
                step_commands_from_polyline(&commands, step_mode)
            } else {
                commands
            }
        };
        if commands.len() < 2 {
            continue;
        }

        let mut stroke_color = series
            .stroke_color
            .unwrap_or_else(|| series_color(style, index, series_count));
        if let Some(emphasized) = emphasized_series
            && series.id != emphasized
        {
            stroke_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
        }
        if let Some(fill) = series.fill
            && let Some(baseline_y) = series_transform.data_y_to_px(f64::from(fill.baseline))
        {
            let fill_commands = area_fill_commands_from_polyline(&commands, baseline_y);
            if !fill_commands.is_empty() {
                let mut fill_color = fill.color.unwrap_or_else(|| {
                    series
                        .stroke_color
                        .unwrap_or_else(|| series_color(style, index, series_count))
                });
                fill_color.a = (fill_color.a * fill.alpha.clamp(0.0, 1.0)).clamp(0.0, 1.0);
                if let Some(emphasized) = emphasized_series
                    && series.id != emphasized
                {
                    fill_color.a *= style.dimmed_series_alpha.clamp(0.0, 1.0);
                }
                painter.path(
                    line_plot_area_fill_path_key(series.id.0),
                    DrawOrder(19),
                    Point::new(Px(0.0), Px(0.0)),
                    &fill_commands,
                    PathStyle::Fill(fret_core::FillStyle::default()),
                    fill_color,
                    raster_scale_factor,
                );
            }
        }
        let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
        painter.path(
            line_plot_series_path_key(series.id.0),
            DrawOrder(20),
            Point::new(Px(0.0), Px(0.0)),
            &commands,
            PathStyle::Stroke(StrokeStyle {
                width: stroke_width,
            }),
            stroke_color,
            raster_scale_factor,
        );
    }

    paint_line_plot_legend(painter, model, plot, pinned_series, legend_hover, style);
    paint_line_plot_query_selection(
        painter,
        plot,
        view_bounds,
        query_selection,
        style,
        x_scale,
        y_scale,
    );
    paint_line_plot_active_selection(painter, plot, active_selection, style);
    if paint_line_plot_selection_tooltip(
        painter,
        bounds,
        plot,
        view_bounds,
        active_selection,
        style,
        x_scale,
        y_scale,
    ) {
        return;
    }
    paint_line_plot_cursor_readout(
        painter,
        model,
        plot,
        output,
        pinned_series,
        hidden_series,
        style,
        y_axis_labels,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        x_scale,
        y_scale,
    );
    paint_line_plot_linked_cursor_readout(
        painter,
        model,
        plot,
        transform.data,
        output.and_then(|snapshot| snapshot.cursor),
        linked_cursor_x,
        pinned_series,
        hidden_series,
        style,
        y_axis_labels,
        y2_axis_labels,
        y3_axis_labels,
        y4_axis_labels,
        x_scale,
        y_scale,
    );
}

fn line_plot_view_bounds_for_y_axis(primary: DataRect, axis_bounds: DataRect) -> DataRect {
    DataRect {
        x_min: primary.x_min,
        x_max: primary.x_max,
        y_min: axis_bounds.y_min,
        y_max: axis_bounds.y_max,
    }
}

fn handle_line_plot_legend_pointer_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
) -> bool {
    let Event::Pointer(fret_core::PointerEvent::Down {
        position,
        button: MouseButton::Left,
        modifiers,
        ..
    }) = event
    else {
        return false;
    };

    let plot = line_plot_inner_rect(bounds, style);
    let Some((series_id, hit)) = line_plot_legend_hit(model, plot, *position) else {
        return false;
    };

    state
        .update(app, |state, _cx| match hit {
            _ if modifiers.shift => {
                let ids: Vec<SeriesId> = model.series.iter().map(|series| series.id).collect();
                let visible_count = ids
                    .iter()
                    .filter(|series_id| !state.hidden_series.contains(series_id))
                    .count();
                let is_solo = visible_count == 1 && !state.hidden_series.contains(&series_id);
                if is_solo {
                    state.hidden_series.clear();
                } else {
                    state.hidden_series = ids.into_iter().filter(|id| *id != series_id).collect();
                    state.hidden_series.remove(&series_id);
                }
                true
            }
            LinePlotLegendHit::Swatch => {
                let total = model.series.len();
                let hidden_count = model
                    .series
                    .iter()
                    .filter(|series| state.hidden_series.contains(&series.id))
                    .count();
                let visible_count = total.saturating_sub(hidden_count);
                if state.hidden_series.contains(&series_id) {
                    state.hidden_series.remove(&series_id);
                    state.pinned_series = state.pinned_series.filter(|id| *id != series_id);
                    true
                } else if visible_count <= 1 {
                    false
                } else {
                    state.hidden_series.insert(series_id);
                    state.pinned_series = state.pinned_series.filter(|id| *id != series_id);
                    true
                }
            }
            LinePlotLegendHit::Label => {
                if state.pinned_series == Some(series_id) {
                    state.pinned_series = None;
                } else {
                    state.pinned_series = Some(series_id);
                    state.hidden_series.remove(&series_id);
                }
                true
            }
        })
        .ok()
        .unwrap_or(false)
}

fn line_plot_panel_event_snapshot(
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
    view_bounds: DataRect,
    query: Option<DataRect>,
) -> Option<PlotOutputSnapshot> {
    let Event::Pointer(fret_core::PointerEvent::Move { position, .. }) = event else {
        return None;
    };
    Some(line_plot_pointer_output_snapshot(
        *position,
        bounds,
        model,
        style,
        x_scale,
        y_scale,
        view_bounds,
        query,
    ))
}

#[allow(clippy::too_many_arguments)]
fn handle_line_plot_draggable_overlay_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    drag_session: &Rc<RefCell<Option<LinePlotDragSession>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<PlotDragOutput> {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }

    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            ..
        }) if plot.contains(*position)
            && PlotInputMap::default().pan.matches(*button, *modifiers)
            && line_plot_legend_hit(model, plot, *position).is_none() =>
        {
            let view_bounds = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let overlays = state
                .read_ref(app, |state| state.overlays.clone())
                .unwrap_or_default();
            let local = local_from_absolute(plot.origin, *position);
            let threshold = style.hover_threshold.0.max(1.0);
            let mut best: Option<(f32, LinePlotDragSession)> = None;

            for point in &overlays.drag_points {
                if !point.point.x.is_finite() || !point.point.y.is_finite() {
                    continue;
                }
                let Some(transform) = line_plot_transform_for_y_axis(
                    plot,
                    view_bounds,
                    model,
                    point.axis,
                    x_scale,
                    y_scale,
                ) else {
                    continue;
                };
                let p_px = transform.data_to_px(point.point);
                if !p_px.x.0.is_finite() || !p_px.y.0.is_finite() {
                    continue;
                }
                let hit_r = point.radius.0.max(threshold);
                let dx = local.x.0 - p_px.x.0;
                let dy = local.y.0 - p_px.y.0;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > hit_r {
                    continue;
                }
                let data = transform.px_to_data(local);
                if !data.x.is_finite() || !data.y.is_finite() {
                    continue;
                };
                let candidate = LinePlotDragSession::Point {
                    id: point.id,
                    axis: point.axis,
                    button: *button,
                    offset: DataPoint {
                        x: data.x - point.point.x,
                        y: data.y - point.point.y,
                    },
                    current: point.point,
                };
                if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                    best = Some((dist, candidate));
                }
            }

            if best.is_none() {
                let transform_x =
                    line_plot_transform_for_x_axis(plot, view_bounds, x_scale, y_scale);
                for line in &overlays.drag_lines_x {
                    if !line.x.is_finite() {
                        continue;
                    }
                    let Some(x_px) = transform_x.data_x_to_px(line.x) else {
                        continue;
                    };
                    let dist = (local.x.0 - x_px.0).abs();
                    if dist > threshold {
                        continue;
                    }
                    let data = transform_x.px_to_data(local);
                    if !data.x.is_finite() {
                        continue;
                    }
                    let candidate = LinePlotDragSession::LineX {
                        id: line.id,
                        button: *button,
                        offset_x: data.x - line.x,
                        current_x: line.x,
                    };
                    if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                        best = Some((dist, candidate));
                    }
                }

                for line in &overlays.drag_lines_y {
                    if !line.y.is_finite() {
                        continue;
                    }
                    let Some(transform) = line_plot_transform_for_y_axis(
                        plot,
                        view_bounds,
                        model,
                        line.axis,
                        x_scale,
                        y_scale,
                    ) else {
                        continue;
                    };
                    let Some(y_px) = transform.data_y_to_px(line.y) else {
                        continue;
                    };
                    let dist = (local.y.0 - y_px.0).abs();
                    if dist > threshold {
                        continue;
                    }
                    let data = transform.px_to_data(local);
                    if !data.y.is_finite() {
                        continue;
                    }
                    let candidate = LinePlotDragSession::LineY {
                        id: line.id,
                        axis: line.axis,
                        button: *button,
                        offset_y: data.y - line.y,
                        current_y: line.y,
                    };
                    if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                        best = Some((dist, candidate));
                    }
                }

                for rect in &overlays.drag_rects {
                    let Some(transform) = line_plot_transform_for_y_axis(
                        plot,
                        view_bounds,
                        model,
                        rect.axis,
                        x_scale,
                        y_scale,
                    ) else {
                        continue;
                    };
                    let a = transform.data_to_px(DataPoint {
                        x: rect.rect.x_min,
                        y: rect.rect.y_min,
                    });
                    let b = transform.data_to_px(DataPoint {
                        x: rect.rect.x_max,
                        y: rect.rect.y_max,
                    });
                    if !a.x.0.is_finite()
                        || !a.y.0.is_finite()
                        || !b.x.0.is_finite()
                        || !b.y.0.is_finite()
                    {
                        continue;
                    }

                    let left = a.x.0.min(b.x.0);
                    let right = a.x.0.max(b.x.0);
                    let top = a.y.0.min(b.y.0);
                    let bottom = a.y.0.max(b.y.0);
                    let inside = local.x.0 >= left
                        && local.x.0 <= right
                        && local.y.0 >= top
                        && local.y.0 <= bottom;
                    if !inside {
                        continue;
                    }

                    let dist_left = (local.x.0 - left).abs();
                    let dist_right = (local.x.0 - right).abs();
                    let dist_top = (local.y.0 - top).abs();
                    let dist_bottom = (local.y.0 - bottom).abs();
                    let mut handle = LinePlotDragRectHandle::Inside;
                    let mut dist = 0.0f32;
                    let mut set_handle = |d: f32, h: LinePlotDragRectHandle| {
                        if d <= threshold && (handle == LinePlotDragRectHandle::Inside || d < dist)
                        {
                            handle = h;
                            dist = d;
                        }
                    };
                    set_handle(dist_left, LinePlotDragRectHandle::Left);
                    set_handle(dist_right, LinePlotDragRectHandle::Right);
                    set_handle(dist_top, LinePlotDragRectHandle::Top);
                    set_handle(dist_bottom, LinePlotDragRectHandle::Bottom);

                    let data = transform.px_to_data(local);
                    if !data.x.is_finite() || !data.y.is_finite() {
                        continue;
                    }
                    let offset = match handle {
                        LinePlotDragRectHandle::Inside => DataPoint {
                            x: data.x - rect.rect.x_min,
                            y: data.y - rect.rect.y_min,
                        },
                        LinePlotDragRectHandle::Left => DataPoint {
                            x: data.x - rect.rect.x_min,
                            y: 0.0,
                        },
                        LinePlotDragRectHandle::Right => DataPoint {
                            x: data.x - rect.rect.x_max,
                            y: 0.0,
                        },
                        LinePlotDragRectHandle::Top => DataPoint {
                            x: 0.0,
                            y: data.y - rect.rect.y_max,
                        },
                        LinePlotDragRectHandle::Bottom => DataPoint {
                            x: 0.0,
                            y: data.y - rect.rect.y_min,
                        },
                    };
                    let candidate = LinePlotDragSession::Rect {
                        id: rect.id,
                        axis: rect.axis,
                        button: *button,
                        handle,
                        offset,
                        start: rect.rect,
                        current: rect.rect,
                    };
                    if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                        best = Some((dist, candidate));
                    }
                }
            }

            let (_, session) = best?;
            *drag_session.borrow_mut() = Some(session);
            Some(line_plot_drag_output(session, PlotDragPhase::Start))
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) => {
            let Some(mut session) = *drag_session.borrow() else {
                return None;
            };
            let button = match session {
                LinePlotDragSession::LineX { button, .. } => button,
                LinePlotDragSession::LineY { button, .. } => button,
                LinePlotDragSession::Point { button, .. } => button,
                LinePlotDragSession::Rect { button, .. } => button,
            };
            let phase = if line_plot_mouse_buttons_contains(*buttons, button) {
                PlotDragPhase::Update
            } else {
                drag_session.borrow_mut().take();
                PlotDragPhase::End
            };
            line_plot_update_drag_session_at_position(
                &mut session,
                *position,
                plot,
                line_plot_current_view_bounds_for_event(
                    app,
                    Some(state),
                    model,
                    style,
                    x_scale,
                    y_scale,
                ),
                model,
                x_scale,
                y_scale,
            );
            if phase != PlotDragPhase::End {
                *drag_session.borrow_mut() = Some(session);
            }
            Some(line_plot_drag_output(session, phase))
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position, button, ..
        }) => {
            let mut session = drag_session.borrow_mut().take()?;
            let session_button = match session {
                LinePlotDragSession::LineX { button, .. } => button,
                LinePlotDragSession::LineY { button, .. } => button,
                LinePlotDragSession::Point { button, .. } => button,
                LinePlotDragSession::Rect { button, .. } => button,
            };
            if session_button != *button {
                *drag_session.borrow_mut() = Some(session);
                return None;
            }
            line_plot_update_drag_session_at_position(
                &mut session,
                *position,
                plot,
                line_plot_current_view_bounds_for_event(
                    app,
                    Some(state),
                    model,
                    style,
                    x_scale,
                    y_scale,
                ),
                model,
                x_scale,
                y_scale,
            );
            Some(line_plot_drag_output(session, PlotDragPhase::End))
        }
        _ => None,
    }
}

fn line_plot_update_drag_session_at_position(
    session: &mut LinePlotDragSession,
    position: Point,
    plot: Rect,
    view_bounds: DataRect,
    model: &PlotPanelModel,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    match session {
        LinePlotDragSession::LineX {
            offset_x,
            current_x,
            ..
        } => {
            let transform = line_plot_transform_for_x_axis(plot, view_bounds, x_scale, y_scale);
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if data.x.is_finite() {
                *current_x = data.x - *offset_x;
            }
        }
        LinePlotDragSession::LineY {
            axis,
            offset_y,
            current_y,
            ..
        } => {
            let Some(transform) =
                line_plot_transform_for_y_axis(plot, view_bounds, model, *axis, x_scale, y_scale)
            else {
                return;
            };
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if data.y.is_finite() {
                *current_y = data.y - *offset_y;
            }
        }
        LinePlotDragSession::Point {
            axis,
            offset,
            current,
            ..
        } => {
            let Some(transform) =
                line_plot_transform_for_y_axis(plot, view_bounds, model, *axis, x_scale, y_scale)
            else {
                return;
            };
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if data.x.is_finite() && data.y.is_finite() {
                *current = DataPoint {
                    x: data.x - offset.x,
                    y: data.y - offset.y,
                };
            }
        }
        LinePlotDragSession::Rect {
            axis,
            handle,
            offset,
            start,
            current,
            ..
        } => {
            let Some(transform) =
                line_plot_transform_for_y_axis(plot, view_bounds, model, *axis, x_scale, y_scale)
            else {
                return;
            };
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if !data.x.is_finite() || !data.y.is_finite() {
                return;
            }

            let mut next = *current;
            match handle {
                LinePlotDragRectHandle::Inside => {
                    let w = start.width();
                    let h = start.height();
                    next.x_min = data.x - offset.x;
                    next.x_max = next.x_min + w;
                    next.y_min = data.y - offset.y;
                    next.y_max = next.y_min + h;
                }
                LinePlotDragRectHandle::Left => {
                    next.x_min = data.x - offset.x;
                    if next.x_min > next.x_max {
                        next.x_min = next.x_max;
                    }
                }
                LinePlotDragRectHandle::Right => {
                    next.x_max = data.x - offset.x;
                    if next.x_max < next.x_min {
                        next.x_max = next.x_min;
                    }
                }
                LinePlotDragRectHandle::Top => {
                    next.y_max = data.y - offset.y;
                    if next.y_max < next.y_min {
                        next.y_max = next.y_min;
                    }
                }
                LinePlotDragRectHandle::Bottom => {
                    next.y_min = data.y - offset.y;
                    if next.y_min > next.y_max {
                        next.y_min = next.y_max;
                    }
                }
            }
            *current = next;
        }
    }
}

fn line_plot_drag_output(session: LinePlotDragSession, phase: PlotDragPhase) -> PlotDragOutput {
    match session {
        LinePlotDragSession::LineX { id, current_x, .. } => PlotDragOutput::LineX {
            id,
            x: current_x,
            phase,
        },
        LinePlotDragSession::LineY {
            id,
            axis,
            current_y,
            ..
        } => PlotDragOutput::LineY {
            id,
            axis,
            y: current_y,
            phase,
        },
        LinePlotDragSession::Point {
            id, axis, current, ..
        } => PlotDragOutput::Point {
            id,
            axis,
            point: current,
            phase,
        },
        LinePlotDragSession::Rect {
            id, axis, current, ..
        } => PlotDragOutput::Rect {
            id,
            axis,
            rect: current,
            phase,
        },
    }
}

fn line_plot_transform_for_x_axis(
    plot: Rect,
    view_bounds: DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> PlotTransform {
    PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size),
        data: view_bounds,
        x_scale,
        y_scale,
    }
}

fn line_plot_transform_for_y_axis(
    plot: Rect,
    primary_view_bounds: DataRect,
    model: &PlotPanelModel,
    axis: YAxis,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<PlotTransform> {
    let data = match axis {
        YAxis::Left => primary_view_bounds,
        YAxis::Right => {
            line_plot_view_bounds_for_y_axis(primary_view_bounds, model.data_bounds_y2?)
        }
        YAxis::Right2 => {
            line_plot_view_bounds_for_y_axis(primary_view_bounds, model.data_bounds_y3?)
        }
        YAxis::Right3 => {
            line_plot_view_bounds_for_y_axis(primary_view_bounds, model.data_bounds_y4?)
        }
    };
    Some(PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size),
        data,
        x_scale,
        y_scale,
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_line_plot_query_drag_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    query_drag_session: &Rc<RefCell<Option<LinePlotQueryDragSession>>>,
    active_selection: &Rc<Cell<Option<LinePlotSelectionOverlay>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    let input_map = PlotInputMap::default();
    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            ..
        }) if plot.contains(*position)
            && input_map
                .query_drag
                .is_some_and(|chord| chord.matches(*button, *modifiers)) =>
        {
            let local = local_from_absolute(plot.origin, *position);
            *query_drag_session.borrow_mut() = Some(LinePlotQueryDragSession {
                start: local,
                current: local,
                button: *button,
            });
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: local,
                current: local,
                kind: LinePlotSelectionKind::Query,
            }));
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) => {
            let Some(mut session) = *query_drag_session.borrow() else {
                return false;
            };
            if !line_plot_mouse_buttons_contains(*buttons, session.button) {
                query_drag_session.borrow_mut().take();
                active_selection.set(None);
                return true;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: session.start,
                current: session.current,
                kind: LinePlotSelectionKind::Query,
            }));
            *query_drag_session.borrow_mut() = Some(session);
            true
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position, button, ..
        }) => {
            let Some(mut session) = query_drag_session.borrow_mut().take() else {
                return false;
            };
            if session.button != *button {
                *query_drag_session.borrow_mut() = Some(session);
                return false;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(None);
            let w = (session.start.x.0 - session.current.x.0).abs();
            let h = (session.start.y.0 - session.current.y.0).abs();
            if w < 4.0 || h < 4.0 {
                return true;
            }

            let current_view = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let Some(next) = line_plot_query_rect_from_plot_points_raw(
                current_view,
                plot.size,
                session.start,
                session.current,
                x_scale,
                y_scale,
            ) else {
                return true;
            };

            state
                .update(app, |state, _cx| {
                    state.query = Some(next);
                    true
                })
                .ok()
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn line_plot_query_rect_from_plot_points_raw(
    view_bounds: DataRect,
    viewport: Size,
    a: Point,
    b: Point,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataRect> {
    let viewport_w = viewport.width.0;
    let viewport_h = viewport.height.0;
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    let x0 = a.x.0.min(b.x.0).clamp(0.0, viewport_w);
    let x1 = a.x.0.max(b.x.0).clamp(0.0, viewport_w);
    let y0 = a.y.0.min(b.y.0).clamp(0.0, viewport_h);
    let y1 = a.y.0.max(b.y.0).clamp(0.0, viewport_h);

    let transform = PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), viewport),
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let a = transform.px_to_data(Point::new(Px(x0), Px(y0)));
    let b = transform.px_to_data(Point::new(Px(x1), Px(y1)));
    if !a.x.is_finite() || !a.y.is_finite() || !b.x.is_finite() || !b.y.is_finite() {
        return None;
    }

    Some(DataRect {
        x_min: a.x.min(b.x),
        x_max: a.x.max(b.x),
        y_min: a.y.min(b.y),
        y_max: a.y.max(b.y),
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_line_plot_box_zoom_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    box_zoom_session: &Rc<RefCell<Option<LinePlotBoxZoomSession>>>,
    active_selection: &Rc<Cell<Option<LinePlotSelectionOverlay>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    let input_map = PlotInputMap::default();
    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            ..
        }) if plot.contains(*position) => {
            let start_box_primary = input_map.box_zoom.matches(*button, *modifiers);
            let start_box_alt = input_map
                .box_zoom_alt
                .is_some_and(|chord| chord.matches(*button, *modifiers));
            if !start_box_primary && !start_box_alt {
                return false;
            }
            if line_plot_legend_hit(model, plot, *position).is_some() {
                return false;
            }

            let local = local_from_absolute(plot.origin, *position);
            *box_zoom_session.borrow_mut() = Some(LinePlotBoxZoomSession {
                start: local,
                current: local,
                button: *button,
                required_mods: if start_box_primary {
                    input_map.box_zoom.modifiers
                } else {
                    input_map
                        .box_zoom_alt
                        .unwrap_or(input_map.box_zoom)
                        .modifiers
                },
            });
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: local,
                current: local,
                kind: LinePlotSelectionKind::BoxZoom,
            }));
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) => {
            let Some(mut session) = *box_zoom_session.borrow() else {
                return false;
            };
            if !line_plot_mouse_buttons_contains(*buttons, session.button) {
                box_zoom_session.borrow_mut().take();
                active_selection.set(None);
                return true;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: session.start,
                current: session.current,
                kind: LinePlotSelectionKind::BoxZoom,
            }));
            *box_zoom_session.borrow_mut() = Some(session);
            true
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button,
            modifiers,
            ..
        }) => {
            let Some(mut session) = box_zoom_session.borrow_mut().take() else {
                return false;
            };
            if session.button != *button {
                *box_zoom_session.borrow_mut() = Some(session);
                return false;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(None);
            let (start, end) = line_plot_apply_box_select_modifiers(
                plot.size,
                session.start,
                session.current,
                *modifiers,
                input_map.box_zoom_expand_x,
                input_map.box_zoom_expand_y,
                session.required_mods,
            );
            let w = (start.x.0 - end.x.0).abs();
            let h = (start.y.0 - end.y.0).abs();
            if w < 4.0 || h < 4.0 {
                return true;
            }

            let current_view = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let axis_locks = state
                .read_ref(app, |state| state.axis_locks)
                .unwrap_or_default();
            if axis_locks.x.zoom && axis_locks.y.zoom {
                return true;
            }

            let Some(mut next) = data_rect_from_plot_points_scaled(
                current_view,
                plot.size,
                start,
                end,
                x_scale,
                y_scale,
            ) else {
                return true;
            };
            if style.clamp_to_data_bounds {
                next = clamp_view_to_data_scaled(
                    next,
                    model.data_bounds,
                    style.overscroll_fraction,
                    x_scale,
                    y_scale,
                );
            }
            if axis_locks.x.zoom {
                next.x_min = current_view.x_min;
                next.x_max = current_view.x_max;
            }
            if axis_locks.y.zoom {
                next.y_min = current_view.y_min;
                next.y_max = current_view.y_max;
            }
            next = sanitize_data_rect_scaled(next, x_scale, y_scale);
            if next == current_view {
                return true;
            }

            state
                .update(app, |state, _cx| {
                    state.view_is_auto = false;
                    state.view_bounds = Some(next);
                    true
                })
                .ok()
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn line_plot_mouse_buttons_contains(buttons: fret_core::MouseButtons, button: MouseButton) -> bool {
    match button {
        MouseButton::Left => buttons.left,
        MouseButton::Right => buttons.right,
        MouseButton::Middle => buttons.middle,
        MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
    }
}

fn line_plot_apply_box_select_modifiers(
    plot_size: Size,
    start: Point,
    end: Point,
    modifiers: fret_core::Modifiers,
    expand_x: Option<ModifierKey>,
    expand_y: Option<ModifierKey>,
    required: ModifiersMask,
) -> (Point, Point) {
    let mut start = start;
    let mut end = end;

    if expand_x.is_some_and(|key| key.is_pressed(modifiers) && !key.is_required_by(required)) {
        start.x = Px(0.0);
        end.x = plot_size.width;
    }
    if expand_y.is_some_and(|key| key.is_pressed(modifiers) && !key.is_required_by(required)) {
        start.y = Px(0.0);
        end.y = plot_size.height;
    }

    (start, end)
}

#[allow(clippy::too_many_arguments)]
fn handle_line_plot_pan_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    pan_session: &Rc<RefCell<Option<LinePlotPanSession>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers,
            ..
        }) if !modifiers.shift && !modifiers.alt && !modifiers.ctrl && plot.contains(*position) => {
            if line_plot_legend_hit(model, plot, *position).is_some() {
                return false;
            }
            *pan_session.borrow_mut() = Some(LinePlotPanSession {
                last_position: *position,
            });
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) if buttons.left => {
            let Some(mut session) = *pan_session.borrow() else {
                return false;
            };
            let current_view = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let dx_px = position.x.0 - session.last_position.x.0;
            let dy_px = position.y.0 - session.last_position.y.0;
            if dx_px == 0.0 && dy_px == 0.0 {
                return true;
            }
            let mut next =
                pan_line_plot_view_bounds(current_view, plot, dx_px, dy_px, x_scale, y_scale);
            let axis_locks = state
                .read_ref(app, |state| state.axis_locks)
                .unwrap_or_default();
            if axis_locks.x.pan {
                next.x_min = current_view.x_min;
                next.x_max = current_view.x_max;
            }
            if axis_locks.y.pan {
                next.y_min = current_view.y_min;
                next.y_max = current_view.y_max;
            }
            let _ = state.update(app, |state, _cx| {
                state.view_is_auto = false;
                state.view_bounds = Some(next);
            });
            session.last_position = *position;
            *pan_session.borrow_mut() = Some(session);
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move { buttons, .. }) if !buttons.left => {
            pan_session.borrow_mut().take().is_some()
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            button: MouseButton::Left,
            ..
        }) => pan_session.borrow_mut().take().is_some(),
        _ => false,
    }
}

fn pan_line_plot_view_bounds(
    view: DataRect,
    plot: Rect,
    dx_px: f32,
    dy_px: f32,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    let pan_axis = |scale: AxisScale, min: f64, max: f64, delta_px: f32, span_px: f32| {
        let Some(axis_min) = scale.to_axis(min) else {
            return (min, max);
        };
        let Some(axis_max) = scale.to_axis(max) else {
            return (min, max);
        };
        if span_px <= 0.0 {
            return (min, max);
        }
        let axis_delta = -(delta_px as f64) / span_px as f64 * (axis_max - axis_min);
        (
            scale.from_axis(axis_min + axis_delta).unwrap_or(min),
            scale.from_axis(axis_max + axis_delta).unwrap_or(max),
        )
    };
    let (x_min, x_max) = pan_axis(x_scale, view.x_min, view.x_max, dx_px, plot.size.width.0);
    let (y_min, y_max) = pan_axis(y_scale, view.y_min, view.y_max, -dy_px, plot.size.height.0);
    sanitize_data_rect_scaled(
        DataRect {
            x_min,
            x_max,
            y_min,
            y_max,
        },
        x_scale,
        y_scale,
    )
}

#[allow(clippy::too_many_arguments)]
fn handle_line_plot_wheel_zoom_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let Event::Pointer(fret_core::PointerEvent::Wheel {
        position,
        delta,
        modifiers,
        ..
    }) = event
    else {
        return false;
    };

    let Some(region) = line_plot_wheel_region_at(bounds, style, *position) else {
        return false;
    };
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    let input_map = PlotInputMap::default();
    if let Some(required) = input_map.wheel_zoom_mod
        && !required.is_pressed(*modifiers)
    {
        return false;
    }

    let delta_y = delta.y.0;
    if !delta_y.is_finite() {
        return false;
    }

    let speed = if input_map.wheel_zoom_log2_per_px.is_finite() {
        input_map.wheel_zoom_log2_per_px
    } else {
        PlotInputMap::default().wheel_zoom_log2_per_px
    };
    let zoom = clamp_zoom_factors(2.0_f32.powf(delta_y * speed));
    let mut zoom_x = zoom;
    let mut zoom_y = zoom;

    match region {
        LinePlotWheelRegion::Plot => {
            let x_only = input_map
                .wheel_zoom_x_only_mod
                .is_some_and(|modifier| modifier.is_pressed(*modifiers));
            let y_only = input_map
                .wheel_zoom_y_only_mod
                .is_some_and(|modifier| modifier.is_pressed(*modifiers));
            if x_only {
                zoom_y = 1.0;
            } else if y_only {
                zoom_x = 1.0;
            }
        }
        LinePlotWheelRegion::XAxis => {
            zoom_y = 1.0;
        }
        LinePlotWheelRegion::YAxis => {
            zoom_x = 1.0;
        }
    }

    let axis_locks = state
        .read_ref(app, |state| state.axis_locks)
        .unwrap_or_default();
    if axis_locks.x.zoom {
        zoom_x = 1.0;
    }
    if axis_locks.y.zoom {
        zoom_y = 1.0;
    }

    if zoom_x == 1.0 && zoom_y == 1.0 {
        return false;
    }

    let current =
        line_plot_current_view_bounds_for_event(app, Some(state), model, style, x_scale, y_scale);
    let local = local_from_absolute(plot.origin, *position);
    let Some(mut next) =
        zoom_view_at_px_scaled(current, plot.size, local, zoom_x, zoom_y, x_scale, y_scale)
    else {
        return false;
    };
    if style.clamp_to_data_bounds {
        next = clamp_view_to_data_scaled(
            next,
            model.data_bounds,
            style.overscroll_fraction,
            x_scale,
            y_scale,
        );
    }
    next = sanitize_data_rect_scaled(next, x_scale, y_scale);
    if next == current {
        return false;
    }

    state
        .update(app, |state, _cx| {
            state.view_is_auto = false;
            state.view_bounds = Some(next);
            true
        })
        .ok()
        .unwrap_or(false)
}

fn line_plot_wheel_region_at(
    bounds: Rect,
    style: LinePlotStyle,
    position: Point,
) -> Option<LinePlotWheelRegion> {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.contains(position) {
        return Some(LinePlotWheelRegion::Plot);
    }

    let pad = style.padding.0.max(0.0);
    let axis_gap = style.axis_gap.0.max(0.0);
    let y_axis = Rect::new(
        Point::new(Px(bounds.origin.x.0 + pad), plot.origin.y),
        Size::new(Px(axis_gap), plot.size.height),
    );
    if y_axis.contains(position) {
        return Some(LinePlotWheelRegion::YAxis);
    }

    let x_axis = Rect::new(
        Point::new(plot.origin.x, Px(plot.origin.y.0 + plot.size.height.0)),
        Size::new(plot.size.width, Px(axis_gap)),
    );
    if x_axis.contains(position) {
        return Some(LinePlotWheelRegion::XAxis);
    }

    None
}

fn line_plot_legend_hover_from_event(
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
) -> Option<Option<SeriesId>> {
    let Event::Pointer(fret_core::PointerEvent::Move { position, .. }) = event else {
        return None;
    };

    let plot = line_plot_inner_rect(bounds, style);
    Some(
        line_plot_legend_hit(model, plot, *position)
            .map(|(series_id, _hit)| series_id)
            .filter(|series_id| model.series.iter().any(|series| series.id == *series_id)),
    )
}

fn publish_line_plot_panel_output<H: UiHost>(
    app: &mut H,
    output: Option<&Model<PlotOutput>>,
    snapshot: PlotOutputSnapshot,
) -> bool {
    let Some(output) = output else {
        return false;
    };
    if output
        .read_ref(app, |state| state.snapshot == snapshot)
        .unwrap_or(false)
    {
        return false;
    }
    output
        .update(app, |state, _cx| {
            state.revision = state.revision.wrapping_add(1);
            state.snapshot = snapshot;
            true
        })
        .ok()
        .unwrap_or(false)
}

fn line_plot_query_from_state<H: UiHost>(
    app: &H,
    state: Option<&Model<PlotState>>,
) -> Option<DataRect> {
    state.and_then(|state| state.read_ref(app, |state| state.query).ok().flatten())
}

fn line_plot_output_snapshot(
    view_bounds: DataRect,
    cursor: Option<DataPoint>,
    query: Option<DataRect>,
) -> PlotOutputSnapshot {
    line_plot_output_snapshot_with_drag(view_bounds, cursor, query, None)
}

fn line_plot_output_snapshot_with_drag(
    view_bounds: DataRect,
    cursor: Option<DataPoint>,
    query: Option<DataRect>,
    drag: Option<PlotDragOutput>,
) -> PlotOutputSnapshot {
    PlotOutputSnapshot {
        view_bounds,
        view_bounds_y2: None,
        view_bounds_y3: None,
        view_bounds_y4: None,
        cursor,
        hover: None,
        query,
        drag,
    }
}

fn line_plot_pointer_output_snapshot(
    pointer: Point,
    bounds: Rect,
    _model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
    view_bounds: DataRect,
    query: Option<DataRect>,
) -> PlotOutputSnapshot {
    let plot = line_plot_inner_rect(bounds, style);
    let cursor = cursor_data_for_line_plot_pointer(pointer, plot, view_bounds, x_scale, y_scale);
    line_plot_output_snapshot(view_bounds, cursor, query)
}

fn line_plot_view_bounds_from_state(
    model: &PlotPanelModel,
    state: Option<&PlotState>,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    if let Some(state) = state
        && !state.view_is_auto
        && let Some(view) = state.view_bounds
    {
        return sanitize_data_rect_scaled(view, x_scale, y_scale);
    }
    let data_bounds = sanitize_data_rect_scaled(model.data_bounds, x_scale, y_scale);
    if style.clamp_to_data_bounds {
        expand_line_plot_data_bounds(data_bounds, style.overscroll_fraction, x_scale, y_scale)
    } else {
        data_bounds
    }
}

fn line_plot_current_view_bounds_for_event<H: UiHost>(
    app: &H,
    state: Option<&Model<PlotState>>,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    state
        .and_then(|state| {
            state
                .read_ref(app, |state| {
                    line_plot_view_bounds_from_state(model, Some(state), style, x_scale, y_scale)
                })
                .ok()
        })
        .unwrap_or_else(|| line_plot_view_bounds_from_state(model, None, style, x_scale, y_scale))
}

fn expand_line_plot_data_bounds(
    bounds: DataRect,
    fraction: f32,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    let fraction = fraction.max(0.0) as f64;
    if fraction <= 0.0 {
        return bounds;
    }
    let expand_axis = |scale: AxisScale, min: f64, max: f64| -> (f64, f64) {
        let Some(axis_min) = scale.to_axis(min) else {
            return (min, max);
        };
        let Some(axis_max) = scale.to_axis(max) else {
            return (min, max);
        };
        let span = axis_max - axis_min;
        if !span.is_finite() || span <= 0.0 {
            return (min, max);
        }
        let pad = span * fraction;
        let next_min = scale.from_axis(axis_min - pad).unwrap_or(min);
        let next_max = scale.from_axis(axis_max + pad).unwrap_or(max);
        (next_min, next_max)
    };
    let (x_min, x_max) = expand_axis(x_scale, bounds.x_min, bounds.x_max);
    let (y_min, y_max) = expand_axis(y_scale, bounds.y_min, bounds.y_max);
    sanitize_data_rect_scaled(
        DataRect {
            x_min,
            x_max,
            y_min,
            y_max,
        },
        x_scale,
        y_scale,
    )
}

fn cursor_data_for_line_plot_pointer(
    pointer: Point,
    plot: Rect,
    view_bounds: crate::cartesian::DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataPoint> {
    if !plot.contains(pointer) || plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }
    let transform = PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let data = transform.px_to_data(pointer);
    (data.x.is_finite() && data.y.is_finite()).then_some(data)
}

fn paint_line_plot_grid_and_axes(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    style: LinePlotStyle,
    x_axis_labels: &AxisLabelFormatter,
    y_axis_labels: &AxisLabelFormatter,
) {
    let plot = transform.viewport;
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let theme = painter.theme().snapshot();
    let mut grid_color = style
        .grid_color
        .unwrap_or_else(|| theme.color_required("border"));
    grid_color.a *= 0.45;
    let axis_color = style
        .axis_color
        .unwrap_or_else(|| theme.color_required("border"));
    let tick_count = style.tick_count.max(2);

    let x_ticks = axis_ticks_scaled(
        transform.data.x_min,
        transform.data.x_max,
        tick_count,
        AxisTicks::Nice,
        transform.x_scale,
    );
    let y_ticks = axis_ticks_scaled(
        transform.data.y_min,
        transform.data.y_max,
        tick_count,
        AxisTicks::Nice,
        transform.y_scale,
    );

    for x in x_ticks.iter().copied() {
        let Some(px) = transform.data_x_to_px(x) else {
            continue;
        };
        push_vertical_line(
            painter,
            px,
            plot.origin.y,
            plot.size.height,
            DrawOrder(2),
            grid_color,
        );
    }

    for y in y_ticks.iter().copied() {
        let Some(py) = transform.data_y_to_px(y) else {
            continue;
        };
        push_horizontal_line(
            painter,
            plot.origin.x,
            py,
            plot.size.width,
            DrawOrder(2),
            grid_color,
        );
    }

    let baseline_y = transform
        .data_y_to_px(0.0)
        .filter(|y| y.0 >= plot.origin.y.0 && y.0 <= plot.origin.y.0 + plot.size.height.0)
        .unwrap_or_else(|| Px(plot.origin.y.0 + plot.size.height.0 - 1.0));
    let baseline_x = transform
        .data_x_to_px(0.0)
        .filter(|x| x.0 >= plot.origin.x.0 && x.0 <= plot.origin.x.0 + plot.size.width.0)
        .unwrap_or(plot.origin.x);

    push_horizontal_line(
        painter,
        plot.origin.x,
        baseline_y,
        plot.size.width,
        DrawOrder(10),
        axis_color,
    );
    push_vertical_line(
        painter,
        baseline_x,
        plot.origin.y,
        plot.size.height,
        DrawOrder(10),
        axis_color,
    );

    paint_line_plot_axis_tick_labels(
        painter,
        transform,
        style,
        &x_ticks,
        &y_ticks,
        x_axis_labels,
        y_axis_labels,
    );
}

fn push_vertical_line(
    painter: &mut CanvasPainter<'_>,
    x: Px,
    y: Px,
    height: Px,
    order: DrawOrder,
    color: Color,
) {
    if !x.0.is_finite() || !y.0.is_finite() || !height.0.is_finite() || height.0 <= 0.0 {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect: Rect::new(Point::new(x, y), Size::new(Px(1.0), height)),
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}

fn push_horizontal_line(
    painter: &mut CanvasPainter<'_>,
    x: Px,
    y: Px,
    width: Px,
    order: DrawOrder,
    color: Color,
) {
    if !x.0.is_finite() || !y.0.is_finite() || !width.0.is_finite() || width.0 <= 0.0 {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect: Rect::new(Point::new(x, y), Size::new(width, Px(1.0))),
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}

fn push_filled_rect(painter: &mut CanvasPainter<'_>, rect: Rect, order: DrawOrder, color: Color) {
    if !rect.origin.x.0.is_finite()
        || !rect.origin.y.0.is_finite()
        || !rect.size.width.0.is_finite()
        || !rect.size.height.0.is_finite()
        || rect.size.width.0 <= 0.0
        || rect.size.height.0 <= 0.0
    {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect,
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}

fn axis_tick_label_text(
    scale: AxisScale,
    formatter: &AxisLabelFormatter,
    value: f64,
    span: f64,
) -> String {
    if scale == AxisScale::Log10 && formatter.is_number_auto() {
        return log10_tick_label_or_empty(value);
    }
    formatter.format(value, span)
}

fn line_plot_inner_rect(bounds: Rect, style: LinePlotStyle) -> Rect {
    let pad = style.padding.0.max(0.0);
    let axis_gap = style.axis_gap.0.max(0.0);
    Rect::new(
        Point::new(
            Px(bounds.origin.x.0 + pad + axis_gap),
            Px(bounds.origin.y.0 + pad),
        ),
        Size::new(
            Px((bounds.size.width.0 - pad * 2.0 - axis_gap).max(0.0)),
            Px((bounds.size.height.0 - pad * 2.0 - axis_gap).max(0.0)),
        ),
    )
}

fn series_color(style: LinePlotStyle, series_index: usize, series_count: usize) -> Color {
    if series_count <= 1 {
        return style.stroke_color;
    }
    style.series_palette[series_index % style.series_palette.len()]
}

#[cfg(test)]
mod tests;
