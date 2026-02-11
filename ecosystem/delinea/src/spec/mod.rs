use fret_core::Rect;

use crate::ids::{
    AxisId, ChartId, DataZoomId, DatasetId, FieldId, GridId, SeriesId, StackId, VisualMapId,
};
use crate::scale::AxisScale;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChartSpec {
    pub id: ChartId,
    pub viewport: Option<Rect>,
    pub datasets: Vec<DatasetSpec>,
    pub grids: Vec<GridSpec>,
    pub axes: Vec<AxisSpec>,
    pub data_zoom_x: Vec<DataZoomXSpec>,
    pub data_zoom_y: Vec<DataZoomYSpec>,
    pub tooltip: Option<TooltipSpecV1>,
    pub axis_pointer: Option<AxisPointerSpec>,
    pub visual_maps: Vec<VisualMapSpec>,
    pub series: Vec<SeriesSpec>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetSpec {
    pub id: DatasetId,
    pub fields: Vec<FieldSpec>,
    /// When set, this dataset is derived from an upstream dataset by applying `transforms` in
    /// order.
    #[cfg_attr(feature = "serde", serde(default))]
    pub from: Option<DatasetId>,
    /// v1 subset: row-preserving transforms only (filter/sort); schema is inherited from `from`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub transforms: Vec<DatasetTransformSpecV1>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DatasetSortOrder {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetFilterSpecV1 {
    pub field: FieldId,
    pub gte: Option<f64>,
    pub gt: Option<f64>,
    pub lte: Option<f64>,
    pub lt: Option<f64>,
    pub eq: Option<f64>,
    pub ne: Option<f64>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetSortSpecV1 {
    pub field: FieldId,
    pub order: DatasetSortOrder,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DatasetTransformSpecV1 {
    Filter(DatasetFilterSpecV1),
    Sort(DatasetSortSpecV1),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldSpec {
    pub id: FieldId,
    /// Column index in the dataset table.
    pub column: usize,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GridSpec {
    pub id: GridId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisKind {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisPosition {
    Top,
    Bottom,
    Left,
    Right,
}

impl AxisPosition {
    pub fn default_for_kind(kind: AxisKind) -> Self {
        match kind {
            AxisKind::X => AxisPosition::Bottom,
            AxisKind::Y => AxisPosition::Left,
        }
    }

    pub fn is_compatible(self, kind: AxisKind) -> bool {
        match kind {
            AxisKind::X => matches!(self, AxisPosition::Top | AxisPosition::Bottom),
            AxisKind::Y => matches!(self, AxisPosition::Left | AxisPosition::Right),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FilterMode {
    #[default]
    Filter,
    /// ECharts-like `weakFilter`.
    ///
    /// v1 note: treated as equivalent to `Filter` until multi-dimensional filtering is introduced
    /// (see ADR 0211).
    WeakFilter,
    /// ECharts-like `empty`.
    ///
    /// Out-of-window samples are treated as "missing" for mark emission while preserving a stable
    /// row/index space (see ADR 0211).
    Empty,
    None,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisRange {
    #[default]
    Auto,
    LockMin {
        min: f64,
    },
    LockMax {
        max: f64,
    },
    Fixed {
        min: f64,
        max: f64,
    },
}

impl AxisRange {
    pub fn clamp_non_degenerate(&mut self) {
        match self {
            AxisRange::Auto => {}
            AxisRange::LockMin { min } => {
                if !min.is_finite() {
                    *min = 0.0;
                }
            }
            AxisRange::LockMax { max } => {
                if !max.is_finite() {
                    *max = 1.0;
                }
            }
            AxisRange::Fixed { min, max } => {
                if !min.is_finite() || !max.is_finite() || *max <= *min {
                    *min = 0.0;
                    *max = 1.0;
                }
            }
        }
    }

    pub fn locked_min(&self) -> Option<f64> {
        match *self {
            AxisRange::Auto => None,
            AxisRange::LockMin { min } => Some(min),
            AxisRange::LockMax { .. } => None,
            AxisRange::Fixed { min, .. } => Some(min),
        }
    }

    pub fn locked_max(&self) -> Option<f64> {
        match *self {
            AxisRange::Auto => None,
            AxisRange::LockMin { .. } => None,
            AxisRange::LockMax { max } => Some(max),
            AxisRange::Fixed { max, .. } => Some(max),
        }
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, AxisRange::Fixed { .. })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisSpec {
    pub id: AxisId,
    /// Optional display name (used by tooltips and legends).
    pub name: Option<String>,
    pub kind: AxisKind,
    pub grid: GridId,
    /// Axis placement in the cartesian grid (presentation-only).
    /// Defaults: X=Bottom, Y=Left.
    pub position: Option<AxisPosition>,
    pub scale: AxisScale,
    /// When set, the axis is constrained in data space.
    /// In v1, `Fixed` fully overrides view windows; partial locks override only one bound.
    pub range: Option<AxisRange>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataZoomXSpec {
    pub id: DataZoomId,
    pub axis: AxisId,
    pub filter_mode: FilterMode,
    /// Minimum allowed span (in data value space) for interaction-derived zoom updates.
    pub min_value_span: Option<f64>,
    /// Maximum allowed span (in data value space) for interaction-derived zoom updates.
    pub max_value_span: Option<f64>,
}

impl Default for DataZoomXSpec {
    fn default() -> Self {
        Self {
            id: DataZoomId::new(0),
            axis: AxisId::new(0),
            filter_mode: FilterMode::Filter,
            min_value_span: None,
            max_value_span: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataZoomYSpec {
    pub id: DataZoomId,
    pub axis: AxisId,
    /// Row filtering mode for Y dataZoom.
    ///
    /// v1 default is `None` (mapping-only; ADR 0198). When enabled, the view/transform pipeline
    /// may materialize sparse selections (ECharts-like) which can be more expensive on large data.
    pub filter_mode: FilterMode,
    /// Minimum allowed span (in data value space) for interaction-derived zoom updates.
    pub min_value_span: Option<f64>,
    /// Maximum allowed span (in data value space) for interaction-derived zoom updates.
    pub max_value_span: Option<f64>,
}

impl Default for DataZoomYSpec {
    fn default() -> Self {
        Self {
            id: DataZoomId::new(0),
            axis: AxisId::new(0),
            filter_mode: FilterMode::None,
            min_value_span: None,
            max_value_span: None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum VisualMapMode {
    /// ECharts-like continuous visualMap (`type: "continuous"`).
    #[default]
    Continuous,
    /// ECharts-like piecewise visualMap (`type: "piecewise"`).
    Piecewise,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VisualMapSpec {
    pub id: VisualMapId,
    pub mode: VisualMapMode,
    /// Optional dataset-wide target.
    ///
    /// When set, `series` must be empty and the visualMap will target all series that reference
    /// the dataset (v1: still restricted to at most one visualMap per series).
    pub dataset: Option<DatasetId>,
    /// Explicit series targets.
    pub series: Vec<SeriesId>,
    /// Input dimension (dataset field id).
    pub field: FieldId,
    /// Full domain used for bucket assignment.
    pub domain: (f64, f64),
    /// Optional initial selected range (used for inRange/outOfRange classification).
    pub initial_range: Option<(f64, f64)>,
    /// Optional initial piecewise selection mask. Bit `i` selects bucket `i` (v1: up to 64 buckets).
    ///
    /// When `None`, all buckets are treated as selected.
    pub initial_piece_mask: Option<u64>,
    /// Optional range mapping for point radius *multipliers* (unitless, adapter-defined base radius).
    ///
    /// v1:
    /// - applied only to point marks (scatter),
    /// - implemented via bucketized batches (no per-item attributes).
    pub point_radius_mul_range: Option<(f32, f32)>,
    /// Optional stroke width range (in pixels).
    ///
    /// v1:
    /// - applied to scatter point borders and bar borders (bucketized batches),
    /// - not yet supported for polyline marks (line/area/band) because v1 does not split paths per bucket.
    pub stroke_width_range: Option<(f32, f32)>,
    /// Optional opacity multiplier range (unitless, 0..=1) applied to in-range buckets.
    ///
    /// v1:
    /// - computed per bucket (no per-item attributes),
    /// - composed with `out_of_range_opacity` for out-of-range buckets.
    pub opacity_mul_range: Option<(f32, f32)>,
    /// Bounded bucket count used for v1 batch-friendly rendering.
    pub buckets: u16,
    /// Opacity multiplier applied to out-of-range items.
    pub out_of_range_opacity: f32,
}

impl Default for VisualMapSpec {
    fn default() -> Self {
        Self {
            id: VisualMapId::new(0),
            mode: VisualMapMode::Continuous,
            dataset: None,
            series: Vec::default(),
            field: FieldId::new(0),
            domain: (0.0, 1.0),
            initial_range: None,
            initial_piece_mask: None,
            point_radius_mul_range: None,
            stroke_width_range: None,
            opacity_mul_range: None,
            buckets: 8,
            out_of_range_opacity: 0.25,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisPointerSpec {
    pub enabled: bool,
    pub trigger: AxisPointerTrigger,
    pub pointer_type: AxisPointerType,
    pub label: AxisPointerLabelSpec,
    /// When true, snapping is enabled:
    /// - `trigger=Item`: the crosshair snaps to the nearest hit point.
    /// - `trigger=Axis`: the crosshair aligns its axis coordinate to a nearest sample on the
    ///   trigger axis (P0: uses the first visible series as the snap reference).
    pub snap: bool,
    /// For `trigger=Item`, this is the maximum distance (in pixels) to activate the pointer/tooltip.
    /// For `trigger=Axis`, this only gates whether `AxisPointerOutput.hit` is populated (marker dot, snap anchor).
    pub trigger_distance_px: f32,
    /// Minimum pointer movement (in pixels) to recompute hit testing.
    pub throttle_px: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisPointerTrigger {
    /// Similar to ECharts tooltip trigger="item": show details for a single series hit.
    Item,
    /// Similar to ECharts tooltip trigger="axis": show values for all visible series at the same X.
    #[default]
    Axis,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisPointerType {
    #[default]
    Line,
    /// Highlights the active category band in the plot region.
    ///
    /// v1:
    /// - only emitted for category trigger axes,
    /// - computed in px space (`AxisPointerOutput.shadow_rect_px`) for adapter-friendly rendering.
    Shadow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisPointerLabelSpec {
    pub show: bool,
    /// Optional label template (string formatter).
    ///
    /// Supported placeholders (v1):
    /// - `{value}`: formatted axis value for the trigger axis.
    /// - `{axis_name}`: axis display name (empty when unset).
    ///
    /// v1 does not support callback-based formatters (for wasm portability and deterministic serialization).
    pub template: String,
}

impl Default for AxisPointerLabelSpec {
    fn default() -> Self {
        Self {
            show: false,
            template: "{value}".to_string(),
        }
    }
}

impl Default for AxisPointerSpec {
    fn default() -> Self {
        Self {
            enabled: true,
            trigger: AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: AxisPointerLabelSpec::default(),
            snap: false,
            trigger_distance_px: 12.0,
            throttle_px: 0.75,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TooltipSpecV1 {
    /// Template for the first axis row in `trigger=Axis` tooltips.
    ///
    /// Supported placeholders:
    /// - `{label}`: axis label (e.g. `x (Time)`),
    /// - `{value}`: formatted axis value.
    pub axis_line_template: String,
    /// Template for series rows in tooltips.
    ///
    /// Supported placeholders:
    /// - `{label}`: series label (e.g. `A`),
    /// - `{value}`: formatted series value.
    pub series_line_template: String,
    /// Controls whether `trigger=Item` tooltips include the axis row.
    ///
    /// v1 behavior:
    /// - `Auto`: show the axis row unless `axisPointer.label.show` is enabled.
    /// - `Show`: always show the axis row.
    /// - `Hide`: never show the axis row.
    #[cfg_attr(feature = "serde", serde(default))]
    pub item_axis_line: TooltipItemAxisLineMode,
    /// Placeholder used when a series cannot be sampled (missing/NaN/out-of-range).
    pub missing_value: String,
    /// Template used for range values (band-like series).
    ///
    /// Supported placeholders:
    /// - `{min}`: formatted low value,
    /// - `{max}`: formatted high value.
    pub range_template: String,
    /// Optional fixed decimal precision for `AxisScale::Value` values.
    ///
    /// This does not apply to category/time axes.
    pub value_decimals: Option<u8>,
    /// When `value_decimals` is set, remove trailing zeros and the trailing decimal point.
    pub trim_trailing_zeros: bool,
    /// Optional per-series overrides applied to series rows.
    pub series_overrides: Vec<TooltipSeriesOverrideV1>,
}

impl Default for TooltipSpecV1 {
    fn default() -> Self {
        Self {
            axis_line_template: "{label}: {value}".to_string(),
            series_line_template: "{label}: {value}".to_string(),
            // ECharts-aligned default: item trigger tooltips usually do not include an axis row
            // (axis values are typically shown via axisPointer labels when enabled).
            item_axis_line: TooltipItemAxisLineMode::Hide,
            missing_value: "-".to_string(),
            range_template: "{min} .. {max}".to_string(),
            value_decimals: None,
            trim_trailing_zeros: true,
            series_overrides: Vec::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum TooltipItemAxisLineMode {
    #[default]
    Auto,
    Show,
    Hide,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TooltipSeriesOverrideV1 {
    pub series: SeriesId,
    pub series_line_template: Option<String>,
    pub missing_value: Option<String>,
    pub range_template: Option<String>,
    pub value_decimals: Option<u8>,
    pub trim_trailing_zeros: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SeriesKind {
    Line,
    Area,
    /// A filled band between `encode.y` (low) and `encode.y2` (high).
    Band,
    Bar,
    /// A point series (ECharts `series.scatter`-like).
    ///
    /// v1 semantics:
    /// - rendering uses point marks (not connected by segments),
    /// - large-data mode may downsample to a pixel-driven budget.
    Scatter,
}

/// Large-data / progressive rendering knobs (v1 subset, ECharts-inspired).
///
/// Notes:
/// - These knobs do not aim for option-schema parity with ECharts; they are a policy surface.
/// - `delinea` is budgeted and incremental by design; `progressive` here is an additional hint to
///   intentionally spread work across multiple `ChartEngine::step` calls, even when the caller
///   provides a large `WorkBudget`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesLodSpecV1 {
    /// ECharts-like `large` switch.
    ///
    /// - `None` uses the engine defaults (typically threshold-based).
    /// - `Some(true)` forces large-mode eligibility (still gated by thresholds).
    /// - `Some(false)` disables large-mode for the series.
    pub large: Option<bool>,
    /// Threshold for enabling large-mode (ECharts `largeThreshold`-like).
    pub large_threshold: Option<u32>,
    /// Maximum work per frame when progressive mode is active (ECharts `progressive`-like).
    ///
    /// This value is interpreted as a cap on processed data items per `ChartEngine::step` for the
    /// series when `visible_len >= progressive_threshold`.
    pub progressive: Option<u32>,
    /// Threshold for enabling progressive mode (ECharts `progressiveThreshold`-like).
    pub progressive_threshold: Option<u32>,
}

/// Bar width specification (ECharts-inspired).
///
/// ECharts semantics:
/// - number: pixels,
/// - percent string (e.g. `"30%"`): fraction of category band width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarWidthSpec {
    Px(f64),
    Band(f64),
}

/// Bar layout parameters (ECharts-inspired).
///
/// v1:
/// - `bar_width` supports ECharts-style px/percent, but is currently only used for category axes.
/// - `bar_gap` and `bar_category_gap` are expressed as ratios (percent strings are accepted when
///   `serde` is enabled).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BarLayoutSpec {
    /// Bar width (`series.barWidth`).
    ///
    /// - `None` means auto width based on group slot count and gaps.
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "deserialize_bar_width_opt")
    )]
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_bar_width_opt"))]
    pub bar_width: Option<BarWidthSpec>,
    /// Gap between adjacent bar slots, expressed as a multiple of `bar_width`.
    ///
    /// Example: `0.3` means the gap is `0.3 * bar_width`.
    ///
    /// Negative values are allowed to overlap bars (ECharts `barGap`-like semantics):
    /// `-1.0` means "full overlap" (all slots share the same center).
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "deserialize_ratio_opt")
    )]
    pub bar_gap: Option<f64>,
    /// Outer padding inside each category band, expressed as a fraction of band width.
    ///
    /// Example: `0.2` means 20% of the band is reserved as padding.
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "deserialize_ratio_opt")
    )]
    pub bar_category_gap: Option<f64>,
}

#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(untagged)]
enum BarWidthInput {
    Num(f64),
    Str(String),
}

#[cfg(feature = "serde")]
fn deserialize_bar_width_opt<'de, D>(deserializer: D) -> Result<Option<BarWidthSpec>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let input = Option::<BarWidthInput>::deserialize(deserializer)?;
    let Some(input) = input else {
        return Ok(None);
    };
    match input {
        BarWidthInput::Num(v) => Ok(Some(BarWidthSpec::Px(v))),
        BarWidthInput::Str(s) => {
            let Some(r) = parse_percent_ratio(&s) else {
                return Err(serde::de::Error::custom("invalid barWidth percent string"));
            };
            Ok(Some(BarWidthSpec::Band(r)))
        }
    }
}

#[cfg(feature = "serde")]
fn serialize_bar_width_opt<S>(
    value: &Option<BarWidthSpec>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        None => serializer.serialize_none(),
        Some(BarWidthSpec::Px(v)) => serializer.serialize_some(v),
        Some(BarWidthSpec::Band(r)) => serializer.serialize_some(&format!("{:.6}%", r * 100.0)),
    }
}

#[cfg(feature = "serde")]
#[derive(Deserialize)]
#[serde(untagged)]
enum RatioInput {
    Num(f64),
    Str(String),
}

#[cfg(feature = "serde")]
fn deserialize_ratio_opt<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let input = Option::<RatioInput>::deserialize(deserializer)?;
    let Some(input) = input else {
        return Ok(None);
    };
    match input {
        RatioInput::Num(v) => Ok(Some(v)),
        RatioInput::Str(s) => parse_percent_ratio(&s)
            .ok_or_else(|| serde::de::Error::custom("invalid percent string")),
    }
}

#[cfg(feature = "serde")]
fn parse_percent_ratio(s: &str) -> Option<f64> {
    let s = s.trim();
    let (number, suffix) = s.split_at(s.len().saturating_sub(1));
    if suffix != "%" {
        return None;
    }
    let v: f64 = number.trim().parse().ok()?;
    if v.is_finite() { Some(v / 100.0) } else { None }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum StackStrategy {
    /// Match ECharts default: stack positive and negative values separately.
    #[default]
    SameSign,
    /// Stack all values together (including mixed sign).
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesEncode {
    pub x: FieldId,
    pub y: FieldId,
    pub y2: Option<FieldId>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AreaBaseline {
    /// Close the area to the minimum of the current Y window.
    #[default]
    AxisMin,
    /// Close the area to Y=0 in data space (clamped to the current Y window).
    Zero,
    /// Close the area to a fixed Y value in data space (clamped to the current Y window).
    Value(f64),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesSpec {
    pub id: SeriesId,
    /// Optional display name (used by tooltips and legends).
    pub name: Option<String>,
    pub kind: SeriesKind,
    pub dataset: DatasetId,
    pub encode: SeriesEncode,
    pub x_axis: AxisId,
    pub y_axis: AxisId,
    /// Stack group id (ECharts `series.stack`).
    ///
    /// v1: only supported for `SeriesKind::Line`, `SeriesKind::Area`, and `SeriesKind::Bar`.
    pub stack: Option<StackId>,
    /// Stacking strategy (ECharts `series.stackStrategy`).
    pub stack_strategy: StackStrategy,
    /// Bar layout parameters (only used when `kind == Bar`).
    pub bar_layout: BarLayoutSpec,
    /// Area baseline configuration (only used when `kind == Area`).
    pub area_baseline: Option<AreaBaseline>,
    /// LOD / progressive rendering knobs (v1 subset).
    pub lod: Option<SeriesLodSpecV1>,
}
