use std::cell::{Cell, RefCell};
use std::rc::Rc;

use fret_core::{
    Color, Corners, DrawOrder, Edges, Paint, PathStyle, Point, Px, Rect, Size, StrokeStyle,
};
use fret_runtime::Model;
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, CanvasProps, Length, ManagedSurfaceProps};
use fret_ui::{ElementContext, UiHost};

use crate::cartesian::{AxisScale, DataRect, PlotTransform, polyline_commands};
use crate::models::{StepMode, YAxis};
use crate::plot::axis::{AxisLabelFormatter, log10_tick_label_or_empty};
use crate::series::SeriesId;
use crate::state::{PlotImageLayer, PlotOutput, PlotOutputSnapshot, PlotOverlays, PlotState};
use crate::style::LinePlotStyle;

mod axis_labels;
mod commands;
mod grid_axes;
mod heatmap;
mod interaction;
mod legend;
mod model;
mod output;
mod overlays;
mod panels;
mod props;
mod readout;
mod selection;

use axis_labels::paint_line_plot_right_axis_tick_labels;
use commands::{
    area_fill_commands_from_polyline, bars_commands_from_series, candlestick_commands_from_series,
    error_bars_commands_from_series, histogram_commands_from_series, line_plot_area_fill_path_key,
    line_plot_candlestick_down_path_key, line_plot_series_path_key,
    line_plot_shaded_lower_path_key, shaded_band_commands_from_series, stems_commands_from_points,
    step_commands_from_polyline,
};
use grid_axes::paint_line_plot_grid_and_axes;
use heatmap::{paint_line_plot_heatmap, paint_line_plot_heatmap_colorbar};
use interaction::{
    LinePlotBoxZoomSession, LinePlotDragSession, LinePlotPanSession, LinePlotQueryDragSession,
    LinePlotSelectionOverlay, handle_line_plot_box_zoom_event,
    handle_line_plot_draggable_overlay_event, handle_line_plot_legend_pointer_event,
    handle_line_plot_pan_event, handle_line_plot_query_drag_event,
    handle_line_plot_wheel_zoom_event, line_plot_legend_hover_from_event,
    line_plot_panel_event_snapshot,
};
use legend::paint_line_plot_legend;
use model::PlotPanelModel;
use output::{
    line_plot_current_view_bounds_for_event, line_plot_output_snapshot,
    line_plot_output_snapshot_with_drag, line_plot_query_from_state,
    line_plot_view_bounds_from_state, publish_line_plot_panel_output,
};
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
