use std::collections::{BTreeMap, BTreeSet};

use fret_core::Rect;

use crate::engine::model::{
    AxisModel, ChartModel, DatasetModel, GridModel, ModelError, SeriesModel,
};
use crate::ids::{AxisId, DatasetId, FieldId, GridId, SeriesId, StackId};
use crate::scale::AxisScale;
use crate::spec::{
    AreaBaseline, AxisKind, AxisPosition, AxisRange, FieldSpec, SeriesEncode, SeriesKind,
    StackStrategy,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PatchMode {
    Merge,
    Replace,
    ReplaceMerge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ReplaceFamily {
    Viewport,
    Datasets,
    Grids,
    Axes,
    Series,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChartPatch {
    pub replace_families: BTreeSet<ReplaceFamily>,

    pub viewport: Option<Option<Rect>>,
    pub plot_viewports_by_grid: BTreeMap<GridId, Option<Rect>>,

    pub datasets: Vec<DatasetOp>,
    pub grids: Vec<GridOp>,
    pub axes: Vec<AxisOp>,
    pub series: Vec<SeriesOp>,
}

impl ChartPatch {
    pub fn apply(self, model: &mut ChartModel, mode: PatchMode) -> Result<PatchReport, ModelError> {
        let mut report = PatchReport::default();
        let mut series_replace_order: Option<Vec<SeriesId>> = None;

        if mode == PatchMode::Replace || self.replace_families.contains(&ReplaceFamily::Viewport) {
            if self.viewport.is_some() || !self.plot_viewports_by_grid.is_empty() {
                report.viewport_changed = true;
                report.marks_changed = true;
            }
        }

        match mode {
            PatchMode::Replace => {
                if let Some(vp) = self.viewport {
                    model.viewport = vp;
                    model.revs.bump_layout();
                    report.marks_changed = true;
                }

                model.datasets.clear();
                model.grids.clear();
                model.axes.clear();
                model.series.clear();
                model.series_order.clear();
                report.structure_changed = true;

                let mut dataset_ids = BTreeSet::<DatasetId>::new();
                for op in self.datasets {
                    if let DatasetOp::Upsert(dataset) = op {
                        if !dataset_ids.insert(dataset.id) {
                            return Err(ModelError::DuplicateId { kind: "dataset" });
                        }
                        model.datasets.insert(
                            dataset.id,
                            DatasetModel {
                                id: dataset.id,
                                fields: dataset_fields_map(&dataset)?,
                            },
                        );
                    }
                }

                let mut grid_ids = BTreeSet::<GridId>::new();
                for op in self.grids {
                    if let GridOp::Upsert { id } = op {
                        if !grid_ids.insert(id) {
                            return Err(ModelError::DuplicateId { kind: "grid" });
                        }
                        model.grids.insert(id, GridModel { id });
                    }
                }

                model.plot_viewports_by_grid.clear();
                for (grid, vp) in self.plot_viewports_by_grid {
                    if !model.grids.contains_key(&grid) {
                        return Err(ModelError::MissingReference {
                            kind: "plot_viewports_by_grid.grid",
                        });
                    }
                    if let Some(vp) = vp {
                        model.plot_viewports_by_grid.insert(grid, vp);
                    }
                }
                if !model.plot_viewports_by_grid.is_empty() {
                    model.revs.bump_layout();
                    report.marks_changed = true;
                }

                let mut axis_ids = BTreeSet::<AxisId>::new();
                for op in self.axes {
                    if let AxisOp::Upsert(axis) = op {
                        if !axis_ids.insert(axis.id) {
                            return Err(ModelError::DuplicateId { kind: "axis" });
                        }
                        if !model.grids.contains_key(&axis.grid) {
                            return Err(ModelError::MissingReference { kind: "grid" });
                        }
                        let model_axis = axis_model_from_patch(axis)?;
                        model.axes.insert(model_axis.id, model_axis);
                    }
                }

                let mut series_ids = BTreeSet::<SeriesId>::new();
                for op in self.series {
                    if let SeriesOp::Upsert(series) = op {
                        if !series_ids.insert(series.id) {
                            return Err(ModelError::DuplicateId { kind: "series" });
                        }
                        if series.kind == SeriesKind::Band && series.encode.y2.is_none() {
                            return Err(ModelError::InvalidSpec {
                                reason: "series.kind=Band requires encode.y2",
                            });
                        }
                        if !model.datasets.contains_key(&series.dataset) {
                            return Err(ModelError::MissingReference { kind: "dataset" });
                        }
                        if !model.axes.contains_key(&series.x_axis) {
                            return Err(ModelError::MissingReference {
                                kind: "axis.x_axis",
                            });
                        }
                        if !model.axes.contains_key(&series.y_axis) {
                            return Err(ModelError::MissingReference {
                                kind: "axis.y_axis",
                            });
                        }
                        model.series_order.push(series.id);
                        model.series.insert(series.id, SeriesModel::from(series));
                    }
                }

                model.revs.bump_spec();
                report.marks_changed = true;
                return Ok(report);
            }
            PatchMode::ReplaceMerge => {
                // ECharts-inspired semantics for "replaceMerge":
                // - families listed in `replace_families` are treated like a new option list:
                //   items not listed are removed,
                //   items with matching IDs are kept and merged,
                //   new IDs are inserted.

                if self.replace_families.contains(&ReplaceFamily::Datasets) {
                    let desired = desired_dataset_ids(&self.datasets)?;
                    let before = model.datasets.len();
                    model.datasets.retain(|id, _| desired.contains(id));
                    if model.datasets.len() != before {
                        report.structure_changed = true;
                    }
                }

                if self.replace_families.contains(&ReplaceFamily::Grids) {
                    let desired = desired_grid_ids(&self.grids)?;
                    let before = model.grids.len();
                    model.grids.retain(|id, _| desired.contains(id));
                    if model.grids.len() != before {
                        report.structure_changed = true;
                    }
                }

                if self.replace_families.contains(&ReplaceFamily::Axes) {
                    let desired = desired_axis_ids(&self.axes)?;
                    let before = model.axes.len();
                    model.axes.retain(|id, _| desired.contains(id));
                    if model.axes.len() != before {
                        report.structure_changed = true;
                    }
                }

                if self.replace_families.contains(&ReplaceFamily::Series) {
                    let (desired, order) = desired_series_ids_and_order(&self.series)?;

                    let before = model.series.len();
                    model.series.retain(|id, _| desired.contains(id));
                    model.series_order.retain(|id| desired.contains(id));
                    if model.series.len() != before {
                        report.structure_changed = true;
                    }

                    series_replace_order = Some(order);
                }
            }
            PatchMode::Merge => {}
        }

        if let Some(vp) = self.viewport {
            model.viewport = vp;
            model.revs.bump_layout();
            report.viewport_changed = true;
            report.marks_changed = true;
        }

        if !self.plot_viewports_by_grid.is_empty() {
            let mut changed = false;
            for (grid, vp) in self.plot_viewports_by_grid {
                if !model.grids.contains_key(&grid) {
                    return Err(ModelError::MissingReference {
                        kind: "plot_viewports_by_grid.grid",
                    });
                }
                match vp {
                    Some(vp) => {
                        if model
                            .plot_viewports_by_grid
                            .get(&grid)
                            .is_none_or(|existing| *existing != vp)
                        {
                            model.plot_viewports_by_grid.insert(grid, vp);
                            changed = true;
                        }
                    }
                    None => {
                        if model.plot_viewports_by_grid.remove(&grid).is_some() {
                            changed = true;
                        }
                    }
                }
            }
            if changed {
                model.revs.bump_layout();
                report.viewport_changed = true;
                report.marks_changed = true;
            }
        }

        for op in self.datasets {
            match op {
                DatasetOp::Upsert(dataset) => {
                    let next = DatasetModel {
                        id: dataset.id,
                        fields: dataset_fields_map(&dataset)?,
                    };
                    if model
                        .datasets
                        .get(&dataset.id)
                        .is_none_or(|existing| existing.fields != next.fields)
                    {
                        model.datasets.insert(dataset.id, next);
                        report.structure_changed = true;
                    }
                }
                DatasetOp::Remove { id } => {
                    if model.datasets.remove(&id).is_some() {
                        report.structure_changed = true;
                    }
                }
            }
        }

        for op in self.grids {
            match op {
                GridOp::Upsert { id } => {
                    model.grids.insert(id, GridModel { id });
                    report.structure_changed = true;
                }
                GridOp::Remove { id } => {
                    if model.grids.remove(&id).is_some() {
                        report.structure_changed = true;
                    }
                }
            }
        }

        for op in self.axes {
            match op {
                AxisOp::Upsert(axis) => {
                    if !model.grids.contains_key(&axis.grid) {
                        return Err(ModelError::MissingReference { kind: "grid" });
                    }
                    let position = axis
                        .position
                        .unwrap_or_else(|| AxisPosition::default_for_kind(axis.kind));
                    if !position.is_compatible(axis.kind) {
                        return Err(ModelError::InvalidSpec {
                            reason: "axis.position must be compatible with axis.kind",
                        });
                    }
                    let Some(existing) = model.axes.get_mut(&axis.id) else {
                        let model_axis = axis_model_from_patch(axis)?;
                        model.axes.insert(model_axis.id, model_axis);
                        report.structure_changed = true;
                        continue;
                    };

                    if let Some(name) = axis.name.and_then(sanitize_name)
                        && existing.name.as_deref() != Some(name.as_str())
                    {
                        existing.name = Some(name);
                    }

                    if existing.kind != axis.kind || existing.grid != axis.grid {
                        existing.kind = axis.kind;
                        existing.grid = axis.grid;
                        report.structure_changed = true;
                    }

                    if existing.position != position {
                        existing.position = position;
                    }

                    if let Some(scale) = axis.scale
                        && existing.scale != scale
                    {
                        existing.scale = scale;
                        model.revs.bump_layout();
                        report.structure_changed = true;
                        report.marks_changed = true;
                    }

                    if let Some(mut range) = axis.range {
                        range.clamp_non_degenerate();
                        if existing.range != range {
                            existing.range = range;
                            model.revs.bump_layout();
                            report.marks_changed = true;
                        }
                    }
                }
                AxisOp::Remove { id } => {
                    if model.axes.remove(&id).is_some() {
                        report.structure_changed = true;
                    }
                }
            }
        }

        for op in self.series {
            match op {
                SeriesOp::Upsert(series) => {
                    if series.kind == SeriesKind::Band && series.encode.y2.is_none() {
                        return Err(ModelError::InvalidSpec {
                            reason: "series.kind=Band requires encode.y2",
                        });
                    }
                    if series.kind != SeriesKind::Bar && series.bar_layout != Default::default() {
                        return Err(ModelError::InvalidSpec {
                            reason: "bar_layout is only supported for Bar series in v1",
                        });
                    }
                    if series.kind == SeriesKind::Bar {
                        let Some(x_axis) = model.axes.get(&series.x_axis) else {
                            return Err(ModelError::MissingReference {
                                kind: "axis.x_axis",
                            });
                        };
                        if !matches!(x_axis.scale, crate::scale::AxisScale::Category(_)) {
                            return Err(ModelError::InvalidSpec {
                                reason: "series.kind=Bar requires a Category x axis",
                            });
                        }
                    }
                    if !model.datasets.contains_key(&series.dataset) {
                        return Err(ModelError::MissingReference { kind: "dataset" });
                    }
                    if !model.axes.contains_key(&series.x_axis) {
                        return Err(ModelError::MissingReference {
                            kind: "axis.x_axis",
                        });
                    }
                    if !model.axes.contains_key(&series.y_axis) {
                        return Err(ModelError::MissingReference {
                            kind: "axis.y_axis",
                        });
                    }

                    let Some(dataset) = model.datasets.get(&series.dataset) else {
                        return Err(ModelError::MissingReference { kind: "dataset" });
                    };
                    if !dataset.fields.contains_key(&series.encode.x) {
                        return Err(ModelError::MissingReference {
                            kind: "dataset.field.x",
                        });
                    }
                    if !dataset.fields.contains_key(&series.encode.y) {
                        return Err(ModelError::MissingReference {
                            kind: "dataset.field.y",
                        });
                    }
                    if let Some(y2) = series.encode.y2
                        && !dataset.fields.contains_key(&y2)
                    {
                        return Err(ModelError::MissingReference {
                            kind: "dataset.field.y2",
                        });
                    }

                    let Some(existing) = model.series.get_mut(&series.id) else {
                        if series_replace_order.is_none() {
                            model.series_order.push(series.id);
                        }
                        model.series.insert(series.id, SeriesModel::from(series));
                        report.structure_changed = true;
                        continue;
                    };

                    if let Some(name) = series.name.and_then(sanitize_name)
                        && existing.name.as_deref() != Some(name.as_str())
                    {
                        existing.name = Some(name);
                    }

                    if existing.kind != series.kind
                        || existing.dataset != series.dataset
                        || existing.encode != series.encode
                        || existing.x_axis != series.x_axis
                        || existing.y_axis != series.y_axis
                    {
                        existing.kind = series.kind;
                        existing.dataset = series.dataset;
                        existing.encode = series.encode;
                        existing.x_axis = series.x_axis;
                        existing.y_axis = series.y_axis;
                        report.structure_changed = true;
                    }

                    if let Some(visible) = series.visible {
                        if existing.visible != visible {
                            existing.visible = visible;
                            model.revs.bump_visual();
                            report.marks_changed = true;
                        }
                    }

                    if let Some(baseline) = series.area_baseline
                        && existing.area_baseline != baseline
                    {
                        existing.area_baseline = baseline;
                        model.revs.bump_visual();
                        report.marks_changed = true;
                    }

                    if let Some(lod) = series.lod
                        && existing.lod != lod
                    {
                        existing.lod = lod;
                        model.revs.bump_visual();
                        report.marks_changed = true;
                    }
                }
                SeriesOp::Remove { id } => {
                    if model.series.remove(&id).is_some() {
                        if series_replace_order.is_none() {
                            model.series_order.retain(|s| *s != id);
                        }
                        report.structure_changed = true;
                    }
                }
            }
        }

        if let Some(order) = series_replace_order {
            let new_order: Vec<SeriesId> = order
                .into_iter()
                .filter(|id| model.series.contains_key(id))
                .collect();

            if model.series_order != new_order {
                model.series_order = new_order;
                report.structure_changed = true;
            }
        }

        if report.structure_changed {
            validate_references(model)?;
            model.revs.bump_spec();
            report.marks_changed = true;
        }

        Ok(report)
    }
}

fn desired_dataset_ids(ops: &[DatasetOp]) -> Result<BTreeSet<DatasetId>, ModelError> {
    let mut desired = BTreeSet::<DatasetId>::new();
    for op in ops {
        if let DatasetOp::Upsert(dataset) = op {
            if !desired.insert(dataset.id) {
                return Err(ModelError::DuplicateId { kind: "dataset" });
            }
        }
    }
    Ok(desired)
}

fn desired_grid_ids(ops: &[GridOp]) -> Result<BTreeSet<GridId>, ModelError> {
    let mut desired = BTreeSet::<GridId>::new();
    for op in ops {
        if let GridOp::Upsert { id } = op {
            if !desired.insert(*id) {
                return Err(ModelError::DuplicateId { kind: "grid" });
            }
        }
    }
    Ok(desired)
}

fn desired_axis_ids(ops: &[AxisOp]) -> Result<BTreeSet<AxisId>, ModelError> {
    let mut desired = BTreeSet::<AxisId>::new();
    for op in ops {
        if let AxisOp::Upsert(axis) = op {
            if !desired.insert(axis.id) {
                return Err(ModelError::DuplicateId { kind: "axis" });
            }
        }
    }
    Ok(desired)
}

fn desired_series_ids_and_order(
    ops: &[SeriesOp],
) -> Result<(BTreeSet<SeriesId>, Vec<SeriesId>), ModelError> {
    let mut desired = BTreeSet::<SeriesId>::new();
    let mut order = Vec::<SeriesId>::new();

    for op in ops {
        if let SeriesOp::Upsert(series) = op {
            if !desired.insert(series.id) {
                return Err(ModelError::DuplicateId { kind: "series" });
            }
            order.push(series.id);
        }
    }

    Ok((desired, order))
}

fn validate_references(model: &ChartModel) -> Result<(), ModelError> {
    for axis in model.axes.values() {
        if !model.grids.contains_key(&axis.grid) {
            return Err(ModelError::MissingReference { kind: "grid" });
        }
    }

    for series in model.series.values() {
        if !model.datasets.contains_key(&series.dataset) {
            return Err(ModelError::MissingReference { kind: "dataset" });
        }
        let Some(dataset) = model.datasets.get(&series.dataset) else {
            return Err(ModelError::MissingReference { kind: "dataset" });
        };
        if series.kind == SeriesKind::Band && series.encode.y2.is_none() {
            return Err(ModelError::InvalidSpec {
                reason: "series.kind=Band requires encode.y2",
            });
        }
        if !dataset.fields.contains_key(&series.encode.x) {
            return Err(ModelError::MissingReference {
                kind: "dataset.field.x",
            });
        }
        if !dataset.fields.contains_key(&series.encode.y) {
            return Err(ModelError::MissingReference {
                kind: "dataset.field.y",
            });
        }
        if let Some(y2) = series.encode.y2
            && !dataset.fields.contains_key(&y2)
        {
            return Err(ModelError::MissingReference {
                kind: "dataset.field.y2",
            });
        }
        if !model.axes.contains_key(&series.x_axis) {
            return Err(ModelError::MissingReference {
                kind: "axis.x_axis",
            });
        }
        if !model.axes.contains_key(&series.y_axis) {
            return Err(ModelError::MissingReference {
                kind: "axis.y_axis",
            });
        }

        if series.stack.is_some()
            && !matches!(
                series.kind,
                SeriesKind::Line | SeriesKind::Area | SeriesKind::Bar
            )
        {
            return Err(ModelError::InvalidSpec {
                reason: "stack is only supported for line/area/bar in v1",
            });
        }
    }

    let mut stack_groups: BTreeMap<StackId, (AxisId, AxisId, DatasetId, FieldId, StackStrategy)> =
        BTreeMap::new();
    for s in model.series.values() {
        let Some(stack) = s.stack else {
            continue;
        };
        let key = (s.x_axis, s.y_axis, s.dataset, s.encode.x, s.stack_strategy);
        if let Some(existing) = stack_groups.insert(stack, key) {
            if existing != key {
                return Err(ModelError::StackGroupMismatch { stack });
            }
        }
    }

    Ok(())
}

fn dataset_fields_map(patch: &DatasetPatch) -> Result<BTreeMap<FieldId, usize>, ModelError> {
    let mut fields = BTreeMap::<FieldId, usize>::new();
    for field in &patch.fields {
        if fields.insert(field.id, field.column).is_some() {
            return Err(ModelError::DuplicateId {
                kind: "dataset.field",
            });
        }
    }
    Ok(fields)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PatchReport {
    pub viewport_changed: bool,
    pub structure_changed: bool,
    pub marks_changed: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DatasetOp {
    Upsert(DatasetPatch),
    Remove { id: DatasetId },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetPatch {
    pub id: DatasetId,
    pub fields: Vec<FieldSpec>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GridOp {
    Upsert { id: GridId },
    Remove { id: GridId },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisOp {
    Upsert(AxisPatch),
    Remove { id: AxisId },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisPatch {
    pub id: AxisId,
    pub name: Option<String>,
    pub kind: AxisKind,
    pub grid: GridId,
    pub position: Option<AxisPosition>,
    pub scale: Option<AxisScale>,
    pub range: Option<AxisRange>,
}

fn axis_model_from_patch(p: AxisPatch) -> Result<AxisModel, ModelError> {
    let position = p
        .position
        .unwrap_or_else(|| AxisPosition::default_for_kind(p.kind));
    if !position.is_compatible(p.kind) {
        return Err(ModelError::InvalidSpec {
            reason: "axis.position must be compatible with axis.kind",
        });
    }

    let mut range = p.range.unwrap_or_default();
    range.clamp_non_degenerate();
    Ok(AxisModel {
        id: p.id,
        name: p.name.and_then(sanitize_name),
        kind: p.kind,
        grid: p.grid,
        position,
        scale: p.scale.unwrap_or_default(),
        range,
    })
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SeriesOp {
    Upsert(SeriesPatch),
    Remove { id: SeriesId },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesPatch {
    pub id: SeriesId,
    pub name: Option<String>,
    pub kind: SeriesKind,
    pub dataset: DatasetId,
    pub encode: SeriesEncode,
    pub x_axis: AxisId,
    pub y_axis: AxisId,
    pub stack: Option<StackId>,
    pub stack_strategy: StackStrategy,
    pub bar_layout: crate::spec::BarLayoutSpec,
    pub visible: Option<bool>,
    pub area_baseline: Option<AreaBaseline>,
    pub lod: Option<crate::spec::SeriesLodSpecV1>,
}

impl From<SeriesPatch> for SeriesModel {
    fn from(p: SeriesPatch) -> Self {
        Self {
            id: p.id,
            name: p.name.and_then(sanitize_name),
            kind: p.kind,
            dataset: p.dataset,
            encode: p.encode,
            x_axis: p.x_axis,
            y_axis: p.y_axis,
            stack: p.stack,
            stack_strategy: p.stack_strategy,
            visible: p.visible.unwrap_or(true),
            area_baseline: p.area_baseline.unwrap_or_default(),
            bar_layout: p.bar_layout,
            lod: p.lod.unwrap_or_default(),
        }
    }
}

fn sanitize_name(name: String) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        None
    } else if trimmed.len() == name.len() {
        Some(name)
    } else {
        Some(trimmed.to_string())
    }
}
