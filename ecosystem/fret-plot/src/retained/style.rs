//! Plot styling types.

use fret_core::geometry::Px;
use fret_core::scene::Color;

use crate::plot::colormap::ColorMapId;

pub const DEFAULT_SERIES_PALETTE: [Color; 10] = [
    Color::from_srgb_hex_rgb(0x59_a6_f2),
    Color::from_srgb_hex_rgb(0xf2_73_8c),
    Color::from_srgb_hex_rgb(0x73_d9_8c),
    Color::from_srgb_hex_rgb(0xf2_bf_59),
    Color::from_srgb_hex_rgb(0xbf_8c_f2),
    Color::from_srgb_hex_rgb(0x59_d9_d9),
    Color::from_srgb_hex_rgb(0xf2_59_d9),
    Color::from_srgb_hex_rgb(0xa6_a6_a6),
    Color::from_srgb_hex_rgb(0x8c_bf_59),
    Color::from_srgb_hex_rgb(0x59_8c_f2),
];
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseReadoutMode {
    /// Show mouse coordinates as a tooltip near the cursor.
    Tooltip,
    /// Show mouse coordinates as a small overlay inside the plot (ImPlot-style).
    Overlay,
    /// Do not show mouse coordinate readout.
    Disabled,
}

impl Default for MouseReadoutMode {
    fn default() -> Self {
        Self::Overlay
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for OverlayAnchor {
    fn default() -> Self {
        Self::TopLeft
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadoutSeriesPolicy {
    /// If a series is pinned, show only that series; otherwise show all visible series.
    PinnedOrAll,
    /// Show only the pinned series. If no series is pinned, show no per-series rows.
    PinnedOnly,
    /// If pinned, show pinned; else if a legend row is hovered, show that series; else show all.
    PinnedOrLegendHoverOrAll,
}

impl Default for ReadoutSeriesPolicy {
    fn default() -> Self {
        Self::PinnedOrAll
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeriesTooltipMode {
    /// Match ImPlot-style hover tooltips: show a series tooltip only when the pointer is close to
    /// a series item (hit-tested within `LinePlotStyle::hover_threshold`).
    HoverOnly,
    /// When the pointer is inside the plot region, show a tooltip for the nearest series at the
    /// cursor X (based on the smallest `|cursor_y - series_y_at_x|` distance).
    ///
    /// This does not change hover emphasis or selection; it only affects tooltip selection.
    NearestAtCursor,
}

impl Default for SeriesTooltipMode {
    fn default() -> Self {
        Self::HoverOnly
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LinePlotStyle {
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_width: Px,
    pub padding: Px,
    pub axis_gap: Px,
    pub axis_color: Option<Color>,
    pub grid_color: Option<Color>,
    pub label_color: Option<Color>,
    pub crosshair_color: Option<Color>,
    pub tooltip_background: Option<Color>,
    pub tooltip_border: Option<Color>,
    pub tooltip_text_color: Option<Color>,
    pub mouse_readout: MouseReadoutMode,
    pub mouse_readout_anchor: OverlayAnchor,
    pub linked_cursor_readout: MouseReadoutMode,
    pub linked_cursor_readout_anchor: OverlayAnchor,
    pub linked_cursor_readout_policy: ReadoutSeriesPolicy,
    pub series_tooltip: SeriesTooltipMode,
    pub hover_threshold: Px,
    /// Minimum number of major tick labels per axis.
    ///
    /// The plot may choose more ticks for large viewports and fewer ticks for small viewports when
    /// labels would overlap.
    pub tick_count: usize,
    pub stroke_color: Color,
    pub stroke_width: Px,
    /// Default palette used to assign colors to series without an explicit per-series override.
    pub series_palette: [Color; 10],
    pub clamp_to_data_bounds: bool,
    /// Extra range around `data_bounds` used by clamping and auto-fit.
    ///
    /// This is expressed as a fraction of the data span (e.g. `0.03` means 3%).
    pub overscroll_fraction: f32,
    pub emphasize_hovered_series: bool,
    pub dimmed_series_alpha: f32,

    pub heatmap_colormap: ColorMapId,
    pub heatmap_show_colorbar: bool,
    pub heatmap_colorbar_width: Px,
    pub heatmap_colorbar_padding: Px,
    pub heatmap_colorbar_steps: usize,

    /// Maximum number of scene-op tiles to build per frame for tiled scene caches (e.g.
    /// heatmap/histogram2d). The plot uses a smaller budget while interacting to stay responsive,
    /// then continues warming tiles while idle.
    pub tile_warmup_tiles_per_frame_idle: u32,
    pub tile_warmup_tiles_per_frame_interactive: u32,
}

impl Default for LinePlotStyle {
    fn default() -> Self {
        Self {
            background: None,
            border: None,
            border_width: Px(1.0),
            padding: Px(8.0),
            axis_gap: Px(18.0),
            axis_color: None,
            grid_color: None,
            label_color: None,
            crosshair_color: None,
            tooltip_background: None,
            tooltip_border: None,
            tooltip_text_color: None,
            mouse_readout: MouseReadoutMode::default(),
            mouse_readout_anchor: OverlayAnchor::BottomLeft,
            linked_cursor_readout: MouseReadoutMode::default(),
            linked_cursor_readout_anchor: OverlayAnchor::TopLeft,
            linked_cursor_readout_policy: ReadoutSeriesPolicy::default(),
            series_tooltip: SeriesTooltipMode::default(),
            hover_threshold: Px(10.0),
            tick_count: 5,
            stroke_color: Color::from_srgb_hex_rgb(0x59_a6_f2),
            stroke_width: Px(1.5),
            series_palette: DEFAULT_SERIES_PALETTE,
            clamp_to_data_bounds: true,
            overscroll_fraction: 0.03,
            emphasize_hovered_series: true,
            dimmed_series_alpha: 0.35,

            heatmap_colormap: ColorMapId::default(),
            heatmap_show_colorbar: false,
            heatmap_colorbar_width: Px(14.0),
            heatmap_colorbar_padding: Px(8.0),
            heatmap_colorbar_steps: 64,

            tile_warmup_tiles_per_frame_idle: 8,
            tile_warmup_tiles_per_frame_interactive: 2,
        }
    }
}
