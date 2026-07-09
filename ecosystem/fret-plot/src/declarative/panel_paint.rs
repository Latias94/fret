//! Declarative line-plot panel paint orchestration owner.

use fret_core::{Color, Corners, DrawOrder, Edges, Paint};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::{AxisScale, DataRect, PlotTransform};
use crate::models::StepMode;
use crate::plot::axis::AxisLabelFormatter;
use crate::series::SeriesId;
use crate::state::{PlotImageLayer, PlotOutputSnapshot, PlotOverlays};
use crate::style::LinePlotStyle;

use super::axis_labels::paint_line_plot_right_axis_tick_labels;
use super::geometry::line_plot_inner_rect;
use super::grid_axes::paint_line_plot_grid_and_axes;
use super::heatmap::{paint_line_plot_heatmap, paint_line_plot_heatmap_colorbar};
use super::interaction::LinePlotSelectionOverlay;
use super::legend::paint_line_plot_legend;
use super::model::PlotPanelModel;
use super::overlays::{
    paint_line_plot_draggable_overlay_labels, paint_line_plot_draggable_shapes,
    paint_line_plot_images, paint_line_plot_reference_lines, paint_line_plot_tag_overlays,
    paint_line_plot_text_overlays,
};
use super::readout::{paint_line_plot_cursor_readout, paint_line_plot_linked_cursor_readout};
use super::selection::{
    paint_line_plot_active_selection, paint_line_plot_query_selection,
    paint_line_plot_selection_tooltip,
};
use super::series_paint::paint_line_plot_series;

pub(super) const PLOT_PANEL_BACKGROUND_TOKEN: &str = "color.surface.background";

pub(super) fn paint_line_plot_panel(
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

    let background = style.background.unwrap_or_else(|| {
        painter
            .theme()
            .snapshot()
            .color_required(PLOT_PANEL_BACKGROUND_TOKEN)
    });
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
    paint_line_plot_grid_and_axes(painter, transform, style, x_axis_labels, y_axis_labels);
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

    paint_line_plot_series(
        painter,
        model,
        transform,
        hidden_series,
        step_mode,
        style,
        pinned_series,
        legend_hover,
    );
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
