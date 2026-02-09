use std::collections::BTreeSet;

use fret_core::{Point, Px, Rect, Size};

use super::*;
use crate::spec::{
    AxisKind, AxisSpec, ChartSpec, DataZoomYSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode,
    SeriesKind, SeriesSpec, VisualMapSpec,
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

            from: None,
            transforms: Vec::new(),
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
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: None,
        visual_maps: vec![],
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
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    }
}

#[test]
fn visual_map_accepts_series_binding() {
    let mut spec = basic_spec();
    let series_id = spec.series[0].id;
    let y_field = spec.series[0].encode.y;

    spec.visual_maps.push(VisualMapSpec {
        id: crate::ids::VisualMapId::new(1),
        mode: crate::spec::VisualMapMode::Continuous,
        dataset: None,
        series: vec![series_id],
        field: y_field,
        domain: (0.0, 1.0),
        initial_range: Some((0.2, 0.8)),
        initial_piece_mask: None,
        point_radius_mul_range: None,
        stroke_width_range: None,
        opacity_mul_range: None,
        buckets: 8,
        out_of_range_opacity: 0.25,
    });

    let model = ChartModel::from_spec(spec).expect("model should accept a visual_map binding");
    assert_eq!(
        model.visual_map_by_series.get(&series_id).copied(),
        Some(crate::ids::VisualMapId::new(1))
    );
}

#[test]
fn visual_map_rejects_multiple_maps_targeting_the_same_series() {
    let mut spec = basic_spec();
    let series_id = spec.series[0].id;
    let y_field = spec.series[0].encode.y;

    spec.visual_maps.push(VisualMapSpec {
        id: crate::ids::VisualMapId::new(1),
        mode: crate::spec::VisualMapMode::Continuous,
        dataset: None,
        series: vec![series_id],
        field: y_field,
        domain: (0.0, 1.0),
        initial_range: None,
        initial_piece_mask: None,
        point_radius_mul_range: None,
        stroke_width_range: None,
        opacity_mul_range: None,
        buckets: 8,
        out_of_range_opacity: 0.25,
    });
    spec.visual_maps.push(VisualMapSpec {
        id: crate::ids::VisualMapId::new(2),
        mode: crate::spec::VisualMapMode::Continuous,
        dataset: None,
        series: vec![series_id],
        field: y_field,
        domain: (0.0, 1.0),
        initial_range: None,
        initial_piece_mask: None,
        point_radius_mul_range: None,
        stroke_width_range: None,
        opacity_mul_range: None,
        buckets: 8,
        out_of_range_opacity: 0.25,
    });

    let err = ChartModel::from_spec(spec).unwrap_err();
    assert!(matches!(err, ModelError::InvalidSpec { .. }));
}

#[test]
fn visual_map_accepts_dataset_binding() {
    let mut spec = basic_spec();
    let dataset_id = spec.series[0].dataset;
    let y_field = spec.series[0].encode.y;

    spec.series.push(SeriesSpec {
        id: crate::ids::SeriesId::new(2),
        name: None,
        kind: SeriesKind::Scatter,
        dataset: dataset_id,
        encode: spec.series[0].encode,
        x_axis: spec.series[0].x_axis,
        y_axis: spec.series[0].y_axis,
        stack: None,
        stack_strategy: Default::default(),
        bar_layout: Default::default(),
        area_baseline: None,
        lod: None,
    });

    spec.visual_maps.push(VisualMapSpec {
        id: crate::ids::VisualMapId::new(1),
        mode: crate::spec::VisualMapMode::Continuous,
        dataset: Some(dataset_id),
        series: vec![],
        field: y_field,
        domain: (0.0, 1.0),
        initial_range: None,
        initial_piece_mask: None,
        point_radius_mul_range: None,
        stroke_width_range: None,
        opacity_mul_range: None,
        buckets: 8,
        out_of_range_opacity: 0.25,
    });

    let model = ChartModel::from_spec(spec).expect("model should accept dataset binding");
    assert_eq!(
        model
            .visual_map_by_series
            .get(&crate::ids::SeriesId::new(1))
            .copied(),
        Some(crate::ids::VisualMapId::new(1))
    );
    assert_eq!(
        model
            .visual_map_by_series
            .get(&crate::ids::SeriesId::new(2))
            .copied(),
        Some(crate::ids::VisualMapId::new(1))
    );
}

#[test]
fn visual_map_rejects_dataset_and_series_both_set() {
    let mut spec = basic_spec();
    let dataset_id = spec.series[0].dataset;
    let series_id = spec.series[0].id;
    let y_field = spec.series[0].encode.y;

    spec.visual_maps.push(VisualMapSpec {
        id: crate::ids::VisualMapId::new(1),
        mode: crate::spec::VisualMapMode::Continuous,
        dataset: Some(dataset_id),
        series: vec![series_id],
        field: y_field,
        domain: (0.0, 1.0),
        initial_range: None,
        initial_piece_mask: None,
        point_radius_mul_range: None,
        stroke_width_range: None,
        opacity_mul_range: None,
        buckets: 8,
        out_of_range_opacity: 0.25,
    });

    let err = ChartModel::from_spec(spec).unwrap_err();
    assert!(matches!(err, ModelError::InvalidSpec { .. }));
}

#[test]
fn data_zoom_y_rejects_non_y_axis_binding() {
    let mut spec = basic_spec();
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: crate::ids::AxisId::new(1),
        filter_mode: FilterMode::None,
        min_value_span: None,
        max_value_span: None,
    });

    let err = ChartModel::from_spec(spec).unwrap_err();
    assert!(matches!(err, ModelError::InvalidSpec { .. }));
}

#[test]
fn data_zoom_y_rejects_multiple_specs_for_same_axis() {
    let mut spec = basic_spec();
    let y_axis = crate::ids::AxisId::new(2);
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: y_axis,
        filter_mode: FilterMode::None,
        min_value_span: None,
        max_value_span: None,
    });
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(2),
        axis: y_axis,
        filter_mode: FilterMode::None,
        min_value_span: None,
        max_value_span: None,
    });

    let err = ChartModel::from_spec(spec).unwrap_err();
    assert!(matches!(err, ModelError::InvalidSpec { .. }));
}

#[test]
fn data_zoom_y_is_accepted_for_y_axes() {
    let mut spec = basic_spec();
    let y_axis = crate::ids::AxisId::new(2);
    spec.data_zoom_y.push(DataZoomYSpec {
        id: crate::ids::DataZoomId::new(1),
        axis: y_axis,
        filter_mode: FilterMode::None,
        min_value_span: Some(10.0),
        max_value_span: Some(100.0),
    });

    let model = ChartModel::from_spec(spec).unwrap();
    assert_eq!(
        model.data_zoom_y_by_axis.get(&y_axis).copied(),
        Some(crate::ids::DataZoomId::new(1))
    );
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
fn bar_requires_exactly_one_category_axis() {
    let mut spec = basic_spec();
    spec.series[0].kind = SeriesKind::Bar;
    let err = ChartModel::from_spec(spec).unwrap_err();
    assert!(matches!(err, ModelError::InvalidSpec { .. }));
}

#[test]
fn bar_allows_category_y_axis_for_horizontal_bars() {
    let mut spec = basic_spec();
    spec.series[0].kind = SeriesKind::Bar;
    spec.axes[1].scale = crate::scale::AxisScale::Category(crate::scale::CategoryAxisScale {
        categories: vec!["A".into(), "B".into()],
    });
    ChartModel::from_spec(spec).unwrap();
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
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    visible: Some(true),
                    area_baseline: None,
                    lod: None,
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
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    visible: Some(false),
                    area_baseline: None,
                    lod: None,
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
                    stack: None,
                    stack_strategy: Default::default(),
                    bar_layout: Default::default(),
                    visible: Some(false),
                    area_baseline: None,
                    lod: None,
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
