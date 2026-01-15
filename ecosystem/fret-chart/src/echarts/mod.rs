//! Minimal ECharts option adapter.
//!
//! This module is intentionally narrow: it targets a small, high-leverage subset of ECharts
//! and translates it into a `delinea::ChartSpec` + datasets that can be inserted into a
//! `delinea::engine::ChartEngine`.

use std::collections::{BTreeMap, BTreeSet};

use delinea::data::{Column, DataTable};
use delinea::ids::{AxisId, ChartId, DatasetId, FieldId, GridId, SeriesId};
use delinea::scale::{AxisScale, CategoryAxisScale, ValueAxisScale};
use delinea::spec::{
    AxisKind, AxisSpec, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind,
    SeriesLodSpecV1, SeriesSpec, TooltipSpecV1,
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
    series: Vec<EchartsSeries>,
    #[serde(default)]
    tooltip: Option<EchartsTooltip>,
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
    source: Option<Vec<Vec<serde_json::Value>>>,
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

    Ok(TranslatedChart { spec, datasets })
}

fn translate_option_with_dataset(option: &EchartsOption) -> Result<TranslatedChart> {
    let grid_id = GridId::new(1);

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

    let x_ids: Vec<AxisId> = (0..x_axes.len())
        .map(|i| AxisId::new(1 + i as u64))
        .collect();
    let y_ids: Vec<AxisId> = (0..y_axes.len())
        .map(|i| AxisId::new(1 + x_axes.len() as u64 + i as u64))
        .collect();

    let mut axes: Vec<AxisSpec> = Vec::with_capacity(x_axes.len() + y_axes.len());

    for (i, axis) in x_axes.iter().enumerate() {
        if axis.grid_index.unwrap_or(0) != 0 {
            return Err(EchartsError::Unsupported("xAxis.gridIndex != 0"));
        }
        let axis_type = axis.kind.as_deref();
        if matches!(axis_type, Some("category")) {
            return Err(EchartsError::Unsupported(
                "xAxis.type = 'category' with dataset (v1 subset)",
            ));
        }
        axes.push(AxisSpec {
            id: x_ids[i],
            name: axis.name.clone(),
            kind: AxisKind::X,
            grid: grid_id,
            position: None,
            scale: match axis_type {
                Some("time") => AxisScale::Time(Default::default()),
                None | Some("value") => AxisScale::Value(ValueAxisScale),
                Some(_) => return Err(EchartsError::Unsupported("xAxis.type")),
            },
            range: None,
        });
    }

    for (i, axis) in y_axes.iter().enumerate() {
        if axis.grid_index.unwrap_or(0) != 0 {
            return Err(EchartsError::Unsupported("yAxis.gridIndex != 0"));
        }
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
            grid: grid_id,
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
    let mut dataset_dimensions_by_index: Vec<Option<Vec<String>>> =
        Vec::with_capacity(datasets.len());

    let mut next_field_id = 1u64;

    for (dataset_index, ds) in datasets.iter().enumerate() {
        let Some(source) = ds.source.as_ref() else {
            return Err(EchartsError::Invalid(
                "dataset.source is required (v1 subset)",
            ));
        };
        if source.is_empty() {
            return Err(EchartsError::Invalid("dataset.source is empty"));
        }

        let width = source[0].len();
        if width == 0 {
            return Err(EchartsError::Invalid("dataset.source row has zero width"));
        }

        for row in source.iter().skip(1) {
            if row.len() != width {
                return Err(EchartsError::Invalid(
                    "dataset.source rows must have consistent width",
                ));
            }
        }

        let mut columns: Vec<Vec<f64>> = vec![Vec::with_capacity(source.len()); width];
        for row in source {
            for (col, v) in row.iter().enumerate() {
                columns[col].push(parse_f64_value(v));
            }
        }

        let mut table = DataTable::default();
        for col in columns {
            table.push_column(Column::F64(col));
        }

        let dataset_id = DatasetId::new((dataset_index as u64).saturating_add(1));
        let mut fields: Vec<FieldSpec> = Vec::with_capacity(width);
        let mut field_ids: Vec<FieldId> = Vec::with_capacity(width);
        for col in 0..width {
            let id = FieldId::new(next_field_id);
            next_field_id = next_field_id.saturating_add(1);
            fields.push(FieldSpec { id, column: col });
            field_ids.push(id);
        }

        out_datasets.push((dataset_id, table));
        dataset_specs.push(DatasetSpec {
            id: dataset_id,
            fields,
        });
        dataset_fields_by_index.push(field_ids);
        dataset_dimensions_by_index.push(ds.dimensions.clone());
    }

    let mut spec = ChartSpec {
        id: ChartId::new(1),
        viewport: None,
        datasets: dataset_specs,
        grids: vec![GridSpec { id: grid_id }],
        axes,
        data_zoom_x: Vec::new(),
        data_zoom_y: Vec::new(),
        tooltip,
        axis_pointer: Some(Default::default()),
        visual_maps: Vec::new(),
        series: Vec::new(),
    };

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
        let Some(dataset) = spec.datasets.get(dataset_index) else {
            return Err(EchartsError::Invalid("series.datasetIndex out of range"));
        };
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
                .and_then(|v| v.as_ref()),
        )?;
        let y_col = resolve_encode_dim(
            &encode_y,
            dataset_dimensions_by_index
                .get(dataset_index)
                .and_then(|v| v.as_ref()),
        )?;

        let x_field = field_ids.get(x_col).copied().ok_or_else(|| {
            EchartsError::Invalid("series.encode.x column out of range for dataset")
        })?;
        let y_field = field_ids.get(y_col).copied().ok_or_else(|| {
            EchartsError::Invalid("series.encode.y column out of range for dataset")
        })?;

        let x_axis_index = series.x_axis_index.unwrap_or(0);
        let y_axis_index = series.y_axis_index.unwrap_or(0);
        let x_axis = *x_ids
            .get(x_axis_index)
            .ok_or(EchartsError::Invalid("series.xAxisIndex out of range"))?;
        let y_axis = *y_ids
            .get(y_axis_index)
            .ok_or(EchartsError::Invalid("series.yAxisIndex out of range"))?;

        let series_id = SeriesId::new((series_index as u64).saturating_add(1));
        spec.series.push(SeriesSpec {
            id: series_id,
            name: series.name.clone(),
            kind,
            dataset: dataset.id,
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

    Ok(TranslatedChart {
        spec,
        datasets: out_datasets,
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
        assert_eq!(ds.row_count, 3);
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
        assert_eq!(translated.datasets[0].1.row_count, 3);
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
        assert_eq!(translated.datasets[0].1.row_count, 3);
        assert_eq!(translated.spec.datasets.len(), 1);
        assert_eq!(translated.spec.datasets[0].fields.len(), 2);
        assert_eq!(translated.spec.axes.len(), 4);

        let series = &translated.spec.series[0];
        assert_eq!(series.x_axis, AxisId::new(2));
        assert_eq!(series.y_axis, AxisId::new(4));
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
}
