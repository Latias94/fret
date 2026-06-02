//! Public plot panel props and private builder-owner facade.

mod area;
mod bars;
mod candlestick;
mod error_bars;
mod heatmap;
mod histogram;
mod histogram2d;
mod line;
mod records;
mod shaded;
mod stems;

pub use records::{
    AreaPlotPanelProps, BarsPlotPanelProps, CandlestickPlotPanelProps, ErrorBarsPlotPanelProps,
    HeatmapPlotPanelProps, Histogram2DPlotPanelProps, HistogramPlotPanelProps, LinePlotPanelProps,
    ShadedPlotPanelProps, StemsPlotPanelProps,
};
