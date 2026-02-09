mod patch;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use fret_core::{Px, Rect};
use thiserror::Error;

use crate::ids::{
    AxisId, ChartId, DataZoomId, DatasetId, FieldId, GridId, Revision, SeriesId, StackId,
    VisualMapId,
};
use crate::scale::AxisScale;
use crate::spec::{
    AreaBaseline, AxisKind, AxisPointerTrigger, AxisPosition, AxisRange, FilterMode, SeriesEncode,
    SeriesKind, StackStrategy, VisualMapSpec,
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
    pub plot_viewports_by_grid: BTreeMap<GridId, Rect>,

    pub datasets: BTreeMap<DatasetId, DatasetModel>,
    pub grids: BTreeMap<GridId, GridModel>,
    pub axes: BTreeMap<AxisId, AxisModel>,
    pub data_zoom_x: BTreeMap<DataZoomId, DataZoomXModel>,
    pub data_zoom_x_by_axis: BTreeMap<AxisId, DataZoomId>,
    pub data_zoom_y: BTreeMap<DataZoomId, DataZoomYModel>,
    pub data_zoom_y_by_axis: BTreeMap<AxisId, DataZoomId>,
    pub tooltip: Option<crate::spec::TooltipSpecV1>,
    pub axis_pointer: Option<AxisPointerModel>,
    pub visual_maps: BTreeMap<VisualMapId, VisualMapModel>,
    pub visual_map_by_series: BTreeMap<SeriesId, VisualMapId>,

    pub series_order: Vec<SeriesId>,
    pub series: BTreeMap<SeriesId, SeriesModel>,

    pub revs: ModelRevisions,
}

impl ChartModel {
    pub fn from_spec(spec: crate::spec::ChartSpec) -> Result<Self, ModelError> {
        let mut model = Self {
            id: spec.id,
            viewport: spec.viewport,
            plot_viewports_by_grid: BTreeMap::default(),
            datasets: BTreeMap::default(),
            grids: BTreeMap::default(),
            axes: BTreeMap::default(),
            data_zoom_x: BTreeMap::default(),
            data_zoom_x_by_axis: BTreeMap::default(),
            data_zoom_y: BTreeMap::default(),
            data_zoom_y_by_axis: BTreeMap::default(),
            tooltip: spec.tooltip,
            axis_pointer: None,
            visual_maps: BTreeMap::default(),
            visual_map_by_series: BTreeMap::default(),
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
            let min_value_span = sanitize_value_span(zoom.min_value_span);
            let max_value_span = sanitize_value_span(zoom.max_value_span);
            let (min_value_span, max_value_span) = match (min_value_span, max_value_span) {
                (Some(min), Some(max)) if min > max => (Some(max), Some(max)),
                other => other,
            };
            model.data_zoom_x.insert(
                zoom.id,
                DataZoomXModel {
                    id: zoom.id,
                    axis: zoom.axis,
                    filter_mode: zoom.filter_mode,
                    min_value_span,
                    max_value_span,
                },
            );
        }

        let mut zoom_ids: BTreeSet<DataZoomId> = BTreeSet::new();
        for zoom in spec.data_zoom_y {
            if !zoom_ids.insert(zoom.id) {
                return Err(ModelError::DuplicateId {
                    kind: "data_zoom_y",
                });
            }
            let Some(axis) = model.axes.get(&zoom.axis) else {
                return Err(ModelError::MissingReference {
                    kind: "data_zoom_y.axis",
                });
            };
            if axis.kind != AxisKind::Y {
                return Err(ModelError::InvalidSpec {
                    reason: "data_zoom_y.axis must reference a Y axis",
                });
            }
            if model
                .data_zoom_y_by_axis
                .insert(zoom.axis, zoom.id)
                .is_some()
            {
                return Err(ModelError::InvalidSpec {
                    reason: "multiple data_zoom_y specs reference the same axis",
                });
            }

            let min_value_span = sanitize_value_span(zoom.min_value_span);
            let max_value_span = sanitize_value_span(zoom.max_value_span);
            let (min_value_span, max_value_span) = match (min_value_span, max_value_span) {
                (Some(min), Some(max)) if min > max => (Some(max), Some(max)),
                other => other,
            };

            model.data_zoom_y.insert(
                zoom.id,
                DataZoomYModel {
                    id: zoom.id,
                    axis: zoom.axis,
                    filter_mode: zoom.filter_mode,
                    min_value_span,
                    max_value_span,
                },
            );
        }

        model.axis_pointer = spec.axis_pointer.map(|mut p| {
            p.trigger_distance_px = sanitize_px(p.trigger_distance_px, 12.0);
            p.throttle_px = sanitize_px(p.throttle_px, 0.75);
            if p.label.template.is_empty() {
                p.label.template = "{value}".to_string();
            }
            AxisPointerModel {
                enabled: p.enabled,
                trigger: p.trigger,
                pointer_type: p.pointer_type,
                label: AxisPointerLabelModel {
                    show: p.label.show,
                    template: p.label.template,
                },
                snap: p.snap,
                trigger_distance_px: p.trigger_distance_px,
                throttle_px: p.throttle_px,
            }
        });

        let visual_maps = spec.visual_maps;

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
                let Some(_) = model.axes.get(&series.x_axis) else {
                    return Err(ModelError::MissingReference {
                        kind: "axis.x_axis",
                    });
                };
                let Some(_) = model.axes.get(&series.y_axis) else {
                    return Err(ModelError::MissingReference {
                        kind: "axis.y_axis",
                    });
                };

                // For bars we support both orientations (ECharts-like):
                // - Vertical: X is category, Y is value -> encode.x is category, encode.y is value.
                // - Horizontal: Y is category, X is value -> encode.y is category, encode.x is value.
                //
                // Exactly one axis must be Category in v1.
                let x_is_category = model
                    .axes
                    .get(&series.x_axis)
                    .is_some_and(|a| matches!(a.scale, AxisScale::Category(_)));
                let y_is_category = model
                    .axes
                    .get(&series.y_axis)
                    .is_some_and(|a| matches!(a.scale, AxisScale::Category(_)));
                if x_is_category == y_is_category {
                    return Err(ModelError::InvalidSpec {
                        reason: "series.kind=Bar requires exactly one Category axis (either x_axis or y_axis)",
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

            if series.kind != SeriesKind::Bar && series.bar_layout != Default::default() {
                return Err(ModelError::InvalidSpec {
                    reason: "bar_layout is only supported for Bar series in v1",
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
                    bar_layout: series.bar_layout,
                    lod: series.lod.unwrap_or_default(),
                },
            );
        }

        apply_visual_maps(&mut model, visual_maps)?;

        let mut stack_groups: BTreeMap<
            StackId,
            (AxisId, AxisId, DatasetId, FieldId, StackStrategy),
        > = BTreeMap::new();
        for s in model.series.values() {
            let Some(stack) = s.stack else {
                continue;
            };
            let key = if s.kind == SeriesKind::Bar {
                let mapping = crate::engine::bar::bar_mapping_for_series(&model, s.id).ok_or(
                    ModelError::InvalidSpec {
                        reason:
                            "stacked bar series requires a valid bar orientation (exactly one Category axis)",
                    },
                )?;
                (
                    mapping.category_axis,
                    mapping.value_axis,
                    s.dataset,
                    mapping.category_field,
                    s.stack_strategy,
                )
            } else {
                (s.x_axis, s.y_axis, s.dataset, s.encode.x, s.stack_strategy)
            };
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualMapDomain {
    pub min: f64,
    pub max: f64,
}

impl VisualMapDomain {
    pub fn sanitize(self) -> Option<Self> {
        if !self.min.is_finite() || !self.max.is_finite() || self.max <= self.min {
            return None;
        }
        Some(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualMapRange {
    pub min: f64,
    pub max: f64,
}

impl VisualMapRange {
    pub fn sanitize(self) -> Option<Self> {
        if !self.min.is_finite() || !self.max.is_finite() {
            return None;
        }
        let (min, max) = if self.max < self.min {
            (self.max, self.min)
        } else {
            (self.min, self.max)
        };
        Some(Self { min, max })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualMapModel {
    pub id: VisualMapId,
    pub mode: crate::spec::VisualMapMode,
    pub field: FieldId,
    pub domain: VisualMapDomain,
    pub initial_range: Option<VisualMapRange>,
    pub initial_piece_mask: Option<u64>,
    pub point_radius_mul_range: Option<(f32, f32)>,
    pub stroke_width_range: Option<(Px, Px)>,
    pub opacity_mul_range: Option<(f32, f32)>,
    pub buckets: u16,
    pub out_of_range_opacity: f32,
}

fn apply_visual_maps(
    model: &mut ChartModel,
    visual_maps: Vec<VisualMapSpec>,
) -> Result<(), ModelError> {
    let mut ids: BTreeSet<VisualMapId> = BTreeSet::new();
    for map in visual_maps {
        if !ids.insert(map.id) {
            return Err(ModelError::DuplicateId { kind: "visual_map" });
        }
        if map.dataset.is_some() && !map.series.is_empty() {
            return Err(ModelError::InvalidSpec {
                reason: "visual_map.dataset is mutually exclusive with visual_map.series",
            });
        }

        let domain = VisualMapDomain {
            min: map.domain.0,
            max: map.domain.1,
        }
        .sanitize()
        .ok_or(ModelError::InvalidSpec {
            reason: "visual_map.domain must be finite and non-degenerate",
        })?;

        let initial_range = map
            .initial_range
            .map(|(min, max)| VisualMapRange { min, max })
            .and_then(VisualMapRange::sanitize);

        let buckets = map.buckets.clamp(1, 64);

        let out_of_range_opacity = if map.out_of_range_opacity.is_finite() {
            map.out_of_range_opacity.clamp(0.0, 1.0)
        } else {
            0.25
        };

        let initial_piece_mask = match map.mode {
            crate::spec::VisualMapMode::Continuous => None,
            crate::spec::VisualMapMode::Piecewise => {
                let full_mask = if buckets >= 64 {
                    u64::MAX
                } else {
                    (1u64 << buckets) - 1
                };
                map.initial_piece_mask.map(|m| m & full_mask)
            }
        };

        let point_radius_mul_range = map
            .point_radius_mul_range
            .filter(|(a, b)| a.is_finite() && b.is_finite())
            .map(|(a, b)| if b < a { (b, a) } else { (a, b) })
            .filter(|(a, b)| *a > 0.0 && *b > 0.0 && (*b - *a).abs() > f32::EPSILON)
            .map(|(a, b)| (a.clamp(0.01, 100.0), b.clamp(0.01, 100.0)));

        let stroke_width_range = map
            .stroke_width_range
            .filter(|(a, b)| a.is_finite() && b.is_finite())
            .map(|(a, b)| if b < a { (b, a) } else { (a, b) })
            .map(|(a, b)| (a.clamp(0.0, 20.0), b.clamp(0.0, 20.0)))
            .filter(|(a, b)| (*b - *a).abs() > f32::EPSILON)
            .map(|(a, b)| (Px(a), Px(b)));

        let opacity_mul_range = map
            .opacity_mul_range
            .filter(|(a, b)| a.is_finite() && b.is_finite())
            .map(|(a, b)| if b < a { (b, a) } else { (a, b) })
            .map(|(a, b)| (a.clamp(0.0, 1.0), b.clamp(0.0, 1.0)))
            .filter(|(a, b)| (*b - *a).abs() > f32::EPSILON);

        let target_series: Vec<SeriesId> = if let Some(dataset_id) = map.dataset {
            let Some(dataset) = model.datasets.get(&dataset_id) else {
                return Err(ModelError::MissingReference {
                    kind: "visual_map.dataset",
                });
            };
            if !dataset.fields.contains_key(&map.field) {
                return Err(ModelError::MissingReference {
                    kind: "visual_map.field",
                });
            }
            model
                .series_in_order()
                .filter(|s| s.dataset == dataset_id)
                .map(|s| s.id)
                .collect()
        } else {
            if map.series.is_empty() {
                return Err(ModelError::InvalidSpec {
                    reason: "visual_map must target at least one series",
                });
            }
            map.series.clone()
        };

        if target_series.is_empty() {
            return Err(ModelError::InvalidSpec {
                reason: "visual_map has no target series",
            });
        }

        for series_id in &target_series {
            if !model.series.contains_key(series_id) {
                return Err(ModelError::MissingReference {
                    kind: "visual_map.series",
                });
            }
            let Some(series) = model.series.get(series_id) else {
                continue;
            };
            let Some(dataset) = model.datasets.get(&series.dataset) else {
                return Err(ModelError::MissingReference { kind: "dataset" });
            };
            if !dataset.fields.contains_key(&map.field) {
                return Err(ModelError::MissingReference {
                    kind: "visual_map.field",
                });
            }

            if model
                .visual_map_by_series
                .insert(*series_id, map.id)
                .is_some()
            {
                return Err(ModelError::InvalidSpec {
                    reason: "multiple visual_maps target the same series (v1 restriction)",
                });
            }
        }

        model.visual_maps.insert(
            map.id,
            VisualMapModel {
                id: map.id,
                mode: map.mode,
                field: map.field,
                domain,
                initial_range,
                initial_piece_mask,
                point_radius_mul_range,
                stroke_width_range,
                opacity_mul_range,
                buckets,
                out_of_range_opacity,
            },
        );
    }
    Ok(())
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
    pub min_value_span: Option<f64>,
    pub max_value_span: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub struct DataZoomYModel {
    pub id: DataZoomId,
    pub axis: AxisId,
    pub filter_mode: FilterMode,
    pub min_value_span: Option<f64>,
    pub max_value_span: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AxisPointerModel {
    pub enabled: bool,
    pub trigger: AxisPointerTrigger,
    pub pointer_type: crate::spec::AxisPointerType,
    pub label: AxisPointerLabelModel,
    pub snap: bool,
    pub trigger_distance_px: f32,
    pub throttle_px: f32,
}

#[derive(Debug, Default, Clone)]
pub struct AxisPointerLabelModel {
    pub show: bool,
    pub template: String,
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
    pub bar_layout: crate::spec::BarLayoutSpec,
    pub lod: crate::spec::SeriesLodSpecV1,
}

fn sanitize_px(v: f32, default: f32) -> f32 {
    if v.is_finite() && v >= 0.0 {
        v
    } else {
        default
    }
}

fn sanitize_value_span(span: Option<f64>) -> Option<f64> {
    span.filter(|v| v.is_finite() && *v > 0.0)
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
