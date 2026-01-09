mod patch;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use fret_core::Rect;
use thiserror::Error;

use crate::ids::{
    AxisId, ChartId, DataZoomId, DatasetId, FieldId, GridId, Revision, SeriesId, StackId,
};
use crate::scale::AxisScale;
use crate::spec::{
    AreaBaseline, AxisKind, AxisPointerTrigger, AxisPosition, AxisRange, FilterMode, SeriesEncode,
    SeriesKind, StackStrategy,
};

pub use patch::*;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("duplicate id: {kind}")]
    DuplicateId { kind: &'static str },
    #[error("missing referenced id: {kind}")]
    MissingReference { kind: &'static str },
    #[error("invalid spec: {reason}")]
    InvalidSpec { reason: &'static str },
    #[error("stack group mismatch: stack={stack:?}")]
    StackGroupMismatch { stack: StackId },
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
    pub data_zoom_x: BTreeMap<DataZoomId, DataZoomXModel>,
    pub data_zoom_x_by_axis: BTreeMap<AxisId, DataZoomId>,
    pub axis_pointer: Option<AxisPointerModel>,

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
            data_zoom_x: BTreeMap::default(),
            data_zoom_x_by_axis: BTreeMap::default(),
            axis_pointer: None,
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
            let position = axis
                .position
                .unwrap_or_else(|| AxisPosition::default_for_kind(axis.kind));
            if !position.is_compatible(axis.kind) {
                return Err(ModelError::InvalidSpec {
                    reason: "axis.position must be compatible with axis.kind",
                });
            }
            model.axes.insert(
                axis.id,
                AxisModel {
                    id: axis.id,
                    name: sanitize_name(axis.name),
                    kind: axis.kind,
                    grid: axis.grid,
                    position,
                    scale: axis.scale,
                    range: {
                        let mut range = axis.range.unwrap_or_default();
                        range.clamp_non_degenerate();
                        range
                    },
                },
            );
        }

        let mut zoom_ids: BTreeSet<DataZoomId> = BTreeSet::new();
        for zoom in spec.data_zoom_x {
            if !zoom_ids.insert(zoom.id) {
                return Err(ModelError::DuplicateId {
                    kind: "data_zoom_x",
                });
            }
            let Some(axis) = model.axes.get(&zoom.axis) else {
                return Err(ModelError::MissingReference {
                    kind: "data_zoom_x.axis",
                });
            };
            if axis.kind != AxisKind::X {
                return Err(ModelError::InvalidSpec {
                    reason: "data_zoom_x.axis must reference an X axis",
                });
            }
            if model
                .data_zoom_x_by_axis
                .insert(zoom.axis, zoom.id)
                .is_some()
            {
                return Err(ModelError::InvalidSpec {
                    reason: "multiple data_zoom_x specs reference the same axis",
                });
            }
            model.data_zoom_x.insert(
                zoom.id,
                DataZoomXModel {
                    id: zoom.id,
                    axis: zoom.axis,
                    filter_mode: zoom.filter_mode,
                },
            );
        }

        model.axis_pointer = spec.axis_pointer.map(|mut p| {
            p.trigger_distance_px = sanitize_px(p.trigger_distance_px, 12.0);
            p.throttle_px = sanitize_px(p.throttle_px, 0.75);
            AxisPointerModel {
                enabled: p.enabled,
                trigger: p.trigger,
                snap: p.snap,
                trigger_distance_px: p.trigger_distance_px,
                throttle_px: p.throttle_px,
            }
        });

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

            if series.stack.is_some() && !matches!(series.kind, SeriesKind::Line) {
                return Err(ModelError::InvalidSpec {
                    reason: "stack is only supported for line in v1",
                });
            }

            model.series_order.push(series.id);
            model.series.insert(
                series.id,
                SeriesModel {
                    id: series.id,
                    name: sanitize_name(series.name),
                    kind: series.kind,
                    dataset: series.dataset,
                    encode: series.encode,
                    x_axis: series.x_axis,
                    y_axis: series.y_axis,
                    stack: series.stack,
                    stack_strategy: series.stack_strategy,
                    visible: true,
                    area_baseline: series.area_baseline.unwrap_or_default(),
                },
            );
        }

        let mut stack_groups: BTreeMap<
            StackId,
            (AxisId, AxisId, DatasetId, FieldId, StackStrategy),
        > = BTreeMap::new();
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
    pub name: Option<String>,
    pub kind: AxisKind,
    pub grid: GridId,
    pub position: AxisPosition,
    pub scale: AxisScale,
    pub range: AxisRange,
}

#[derive(Debug, Clone, Copy)]
pub struct DataZoomXModel {
    pub id: DataZoomId,
    pub axis: AxisId,
    pub filter_mode: FilterMode,
}

#[derive(Debug, Clone, Copy)]
pub struct AxisPointerModel {
    pub enabled: bool,
    pub trigger: AxisPointerTrigger,
    pub snap: bool,
    pub trigger_distance_px: f32,
    pub throttle_px: f32,
}

#[derive(Debug, Clone)]
pub struct SeriesModel {
    pub id: SeriesId,
    pub name: Option<String>,
    pub kind: SeriesKind,
    pub dataset: DatasetId,
    pub encode: SeriesEncode,
    pub x_axis: AxisId,
    pub y_axis: AxisId,
    pub stack: Option<StackId>,
    pub stack_strategy: StackStrategy,
    pub visible: bool,
    pub area_baseline: AreaBaseline,
}

fn sanitize_px(v: f32, default: f32) -> f32 {
    if v.is_finite() && v >= 0.0 {
        v
    } else {
        default
    }
}

fn sanitize_name(name: Option<String>) -> Option<String> {
    let name = name?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        None
    } else if trimmed.len() == name.len() {
        Some(name)
    } else {
        Some(trimmed.to_string())
    }
}
