#![cfg(feature = "echarts")]

use delinea::engine::ChartEngine;
use delinea::ids::StringId;
use delinea::scheduler::WorkBudget;
use delinea::text::{TextMeasurer, TextMetrics, TextStyleId};
use fret_chart::echarts::translate_json_str;
use fret_core::{Point, Px, Rect, Size};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
struct NullTextMeasurer;

impl TextMeasurer for NullTextMeasurer {
    fn measure(&mut self, _text: StringId, _style: TextStyleId) -> TextMetrics {
        TextMetrics::default()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct HeadlessSnapshot {
    axis_windows: Vec<AxisWindowSnapshot>,
    filter_plan: FilterPlanSnapshot,
    marks: MarksSnapshot,
    participation: ParticipationSnapshot,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct AxisWindowSnapshot {
    axis: u64,
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct FilterPlanSnapshot {
    revision: u64,
    grids: Vec<GridFilterSnapshot>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct GridFilterSnapshot {
    grid: u64,
    series: Vec<u64>,
    y_percent_extents: Vec<AxisExtentSnapshot>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct AxisExtentSnapshot {
    axis: u64,
    min: f64,
    max: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct MarksSnapshot {
    revision: u64,
    total_nodes: usize,
    arena_points: usize,
    arena_rects: usize,
    data_indices: IndicesSnapshot,
    rect_data_indices: IndicesSnapshot,
    by_series: Vec<SeriesMarksSnapshot>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct IndicesSnapshot {
    len: usize,
    head: Vec<u32>,
    tail: Vec<u32>,
    hash: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct SeriesMarksSnapshot {
    series: u64,
    nodes: usize,
    points: usize,
    polylines: usize,
    rects: usize,
    texts: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ParticipationSnapshot {
    revision: u64,
    series: Vec<SeriesParticipationSnapshot>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct SeriesParticipationSnapshot {
    series: u64,
    dataset: u64,
    revision: u64,
    data_revision: u64,
    selection: SelectionSnapshot,
    x_filter_mode: String,
    y_filter_mode: String,
    empty_mask_active: bool,
    empty_mask_y_is_interval: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum SelectionSnapshot {
    All {
        view_len: usize,
    },
    Range {
        start: usize,
        end: usize,
        view_len: usize,
    },
    Indices {
        view_len: usize,
        head: Vec<u32>,
        tail: Vec<u32>,
        hash: u64,
    },
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn goldens_dir() -> PathBuf {
    repo_root()
        .join("goldens")
        .join("echarts-headless")
        .join("v1")
}

fn viewport_320x200() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
    )
}

fn snapshot_from_engine(
    engine: &ChartEngine,
    row_count_by_dataset: &BTreeMap<u64, usize>,
) -> HeadlessSnapshot {
    let output = engine.output();

    let mut axis_windows = Vec::with_capacity(output.axis_windows.len());
    for (axis, w) in &output.axis_windows {
        axis_windows.push(AxisWindowSnapshot {
            axis: axis.0 as u64,
            min: w.min,
            max: w.max,
        });
    }
    axis_windows.sort_by_key(|w| w.axis);

    let filter_plan_output = engine.filter_plan_output();
    let mut grids = Vec::with_capacity(filter_plan_output.grids.len());
    for g in &filter_plan_output.grids {
        let mut extents = Vec::with_capacity(g.y_percent_extents.len());
        for (axis, (min, max)) in &g.y_percent_extents {
            extents.push(AxisExtentSnapshot {
                axis: axis.0 as u64,
                min: *min,
                max: *max,
            });
        }
        extents.sort_by_key(|e| e.axis);

        let mut series: Vec<u64> = g.series.iter().map(|s| s.0 as u64).collect();
        series.sort_unstable();

        grids.push(GridFilterSnapshot {
            grid: g.grid.0 as u64,
            series,
            y_percent_extents: extents,
        });
    }
    grids.sort_by_key(|g| g.grid);

    let filter_plan = FilterPlanSnapshot {
        revision: filter_plan_output.revision.0 as u64,
        grids,
    };

    let marks = &output.marks;
    let mut by_series: BTreeMap<u64, SeriesMarksSnapshot> = BTreeMap::new();
    for node in &marks.nodes {
        let Some(series) = node.source_series else {
            continue;
        };
        let entry = by_series
            .entry(series.0 as u64)
            .or_insert(SeriesMarksSnapshot {
                series: series.0 as u64,
                nodes: 0,
                points: 0,
                polylines: 0,
                rects: 0,
                texts: 0,
            });
        entry.nodes += 1;
        match &node.payload {
            delinea::marks::MarkPayloadRef::Group(_) => {}
            delinea::marks::MarkPayloadRef::Polyline(p) => {
                entry.polylines += 1;
                entry.points += p.points.len();
            }
            delinea::marks::MarkPayloadRef::Points(p) => {
                entry.points += p.points.len();
            }
            delinea::marks::MarkPayloadRef::Rect(r) => {
                entry.rects += r.rects.len();
            }
            delinea::marks::MarkPayloadRef::Text(_) => {
                entry.texts += 1;
            }
        }
    }

    let data_indices = snapshot_indices(&marks.arena.data_indices);
    let rect_data_indices = snapshot_indices(&marks.arena.rect_data_indices);
    let marks = MarksSnapshot {
        revision: marks.revision.0 as u64,
        total_nodes: marks.nodes.len(),
        arena_points: marks.arena.points.len(),
        arena_rects: marks.arena.rects.len(),
        data_indices,
        rect_data_indices,
        by_series: by_series.into_values().collect(),
    };

    let participation = engine.participation();
    let mut series = Vec::with_capacity(participation.series.len());
    for s in &participation.series {
        let row_count = row_count_by_dataset
            .get(&(s.dataset.0 as u64))
            .copied()
            .unwrap_or(0);
        let view_len = s.selection.view_len(row_count);

        let selection = match &s.selection {
            delinea::transform::RowSelection::All => SelectionSnapshot::All { view_len },
            delinea::transform::RowSelection::Range(r) => SelectionSnapshot::Range {
                start: r.start,
                end: r.end,
                view_len,
            },
            delinea::transform::RowSelection::Indices(indices) => {
                let head: Vec<u32> = indices.iter().copied().take(6).collect();
                let tail: Vec<u32> = indices.iter().copied().rev().take(6).collect();
                let hash = indices.iter().fold(14695981039346656037u64, |h, v| {
                    let v = *v as u64;
                    (h ^ v).wrapping_mul(1099511628211u64)
                });
                SelectionSnapshot::Indices {
                    view_len,
                    head,
                    tail,
                    hash,
                }
            }
        };

        series.push(SeriesParticipationSnapshot {
            series: s.series.0 as u64,
            dataset: s.dataset.0 as u64,
            revision: s.revision.0 as u64,
            data_revision: s.data_revision.0 as u64,
            selection,
            x_filter_mode: format!("{:?}", s.x_filter_mode),
            y_filter_mode: format!("{:?}", s.y_filter_mode),
            empty_mask_active: s.empty_mask.x_active || s.empty_mask.y_active,
            empty_mask_y_is_interval: s.empty_mask.y_is_interval,
        });
    }
    series.sort_by_key(|s| s.series);

    let participation = ParticipationSnapshot {
        revision: participation.revision.0 as u64,
        series,
    };

    HeadlessSnapshot {
        axis_windows,
        filter_plan,
        marks,
        participation,
    }
}

fn snapshot_indices(indices: &[u32]) -> IndicesSnapshot {
    let take = 8usize;
    let head: Vec<u32> = indices.iter().copied().take(take).collect();
    let mut tail: Vec<u32> = indices.iter().copied().rev().take(take).collect();
    tail.reverse();
    let hash = indices.iter().fold(14695981039346656037u64, |h, v| {
        let v = *v as u64;
        (h ^ v).wrapping_mul(1099511628211u64)
    });
    IndicesSnapshot {
        len: indices.len(),
        head,
        tail,
        hash,
    }
}

fn run_option_snapshot_with_viewport(option_json: &str, viewport: Rect) -> HeadlessSnapshot {
    let translated = translate_json_str(option_json).expect("translate");

    let mut spec = translated.spec.clone();
    if spec.viewport.is_none() {
        spec.viewport = Some(viewport);
    }

    let mut row_count_by_dataset: BTreeMap<u64, usize> = BTreeMap::new();
    for (id, table) in &translated.datasets {
        row_count_by_dataset.insert(id.0 as u64, table.row_count);
    }

    let mut engine = ChartEngine::new(spec).expect("engine");
    for (id, table) in translated.datasets {
        engine.datasets_mut().insert(id, table);
    }
    for action in translated.actions {
        engine.apply_action(action);
    }

    let mut measurer = NullTextMeasurer::default();
    for _ in 0..64 {
        let result = engine
            .step(
                &mut measurer,
                WorkBudget::new(10_000_000, 10_000_000, 10_000_000),
            )
            .expect("step");
        if !result.unfinished {
            return snapshot_from_engine(&engine, &row_count_by_dataset);
        }
    }

    panic!("engine did not finish within the step cap");
}

fn assert_matches_golden(name: &str, option_json: &str) {
    assert_matches_golden_with_viewport(name, option_json, viewport_320x200());
}

fn assert_matches_golden_with_viewport(name: &str, option_json: &str, viewport: Rect) {
    let actual = run_option_snapshot_with_viewport(option_json, viewport);
    let actual_json = serde_json::to_string_pretty(&actual).expect("serialize");
    let actual_json = format!("{actual_json}\n");

    let path = goldens_dir().join(format!("{name}.json"));
    if std::env::var_os("FRET_UPDATE_GOLDENS").is_some() {
        std::fs::create_dir_all(goldens_dir()).expect("create goldens dir");
        std::fs::write(&path, actual_json).expect("write golden");
        return;
    }

    let expected = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing golden: {}\nerror: {err}\n\nTo (re)generate:\n  $env:FRET_UPDATE_GOLDENS='1'; cargo test -p fret-chart -F echarts --test echarts_headless_goldens\n",
            path.display()
        )
    });
    assert_eq!(expected, actual_json, "golden mismatch: {}", path.display());
}

#[test]
fn golden_line_category_basic() {
    let json = r#"
    {
      "xAxis": { "type": "category", "data": ["Mon","Tue","Wed","Thu","Fri"] },
      "yAxis": { "type": "value" },
      "series": [{ "type": "line", "data": [120, 132, 101, 134, 90] }]
    }
    "#;
    assert_matches_golden("line-category-basic", json);
}

#[test]
fn golden_dataset_encode_scatter() {
    let json = r#"
    {
      "dataset": {
        "source": [
          ["x","y"],
          [0, 100],
          [1, 10],
          [2, 20],
          [3, 30],
          [4, 0]
        ]
      },
      "xAxis": [{ "type": "value" }],
      "yAxis": [{ "type": "value" }],
      "series": [
        { "type": "scatter", "datasetIndex": 0, "encode": { "x": "x", "y": "y" } }
      ]
    }
    "#;
    assert_matches_golden("dataset-encode-scatter", json);
}

#[test]
fn golden_datazoom_percent_order_sensitive() {
    let json = r#"
    {
      "dataset": {
        "source": [
          ["x","y"],
          [0, 100],
          [1, 10],
          [2, 20],
          [3, 30],
          [4, 0]
        ]
      },
      "xAxis": [{ "type": "value" }],
      "yAxis": [{ "type": "value" }],
      "dataZoom": [
        { "type": "inside", "xAxisIndex": 0, "filterMode": "filter", "start": 20, "end": 80 },
        { "type": "inside", "yAxisIndex": 0, "filterMode": "filter", "start": 0, "end": 100 }
      ],
      "series": [
        { "type": "scatter", "datasetIndex": 0, "encode": { "x": "x", "y": "y" } }
      ]
    }
    "#;
    assert_matches_golden("datazoom-percent-order-sensitive", json);
}

#[test]
fn golden_visualmap_scatter_opacity_and_size() {
    let json = r#"
    {
      "dataset": {
        "source": [
          ["x","y"],
          [0, -1],
          [1, -0.5],
          [2, 0],
          [3, 0.5],
          [4, 1]
        ]
      },
      "xAxis": [{ "type": "value" }],
      "yAxis": [{ "type": "value" }],
      "visualMap": {
        "type": "continuous",
        "dimension": "y",
        "min": -1,
        "max": 1,
        "range": [-0.25, 0.75],
        "inRange": { "opacity": [0.2, 1.0], "symbolSize": [5, 20] },
        "outOfRange": { "opacity": 0.05 }
      },
      "series": [
        { "type": "scatter", "datasetIndex": 0, "encode": { "x": "x", "y": "y" } }
      ]
    }
    "#;
    assert_matches_golden("visualmap-scatter-opacity-and-size", json);
}

#[test]
fn golden_scatter_lod_forced_large_mode_is_pixel_bounded() {
    let mut source = Vec::with_capacity(1 + 200);
    source.push(serde_json::json!(["x", "y"]));
    for i in 0..200 {
        source.push(serde_json::json!([i as f64, (i % 50) as f64]));
    }

    let option = serde_json::json!({
      "dataset": { "source": source },
      "xAxis": [{ "type": "value" }],
      "yAxis": [{ "type": "value" }],
      "series": [
        {
          "type": "scatter",
          "datasetIndex": 0,
          "encode": { "x": "x", "y": "y" },
          "large": true,
          "largeThreshold": 1
        }
      ]
    });
    let json = serde_json::to_string(&option).expect("option json");

    let viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(200.0)));
    assert_matches_golden_with_viewport("scatter-lod-forced-large", &json, viewport);
}

#[test]
fn golden_bar_category_basic() {
    let json = r#"
    {
      "xAxis": { "type": "category", "data": ["A","B","C","D"] },
      "yAxis": { "type": "value" },
      "series": [{ "type": "bar", "data": [5, 20, 36, 10] }]
    }
    "#;
    assert_matches_golden("bar-category-basic", json);
}

#[test]
fn golden_multi_grid_two_series_with_shared_datazoom() {
    let mut source = Vec::with_capacity(1 + 21);
    source.push(serde_json::json!(["x", "y0", "y1"]));
    for i in 0..20 {
        source.push(serde_json::json!([i as f64, i as f64, (100 - i) as f64]));
    }

    let option = serde_json::json!({
      "grid": [{}, {}],
      "xAxis": [
        { "type": "value", "gridIndex": 0 },
        { "type": "value", "gridIndex": 1 }
      ],
      "yAxis": [
        { "type": "value", "gridIndex": 0 },
        { "type": "value", "gridIndex": 1 }
      ],
      "dataset": { "source": source },
      "dataZoom": [
        { "type": "inside", "xAxisIndex": [0, 1], "filterMode": "filter", "start": 25, "end": 75 }
      ],
      "series": [
        { "type": "scatter", "datasetIndex": 0, "xAxisIndex": 0, "yAxisIndex": 0, "encode": { "x": "x", "y": "y0" } },
        { "type": "line", "datasetIndex": 0, "xAxisIndex": 1, "yAxisIndex": 1, "encode": { "x": "x", "y": "y1" } }
      ]
    });
    let json = serde_json::to_string(&option).expect("option json");

    assert_matches_golden("multi-grid-two-series-datazoom", &json);
}

#[test]
fn golden_dataset_transform_filter_from_dataset() {
    let mut source = Vec::with_capacity(1 + 20);
    source.push(serde_json::json!(["x", "y"]));
    for i in 0..20 {
        source.push(serde_json::json!([i as f64, i as f64]));
    }

    let option = serde_json::json!({
      "dataset": [
        { "source": source },
        {
          "fromDatasetIndex": 0,
          "transform": { "type": "filter", "config": { "dimension": "y", "gte": 10 } }
        }
      ],
      "xAxis": [{ "type": "value" }],
      "yAxis": [{ "type": "value" }],
      "series": [
        { "type": "scatter", "datasetIndex": 1, "encode": { "x": "x", "y": "y" } }
      ]
    });
    let json = serde_json::to_string(&option).expect("option json");
    assert_matches_golden("dataset-transform-filter", &json);
}

#[test]
fn golden_dataset_transform_sort_desc_from_dataset() {
    let mut source = Vec::with_capacity(1 + 10);
    source.push(serde_json::json!(["x", "y"]));
    for i in 0..10 {
        source.push(serde_json::json!([i as f64, i as f64]));
    }

    let option = serde_json::json!({
      "dataset": [
        { "source": source },
        {
          "fromDatasetIndex": 0,
          "transform": { "type": "sort", "config": { "dimension": "y", "order": "desc" } }
        }
      ],
      "xAxis": [{ "type": "value" }],
      "yAxis": [{ "type": "value" }],
      "series": [
        { "type": "line", "datasetIndex": 1, "encode": { "x": "x", "y": "y" } }
      ]
    });
    let json = serde_json::to_string(&option).expect("option json");
    assert_matches_golden("dataset-transform-sort-desc", &json);
}

#[test]
fn golden_dataset_transform_chain_filter_then_sort_desc_from_dataset() {
    let mut source = Vec::with_capacity(1 + 20);
    source.push(serde_json::json!(["x", "y"]));
    for i in 0..20 {
        source.push(serde_json::json!([i as f64, i as f64]));
    }

    let option = serde_json::json!({
      "dataset": [
        { "source": source },
        {
          "fromDatasetIndex": 0,
          "transform": { "type": "filter", "config": { "dimension": "y", "gte": 5, "lte": 14 } }
        },
        {
          "fromDatasetIndex": 1,
          "transform": { "type": "sort", "config": { "dimension": "y", "order": "desc" } }
        }
      ],
      "xAxis": [{ "type": "value" }],
      "yAxis": [{ "type": "value" }],
      "series": [
        { "type": "scatter", "datasetIndex": 2, "encode": { "x": "x", "y": "y" } }
      ]
    });
    let json = serde_json::to_string(&option).expect("option json");
    assert_matches_golden("dataset-transform-chain-filter-sort-desc", &json);
}
