use std::cell::{Cell, RefCell};
use std::rc::Rc;

use fret_runtime::Model;
use fret_ui::element::{AnyElement, CanvasProps, Length, ManagedSurfaceProps};
use fret_ui::{ElementContext, UiHost};

use crate::cartesian::{AxisScale, DataRect};
use crate::models::StepMode;
use crate::plot::axis::AxisLabelFormatter;
use crate::series::SeriesId;
use crate::state::{PlotOutput, PlotOverlays, PlotState};
use crate::style::LinePlotStyle;

mod axis_labels;
mod commands;
mod geometry;
mod grid_axes;
mod heatmap;
mod interaction;
mod legend;
mod model;
mod output;
mod overlays;
mod paint_primitives;
mod panel_paint;
mod panels;
mod props;
mod readout;
mod selection;
mod style_helpers;

use interaction::{
    LinePlotBoxZoomSession, LinePlotDragSession, LinePlotPanSession, LinePlotQueryDragSession,
    LinePlotSelectionOverlay, handle_line_plot_box_zoom_event,
    handle_line_plot_draggable_overlay_event, handle_line_plot_legend_pointer_event,
    handle_line_plot_pan_event, handle_line_plot_query_drag_event,
    handle_line_plot_wheel_zoom_event, line_plot_legend_hover_from_event,
    line_plot_panel_event_snapshot,
};
use model::PlotPanelModel;
use output::{
    line_plot_current_view_bounds_for_event, line_plot_output_snapshot,
    line_plot_output_snapshot_with_drag, line_plot_query_from_state,
    line_plot_view_bounds_from_state, publish_line_plot_panel_output,
};
use panel_paint::paint_line_plot_panel;

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

#[cfg(test)]
mod tests;
