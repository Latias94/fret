use std::collections::BTreeSet;

use fret_core::{Point, Px, Rect, Size};

use super::*;
use crate::spec::{
    AxisKind, AxisSpec, ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind,
    SeriesSpec,
};

fn basic_spec() -> ChartSpec {
    let dataset_id = crate::ids::DatasetId::new(1);
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);
    ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: None,
        datasets: vec![DatasetSpec {
            id: dataset_id,
            fields: vec![
                FieldSpec {
                    id: x_field,
                    column: 0,
                },
                FieldSpec {
                    id: y_field,
                    column: 1,
                },
            ],
        }],
        grids: vec![GridSpec {
            id: crate::ids::GridId::new(1),
        }],
        axes: vec![
            AxisSpec {
                id: crate::ids::AxisId::new(1),
                name: None,
                kind: AxisKind::X,
                grid: crate::ids::GridId::new(1),
                position: None,
                scale: Default::default(),
                range: None,
            },
            AxisSpec {
                id: crate::ids::AxisId::new(2),
                name: None,
                kind: AxisKind::Y,
                grid: crate::ids::GridId::new(1),
                position: None,
                scale: Default::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        axis_pointer: None,
        series: vec![SeriesSpec {
            id: crate::ids::SeriesId::new(1),
            name: None,
            kind: SeriesKind::Line,
            dataset: dataset_id,
            encode: SeriesEncode {
                x: x_field,
                y: y_field,
                y2: None,
            },
            x_axis: crate::ids::AxisId::new(1),
            y_axis: crate::ids::AxisId::new(2),
            area_baseline: None,
        }],
    }
}

#[test]
fn band_requires_y2_col() {
    let mut spec = basic_spec();
    spec.series[0].kind = SeriesKind::Band;
    spec.series[0].encode.y2 = None;
    let err = ChartModel::from_spec(spec).unwrap_err();
    assert!(matches!(err, ModelError::InvalidSpec { .. }));
}

#[test]
fn bar_requires_category_x_axis() {
    let mut spec = basic_spec();
    spec.series[0].kind = SeriesKind::Bar;
    let err = ChartModel::from_spec(spec).unwrap_err();
    assert!(matches!(err, ModelError::InvalidSpec { .. }));
}

#[test]
fn from_spec_validates_references() {
    let mut spec = basic_spec();
    spec.series[0].dataset = crate::ids::DatasetId::new(999);

    let err = ChartModel::from_spec(spec).unwrap_err();
    assert!(matches!(
        err,
        ModelError::MissingReference { kind: "dataset" }
    ));
}

#[test]
fn merge_patch_updates_viewport_and_revs() {
    let spec = basic_spec();
    let mut model = ChartModel::from_spec(spec).unwrap();
    let before = model.revs;

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let report = model
        .apply_patch(
            ChartPatch {
                viewport: Some(Some(viewport)),
                ..ChartPatch::default()
            },
            PatchMode::Merge,
        )
        .unwrap();

    assert!(report.viewport_changed);
    assert!(report.marks_changed);
    assert!(!report.structure_changed);
    assert_eq!(model.viewport, Some(viewport));
    assert_eq!(model.revs.spec, before.spec);
    assert!(model.revs.layout.0 > before.layout.0);
    assert!(model.revs.marks.0 > before.marks.0);
}

#[test]
fn replace_merge_can_replace_series_only() {
    let spec = basic_spec();
    let mut model = ChartModel::from_spec(spec).unwrap();

    let replace_families: BTreeSet<ReplaceFamily> = [ReplaceFamily::Series].into_iter().collect();
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);
    let report = model
        .apply_patch(
            ChartPatch {
                replace_families,
                series: vec![SeriesOp::Upsert(SeriesPatch {
                    id: crate::ids::SeriesId::new(2),
                    name: None,
                    kind: SeriesKind::Line,
                    dataset: crate::ids::DatasetId::new(1),
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis: crate::ids::AxisId::new(1),
                    y_axis: crate::ids::AxisId::new(2),
                    visible: Some(true),
                    area_baseline: None,
                })],
                ..ChartPatch::default()
            },
            PatchMode::ReplaceMerge,
        )
        .unwrap();

    assert!(report.structure_changed);
    assert!(report.marks_changed);
    assert_eq!(model.series_order, vec![crate::ids::SeriesId::new(2)]);
    assert!(model.series.contains_key(&crate::ids::SeriesId::new(2)));
    assert!(!model.series.contains_key(&crate::ids::SeriesId::new(1)));
}

#[test]
fn replace_merge_keeps_and_merges_matching_ids() {
    let spec = basic_spec();
    let mut model = ChartModel::from_spec(spec).unwrap();

    let replace_families: BTreeSet<ReplaceFamily> = [ReplaceFamily::Series].into_iter().collect();
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);
    let report = model
        .apply_patch(
            ChartPatch {
                replace_families,
                series: vec![SeriesOp::Upsert(SeriesPatch {
                    id: crate::ids::SeriesId::new(1),
                    name: None,
                    kind: SeriesKind::Line,
                    dataset: crate::ids::DatasetId::new(1),
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis: crate::ids::AxisId::new(1),
                    y_axis: crate::ids::AxisId::new(2),
                    visible: Some(false),
                    area_baseline: None,
                })],
                ..ChartPatch::default()
            },
            PatchMode::ReplaceMerge,
        )
        .unwrap();

    assert!(!report.structure_changed);
    assert!(report.marks_changed);
    assert_eq!(model.series_order, vec![crate::ids::SeriesId::new(1)]);
    assert_eq!(
        model
            .series
            .get(&crate::ids::SeriesId::new(1))
            .unwrap()
            .visible,
        false
    );
}

#[test]
fn replace_merge_rejects_dangling_references() {
    let spec = basic_spec();
    let mut model = ChartModel::from_spec(spec).unwrap();

    let replace_families: BTreeSet<ReplaceFamily> = [ReplaceFamily::Axes].into_iter().collect();
    let err = model
        .apply_patch(
            ChartPatch {
                replace_families,
                axes: vec![AxisOp::Upsert(AxisPatch {
                    id: crate::ids::AxisId::new(1),
                    name: None,
                    kind: AxisKind::X,
                    grid: crate::ids::GridId::new(1),
                    position: None,
                    scale: None,
                    range: None,
                })],
                ..ChartPatch::default()
            },
            PatchMode::ReplaceMerge,
        )
        .unwrap_err();

    assert!(matches!(
        err,
        ModelError::MissingReference {
            kind: "axis.y_axis"
        }
    ));
}

#[test]
fn replace_validates_references() {
    let spec = basic_spec();
    let mut model = ChartModel::from_spec(spec).unwrap();

    let err = model
        .apply_patch(
            ChartPatch {
                grids: vec![GridOp::Upsert {
                    id: crate::ids::GridId::new(1),
                }],
                axes: vec![AxisOp::Upsert(AxisPatch {
                    id: crate::ids::AxisId::new(1),
                    name: None,
                    kind: AxisKind::X,
                    grid: crate::ids::GridId::new(999),
                    position: None,
                    scale: None,
                    range: None,
                })],
                ..ChartPatch::default()
            },
            PatchMode::Replace,
        )
        .unwrap_err();

    assert!(matches!(err, ModelError::MissingReference { kind: "grid" }));
}

#[test]
fn merge_axis_range_updates_layout_without_structure() {
    let spec = basic_spec();
    let mut model = ChartModel::from_spec(spec).unwrap();
    let before = model.revs;

    let report = model
        .apply_patch(
            ChartPatch {
                axes: vec![AxisOp::Upsert(AxisPatch {
                    id: crate::ids::AxisId::new(1),
                    name: None,
                    kind: AxisKind::X,
                    grid: crate::ids::GridId::new(1),
                    position: None,
                    scale: None,
                    range: Some(crate::spec::AxisRange::LockMin { min: 10.0 }),
                })],
                ..ChartPatch::default()
            },
            PatchMode::Merge,
        )
        .unwrap();

    assert!(!report.viewport_changed);
    assert!(!report.structure_changed);
    assert!(report.marks_changed);

    let axis = model.axes.get(&crate::ids::AxisId::new(1)).unwrap();
    assert_eq!(axis.range, crate::spec::AxisRange::LockMin { min: 10.0 });
    assert_eq!(model.revs.spec, before.spec);
    assert!(model.revs.layout.0 > before.layout.0);
    assert!(model.revs.marks.0 > before.marks.0);
}

#[test]
fn merge_series_visibility_updates_visual_without_structure() {
    let spec = basic_spec();
    let mut model = ChartModel::from_spec(spec).unwrap();
    let x_field = crate::ids::FieldId::new(1);
    let y_field = crate::ids::FieldId::new(2);

    let report = model
        .apply_patch(
            ChartPatch {
                series: vec![SeriesOp::Upsert(SeriesPatch {
                    id: crate::ids::SeriesId::new(1),
                    name: None,
                    kind: SeriesKind::Line,
                    dataset: crate::ids::DatasetId::new(1),
                    encode: SeriesEncode {
                        x: x_field,
                        y: y_field,
                        y2: None,
                    },
                    x_axis: crate::ids::AxisId::new(1),
                    y_axis: crate::ids::AxisId::new(2),
                    visible: Some(false),
                    area_baseline: None,
                })],
                ..ChartPatch::default()
            },
            PatchMode::Merge,
        )
        .unwrap();

    assert!(!report.structure_changed);
    assert!(report.marks_changed);
    assert_eq!(
        model
            .series
            .get(&crate::ids::SeriesId::new(1))
            .unwrap()
            .visible,
        false
    );
}
