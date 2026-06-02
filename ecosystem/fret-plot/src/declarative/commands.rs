//! Declarative line-plot path-command projection hub.

mod bar_histogram;
mod candlestick;
mod error_bars;
mod line_area;
mod shaded;

pub(super) use bar_histogram::{bars_commands_from_series, histogram_commands_from_series};
pub(super) use candlestick::{
    candlestick_commands_from_series, line_plot_candlestick_down_path_key,
};
pub(super) use error_bars::error_bars_commands_from_series;
pub(super) use line_area::{
    area_fill_commands_from_polyline, stems_commands_from_points, step_commands_from_polyline,
};
pub(super) use shaded::{line_plot_shaded_lower_path_key, shaded_band_commands_from_series};

pub(super) fn line_plot_series_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6c69_6e65_u64 ^ series_id
}

pub(super) fn line_plot_area_fill_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6172_6561_u64 ^ series_id
}
