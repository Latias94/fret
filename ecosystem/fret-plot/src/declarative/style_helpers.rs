//! Declarative line-plot shared style and formatting owner.

use fret_core::Color;

use crate::cartesian::AxisScale;
use crate::plot::axis::{AxisLabelFormatter, log10_tick_label_or_empty};
use crate::style::LinePlotStyle;

pub(super) fn axis_tick_label_text(
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

pub(super) fn series_color(
    style: LinePlotStyle,
    series_index: usize,
    series_count: usize,
) -> Color {
    if series_count <= 1 {
        return style.stroke_color;
    }
    style.series_palette[series_index % style.series_palette.len()]
}
