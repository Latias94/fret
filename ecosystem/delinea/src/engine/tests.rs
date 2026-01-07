use std::collections::BTreeMap;

use crate::action::Action;
use crate::engine::ChartEngine;
use crate::engine::window::DataWindow;
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
            area_baseline: None,
        }],
    }
}

#[test]
fn set_view_window_2d_updates_both_axes() {
    let mut engine = ChartEngine::new(basic_spec()).unwrap();
    engine.apply_action(Action::SetViewWindow2D {
        x_axis: crate::ids::AxisId::new(1),
        y_axis: crate::ids::AxisId::new(2),
        x: Some(DataWindow {
            min: 10.0,
            max: 20.0,
        }),
        y: Some(DataWindow {
            min: -5.0,
            max: 5.0,
        }),
    });

    let mut expected_x = BTreeMap::new();
    expected_x.insert(
        crate::ids::AxisId::new(1),
        DataWindow {
            min: 10.0,
            max: 20.0,
        },
    );
    assert_eq!(engine.state().data_window_x, expected_x);

    let mut expected_y = BTreeMap::new();
    expected_y.insert(
        crate::ids::AxisId::new(2),
        DataWindow {
            min: -5.0,
            max: 5.0,
        },
    );
    assert_eq!(engine.state().data_window_y, expected_y);
}
