use std::collections::BTreeSet;

use fret_core::Rect;

use crate::engine::model::{AxisModel, ChartModel, GridModel, ModelError, SeriesModel};
use crate::ids::{AxisId, DatasetId, GridId, SeriesId};
use crate::spec::{AxisKind, AxisRange, SeriesKind};

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
            if self.viewport.is_some() {
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
                    if let DatasetOp::Upsert { id } = op {
                        if !dataset_ids.insert(id) {
                            return Err(ModelError::DuplicateId { kind: "dataset" });
                        }
                        model
                            .datasets
                            .insert(id, crate::engine::model::DatasetModel { id });
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

                let mut axis_ids = BTreeSet::<AxisId>::new();
                for op in self.axes {
                    if let AxisOp::Upsert(axis) = op {
                        if !axis_ids.insert(axis.id) {
                            return Err(ModelError::DuplicateId { kind: "axis" });
                        }
                        if !model.grids.contains_key(&axis.grid) {
                            return Err(ModelError::MissingReference { kind: "grid" });
                        }
                        model.axes.insert(axis.id, AxisModel::from(axis));
                    }
                }

                let mut series_ids = BTreeSet::<SeriesId>::new();
                for op in self.series {
                    if let SeriesOp::Upsert(series) = op {
                        if !series_ids.insert(series.id) {
                            return Err(ModelError::DuplicateId { kind: "series" });
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

        for op in self.datasets {
            match op {
                DatasetOp::Upsert { id } => {
                    model
                        .datasets
                        .insert(id, crate::engine::model::DatasetModel { id });
                    report.structure_changed = true;
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
                    let Some(existing) = model.axes.get_mut(&axis.id) else {
                        model.axes.insert(axis.id, AxisModel::from(axis));
                        report.structure_changed = true;
                        continue;
                    };

                    if existing.kind != axis.kind || existing.grid != axis.grid {
                        existing.kind = axis.kind;
                        existing.grid = axis.grid;
                        report.structure_changed = true;
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

                    let Some(existing) = model.series.get_mut(&series.id) else {
                        if series_replace_order.is_none() {
                            model.series_order.push(series.id);
                        }
                        model.series.insert(series.id, SeriesModel::from(series));
                        report.structure_changed = true;
                        continue;
                    };

                    if existing.kind != series.kind
                        || existing.dataset != series.dataset
                        || existing.x_col != series.x_col
                        || existing.y_col != series.y_col
                        || existing.x_axis != series.x_axis
                        || existing.y_axis != series.y_axis
                    {
                        existing.kind = series.kind;
                        existing.dataset = series.dataset;
                        existing.x_col = series.x_col;
                        existing.y_col = series.y_col;
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
        if let DatasetOp::Upsert { id } = op {
            if !desired.insert(*id) {
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
    }

    Ok(())
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
    Upsert { id: DatasetId },
    Remove { id: DatasetId },
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
    pub kind: AxisKind,
    pub grid: GridId,
    pub range: Option<AxisRange>,
}

impl From<AxisPatch> for AxisModel {
    fn from(p: AxisPatch) -> Self {
        let mut range = p.range.unwrap_or_default();
        range.clamp_non_degenerate();
        Self {
            id: p.id,
            kind: p.kind,
            grid: p.grid,
            range,
        }
    }
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
    pub kind: SeriesKind,
    pub dataset: DatasetId,
    pub x_col: usize,
    pub y_col: usize,
    pub x_axis: AxisId,
    pub y_axis: AxisId,
    pub visible: Option<bool>,
}

impl From<SeriesPatch> for SeriesModel {
    fn from(p: SeriesPatch) -> Self {
        Self {
            id: p.id,
            kind: p.kind,
            dataset: p.dataset,
            x_col: p.x_col,
            y_col: p.y_col,
            x_axis: p.x_axis,
            y_axis: p.y_axis,
            visible: p.visible.unwrap_or(true),
        }
    }
}
