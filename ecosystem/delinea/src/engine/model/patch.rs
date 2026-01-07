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

        if mode == PatchMode::Replace || self.replace_families.contains(&ReplaceFamily::Viewport) {
            if self.viewport.is_some() {
                report.viewport_changed = true;
                report.marks_changed = true;
            }
        }

        match mode {
            PatchMode::Replace => {
                if self.viewport.is_some() {
                    model.viewport = self.viewport.unwrap();
                    model.revs.bump_layout();
                    report.marks_changed = true;
                }

                model.datasets.clear();
                model.grids.clear();
                model.axes.clear();
                model.series.clear();
                model.series_order.clear();
                report.structure_changed = true;

                for op in self.datasets {
                    if let DatasetOp::Upsert { id } = op {
                        model
                            .datasets
                            .insert(id, crate::engine::model::DatasetModel { id });
                    }
                }
                for op in self.grids {
                    if let GridOp::Upsert { id } = op {
                        model.grids.insert(id, GridModel { id });
                    }
                }
                for op in self.axes {
                    if let AxisOp::Upsert(axis) = op {
                        if !model.grids.contains_key(&axis.grid) {
                            return Err(ModelError::MissingReference { kind: "grid" });
                        }
                        model.axes.insert(axis.id, AxisModel::from(axis));
                    }
                }
                for op in self.series {
                    if let SeriesOp::Upsert(series) = op {
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
                // Apply replacement for explicit families, otherwise merge.
                if self.replace_families.contains(&ReplaceFamily::Datasets) {
                    model.datasets.clear();
                    report.structure_changed = true;
                }
                if self.replace_families.contains(&ReplaceFamily::Grids) {
                    model.grids.clear();
                    report.structure_changed = true;
                }
                if self.replace_families.contains(&ReplaceFamily::Axes) {
                    model.axes.clear();
                    report.structure_changed = true;
                }
                if self.replace_families.contains(&ReplaceFamily::Series) {
                    model.series.clear();
                    model.series_order.clear();
                    report.structure_changed = true;
                }
                if self.replace_families.contains(&ReplaceFamily::Viewport)
                    && self.viewport.is_some()
                {
                    model.viewport = self.viewport.unwrap();
                    model.revs.bump_layout();
                    report.viewport_changed = true;
                    report.marks_changed = true;
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
                        model.series_order.push(series.id);
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
                        model.series_order.retain(|s| *s != id);
                        report.structure_changed = true;
                    }
                }
            }
        }

        if report.structure_changed {
            model.revs.bump_spec();
            report.marks_changed = true;
        }

        Ok(report)
    }
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
