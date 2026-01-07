use std::collections::BTreeSet;

use fret_core::Rect;

use crate::engine::model::{AxisModel, ChartModel, GridModel, ModelError, SeriesModel};
use crate::ids::{AxisId, DatasetId, GridId, SeriesId};
use crate::spec::{AxisKind, SeriesKind};

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
            }
        }

        match mode {
            PatchMode::Replace => {
                if self.viewport.is_some() {
                    model.viewport = self.viewport.unwrap();
                    model.revs.bump_layout();
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
                }
            }
            PatchMode::Merge => {}
        }

        if let Some(vp) = self.viewport {
            model.viewport = vp;
            model.revs.bump_layout();
            report.viewport_changed = true;
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
                    model.axes.insert(axis.id, AxisModel::from(axis));
                    report.structure_changed = true;
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

                    if model.series.contains_key(&series.id) {
                        model.series.insert(series.id, SeriesModel::from(series));
                    } else {
                        model.series_order.push(series.id);
                        model.series.insert(series.id, SeriesModel::from(series));
                    }
                    report.structure_changed = true;
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
        }

        Ok(report)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PatchReport {
    pub viewport_changed: bool,
    pub structure_changed: bool,
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
}

impl From<AxisPatch> for AxisModel {
    fn from(p: AxisPatch) -> Self {
        Self {
            id: p.id,
            kind: p.kind,
            grid: p.grid,
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
