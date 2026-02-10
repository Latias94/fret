//! Minimal ECharts option adapter.
//!
//! This module is intentionally narrow: it targets a small, high-leverage subset of ECharts
//! and translates it into a `delinea::ChartSpec` + datasets that can be inserted into a
//! `delinea::engine::ChartEngine`.

use std::collections::{BTreeMap, BTreeSet};

use delinea::Action;
use delinea::data::{Column, DataTable};
use delinea::engine::window::DataWindow;
use delinea::ids::{AxisId, ChartId, DatasetId, FieldId, GridId, SeriesId, VisualMapId};
use delinea::scale::{AxisScale, CategoryAxisScale, ValueAxisScale};
use delinea::spec::{
    AxisKind, AxisSpec, ChartSpec, DataZoomXSpec, DataZoomYSpec, DatasetFilterSpecV1,
    DatasetSortOrder, DatasetSortSpecV1, DatasetSpec, DatasetTransformSpecV1, FieldSpec,
    FilterMode, GridSpec, SeriesEncode, SeriesKind, SeriesLodSpecV1, SeriesSpec, TooltipSpecV1,
    VisualMapMode, VisualMapSpec,
};

use serde::Deserialize;

pub type Result<T> = std::result::Result<T, EchartsError>;

#[derive(Debug)]
pub enum EchartsError {
    Json(serde_json::Error),
    Unsupported(&'static str),
    Invalid(&'static str),
}

impl std::fmt::Display for EchartsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(err) => write!(f, "failed to parse ECharts JSON: {err}"),
            Self::Unsupported(msg) => write!(f, "unsupported ECharts feature: {msg}"),
            Self::Invalid(msg) => write!(f, "invalid ECharts option: {msg}"),
        }
    }
}

impl std::error::Error for EchartsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for EchartsError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Debug, Clone)]
pub struct TranslatedChart {
    pub spec: ChartSpec,
    pub datasets: Vec<(DatasetId, DataTable)>,
    pub actions: Vec<Action>,
}

impl TranslatedChart {
    pub fn primary_dataset_id(&self) -> Option<DatasetId> {
        self.datasets.first().map(|(id, _)| *id)
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsOption {
    #[serde(default)]
    x_axis: Option<AxisOrAxes>,
    #[serde(default)]
    y_axis: Option<AxisOrAxes>,
    #[serde(default)]
    dataset: Option<DatasetOrDatasets>,
    #[serde(default)]
    grid: Option<GridOrGrids>,
    #[serde(default)]
    series: Vec<EchartsSeries>,
    #[serde(default)]
    tooltip: Option<EchartsTooltip>,
    #[serde(default)]
    data_zoom: Option<DataZoomOrDataZooms>,
    #[serde(default)]
    visual_map: Option<VisualMapOrVisualMaps>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum VisualMapOrVisualMaps {
    One(EchartsVisualMap),
    Many(Vec<EchartsVisualMap>),
}

impl VisualMapOrVisualMaps {
    fn to_vec(&self) -> Vec<EchartsVisualMap> {
        match self {
            Self::One(v) => vec![v.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsVisualMap {
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    series_index: Option<SeriesIndexOrIndices>,
    #[serde(default)]
    dimension: Option<EncodeDim>,
    #[serde(default)]
    min: Option<f64>,
    #[serde(default)]
    max: Option<f64>,
    #[serde(default)]
    range: Option<[f64; 2]>,
    #[serde(default)]
    split_number: Option<u16>,
    #[serde(default)]
    in_range: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default)]
    out_of_range: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SeriesIndexOrIndices {
    One(usize),
    Many(Vec<usize>),
}

impl SeriesIndexOrIndices {
    fn to_vec(&self) -> Vec<usize> {
        match self {
            Self::One(v) => vec![*v],
            Self::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AxisOrAxes {
    One(EchartsAxis),
    Many(Vec<EchartsAxis>),
}

impl AxisOrAxes {
    fn first(&self) -> Option<&EchartsAxis> {
        match self {
            Self::One(a) => Some(a),
            Self::Many(v) => v.first(),
        }
    }

    fn to_vec(&self) -> Vec<EchartsAxis> {
        match self {
            Self::One(a) => vec![a.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsAxis {
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    data: Option<Vec<String>>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    grid_index: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GridOrGrids {
    One(EchartsGrid),
    Many(Vec<EchartsGrid>),
}

impl GridOrGrids {
    fn to_len(&self) -> usize {
        match self {
            Self::One(_) => 1,
            Self::Many(v) => v.len(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsGrid {}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsTooltip {
    #[serde(default)]
    show: Option<bool>,
    #[serde(default)]
    trigger: Option<String>,
    #[serde(default)]
    axis_pointer: Option<EchartsAxisPointer>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsAxisPointer {
    #[serde(default, rename = "type")]
    kind: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsSeries {
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    data: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    dataset_index: Option<usize>,
    #[serde(default)]
    x_axis_index: Option<usize>,
    #[serde(default)]
    y_axis_index: Option<usize>,
    #[serde(default)]
    encode: Option<EchartsSeriesEncode>,
    #[serde(default)]
    large: Option<bool>,
    #[serde(default)]
    large_threshold: Option<u32>,
    #[serde(default)]
    progressive: Option<u32>,
    #[serde(default)]
    progressive_threshold: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsSeriesEncode {
    #[serde(default)]
    x: Option<EncodeDim>,
    #[serde(default)]
    y: Option<EncodeDim>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum EncodeDim {
    Index(usize),
    Name(String),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DatasetOrDatasets {
    One(EchartsDataset),
    Many(Vec<EchartsDataset>),
}

impl DatasetOrDatasets {
    fn to_vec(&self) -> Vec<EchartsDataset> {
        match self {
            Self::One(d) => vec![d.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsDataset {
    #[serde(default)]
    dimensions: Option<Vec<String>>,
    #[serde(default)]
    source: Option<DatasetSource>,
    #[serde(default)]
    from_dataset_index: Option<usize>,
    #[serde(default)]
    transform: Option<DatasetTransformOrTransforms>,
    #[serde(default)]
    source_header: Option<DatasetSourceHeader>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum DatasetTransformOrTransforms {
    One(EchartsDatasetTransform),
    Many(Vec<EchartsDatasetTransform>),
}

impl DatasetTransformOrTransforms {
    fn to_vec(&self) -> Vec<EchartsDatasetTransform> {
        match self {
            Self::One(v) => vec![v.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsDatasetTransform {
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum DatasetSource {
    Rows(Vec<Vec<serde_json::Value>>),
    Objects(Vec<serde_json::Map<String, serde_json::Value>>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum DatasetSourceHeader {
    Num(u8),
    Str(String),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DataZoomOrDataZooms {
    One(EchartsDataZoom),
    Many(Vec<EchartsDataZoom>),
}

impl DataZoomOrDataZooms {
    fn to_vec(&self) -> Vec<EchartsDataZoom> {
        match self {
            Self::One(v) => vec![v.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EchartsDataZoom {
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    orient: Option<String>,
    #[serde(default)]
    x_axis_index: Option<AxisIndexOrIndices>,
    #[serde(default)]
    y_axis_index: Option<AxisIndexOrIndices>,
    #[serde(default)]
    filter_mode: Option<String>,
    #[serde(default)]
    range_mode: Option<EchartsDataZoomRangeMode>,
    #[serde(default)]
    start: Option<f64>,
    #[serde(default)]
    end: Option<f64>,
    #[serde(default)]
    start_value: Option<serde_json::Value>,
    #[serde(default)]
    end_value: Option<serde_json::Value>,
    #[serde(default)]
    min_value_span: Option<f64>,
    #[serde(default)]
    max_value_span: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum EchartsDataZoomRangeMode {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RangeMode {
    Percent,
    Value,
}

impl EchartsDataZoomRangeMode {
    fn parse_homogeneous(&self) -> Result<RangeMode> {
        let mode_str = match self {
            Self::One(v) => v.as_str(),
            Self::Many(v) => {
                let first = v.first().map(|s| s.as_str()).unwrap_or("");
                if v.iter().any(|s| s.as_str() != first) {
                    return Err(EchartsError::Unsupported(
                        "dataZoom.rangeMode mixed per bound (v1 subset)",
                    ));
                }
                first
            }
        };

        match mode_str {
            "percent" => Ok(RangeMode::Percent),
            "value" => Ok(RangeMode::Value),
            _ => Err(EchartsError::Unsupported("dataZoom.rangeMode")),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum AxisIndexOrIndices {
    One(usize),
    Many(Vec<usize>),
}

impl AxisIndexOrIndices {
    fn to_vec(&self) -> Vec<usize> {
        match self {
            Self::One(v) => vec![*v],
            Self::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum XValue {
    Number(f64),
    Category(String),
}

#[derive(Debug, Clone)]
struct ParsedSeries {
    kind: SeriesKind,
    name: Option<String>,
    points: Vec<(XValue, f64)>,
    lod: Option<SeriesLodSpecV1>,
}

#[derive(Debug, Clone)]
enum XLayoutMode {
    Value {
        index_by_bits: BTreeMap<u64, usize>,
    },
    Category {
        index_by_category: BTreeMap<String, usize>,
    },
}

#[derive(Debug, Clone)]
struct XLayout {
    scale: AxisScale,
    values: Vec<f64>,
    mode: XLayoutMode,
}

pub fn translate_json_str(option_json: &str) -> Result<TranslatedChart> {
    let option: EchartsOption = serde_json::from_str(option_json)?;
    translate_option(&option)
}

fn translate_option(option: &EchartsOption) -> Result<TranslatedChart> {
    if option.series.is_empty() {
        return Err(EchartsError::Invalid("missing `series`"));
    }

    if option.dataset.is_some() {
        return translate_option_with_dataset(option);
    }
    if option.visual_map.is_some() {
        return Err(EchartsError::Unsupported(
            "visualMap without dataset (v1 subset)",
        ));
    }

    let grid_id = GridId::new(1);
    let dataset_id = DatasetId::new(1);
    let x_axis_id = AxisId::new(1);
    let y_axis_id = AxisId::new(2);
    let x_field = FieldId::new(1);

    let x_axis_opt = option.x_axis.as_ref().and_then(|a| a.first());
    let y_axis_opt = option.y_axis.as_ref().and_then(|a| a.first());

    let series = extract_series(option)?;
    let x_layout = build_x_layout(x_axis_opt, &series)?;
    let y_scale = build_y_scale(y_axis_opt)?;

    let mut table = DataTable::default();
    table.push_column(Column::F64(x_layout.values.clone()));

    let mut fields = vec![FieldSpec {
        id: x_field,
        column: 0,
    }];
    for (series_index, s) in series.iter().enumerate() {
        let y_field = FieldId::new((series_index as u64).saturating_add(2));
        fields.push(FieldSpec {
            id: y_field,
            column: 1 + series_index,
        });

        let y_values = build_y_values_for_series(&x_layout, s)?;
        table.push_column(Column::F64(y_values));
    }

    let datasets = vec![(dataset_id, table)];

    let tooltip = translate_tooltip(option.tooltip.as_ref())?;

    let mut spec = ChartSpec {
        id: ChartId::new(1),
        viewport: None,
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields,

            from: None,
            transforms: Vec::new(),
        }],
        grids: vec![GridSpec { id: grid_id }],
        axes: vec![
            AxisSpec {
                id: x_axis_id,
                name: x_axis_opt.and_then(|a| a.name.clone()),
                kind: AxisKind::X,
                grid: grid_id,
                position: None,
                scale: x_layout.scale.clone(),
                range: None,
            },
            AxisSpec {
                id: y_axis_id,
                name: y_axis_opt.and_then(|a| a.name.clone()),
                kind: AxisKind::Y,
                grid: grid_id,
                position: None,
                scale: y_scale,
                range: None,
            },
        ],
        data_zoom_x: Vec::new(),
        data_zoom_y: Vec::new(),
        tooltip,
        axis_pointer: Some(Default::default()),
        visual_maps: Vec::new(),
        series: Vec::new(),
    };

    for (series_index, s) in series.iter().enumerate() {
        let series_id = SeriesId::new((series_index as u64).saturating_add(1));
        let y_field = FieldId::new((series_index as u64).saturating_add(2));
        spec.series.push(SeriesSpec {
            id: series_id,
            name: s.name.clone(),
            kind: s.kind,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis: x_axis_id,
            y_axis: y_axis_id,
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: s.lod,
        });
    }

    let axis_grids = compute_axis_grids(&spec);
    let (data_zoom_x, data_zoom_y, actions) = translate_data_zoom_v1(
        option.data_zoom.as_ref(),
        &[x_axis_id],
        &[y_axis_id],
        &axis_grids,
    )?;
    spec.data_zoom_x = data_zoom_x;
    spec.data_zoom_y = data_zoom_y;

    Ok(TranslatedChart {
        spec,
        datasets,
        actions,
    })
}

fn translate_option_with_dataset(option: &EchartsOption) -> Result<TranslatedChart> {
    let grid_count_from_option = option.grid.as_ref().map(|g| g.to_len()).unwrap_or(0);

    let datasets = option
        .dataset
        .as_ref()
        .map(|d| d.to_vec())
        .unwrap_or_default();
    if datasets.is_empty() {
        return Err(EchartsError::Invalid("missing `dataset`"));
    }

    let x_axes: Vec<EchartsAxis> = option
        .x_axis
        .as_ref()
        .map(|a| a.to_vec())
        .unwrap_or_else(|| vec![EchartsAxis::default()]);
    let y_axes: Vec<EchartsAxis> = option
        .y_axis
        .as_ref()
        .map(|a| a.to_vec())
        .unwrap_or_else(|| vec![EchartsAxis::default()]);

    let mut required_grid_count = grid_count_from_option.max(1);
    for axis in x_axes.iter().chain(y_axes.iter()) {
        if let Some(i) = axis.grid_index {
            required_grid_count = required_grid_count.max(i.saturating_add(1));
        }
    }

    let grid_ids: Vec<GridId> = (0..required_grid_count)
        .map(|i| GridId::new(1 + i as u64))
        .collect();
    let grids: Vec<GridSpec> = grid_ids.iter().copied().map(|id| GridSpec { id }).collect();

    let x_ids: Vec<AxisId> = (0..x_axes.len())
        .map(|i| AxisId::new(1 + i as u64))
        .collect();
    let y_ids: Vec<AxisId> = (0..y_axes.len())
        .map(|i| AxisId::new(1 + x_axes.len() as u64 + i as u64))
        .collect();

    let mut axes: Vec<AxisSpec> = Vec::with_capacity(x_axes.len() + y_axes.len());

    for (i, axis) in x_axes.iter().enumerate() {
        let grid_index = axis.grid_index.unwrap_or(0);
        let grid = *grid_ids
            .get(grid_index)
            .ok_or(EchartsError::Invalid("xAxis.gridIndex out of range"))?;
        let axis_type = axis.kind.as_deref();
        axes.push(AxisSpec {
            id: x_ids[i],
            name: axis.name.clone(),
            kind: AxisKind::X,
            grid,
            position: None,
            scale: match axis_type {
                Some("time") => AxisScale::Time(Default::default()),
                Some("category") => AxisScale::Value(ValueAxisScale),
                None | Some("value") => AxisScale::Value(ValueAxisScale),
                Some(_) => return Err(EchartsError::Unsupported("xAxis.type")),
            },
            range: None,
        });
    }

    for (i, axis) in y_axes.iter().enumerate() {
        let grid_index = axis.grid_index.unwrap_or(0);
        let grid = *grid_ids
            .get(grid_index)
            .ok_or(EchartsError::Invalid("yAxis.gridIndex out of range"))?;
        let axis_type = axis.kind.as_deref();
        if matches!(axis_type, Some("category")) {
            return Err(EchartsError::Unsupported(
                "yAxis.type = 'category' with dataset (v1 subset)",
            ));
        }
        axes.push(AxisSpec {
            id: y_ids[i],
            name: axis.name.clone(),
            kind: AxisKind::Y,
            grid,
            position: None,
            scale: match axis_type {
                Some("time") => AxisScale::Time(Default::default()),
                None | Some("value") => AxisScale::Value(ValueAxisScale),
                Some(_) => return Err(EchartsError::Unsupported("yAxis.type")),
            },
            range: None,
        });
    }

    let tooltip = translate_tooltip(option.tooltip.as_ref())?;

    let mut out_datasets: Vec<(DatasetId, DataTable)> = Vec::with_capacity(datasets.len());
    let mut dataset_specs: Vec<DatasetSpec> = Vec::with_capacity(datasets.len());
    let mut dataset_fields_by_index: Vec<Vec<FieldId>> = Vec::with_capacity(datasets.len());
    let mut dataset_dimensions_by_index: Vec<Vec<String>> = Vec::with_capacity(datasets.len());
    let mut dataset_rows_by_index: Vec<Option<Vec<Vec<serde_json::Value>>>> =
        vec![None; datasets.len()];
    let mut root_index_by_dataset_index: Vec<usize> = vec![0; datasets.len()];
    let mut table_index_by_dataset_index: Vec<Option<usize>> = vec![None; datasets.len()];

    let mut next_field_id = 1u64;

    for (dataset_index, ds) in datasets.iter().enumerate() {
        let dataset_id = DatasetId::new((dataset_index as u64).saturating_add(1));

        if ds.source.is_some() {
            let parsed = parse_raw_dataset_v1(ds, &mut next_field_id)?;
            let table_index = out_datasets.len();
            out_datasets.push((dataset_id, parsed.table));
            dataset_rows_by_index[dataset_index] = Some(parsed.rows);
            root_index_by_dataset_index[dataset_index] = dataset_index;
            table_index_by_dataset_index[dataset_index] = Some(table_index);
            dataset_specs.push(DatasetSpec {
                id: dataset_id,
                fields: parsed.fields,
                from: None,
                transforms: Vec::new(),
            });
            dataset_fields_by_index.push(parsed.field_ids);
            dataset_dimensions_by_index.push(parsed.dimensions);
            continue;
        }

        let base_index = ds.from_dataset_index.ok_or(EchartsError::Invalid(
            "dataset.source is required unless dataset.fromDatasetIndex is set (v1 subset)",
        ))?;
        if base_index >= dataset_index {
            return Err(EchartsError::Unsupported(
                "dataset.fromDatasetIndex must refer to a previous dataset (v1 subset)",
            ));
        }
        let base_dimensions = dataset_dimensions_by_index
            .get(base_index)
            .ok_or(EchartsError::Invalid(
                "dataset.fromDatasetIndex out of range",
            ))?
            .clone();
        let base_field_ids = dataset_fields_by_index
            .get(base_index)
            .ok_or(EchartsError::Invalid(
                "dataset.fromDatasetIndex out of range",
            ))?
            .clone();
        let base_dataset_id = DatasetId::new((base_index as u64).saturating_add(1));
        root_index_by_dataset_index[dataset_index] = *root_index_by_dataset_index
            .get(base_index)
            .ok_or(EchartsError::Invalid(
            "dataset.fromDatasetIndex out of range",
        ))?;

        if ds.dimensions.is_some() {
            return Err(EchartsError::Unsupported(
                "dataset.dimensions with fromDatasetIndex (v1 subset)",
            ));
        }

        let width = base_dimensions.len();
        if width == 0 {
            return Err(EchartsError::Invalid("dataset.dimensions is empty"));
        }

        let mut fields: Vec<FieldSpec> = Vec::with_capacity(width);
        for (column, id) in base_field_ids.iter().copied().enumerate() {
            fields.push(FieldSpec { id, column });
        }

        let mut transforms: Vec<DatasetTransformSpecV1> = Vec::new();
        if let Some(ts) = ds.transform.as_ref() {
            for t in ts.to_vec() {
                let kind = t.kind.as_deref().unwrap_or_default();
                let cfg = as_object(t.config.as_ref())?.ok_or(EchartsError::Invalid(
                    "dataset.transform.config is required (v1 subset)",
                ))?;
                let dim = parse_transform_dimension(cfg.get("dimension"))?;
                let col = resolve_encode_dim(&dim, Some(&base_dimensions))?;
                let field = *base_field_ids.get(col).ok_or(EchartsError::Invalid(
                    "dataset.transform.config.dimension out of range",
                ))?;

                match kind {
                    "filter" => {
                        transforms.push(DatasetTransformSpecV1::Filter(DatasetFilterSpecV1 {
                            field,
                            gte: cfg.get("gte").map(parse_f64_value),
                            gt: cfg.get("gt").map(parse_f64_value),
                            lte: cfg.get("lte").map(parse_f64_value),
                            lt: cfg.get("lt").map(parse_f64_value),
                            eq: cfg.get("eq").map(parse_f64_value),
                            ne: cfg.get("ne").map(parse_f64_value),
                        }));
                    }
                    "sort" => {
                        let order = cfg.get("order").and_then(|v| v.as_str()).unwrap_or("asc");
                        let order = match order {
                            "asc" => DatasetSortOrder::Asc,
                            "desc" => DatasetSortOrder::Desc,
                            _ => {
                                return Err(EchartsError::Unsupported(
                                    "dataset.transform.config.order (only asc/desc)",
                                ));
                            }
                        };
                        transforms.push(DatasetTransformSpecV1::Sort(DatasetSortSpecV1 {
                            field,
                            order,
                        }));
                    }
                    _ => return Err(EchartsError::Unsupported("dataset.transform.type")),
                }
            }
        }

        dataset_specs.push(DatasetSpec {
            id: dataset_id,
            fields,
            from: Some(base_dataset_id),
            transforms,
        });
        dataset_fields_by_index.push(base_field_ids);
        dataset_dimensions_by_index.push(base_dimensions);
    }

    #[derive(Debug, Clone)]
    struct SeriesDraft {
        series_index: usize,
        kind: SeriesKind,
        name: Option<String>,
        dataset_index: usize,
        x_col: usize,
        y_col: usize,
        x_axis_index: usize,
        y_axis_index: usize,
        lod: Option<SeriesLodSpecV1>,
    }

    let mut drafts: Vec<SeriesDraft> = Vec::with_capacity(option.series.len());

    for (series_index, series) in option.series.iter().enumerate() {
        if series.data.is_some() {
            return Err(EchartsError::Unsupported(
                "series.data with dataset (v1 subset)",
            ));
        }

        let kind = match series.kind.as_deref().unwrap_or("line") {
            "line" => SeriesKind::Line,
            "scatter" => SeriesKind::Scatter,
            "bar" => SeriesKind::Bar,
            _ => return Err(EchartsError::Unsupported("series.type")),
        };

        let dataset_index = series.dataset_index.unwrap_or(0);
        let Some(field_ids) = dataset_fields_by_index.get(dataset_index) else {
            return Err(EchartsError::Invalid("series.datasetIndex out of range"));
        };

        let encode_x = series
            .encode
            .as_ref()
            .and_then(|e| e.x.as_ref())
            .cloned()
            .unwrap_or(EncodeDim::Index(0));
        let encode_y = series
            .encode
            .as_ref()
            .and_then(|e| e.y.as_ref())
            .cloned()
            .unwrap_or(EncodeDim::Index(1));

        let x_col = resolve_encode_dim(
            &encode_x,
            dataset_dimensions_by_index
                .get(dataset_index)
                .map(|v| v.as_slice()),
        )?;
        let y_col = resolve_encode_dim(
            &encode_y,
            dataset_dimensions_by_index
                .get(dataset_index)
                .map(|v| v.as_slice()),
        )?;

        if x_col >= field_ids.len() {
            return Err(EchartsError::Invalid(
                "series.encode.x column out of range for dataset",
            ));
        }
        if y_col >= field_ids.len() {
            return Err(EchartsError::Invalid(
                "series.encode.y column out of range for dataset",
            ));
        }

        let x_axis_index = series.x_axis_index.unwrap_or(0);
        let y_axis_index = series.y_axis_index.unwrap_or(0);

        let lod = {
            let lod = SeriesLodSpecV1 {
                large: series.large,
                large_threshold: series.large_threshold,
                progressive: series.progressive,
                progressive_threshold: series.progressive_threshold,
            };
            let any = lod.large.is_some()
                || lod.large_threshold.is_some()
                || lod.progressive.is_some()
                || lod.progressive_threshold.is_some();
            any.then_some(lod)
        };

        drafts.push(SeriesDraft {
            series_index,
            kind,
            name: series.name.clone(),
            dataset_index,
            x_col,
            y_col,
            x_axis_index,
            y_axis_index,
            lod,
        });
    }

    fn category_label(v: &serde_json::Value) -> Option<String> {
        match v {
            serde_json::Value::Null => None,
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            other => Some(other.to_string()),
        }
    }

    #[derive(Debug, Clone)]
    struct CategoryAxisPlan {
        categories: Vec<String>,
        explicit_data: bool,
        index_by_label: BTreeMap<String, usize>,
    }

    let mut category_plans: BTreeMap<usize, CategoryAxisPlan> = BTreeMap::new();
    for (axis_index, axis) in x_axes.iter().enumerate() {
        let axis_type = axis.kind.as_deref();
        if !matches!(axis_type, Some("category")) {
            continue;
        }

        let mut categories: Vec<String> = Vec::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let explicit_data = axis.data.as_ref().is_some_and(|d| !d.is_empty());

        if let Some(data) = axis.data.as_ref() {
            for c in data {
                if seen.insert(c.clone()) {
                    categories.push(c.clone());
                }
            }
        }

        if !explicit_data {
            for d in drafts.iter().filter(|d| d.x_axis_index == axis_index) {
                let Some(root_index) = root_index_by_dataset_index.get(d.dataset_index).copied()
                else {
                    continue;
                };
                let Some(rows) = dataset_rows_by_index
                    .get(root_index)
                    .and_then(|r| r.as_ref())
                else {
                    continue;
                };
                for row in rows {
                    let Some(v) = row.get(d.x_col) else {
                        continue;
                    };
                    let Some(label) = category_label(v) else {
                        continue;
                    };
                    if seen.insert(label.clone()) {
                        categories.push(label);
                    }
                }
            }
        }

        if categories.is_empty() {
            return Err(EchartsError::Invalid("category xAxis has no categories"));
        }

        let mut index_by_label: BTreeMap<String, usize> = BTreeMap::new();
        for (i, c) in categories.iter().enumerate() {
            index_by_label.insert(c.clone(), i);
        }

        category_plans.insert(
            axis_index,
            CategoryAxisPlan {
                categories,
                explicit_data,
                index_by_label,
            },
        );
    }

    let mut spec = ChartSpec {
        id: ChartId::new(1),
        viewport: None,
        datasets: dataset_specs,
        grids,
        axes,
        data_zoom_x: Vec::new(),
        data_zoom_y: Vec::new(),
        tooltip,
        axis_pointer: Some(Default::default()),
        visual_maps: Vec::new(),
        series: Vec::new(),
    };

    // Update X axis scales after dataset-driven category planning.
    for (axis_index, _axis) in x_axes.iter().enumerate() {
        let axis_id = x_ids
            .get(axis_index)
            .copied()
            .ok_or(EchartsError::Invalid("xAxis index out of range"))?;

        let Some(target) = spec.axes.iter_mut().find(|a| a.id == axis_id) else {
            continue;
        };

        if let Some(plan) = category_plans.get(&axis_index) {
            target.scale = AxisScale::Category(CategoryAxisScale {
                categories: plan.categories.clone(),
            });
        }
    }

    // For category X axes, create derived numeric X columns that map category -> index.
    let mut derived_x_field_by_key: BTreeMap<(usize, usize, usize), FieldId> = BTreeMap::new();

    for d in &drafts {
        let dataset_index = d.dataset_index;
        let Some(dataset_id_for_series) = spec.datasets.get(dataset_index).map(|ds| ds.id) else {
            return Err(EchartsError::Invalid("series.datasetIndex out of range"));
        };
        let Some(field_ids) = dataset_fields_by_index.get(dataset_index) else {
            return Err(EchartsError::Invalid("series.datasetIndex out of range"));
        };
        let Some(root_index) = root_index_by_dataset_index.get(dataset_index).copied() else {
            return Err(EchartsError::Invalid("series.datasetIndex out of range"));
        };

        let x_axis_index = d.x_axis_index;
        let y_axis_index = d.y_axis_index;
        let x_axis = *x_ids
            .get(x_axis_index)
            .ok_or(EchartsError::Invalid("series.xAxisIndex out of range"))?;
        let y_axis = *y_ids
            .get(y_axis_index)
            .ok_or(EchartsError::Invalid("series.yAxisIndex out of range"))?;

        let base_x_field = field_ids
            .get(d.x_col)
            .copied()
            .ok_or(EchartsError::Invalid(
                "series.encode.x column out of range for dataset",
            ))?;
        let y_field = field_ids
            .get(d.y_col)
            .copied()
            .ok_or(EchartsError::Invalid(
                "series.encode.y column out of range for dataset",
            ))?;

        let x_field = if let Some(plan) = category_plans.get(&x_axis_index) {
            let key = (root_index, d.x_col, x_axis_index);
            if let Some(field) = derived_x_field_by_key.get(&key).copied() {
                field
            } else {
                let Some(table_index) = table_index_by_dataset_index
                    .get(root_index)
                    .and_then(|v| *v)
                else {
                    return Err(EchartsError::Invalid(
                        "category xAxis requires a root dataset with `source` (v1 subset)",
                    ));
                };
                let Some(rows) = dataset_rows_by_index
                    .get(root_index)
                    .and_then(|r| r.as_ref())
                else {
                    return Err(EchartsError::Invalid(
                        "category xAxis requires a root dataset with `source` (v1 subset)",
                    ));
                };

                let (_dataset_id, table) = out_datasets
                    .get_mut(table_index)
                    .ok_or(EchartsError::Invalid("dataset table index out of range"))?;

                let mut codes: Vec<f64> = Vec::with_capacity(rows.len());
                for row in rows {
                    let v = row.get(d.x_col).unwrap_or(&serde_json::Value::Null);
                    let code = if plan.explicit_data {
                        match v {
                            serde_json::Value::Number(n) => n
                                .as_i64()
                                .and_then(|i| (i >= 0).then_some(i as usize))
                                .and_then(|i| (i < plan.categories.len()).then_some(i))
                                .map(|i| i as f64)
                                .unwrap_or(f64::NAN),
                            serde_json::Value::String(s) => plan
                                .index_by_label
                                .get(s.as_str())
                                .copied()
                                .map(|i| i as f64)
                                .unwrap_or(f64::NAN),
                            _ => f64::NAN,
                        }
                    } else {
                        category_label(v)
                            .and_then(|label| plan.index_by_label.get(&label).copied())
                            .map(|i| i as f64)
                            .unwrap_or(f64::NAN)
                    };
                    codes.push(code);
                }

                table.push_column(Column::F64(codes));
                let column = table.column_count().saturating_sub(1);

                let derived_id = FieldId::new(next_field_id);
                next_field_id = next_field_id.saturating_add(1);
                for (i, spec_dataset) in spec.datasets.iter_mut().enumerate() {
                    if root_index_by_dataset_index
                        .get(i)
                        .is_some_and(|root| *root == root_index)
                    {
                        spec_dataset.fields.push(FieldSpec {
                            id: derived_id,
                            column,
                        });
                        if let Some(ids) = dataset_fields_by_index.get_mut(i) {
                            ids.push(derived_id);
                        }
                        if let Some(dims) = dataset_dimensions_by_index.get_mut(i) {
                            dims.push(format!("__category_index_x{}_col{}", x_axis_index, d.x_col));
                        }
                    }
                }
                derived_x_field_by_key.insert(key, derived_id);

                derived_id
            }
        } else {
            base_x_field
        };

        let series_id = SeriesId::new((d.series_index as u64).saturating_add(1));
        spec.series.push(SeriesSpec {
            id: series_id,
            name: d.name.clone(),
            kind: d.kind,
            dataset: dataset_id_for_series,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis,
            y_axis,
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: d.lod,
        });
    }

    spec.visual_maps = translate_visual_maps_v1(
        option.visual_map.as_ref(),
        &spec,
        &out_datasets,
        &dataset_dimensions_by_index,
        &dataset_fields_by_index,
    )?;

    let axis_grids = compute_axis_grids(&spec);
    let (data_zoom_x, data_zoom_y, actions) =
        translate_data_zoom_v1(option.data_zoom.as_ref(), &x_ids, &y_ids, &axis_grids)?;
    spec.data_zoom_x = data_zoom_x;
    spec.data_zoom_y = data_zoom_y;

    Ok(TranslatedChart {
        spec,
        datasets: out_datasets,
        actions,
    })
}

fn translate_tooltip(tooltip: Option<&EchartsTooltip>) -> Result<Option<TooltipSpecV1>> {
    let Some(tooltip) = tooltip else {
        return Ok(Some(TooltipSpecV1::default()));
    };
    if matches!(tooltip.show, Some(false)) {
        return Ok(None);
    }
    if let Some(trigger) = tooltip.trigger.as_deref()
        && trigger != "axis"
        && trigger != "item"
    {
        return Err(EchartsError::Unsupported(
            "tooltip.trigger (only 'axis'/'item')",
        ));
    }
    if let Some(pointer) = tooltip.axis_pointer.as_ref()
        && let Some(kind) = pointer.kind.as_deref()
        && kind != "line"
        && kind != "shadow"
    {
        return Err(EchartsError::Unsupported(
            "tooltip.axisPointer.type (only 'line'/'shadow')",
        ));
    }
    Ok(Some(TooltipSpecV1::default()))
}

fn translate_data_zoom_v1(
    zoom: Option<&DataZoomOrDataZooms>,
    x_axes: &[AxisId],
    y_axes: &[AxisId],
    axis_grids: &BTreeMap<AxisId, GridId>,
) -> Result<(Vec<DataZoomXSpec>, Vec<DataZoomYSpec>, Vec<Action>)> {
    let Some(zoom) = zoom else {
        return Ok((Vec::new(), Vec::new(), Vec::new()));
    };

    let mut next_id = 1u64;
    let mut x_specs: Vec<DataZoomXSpec> = Vec::new();
    let mut y_specs: Vec<DataZoomYSpec> = Vec::new();
    let mut actions: Vec<Action> = Vec::new();

    #[derive(Debug, Clone, Copy)]
    enum ZoomWindowInput {
        Value { start: f64, end: f64 },
        Percent { start: f64, end: f64 },
    }

    #[derive(Debug, Clone, Copy)]
    struct ZoomAxisAccum {
        filter_mode: Option<FilterMode>,
        min_value_span: Option<f64>,
        max_value_span: Option<f64>,
        window: Option<ZoomWindowInput>,
    }

    impl ZoomAxisAccum {
        fn apply(
            &mut self,
            filter_mode: FilterMode,
            min_value_span: Option<f64>,
            max_value_span: Option<f64>,
            window: Option<ZoomWindowInput>,
        ) -> Result<()> {
            if let Some(existing) = self.filter_mode
                && existing != filter_mode
            {
                return Err(EchartsError::Unsupported(
                    "multiple dataZoom components with different filterMode for the same axis (v1 subset)",
                ));
            }

            if let (Some(existing), Some(next)) = (self.min_value_span, min_value_span)
                && existing != next
            {
                return Err(EchartsError::Unsupported(
                    "multiple dataZoom components with different minValueSpan for the same axis (v1 subset)",
                ));
            }

            if let (Some(existing), Some(next)) = (self.max_value_span, max_value_span)
                && existing != next
            {
                return Err(EchartsError::Unsupported(
                    "multiple dataZoom components with different maxValueSpan for the same axis (v1 subset)",
                ));
            }

            self.filter_mode = Some(filter_mode);
            if min_value_span.is_some() {
                self.min_value_span = min_value_span;
            }
            if max_value_span.is_some() {
                self.max_value_span = max_value_span;
            }
            if window.is_some() {
                self.window = window;
            }
            Ok(())
        }
    }

    let mut x_accum: BTreeMap<usize, ZoomAxisAccum> = BTreeMap::new();
    let mut y_accum: BTreeMap<usize, ZoomAxisAccum> = BTreeMap::new();

    fn auto_target_parallel_axes_in_first_grid(
        axes: &[AxisId],
        axis_grids: &BTreeMap<AxisId, GridId>,
    ) -> Vec<usize> {
        let Some(first_axis) = axes.first().copied() else {
            return Vec::new();
        };
        let Some(grid) = axis_grids.get(&first_axis).copied() else {
            return vec![0];
        };

        axes.iter()
            .enumerate()
            .filter_map(|(i, axis)| (axis_grids.get(axis).copied() == Some(grid)).then_some(i))
            .collect()
    }

    for entry in zoom.to_vec() {
        let _kind = entry.kind.as_deref().unwrap_or("inside");
        let orient = entry.orient.as_deref().unwrap_or("horizontal");
        if !matches!(orient, "horizontal" | "vertical") {
            return Err(EchartsError::Unsupported("dataZoom.orient"));
        }

        let filter_mode = match entry.filter_mode.as_deref().unwrap_or("filter") {
            "filter" => FilterMode::Filter,
            "weakFilter" => FilterMode::WeakFilter,
            "empty" => FilterMode::Empty,
            "none" => FilterMode::None,
            _ => return Err(EchartsError::Unsupported("dataZoom.filterMode")),
        };

        let window_input = {
            let range_mode = entry
                .range_mode
                .as_ref()
                .map(EchartsDataZoomRangeMode::parse_homogeneous)
                .transpose()?;

            let start_value = entry.start_value.as_ref().map(parse_f64_value);
            let end_value = entry.end_value.as_ref().map(parse_f64_value);
            let value_window = match (start_value, end_value) {
                (Some(a), Some(b)) if a.is_finite() && b.is_finite() => {
                    Some(ZoomWindowInput::Value { start: a, end: b })
                }
                _ => None,
            };

            let percent_window = if entry.start.is_some() || entry.end.is_some() {
                let start = entry.start.unwrap_or(0.0);
                let end = entry.end.unwrap_or(100.0);
                if start.is_finite() && end.is_finite() {
                    Some(ZoomWindowInput::Percent { start, end })
                } else {
                    None
                }
            } else {
                None
            };

            match range_mode {
                Some(RangeMode::Value) => value_window,
                Some(RangeMode::Percent) => percent_window,
                None => value_window.or(percent_window),
            }
        };

        let x_axis_indices: Vec<usize> = if let Some(indices) = entry.x_axis_index.as_ref() {
            indices.to_vec()
        } else if entry.y_axis_index.is_some() {
            Vec::new()
        } else if orient == "horizontal" {
            // ECharts-style auto-target: all parallel axes in the same grid as the first axis.
            auto_target_parallel_axes_in_first_grid(x_axes, axis_grids)
        } else {
            Vec::new()
        };

        let y_axis_indices: Vec<usize> = if let Some(indices) = entry.y_axis_index.as_ref() {
            indices.to_vec()
        } else if entry.x_axis_index.is_some() {
            Vec::new()
        } else if orient == "vertical" {
            auto_target_parallel_axes_in_first_grid(y_axes, axis_grids)
        } else {
            Vec::new()
        };

        for axis_index in x_axis_indices {
            if x_axes.get(axis_index).is_none() {
                return Err(EchartsError::Invalid("dataZoom.xAxisIndex out of range"));
            }
            x_accum
                .entry(axis_index)
                .or_insert(ZoomAxisAccum {
                    filter_mode: None,
                    min_value_span: None,
                    max_value_span: None,
                    window: None,
                })
                .apply(
                    filter_mode,
                    entry.min_value_span,
                    entry.max_value_span,
                    window_input,
                )?;
        }

        for axis_index in y_axis_indices {
            if y_axes.get(axis_index).is_none() {
                return Err(EchartsError::Invalid("dataZoom.yAxisIndex out of range"));
            }
            y_accum
                .entry(axis_index)
                .or_insert(ZoomAxisAccum {
                    filter_mode: None,
                    min_value_span: None,
                    max_value_span: None,
                    window: None,
                })
                .apply(
                    filter_mode,
                    entry.min_value_span,
                    entry.max_value_span,
                    window_input,
                )?;
        }
    }

    for (axis_index, acc) in x_accum {
        let axis = x_axes[axis_index];
        let id = delinea::ids::DataZoomId::new(next_id);
        next_id = next_id.saturating_add(1);
        x_specs.push(DataZoomXSpec {
            id,
            axis,
            filter_mode: acc.filter_mode.unwrap_or(FilterMode::Filter),
            min_value_span: acc.min_value_span,
            max_value_span: acc.max_value_span,
        });
        if let Some(w) = acc.window {
            match w {
                ZoomWindowInput::Value { start, end } => actions.push(Action::SetDataWindowX {
                    axis,
                    window: Some(DataWindow {
                        min: start,
                        max: end,
                    }),
                }),
                ZoomWindowInput::Percent { start, end } => {
                    actions.push(Action::SetAxisWindowPercent {
                        axis,
                        range: Some((start, end)),
                    })
                }
            }
        }
    }

    for (axis_index, acc) in y_accum {
        let axis = y_axes[axis_index];
        let id = delinea::ids::DataZoomId::new(next_id);
        next_id = next_id.saturating_add(1);
        y_specs.push(DataZoomYSpec {
            id,
            axis,
            filter_mode: acc.filter_mode.unwrap_or(FilterMode::Filter),
            min_value_span: acc.min_value_span,
            max_value_span: acc.max_value_span,
        });
        if let Some(w) = acc.window {
            match w {
                ZoomWindowInput::Value { start, end } => actions.push(Action::SetDataWindowY {
                    axis,
                    window: Some(DataWindow {
                        min: start,
                        max: end,
                    }),
                }),
                ZoomWindowInput::Percent { start, end } => {
                    actions.push(Action::SetAxisWindowPercent {
                        axis,
                        range: Some((start, end)),
                    })
                }
            }
        }
    }

    Ok((x_specs, y_specs, actions))
}

fn translate_visual_maps_v1(
    visual_map: Option<&VisualMapOrVisualMaps>,
    spec: &ChartSpec,
    datasets: &[(DatasetId, DataTable)],
    dataset_dimensions_by_index: &[Vec<String>],
    dataset_fields_by_index: &[Vec<FieldId>],
) -> Result<Vec<VisualMapSpec>> {
    let Some(visual_map) = visual_map else {
        return Ok(Vec::new());
    };

    let mut dataset_index_by_id: BTreeMap<DatasetId, usize> = BTreeMap::new();
    for (i, ds) in spec.datasets.iter().enumerate() {
        dataset_index_by_id.insert(ds.id, i);
    }

    let mut table_by_dataset: BTreeMap<DatasetId, &DataTable> = BTreeMap::new();
    for (id, table) in datasets {
        table_by_dataset.insert(*id, table);
    }

    fn parse_range_f32(
        map: Option<&serde_json::Map<String, serde_json::Value>>,
        key: &str,
    ) -> Option<(f32, f32)> {
        let v = map?.get(key)?;
        match v {
            serde_json::Value::Number(n) => n.as_f64().map(|v| (v as f32, v as f32)),
            serde_json::Value::Array(arr) => {
                if arr.len() != 2 {
                    return None;
                }
                let a = arr[0].as_f64()? as f32;
                let b = arr[1].as_f64()? as f32;
                Some((a, b))
            }
            _ => None,
        }
    }

    fn parse_number_f32(
        map: Option<&serde_json::Map<String, serde_json::Value>>,
        key: &str,
    ) -> Option<f32> {
        let v = map?.get(key)?;
        match v {
            serde_json::Value::Number(n) => n.as_f64().map(|v| v as f32),
            _ => None,
        }
    }

    fn clamp01(v: f32) -> f32 {
        v.clamp(0.0, 1.0)
    }

    fn compute_domain_from_table(
        table_by_dataset: &BTreeMap<DatasetId, &DataTable>,
        spec: &ChartSpec,
        dataset_id: DatasetId,
        field: FieldId,
    ) -> Result<(f64, f64)> {
        let dataset_spec = spec
            .datasets
            .iter()
            .find(|d| d.id == dataset_id)
            .ok_or(EchartsError::Invalid("visualMap target dataset missing"))?;
        let column = dataset_spec
            .fields
            .iter()
            .find(|f| f.id == field)
            .map(|f| f.column)
            .ok_or(EchartsError::Invalid("visualMap target field missing"))?;
        let table = table_by_dataset
            .get(&dataset_id)
            .copied()
            .ok_or(EchartsError::Invalid("visualMap dataset table missing"))?;
        let values = table
            .columns()
            .get(column)
            .and_then(|c| c.as_f64_slice())
            .ok_or(EchartsError::Unsupported(
                "visualMap domain inference requires f64 column (v1 subset)",
            ))?;

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for v in values.iter().copied().take(table.row_count()) {
            if v.is_nan() {
                continue;
            }
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        if !min.is_finite() || !max.is_finite() {
            return Err(EchartsError::Invalid("visualMap domain is empty"));
        }
        Ok((min, max))
    }

    fn resolve_field_from_dimension(
        dim: &EncodeDim,
        dataset_index: usize,
        dataset_dimensions_by_index: &[Vec<String>],
        dataset_fields_by_index: &[Vec<FieldId>],
    ) -> Result<FieldId> {
        let field_ids = dataset_fields_by_index
            .get(dataset_index)
            .ok_or(EchartsError::Invalid("visualMap.datasetIndex out of range"))?;
        match dim {
            EncodeDim::Index(i) => field_ids
                .get(*i)
                .copied()
                .ok_or(EchartsError::Invalid("visualMap.dimension out of range")),
            EncodeDim::Name(name) => {
                let dims =
                    dataset_dimensions_by_index
                        .get(dataset_index)
                        .ok_or(EchartsError::Invalid(
                            "visualMap dataset dimensions missing",
                        ))?;
                let i = dims
                    .iter()
                    .position(|d| d == name)
                    .ok_or(EchartsError::Invalid("visualMap.dimension name not found"))?;
                field_ids
                    .get(i)
                    .copied()
                    .ok_or(EchartsError::Invalid("visualMap.dimension out of range"))
            }
        }
    }

    let mut out: Vec<VisualMapSpec> = Vec::new();
    for (i, vm) in visual_map.to_vec().into_iter().enumerate() {
        let mode = match vm.kind.as_deref().unwrap_or("continuous") {
            "continuous" => VisualMapMode::Continuous,
            "piecewise" => VisualMapMode::Piecewise,
            _ => return Err(EchartsError::Unsupported("visualMap.type")),
        };

        let (dataset_target, series_targets) = if let Some(series) = vm.series_index.as_ref() {
            let indices = series.to_vec();
            if indices.is_empty() {
                return Err(EchartsError::Invalid("visualMap.seriesIndex is empty"));
            }
            let mut series_ids: Vec<SeriesId> = Vec::with_capacity(indices.len());
            for idx in indices {
                let s = spec
                    .series
                    .get(idx)
                    .ok_or(EchartsError::Invalid("visualMap.seriesIndex out of range"))?;
                series_ids.push(s.id);
            }
            (None, series_ids)
        } else {
            if spec.series.is_empty() {
                return Err(EchartsError::Invalid("visualMap requires series"));
            }
            let dataset_id = spec.series[0].dataset;
            if spec.series.iter().any(|s| s.dataset != dataset_id) {
                return Err(EchartsError::Unsupported(
                    "visualMap without seriesIndex requires a single dataset (v1 subset)",
                ));
            }
            (Some(dataset_id), Vec::new())
        };

        let dataset_id = if let Some(ds) = dataset_target {
            ds
        } else {
            let first = series_targets
                .first()
                .and_then(|id| spec.series.iter().find(|s| s.id == *id))
                .ok_or(EchartsError::Invalid("visualMap target series missing"))?;
            let dataset_id = first.dataset;
            if series_targets.iter().any(|id| {
                spec.series
                    .iter()
                    .find(|s| s.id == *id)
                    .is_some_and(|s| s.dataset != dataset_id)
            }) {
                return Err(EchartsError::Unsupported(
                    "visualMap across multiple datasets (v1 subset)",
                ));
            }
            dataset_id
        };

        let dataset_index = *dataset_index_by_id
            .get(&dataset_id)
            .ok_or(EchartsError::Invalid("visualMap target dataset missing"))?;

        let field = if let Some(dim) = vm.dimension.as_ref() {
            resolve_field_from_dimension(
                dim,
                dataset_index,
                dataset_dimensions_by_index,
                dataset_fields_by_index,
            )?
        } else if let Some(series_id) = series_targets.first() {
            let series = spec
                .series
                .iter()
                .find(|s| s.id == *series_id)
                .ok_or(EchartsError::Invalid("visualMap target series missing"))?;
            series.encode.y
        } else {
            return Err(EchartsError::Invalid("visualMap requires dimension"));
        };

        let (mut min, mut max) = match (vm.min, vm.max) {
            (Some(min), Some(max)) => (min, max),
            _ => compute_domain_from_table(&table_by_dataset, spec, dataset_id, field)?,
        };
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }

        let initial_range = vm.range.map(|r| {
            let mut a = r[0];
            let mut b = r[1];
            if a > b {
                std::mem::swap(&mut a, &mut b);
            }
            (a, b)
        });

        let point_radius_mul_range =
            parse_range_f32(vm.in_range.as_ref(), "symbolSize").map(|(a, b)| {
                let base = 10.0f32;
                ((a / base).max(0.0), (b / base).max(0.0))
            });

        let opacity_mul_range =
            parse_range_f32(vm.in_range.as_ref(), "opacity").map(|(a, b)| (clamp01(a), clamp01(b)));
        let out_of_range_opacity = parse_number_f32(vm.out_of_range.as_ref(), "opacity")
            .map(clamp01)
            .unwrap_or(0.25);

        let buckets = vm.split_number.unwrap_or(8).max(1).min(64);

        out.push(VisualMapSpec {
            id: VisualMapId::new((i as u64).saturating_add(1)),
            mode,
            dataset: dataset_target,
            series: series_targets,
            field,
            domain: (min, max),
            initial_range,
            initial_piece_mask: None,
            point_radius_mul_range,
            stroke_width_range: None,
            opacity_mul_range,
            buckets,
            out_of_range_opacity,
        });
    }

    Ok(out)
}

fn compute_axis_grids(spec: &ChartSpec) -> BTreeMap<AxisId, GridId> {
    spec.axes.iter().map(|a| (a.id, a.grid)).collect()
}

fn build_y_scale(y_axis: Option<&EchartsAxis>) -> Result<AxisScale> {
    let axis_type = y_axis.and_then(|a| a.kind.as_deref());
    match axis_type {
        None | Some("value") => Ok(AxisScale::Value(ValueAxisScale)),
        Some("time") => Ok(AxisScale::Time(Default::default())),
        Some("category") => Err(EchartsError::Unsupported(
            "yAxis.type = 'category' (not supported yet)",
        )),
        Some(_) => Err(EchartsError::Unsupported("yAxis.type")),
    }
}

fn build_x_layout(x_axis: Option<&EchartsAxis>, series: &[ParsedSeries]) -> Result<XLayout> {
    let axis_type = x_axis.and_then(|a| a.kind.as_deref());
    if let Some(axis_type) = axis_type
        && axis_type != "value"
        && axis_type != "category"
        && axis_type != "time"
    {
        return Err(EchartsError::Unsupported("xAxis.type"));
    }

    let mut force_category = matches!(axis_type, Some("category"));
    if !force_category {
        force_category = x_axis
            .and_then(|a| a.data.as_ref())
            .is_some_and(|d| !d.is_empty());
    }
    if !force_category {
        force_category = series.iter().any(|s| {
            s.points
                .iter()
                .any(|(x, _)| matches!(x, XValue::Category(_)))
        });
    }

    if force_category {
        let mut categories: Vec<String> = Vec::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();

        if let Some(axis_categories) = x_axis.and_then(|a| a.data.as_ref()) {
            for c in axis_categories {
                if seen.insert(c.clone()) {
                    categories.push(c.clone());
                }
            }
        }

        let mut max_len = 0usize;
        for s in series {
            max_len = max_len.max(s.points.len());
        }
        if categories.is_empty() {
            for i in 0..max_len {
                let c = i.to_string();
                seen.insert(c.clone());
                categories.push(c);
            }
        }

        for s in series {
            for (x, _) in &s.points {
                let Some(c) = (match x {
                    XValue::Category(v) => Some(v.clone()),
                    // Numeric X values are treated as indices in category mode.
                    XValue::Number(_) => None,
                }) else {
                    continue;
                };
                if seen.insert(c.clone()) {
                    categories.push(c);
                }
            }
        }

        if categories.is_empty() {
            return Err(EchartsError::Invalid("category xAxis has no categories"));
        }

        let mut index_by_category: BTreeMap<String, usize> = BTreeMap::new();
        for (i, c) in categories.iter().enumerate() {
            index_by_category.insert(c.clone(), i);
        }

        let values: Vec<f64> = (0..categories.len()).map(|i| i as f64).collect();
        return Ok(XLayout {
            scale: AxisScale::Category(CategoryAxisScale { categories }),
            values,
            mode: XLayoutMode::Category { index_by_category },
        });
    }

    let Some(base) = series.first() else {
        return Err(EchartsError::Invalid("missing series"));
    };
    if base.points.is_empty() {
        return Err(EchartsError::Invalid("series.data is empty"));
    }

    let base_x: Vec<f64> = base
        .points
        .iter()
        .map(|(x, _)| match x {
            XValue::Number(v) => *v,
            XValue::Category(_) => f64::NAN,
        })
        .collect();

    for other in series.iter().skip(1) {
        if other.points.len() != base.points.len() {
            return Err(EchartsError::Unsupported(
                "multiple series with mismatched x lengths on value axis",
            ));
        }
        for ((x0, _), (x1, _)) in base.points.iter().zip(other.points.iter()) {
            match (x0, x1) {
                (XValue::Number(a), XValue::Number(b))
                    if a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()) => {}
                _ => {
                    return Err(EchartsError::Unsupported(
                        "multiple series with mismatched x values on value axis",
                    ));
                }
            }
        }
    }

    let mut index_by_bits: BTreeMap<u64, usize> = BTreeMap::new();
    for (i, x) in base_x.iter().enumerate() {
        if x.is_finite() {
            index_by_bits.entry(x.to_bits()).or_insert(i);
        }
    }

    let scale = match axis_type {
        Some("time") => AxisScale::Time(Default::default()),
        _ => AxisScale::Value(ValueAxisScale),
    };
    Ok(XLayout {
        scale,
        values: base_x,
        mode: XLayoutMode::Value { index_by_bits },
    })
}

fn build_y_values_for_series(x_layout: &XLayout, series: &ParsedSeries) -> Result<Vec<f64>> {
    let mut y_values = vec![f64::NAN; x_layout.values.len()];
    match &x_layout.mode {
        XLayoutMode::Category { index_by_category } => {
            for (i, (x, y)) in series.points.iter().enumerate() {
                let idx = match x {
                    XValue::Category(v) => index_by_category.get(v).copied(),
                    XValue::Number(v) => {
                        if !v.is_finite() {
                            None
                        } else {
                            let idx = v.round() as isize;
                            (idx >= 0)
                                .then_some(idx as usize)
                                .filter(|idx| *idx < y_values.len())
                        }
                    }
                }
                .or_else(|| (i < y_values.len()).then_some(i));

                if let Some(idx) = idx {
                    y_values[idx] = *y;
                }
            }
        }
        XLayoutMode::Value { index_by_bits } => {
            for (x, y) in &series.points {
                let XValue::Number(x) = x else {
                    return Err(EchartsError::Unsupported("category x values on value axis"));
                };
                if !x.is_finite() {
                    continue;
                }
                if let Some(idx) = index_by_bits.get(&x.to_bits()).copied() {
                    y_values[idx] = *y;
                }
            }
        }
    }
    Ok(y_values)
}

fn extract_series(option: &EchartsOption) -> Result<Vec<ParsedSeries>> {
    let mut out = Vec::with_capacity(option.series.len());

    for series in &option.series {
        let kind = match series.kind.as_deref().unwrap_or("line") {
            "line" => SeriesKind::Line,
            "scatter" => SeriesKind::Scatter,
            "bar" => SeriesKind::Bar,
            _ => return Err(EchartsError::Unsupported("series.type")),
        };

        let Some(data) = series.data.as_ref() else {
            return Err(EchartsError::Invalid("series.data is required (v1 subset)"));
        };

        let mut points: Vec<(XValue, f64)> = Vec::with_capacity(data.len());
        for (i, item) in data.iter().enumerate() {
            if item.is_null() {
                points.push((XValue::Number(i as f64), f64::NAN));
                continue;
            }

            match item {
                serde_json::Value::Number(n) => {
                    let y = n.as_f64().unwrap_or(f64::NAN);
                    points.push((XValue::Number(i as f64), y));
                }
                serde_json::Value::Array(arr) => {
                    if arr.len() < 2 {
                        return Err(EchartsError::Invalid(
                            "series.data item array must have at least 2 entries",
                        ));
                    }
                    let x = parse_x_value(&arr[0])?;
                    let y = parse_f64_value(&arr[1]);
                    points.push((x, y));
                }
                serde_json::Value::String(s) => {
                    let y = s.parse::<f64>().unwrap_or(f64::NAN);
                    points.push((XValue::Number(i as f64), y));
                }
                _ => return Err(EchartsError::Unsupported("series.data item type")),
            }
        }

        out.push(ParsedSeries {
            kind,
            name: series.name.clone(),
            points,
            lod: {
                let lod = SeriesLodSpecV1 {
                    large: series.large,
                    large_threshold: series.large_threshold,
                    progressive: series.progressive,
                    progressive_threshold: series.progressive_threshold,
                };
                let any = lod.large.is_some()
                    || lod.large_threshold.is_some()
                    || lod.progressive.is_some()
                    || lod.progressive_threshold.is_some();
                any.then_some(lod)
            },
        });
    }

    Ok(out)
}

fn resolve_encode_dim(dim: &EncodeDim, dimensions: Option<&[String]>) -> Result<usize> {
    match dim {
        EncodeDim::Index(i) => Ok(*i),
        EncodeDim::Name(name) => {
            let Some(dims) = dimensions else {
                return Err(EchartsError::Unsupported(
                    "series.encode by name without dataset.dimensions (v1 subset)",
                ));
            };
            dims.iter()
                .position(|d| d == name)
                .ok_or(EchartsError::Invalid(
                    "series.encode dimension name not found",
                ))
        }
    }
}

struct ParsedRawDatasetV1 {
    table: DataTable,
    rows: Vec<Vec<serde_json::Value>>,
    fields: Vec<FieldSpec>,
    field_ids: Vec<FieldId>,
    dimensions: Vec<String>,
}

fn parse_raw_dataset_v1(
    ds: &EchartsDataset,
    next_field_id: &mut u64,
) -> Result<ParsedRawDatasetV1> {
    if ds.source.is_none() {
        return Err(EchartsError::Invalid(
            "dataset.source is required (v1 subset)",
        ));
    }
    if ds.from_dataset_index.is_some() {
        return Err(EchartsError::Unsupported(
            "dataset.fromDatasetIndex with dataset.source (v1 subset)",
        ));
    }
    if ds.transform.is_some() {
        return Err(EchartsError::Unsupported(
            "dataset.transform with dataset.source (v1 subset)",
        ));
    }

    let header_mode = match ds.source_header.as_ref() {
        None => "auto",
        Some(DatasetSourceHeader::Num(0)) => "none",
        Some(DatasetSourceHeader::Num(1)) => "header",
        Some(DatasetSourceHeader::Num(_)) => {
            return Err(EchartsError::Unsupported(
                "dataset.sourceHeader (only 0/1/'auto')",
            ));
        }
        Some(DatasetSourceHeader::Str(s)) if s == "auto" => "auto",
        Some(DatasetSourceHeader::Str(_)) => {
            return Err(EchartsError::Unsupported(
                "dataset.sourceHeader (only 0/1/'auto')",
            ));
        }
    };

    let mut dimensions = ds.dimensions.clone().unwrap_or_default();

    let rows: Vec<Vec<serde_json::Value>> = match ds.source.as_ref().expect("checked") {
        DatasetSource::Rows(rows) => {
            if rows.is_empty() {
                return Err(EchartsError::Invalid("dataset.source is empty"));
            }

            let mut data_start = 0usize;
            let header_candidate_is_strings = rows[0].iter().all(|v| v.is_string());
            let has_header = match header_mode {
                "header" => true,
                "none" => false,
                _ => header_candidate_is_strings,
            };

            if has_header {
                if !header_candidate_is_strings {
                    return Err(EchartsError::Invalid(
                        "dataset.source header row must be strings (v1 subset)",
                    ));
                }
                dimensions = rows[0]
                    .iter()
                    .map(|v| v.as_str().unwrap_or_default().to_string())
                    .collect();
                data_start = 1;
            }

            if data_start >= rows.len() {
                return Err(EchartsError::Invalid("dataset.source has no data rows"));
            }

            let data_rows = &rows[data_start..];
            let width = data_rows[0].len();
            if width == 0 {
                return Err(EchartsError::Invalid("dataset.source row has zero width"));
            }
            for row in data_rows.iter().skip(1) {
                if row.len() != width {
                    return Err(EchartsError::Invalid(
                        "dataset.source rows must have consistent width",
                    ));
                }
            }

            if dimensions.is_empty() {
                dimensions = (0..width).map(|i| format!("dim{i}")).collect();
            }

            data_rows.to_vec()
        }
        DatasetSource::Objects(rows) => {
            if rows.is_empty() {
                return Err(EchartsError::Invalid("dataset.source is empty"));
            }

            let rows = rows.clone();
            if dimensions.is_empty() {
                let mut keys: BTreeSet<String> = BTreeSet::new();
                for row in &rows {
                    for k in row.keys() {
                        keys.insert(k.clone());
                    }
                }
                dimensions = keys.into_iter().collect();
            }

            if dimensions.is_empty() {
                return Err(EchartsError::Invalid("dataset.dimensions is empty"));
            }

            let width = dimensions.len();
            let mut normalized_rows: Vec<Vec<serde_json::Value>> = Vec::with_capacity(rows.len());
            for row in rows {
                let mut out_row: Vec<serde_json::Value> = Vec::with_capacity(width);
                for key in &dimensions {
                    let v = row.get(key).unwrap_or(&serde_json::Value::Null);
                    out_row.push(v.clone());
                }
                normalized_rows.push(out_row);
            }
            normalized_rows
        }
    };

    if dimensions.is_empty() {
        return Err(EchartsError::Invalid("dataset.dimensions is empty"));
    }
    let width = dimensions.len();
    for row in &rows {
        if row.len() != width {
            return Err(EchartsError::Invalid(
                "dataset.source rows must have consistent width",
            ));
        }
    }

    let row_count = rows.len();
    let mut columns: Vec<Vec<f64>> = vec![Vec::with_capacity(row_count); width];
    for row in &rows {
        for col in 0..width {
            let v = row.get(col).unwrap_or(&serde_json::Value::Null);
            columns[col].push(parse_f64_value(v));
        }
    }

    let mut table = DataTable::default();
    for col in columns {
        table.push_column(Column::F64(col));
    }
    if table.row_count() != row_count {
        return Err(EchartsError::Invalid("dataset.source rowCount mismatch"));
    }

    let mut fields: Vec<FieldSpec> = Vec::with_capacity(width);
    let mut field_ids: Vec<FieldId> = Vec::with_capacity(width);
    for col in 0..width {
        let id = FieldId::new(*next_field_id);
        *next_field_id = (*next_field_id).saturating_add(1);
        fields.push(FieldSpec { id, column: col });
        field_ids.push(id);
    }

    Ok(ParsedRawDatasetV1 {
        table,
        rows,
        fields,
        field_ids,
        dimensions,
    })
}

fn as_object<'a>(
    v: Option<&'a serde_json::Value>,
) -> Result<Option<&'a serde_json::Map<String, serde_json::Value>>> {
    match v {
        None => Ok(None),
        Some(serde_json::Value::Object(o)) => Ok(Some(o)),
        Some(_) => Err(EchartsError::Unsupported(
            "dataset.transform.config (object only)",
        )),
    }
}

fn parse_transform_dimension(v: Option<&serde_json::Value>) -> Result<EncodeDim> {
    let Some(v) = v else {
        return Err(EchartsError::Invalid(
            "dataset.transform.config.dimension is required (v1 subset)",
        ));
    };
    match v {
        serde_json::Value::Number(n) => Ok(EncodeDim::Index(n.as_u64().unwrap_or(0) as usize)),
        serde_json::Value::String(s) => Ok(EncodeDim::Name(s.clone())),
        _ => Err(EchartsError::Unsupported(
            "dataset.transform.config.dimension (only string/number)",
        )),
    }
}

fn parse_x_value(v: &serde_json::Value) -> Result<XValue> {
    match v {
        serde_json::Value::Number(n) => Ok(XValue::Number(n.as_f64().unwrap_or(f64::NAN))),
        serde_json::Value::String(s) => Ok(XValue::Category(s.clone())),
        serde_json::Value::Null => Ok(XValue::Number(f64::NAN)),
        _ => Err(EchartsError::Unsupported("series.data[x]")),
    }
}

fn parse_f64_value(v: &serde_json::Value) -> f64 {
    match v {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
        serde_json::Value::Null => f64::NAN,
        _ => f64::NAN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn translate_minimal_line_series_y_only() {
        let json = r#"
        {
          "xAxis": { "type": "category", "data": ["Mon","Tue","Wed"] },
          "yAxis": { "type": "value" },
          "series": [
            { "type": "line", "name": "A", "data": [120, 132, 101] }
          ],
          "tooltip": { "trigger": "axis" }
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.datasets.len(), 1);

        let ds = translated.datasets[0].1.clone();
        assert_eq!(ds.row_count(), 3);
        assert!(translated.spec.axes.len() >= 2);
    }

    #[test]
    fn translate_xy_scatter() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "series": [
            { "type": "scatter", "data": [[0, 1], [1, 2], [2, 3]] }
          ]
        }
        "#;
        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.datasets.len(), 1);
        assert_eq!(translated.datasets[0].1.row_count(), 3);
    }

    #[test]
    fn translate_dataset_encode_and_axis_indices() {
        let json = r#"
        {
          "dataset": {
            "dimensions": ["x", "y"],
            "source": [[0, 1], [1, 2], [2, 3]]
          },
          "xAxis": [{ "type": "value" }, { "type": "value", "name": "X2" }],
          "yAxis": [{ "type": "value" }, { "type": "value", "name": "Y2" }],
          "series": [
            {
              "type": "scatter",
              "datasetIndex": 0,
              "xAxisIndex": 1,
              "yAxisIndex": 1,
              "encode": { "x": "x", "y": "y" }
            }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.datasets.len(), 1);
        assert_eq!(translated.datasets[0].1.row_count(), 3);
        assert_eq!(translated.spec.datasets.len(), 1);
        assert_eq!(translated.spec.datasets[0].fields.len(), 2);
        assert_eq!(translated.spec.axes.len(), 4);

        let series = &translated.spec.series[0];
        assert_eq!(series.x_axis, AxisId::new(2));
        assert_eq!(series.y_axis, AxisId::new(4));
    }

    #[test]
    fn translate_dataset_source_header_auto() {
        let json = r#"
        {
          "dataset": {
            "sourceHeader": "auto",
            "source": [["x", "y"], [0, 1], [1, 2], [2, 3]]
          },
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "series": [
            { "type": "line", "encode": { "x": "x", "y": "y" } }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.datasets.len(), 1);
        assert_eq!(translated.datasets[0].1.row_count(), 3);
        assert_eq!(translated.spec.datasets[0].fields.len(), 2);
    }

    #[test]
    fn translate_dataset_source_object_rows() {
        let json = r#"
        {
          "dataset": {
            "source": [
              { "x": 0, "y": 1 },
              { "x": 1, "y": 2 },
              { "x": 2, "y": 3 }
            ]
          },
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "series": [
            { "type": "scatter", "encode": { "x": "x", "y": "y" } }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.datasets.len(), 1);
        assert_eq!(translated.datasets[0].1.row_count(), 3);
        assert_eq!(translated.spec.datasets[0].fields.len(), 2);
    }

    #[test]
    fn translate_dataset_category_x_axis_from_source() {
        let json = r#"
        {
          "dataset": {
            "source": [["cat", "y"], ["A", 1], ["B", 2], ["C", 3]]
          },
          "xAxis": { "type": "category" },
          "yAxis": { "type": "value" },
          "series": [
            { "type": "scatter", "encode": { "x": "cat", "y": "y" } }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.datasets.len(), 1);
        assert_eq!(translated.datasets[0].1.row_count(), 3);

        let axis = translated
            .spec
            .axes
            .iter()
            .find(|a| a.kind == AxisKind::X)
            .expect("x axis");
        let AxisScale::Category(scale) = &axis.scale else {
            panic!("expected category x axis");
        };
        assert_eq!(scale.categories, vec!["A", "B", "C"]);

        let series = &translated.spec.series[0];
        let dataset = &translated.spec.datasets[0];
        let x_field = series.encode.x;
        let x_col = dataset
            .fields
            .iter()
            .find(|f| f.id == x_field)
            .expect("x field")
            .column;

        let table = &translated.datasets[0].1;
        let x = table.column_f64(x_col).expect("f64 column");
        assert_eq!(x, &[0.0, 1.0, 2.0]);
    }

    #[test]
    fn translate_dataset_category_x_axis_with_explicit_data() {
        let json = r#"
        {
          "dataset": {
            "source": [[0, 1], [1, 2], [2, 3]]
          },
          "xAxis": { "type": "category", "data": ["A", "B", "C"] },
          "yAxis": { "type": "value" },
          "series": [
            { "type": "line", "encode": { "x": 0, "y": 1 } }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.datasets.len(), 1);
        assert_eq!(translated.datasets[0].1.row_count(), 3);

        let axis = translated
            .spec
            .axes
            .iter()
            .find(|a| a.kind == AxisKind::X)
            .expect("x axis");
        let AxisScale::Category(scale) = &axis.scale else {
            panic!("expected category x axis");
        };
        assert_eq!(scale.categories, vec!["A", "B", "C"]);

        let series = &translated.spec.series[0];
        let dataset = &translated.spec.datasets[0];
        let x_field = series.encode.x;
        let x_col = dataset
            .fields
            .iter()
            .find(|f| f.id == x_field)
            .expect("x field")
            .column;

        let table = &translated.datasets[0].1;
        let x = table.column_f64(x_col).expect("f64 column");
        assert_eq!(x, &[0.0, 1.0, 2.0]);
    }

    #[test]
    fn translate_dataset_multi_grid_axis_binding() {
        let json = r#"
        {
          "grid": [{}, {}],
          "dataset": {
            "dimensions": ["x", "y0", "y1"],
            "source": [[0, 1, 10], [1, 2, 20], [2, 3, 30]]
          },
          "xAxis": [
            { "type": "value", "gridIndex": 0 },
            { "type": "value", "gridIndex": 1 }
          ],
          "yAxis": [
            { "type": "value", "gridIndex": 0 },
            { "type": "value", "gridIndex": 1 }
          ],
          "series": [
            { "type": "line", "encode": { "x": "x", "y": "y0" }, "xAxisIndex": 0, "yAxisIndex": 0 },
            { "type": "scatter", "encode": { "x": "x", "y": "y1" }, "xAxisIndex": 1, "yAxisIndex": 1 }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.spec.grids.len(), 2);
        assert_eq!(translated.spec.grids[0].id, GridId::new(1));
        assert_eq!(translated.spec.grids[1].id, GridId::new(2));

        let x1 = translated
            .spec
            .axes
            .iter()
            .find(|a| a.id == AxisId::new(2))
            .expect("xAxis[1]");
        assert_eq!(x1.grid, GridId::new(2));

        let y1 = translated
            .spec
            .axes
            .iter()
            .find(|a| a.id == AxisId::new(4))
            .expect("yAxis[1]");
        assert_eq!(y1.grid, GridId::new(2));
    }

    #[test]
    fn translate_tooltip_rejects_unknown_trigger() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "series": [{ "type": "line", "data": [1,2,3] }],
          "tooltip": { "trigger": "foobar" }
        }
        "#;
        let err = translate_json_str(json).unwrap_err();
        assert!(matches!(err, EchartsError::Unsupported(_)));
    }

    #[test]
    fn translate_defaults_to_viewportless_spec() {
        let json = r#"
        {
          "series": [{ "data": [1,2,3] }]
        }
        "#;
        let translated = translate_json_str(json).expect("translate");
        assert!(translated.spec.viewport.is_none());
    }

    #[test]
    fn translate_can_be_used_to_build_engine() {
        let json = r#"
        {
          "xAxis": { "type": "category", "data": ["Mon","Tue","Wed"] },
          "yAxis": { "type": "value" },
          "series": [{ "type": "line", "data": [120, 132, 101] }]
        }
        "#;
        let translated = translate_json_str(json).expect("translate");

        let mut spec = translated.spec.clone();
        spec.viewport = Some(Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        ));
        let mut engine = delinea::engine::ChartEngine::new(spec).expect("engine");
        let (dataset_id, table) = translated.datasets.into_iter().next().expect("dataset");
        engine.datasets_mut().insert(dataset_id, table);
    }

    #[test]
    fn translate_series_lod_knobs() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "series": [
            {
              "type": "scatter",
              "name": "A",
              "data": [[0, 1], [1, 2], [2, 3]],
              "large": true,
              "largeThreshold": 123,
              "progressive": 456,
              "progressiveThreshold": 789
            }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        let series = &translated.spec.series[0];
        let lod = series.lod.expect("expected lod knobs");
        assert_eq!(lod.large, Some(true));
        assert_eq!(lod.large_threshold, Some(123));
        assert_eq!(lod.progressive, Some(456));
        assert_eq!(lod.progressive_threshold, Some(789));
    }

    #[test]
    fn translate_data_zoom_start_end_value_to_specs_and_actions() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "dataZoom": [
            { "type": "inside", "xAxisIndex": 0, "filterMode": "empty", "startValue": 1, "endValue": 3 },
            { "type": "slider", "yAxisIndex": 0, "filterMode": "filter", "startValue": 10, "endValue": 30 }
          ],
          "series": [
            { "type": "scatter", "data": [[0, 0], [1, 10], [2, 20], [3, 30], [4, 40]] }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.spec.data_zoom_x.len(), 1);
        assert_eq!(
            translated.spec.data_zoom_x[0].filter_mode,
            FilterMode::Empty
        );
        assert_eq!(translated.spec.data_zoom_y.len(), 1);
        assert_eq!(
            translated.spec.data_zoom_y[0].filter_mode,
            FilterMode::Filter
        );

        assert!(
            translated.actions.iter().any(|a| matches!(
                a,
                Action::SetDataWindowX { window: Some(w), .. }
                    if (w.min - 1.0).abs() < 1e-9 && (w.max - 3.0).abs() < 1e-9
            )),
            "expected SetDataWindowX action"
        );
        assert!(
            translated.actions.iter().any(|a| matches!(
                a,
                Action::SetDataWindowY { window: Some(w), .. }
                    if (w.min - 10.0).abs() < 1e-9 && (w.max - 30.0).abs() < 1e-9
            )),
            "expected SetDataWindowY action"
        );
    }

    #[test]
    fn translate_data_zoom_start_end_percent_to_actions() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "dataZoom": [
            { "type": "slider", "xAxisIndex": 0, "filterMode": "filter", "start": 25, "end": 75 }
          ],
          "series": [
            { "type": "scatter", "data": [[0, 0], [1, 10], [2, 20], [3, 30], [4, 40]] }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.spec.data_zoom_x.len(), 1);
        assert!(
            translated.actions.iter().any(|a| matches!(
                a,
                Action::SetAxisWindowPercent { range: Some((start, end)), .. }
                    if (start - 25.0).abs() < 1e-9 && (end - 75.0).abs() < 1e-9
            )),
            "expected SetAxisWindowPercent action for percent window"
        );
    }

    #[test]
    fn translate_data_zoom_defaults_to_first_x_axis_when_unspecified() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "dataZoom": [
            { "type": "slider", "filterMode": "empty", "startValue": 1, "endValue": 3 }
          ],
          "series": [
            { "type": "scatter", "data": [[0, 0], [1, 10], [2, 20], [3, 30], [4, 40]] }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.spec.data_zoom_x.len(), 1);
        assert_eq!(translated.spec.data_zoom_y.len(), 0);
        assert!(
            translated.actions.iter().any(|a| matches!(
                a,
                Action::SetDataWindowX { window: Some(w), .. }
                    if (w.min - 1.0).abs() < 1e-9 && (w.max - 3.0).abs() < 1e-9
            )),
            "expected SetDataWindowX action"
        );
    }

    #[test]
    fn translate_data_zoom_defaults_to_first_y_axis_when_orient_is_vertical() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "dataZoom": [
            { "type": "slider", "orient": "vertical", "filterMode": "filter", "startValue": 10, "endValue": 30 }
          ],
          "series": [
            { "type": "scatter", "data": [[0, 0], [1, 10], [2, 20], [3, 30], [4, 40]] }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.spec.data_zoom_x.len(), 0);
        assert_eq!(translated.spec.data_zoom_y.len(), 1);
        assert!(
            translated.actions.iter().any(|a| matches!(
                a,
                Action::SetDataWindowY { window: Some(w), .. }
                    if (w.min - 10.0).abs() < 1e-9 && (w.max - 30.0).abs() < 1e-9
            )),
            "expected SetDataWindowY action"
        );
    }

    #[test]
    fn translate_data_zoom_auto_targets_all_parallel_x_axes_in_first_grid() {
        let json = r#"
        {
          "dataset": { "source": [[0, 0], [1, 10], [2, 20], [3, 30], [4, 40]] },
          "xAxis": [{ "type": "value" }, { "type": "value" }],
          "yAxis": { "type": "value" },
          "dataZoom": [
            { "type": "slider", "filterMode": "empty", "startValue": 1, "endValue": 3 }
          ],
          "series": [
            { "type": "scatter", "datasetIndex": 0, "encode": { "x": 0, "y": 1 }, "xAxisIndex": 0, "yAxisIndex": 0 }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.spec.data_zoom_x.len(), 2);
        assert_eq!(translated.spec.data_zoom_y.len(), 0);
        assert_eq!(translated.actions.len(), 2);
        assert!(
            translated.actions.iter().all(|a| matches!(
                a,
                Action::SetDataWindowX { window: Some(w), .. }
                    if (w.min - 1.0).abs() < 1e-9 && (w.max - 3.0).abs() < 1e-9
            )),
            "expected SetDataWindowX actions for all targeted axes"
        );
    }

    #[test]
    fn translate_data_zoom_auto_targets_all_parallel_y_axes_in_first_grid_when_orient_vertical() {
        let json = r#"
        {
          "dataset": { "source": [[0, 0], [1, 10], [2, 20], [3, 30], [4, 40]] },
          "xAxis": { "type": "value" },
          "yAxis": [{ "type": "value" }, { "type": "value" }],
          "dataZoom": [
            { "type": "slider", "orient": "vertical", "filterMode": "filter", "startValue": 10, "endValue": 30 }
          ],
          "series": [
            { "type": "scatter", "datasetIndex": 0, "encode": { "x": 0, "y": 1 }, "xAxisIndex": 0, "yAxisIndex": 0 }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.spec.data_zoom_x.len(), 0);
        assert_eq!(translated.spec.data_zoom_y.len(), 2);
        assert_eq!(translated.actions.len(), 2);
        assert!(
            translated.actions.iter().all(|a| matches!(
                a,
                Action::SetDataWindowY { window: Some(w), .. }
                    if (w.min - 10.0).abs() < 1e-9 && (w.max - 30.0).abs() < 1e-9
            )),
            "expected SetDataWindowY actions for all targeted axes"
        );
    }

    #[test]
    fn translate_data_zoom_range_mode_value_ignores_start_end_percent() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "dataZoom": [
            {
              "type": "slider",
              "xAxisIndex": 0,
              "filterMode": "filter",
              "rangeMode": "value",
              "start": 25,
              "end": 75,
              "startValue": 1,
              "endValue": 3
            }
          ],
          "series": [
            { "type": "scatter", "data": [[0, 0], [1, 10], [2, 20], [3, 30], [4, 40]] }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.spec.data_zoom_x.len(), 1);
        assert!(
            translated.actions.iter().any(|a| matches!(
                a,
                Action::SetDataWindowX { window: Some(w), .. }
                    if (w.min - 1.0).abs() < 1e-9 && (w.max - 3.0).abs() < 1e-9
            )),
            "expected startValue/endValue to win under rangeMode=value"
        );
    }

    #[test]
    fn translate_data_zoom_range_mode_percent_ignores_start_value_end_value() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "dataZoom": [
            {
              "type": "slider",
              "xAxisIndex": 0,
              "filterMode": "filter",
              "rangeMode": "percent",
              "start": 25,
              "end": 75,
              "startValue": 123,
              "endValue": 456
            }
          ],
          "series": [
            { "type": "scatter", "data": [[0, 0], [1, 10], [2, 20], [3, 30], [4, 40]] }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.spec.data_zoom_x.len(), 1);
        assert!(
            translated.actions.iter().any(|a| matches!(
                a,
                Action::SetAxisWindowPercent { range: Some((start, end)), .. }
                    if (start - 25.0).abs() < 1e-9 && (end - 75.0).abs() < 1e-9
            )),
            "expected start/end percent to win under rangeMode=percent"
        );
    }

    #[test]
    fn translate_multiple_datazoom_same_axis_is_accepted_when_compatible() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "dataZoom": [
            { "type": "inside", "xAxisIndex": 0, "filterMode": "filter" },
            { "type": "slider", "xAxisIndex": 0, "filterMode": "filter", "startValue": 1, "endValue": 3 }
          ],
          "series": [
            { "type": "scatter", "data": [[0, 0], [1, 10], [2, 20], [3, 30], [4, 40]] }
          ]
        }
        "#;

        let translated = translate_json_str(json).expect("translate");
        assert_eq!(translated.spec.data_zoom_x.len(), 1);
        assert!(
            translated.actions.iter().any(|a| matches!(
                a,
                Action::SetDataWindowX { window: Some(w), .. }
                    if (w.min - 1.0).abs() < 1e-9 && (w.max - 3.0).abs() < 1e-9
            )),
            "expected SetDataWindowX action"
        );
    }

    #[test]
    fn translate_multiple_datazoom_same_axis_rejects_conflicting_filter_mode() {
        let json = r#"
        {
          "xAxis": { "type": "value" },
          "yAxis": { "type": "value" },
          "dataZoom": [
            { "type": "inside", "xAxisIndex": 0, "filterMode": "filter" },
            { "type": "slider", "xAxisIndex": 0, "filterMode": "empty" }
          ],
          "series": [
            { "type": "scatter", "data": [[0, 0], [1, 10]] }
          ]
        }
        "#;

        let err = translate_json_str(json).unwrap_err();
        assert!(matches!(err, EchartsError::Unsupported(_)));
    }
}
