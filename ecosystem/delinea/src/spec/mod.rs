use fret_core::Rect;

use crate::ids::{AxisId, ChartId, DataZoomId, DatasetId, FieldId, GridId, SeriesId, StackId};
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
    pub axis_pointer: Option<AxisPointerSpec>,
    pub series: Vec<SeriesSpec>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetSpec {
    pub id: DatasetId,
    pub fields: Vec<FieldSpec>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataZoomXSpec {
    pub id: DataZoomId,
    pub axis: AxisId,
    pub filter_mode: FilterMode,
}

impl Default for DataZoomXSpec {
    fn default() -> Self {
        Self {
            id: DataZoomId::new(0),
            axis: AxisId::new(0),
            filter_mode: FilterMode::Filter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisPointerSpec {
    pub enabled: bool,
    pub trigger: AxisPointerTrigger,
    /// When true, crosshair snaps to the nearest hit point (P0: single series hit).
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

impl Default for AxisPointerSpec {
    fn default() -> Self {
        Self {
            enabled: true,
            trigger: AxisPointerTrigger::Axis,
            snap: false,
            trigger_distance_px: 12.0,
            throttle_px: 0.75,
        }
    }
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
}
