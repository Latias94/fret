use std::collections::BTreeSet;

use fret_core::{Point, Px, Rect, Size};

use super::*;
use crate::spec::{AxisKind, AxisSpec, ChartSpec, DatasetSpec, GridSpec, SeriesKind, SeriesSpec};

fn basic_spec() -> ChartSpec {
    ChartSpec {
        id: crate::ids::ChartId::new(1),
        viewport: None,
        datasets: vec![DatasetSpec {
            id: crate::ids::DatasetId::new(1),
        }],
        grids: vec![GridSpec {
            id: crate::ids::GridId::new(1),
        }],
        axes: vec![
            AxisSpec {
                id: crate::ids::AxisId::new(1),
                kind: AxisKind::X,
                grid: crate::ids::GridId::new(1),
                range: None,
            },
            AxisSpec {
                id: crate::ids::AxisId::new(2),
                kind: AxisKind::Y,
                grid: crate::ids::GridId::new(1),
                range: None,
            },
        ],
        series: vec![SeriesSpec {
            id: crate::ids::SeriesId::new(1),
            kind: SeriesKind::Line,
            dataset: crate::ids::DatasetId::new(1),
            x_col: 0,
            y_col: 1,
            x_axis: crate::ids::AxisId::new(1),
            y_axis: crate::ids::AxisId::new(2),
        }],
    }
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
    let report = model
        .apply_patch(
            ChartPatch {
                replace_families,
                series: vec![SeriesOp::Upsert(SeriesPatch {
                    id: crate::ids::SeriesId::new(2),
                    kind: SeriesKind::Line,
                    dataset: crate::ids::DatasetId::new(1),
                    x_col: 0,
                    y_col: 1,
                    x_axis: crate::ids::AxisId::new(1),
                    y_axis: crate::ids::AxisId::new(2),
                    visible: Some(true),
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
    let report = model
        .apply_patch(
            ChartPatch {
                replace_families,
                series: vec![SeriesOp::Upsert(SeriesPatch {
                    id: crate::ids::SeriesId::new(1),
                    kind: SeriesKind::Line,
                    dataset: crate::ids::DatasetId::new(1),
                    x_col: 0,
                    y_col: 1,
                    x_axis: crate::ids::AxisId::new(1),
                    y_axis: crate::ids::AxisId::new(2),
                    visible: Some(false),
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
                    kind: AxisKind::X,
                    grid: crate::ids::GridId::new(1),
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
                    kind: AxisKind::X,
                    grid: crate::ids::GridId::new(999),
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
                    kind: AxisKind::X,
                    grid: crate::ids::GridId::new(1),
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

    let report = model
        .apply_patch(
            ChartPatch {
                series: vec![SeriesOp::Upsert(SeriesPatch {
                    id: crate::ids::SeriesId::new(1),
                    kind: SeriesKind::Line,
                    dataset: crate::ids::DatasetId::new(1),
                    x_col: 0,
                    y_col: 1,
                    x_axis: crate::ids::AxisId::new(1),
                    y_axis: crate::ids::AxisId::new(2),
                    visible: Some(false),
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
