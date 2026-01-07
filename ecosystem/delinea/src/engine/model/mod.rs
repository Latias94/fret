mod patch;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use fret_core::Rect;
use thiserror::Error;

use crate::ids::{AxisId, ChartId, DatasetId, FieldId, GridId, Revision, SeriesId};
use crate::spec::{AreaBaseline, AxisKind, AxisRange, SeriesEncode, SeriesKind};

pub use patch::*;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("duplicate id: {kind}")]
    DuplicateId { kind: &'static str },
    #[error("missing referenced id: {kind}")]
    MissingReference { kind: &'static str },
    #[error("invalid spec: {reason}")]
    InvalidSpec { reason: &'static str },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ModelRevisions {
    pub spec: Revision,
    pub data: Revision,
    pub layout: Revision,
    pub visual: Revision,
    pub marks: Revision,
}

impl ModelRevisions {
    pub fn bump_spec(&mut self) {
        self.spec.bump();
        self.marks.bump();
    }

    pub fn bump_data(&mut self) {
        self.data.bump();
        self.marks.bump();
    }

    pub fn bump_layout(&mut self) {
        self.layout.bump();
        self.marks.bump();
    }

    pub fn bump_visual(&mut self) {
        self.visual.bump();
        self.marks.bump();
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChartModel {
    pub id: ChartId,
    pub viewport: Option<Rect>,

    pub datasets: BTreeMap<DatasetId, DatasetModel>,
    pub grids: BTreeMap<GridId, GridModel>,
    pub axes: BTreeMap<AxisId, AxisModel>,

    pub series_order: Vec<SeriesId>,
    pub series: BTreeMap<SeriesId, SeriesModel>,

    pub revs: ModelRevisions,
}

impl ChartModel {
    pub fn from_spec(spec: crate::spec::ChartSpec) -> Result<Self, ModelError> {
        let mut model = Self {
            id: spec.id,
            viewport: spec.viewport,
            datasets: BTreeMap::default(),
            grids: BTreeMap::default(),
            axes: BTreeMap::default(),
            series_order: Vec::default(),
            series: BTreeMap::default(),
            revs: ModelRevisions::default(),
        };

        let mut dataset_ids: BTreeSet<DatasetId> = BTreeSet::new();
        for dataset in spec.datasets {
            if !dataset_ids.insert(dataset.id) {
                return Err(ModelError::DuplicateId { kind: "dataset" });
            }
            let mut fields = BTreeMap::<FieldId, usize>::new();
            for field in dataset.fields {
                if fields.insert(field.id, field.column).is_some() {
                    return Err(ModelError::DuplicateId {
                        kind: "dataset.field",
                    });
                }
            }
            model.datasets.insert(
                dataset.id,
                DatasetModel {
                    id: dataset.id,
                    fields,
                },
            );
        }

        let mut grid_ids: BTreeSet<GridId> = BTreeSet::new();
        for grid in spec.grids {
            if !grid_ids.insert(grid.id) {
                return Err(ModelError::DuplicateId { kind: "grid" });
            }
            model.grids.insert(grid.id, GridModel { id: grid.id });
        }

        let mut axis_ids: BTreeSet<AxisId> = BTreeSet::new();
        for axis in spec.axes {
            if !axis_ids.insert(axis.id) {
                return Err(ModelError::DuplicateId { kind: "axis" });
            }
            if !model.grids.contains_key(&axis.grid) {
                return Err(ModelError::MissingReference { kind: "grid" });
            }
            model.axes.insert(
                axis.id,
                AxisModel {
                    id: axis.id,
                    kind: axis.kind,
                    grid: axis.grid,
                    range: {
                        let mut range = axis.range.unwrap_or_default();
                        range.clamp_non_degenerate();
                        range
                    },
                },
            );
        }

        let mut series_ids: BTreeSet<SeriesId> = BTreeSet::new();
        for series in spec.series {
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

            model.series_order.push(series.id);
            model.series.insert(
                series.id,
                SeriesModel {
                    id: series.id,
                    kind: series.kind,
                    dataset: series.dataset,
                    encode: series.encode,
                    x_axis: series.x_axis,
                    y_axis: series.y_axis,
                    visible: true,
                    area_baseline: series.area_baseline.unwrap_or_default(),
                },
            );
        }

        model.revs.bump_spec();
        Ok(model)
    }

    pub fn series_in_order(&self) -> impl Iterator<Item = &SeriesModel> {
        self.series_order
            .iter()
            .filter_map(|id| self.series.get(id))
    }

    pub fn apply_patch(
        &mut self,
        patch: ChartPatch,
        mode: PatchMode,
    ) -> Result<PatchReport, ModelError> {
        patch.apply(self, mode)
    }
}

#[derive(Debug, Clone)]
pub struct DatasetModel {
    pub id: DatasetId,
    pub fields: BTreeMap<FieldId, usize>,
}

#[derive(Debug, Clone)]
pub struct GridModel {
    pub id: GridId,
}

#[derive(Debug, Clone)]
pub struct AxisModel {
    pub id: AxisId,
    pub kind: AxisKind,
    pub grid: GridId,
    pub range: AxisRange,
}

#[derive(Debug, Clone)]
pub struct SeriesModel {
    pub id: SeriesId,
    pub kind: SeriesKind,
    pub dataset: DatasetId,
    pub encode: SeriesEncode,
    pub x_axis: AxisId,
    pub y_axis: AxisId,
    pub visible: bool,
    pub area_baseline: AreaBaseline,
}
