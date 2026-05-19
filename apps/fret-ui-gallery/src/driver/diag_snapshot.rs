use fret_app::App;
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
#[cfg(feature = "gallery-dev")]
use fret_core::AppWindowId;
use std::sync::Arc;

use crate::spec::{
    BISECT_DISABLE_CARD_CODE_TABS, BISECT_DISABLE_CARD_PAGE_INTRO,
    BISECT_DISABLE_CARD_SECTION_CARD_CONTENT, BISECT_DISABLE_CARD_SECTION_COMPOSITIONS,
    BISECT_DISABLE_CARD_SECTION_DEMO, BISECT_DISABLE_CARD_SECTION_IMAGE,
    BISECT_DISABLE_CARD_SECTION_MEETING_NOTES, BISECT_DISABLE_CARD_SECTION_NOTES,
    BISECT_DISABLE_CARD_SECTION_RTL, BISECT_DISABLE_CARD_SECTION_SIZE,
    BISECT_DISABLE_CARD_SECTION_USAGE, BISECT_MINIMAL_ROOT, BISECT_SIMPLE_CONTENT,
    BISECT_SIMPLE_SIDEBAR, PAGE_GROUPS, ui_gallery_bisect_flags,
};
use crate::ui::{card_doc_scaffold_metrics_json, nav_visibility_summary};

#[cfg(feature = "gallery-dev")]
use crate::harness::UiGalleryChartTortureOutputStore;
#[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
use crate::harness::{
    UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER, UiGalleryCodeEditorHandlesStore,
    UiGalleryMarkdownEditorHandlesStore,
};

use super::UiGalleryHarnessDiagnosticsStore;

fn arc_str_len(value: &Arc<str>) -> u64 {
    value.len() as u64
}

fn opt_arc_str_len(value: Option<&Arc<str>>) -> u64 {
    value.map(arc_str_len).unwrap_or(0)
}

fn vec_arc_str_len(values: &[Arc<str>]) -> u64 {
    values.iter().map(arc_str_len).sum()
}

fn opt_arc_str_json(value: Option<&Arc<str>>) -> serde_json::Value {
    value
        .map(|value| serde_json::Value::String(value.to_string()))
        .unwrap_or(serde_json::Value::Null)
}

fn theme_color_scheme_json(scheme: Option<fret_core::window::ColorScheme>) -> serde_json::Value {
    match scheme {
        Some(fret_core::window::ColorScheme::Light) => serde_json::json!("light"),
        Some(fret_core::window::ColorScheme::Dark) => serde_json::json!("dark"),
        None => serde_json::Value::Null,
    }
}

fn window_metrics_preferences_snapshot_json(
    app: &App,
    window: fret_core::AppWindowId,
) -> serde_json::Value {
    app.global::<fret_core::WindowMetricsService>()
        .map(|svc| {
            let text_scale_factor = svc.text_scale_factor(window);
            serde_json::json!({
                "schema_version": 1,
                "color_scheme_known": svc.color_scheme_is_known(window),
                "color_scheme": theme_color_scheme_json(svc.color_scheme(window)),
                "prefers_reduced_motion_known": svc.prefers_reduced_motion_is_known(window),
                "prefers_reduced_motion": svc.prefers_reduced_motion(window),
                "text_scale_factor_known": svc.text_scale_factor_is_known(window),
                "text_scale_factor": text_scale_factor.map(rounded_f32),
                "text_scale_factor_milli": text_scale_factor.map(|value| scaled_f32(value, 1000.0)),
            })
        })
        .unwrap_or_else(|| {
            serde_json::json!({
                "schema_version": 1,
                "color_scheme_known": false,
                "color_scheme": null,
                "prefers_reduced_motion_known": false,
                "prefers_reduced_motion": null,
                "text_scale_factor_known": false,
                "text_scale_factor": null,
                "text_scale_factor_milli": null,
            })
        })
}

fn scaled_f32(value: f32, scale: f32) -> i64 {
    (value * scale).round() as i64
}

fn rounded_f32(value: f32) -> f64 {
    (value as f64 * 1_000_000.0).round() / 1_000_000.0
}

fn cubic_bezier_json(value: fret_ui::theme::CubicBezier) -> serde_json::Value {
    serde_json::json!({
        "x1": rounded_f32(value.x1),
        "y1": rounded_f32(value.y1),
        "x2": rounded_f32(value.x2),
        "y2": rounded_f32(value.y2),
    })
}

fn cubic_bezier_milli_json(value: fret_ui::theme::CubicBezier) -> serde_json::Value {
    serde_json::json!({
        "x1": scaled_f32(value.x1, 1000.0),
        "y1": scaled_f32(value.y1, 1000.0),
        "x2": scaled_f32(value.x2, 1000.0),
        "y2": scaled_f32(value.y2, 1000.0),
    })
}

fn theme_runtime_snapshot_json(app: &App) -> serde_json::Value {
    let theme = fret_ui::Theme::global(app);
    let drag_release_settle_bounce =
        theme.number_token("number.motion.spring.drag_release_settle.bounce");
    let easing_standard = theme.easing_token("easing.motion.standard");
    let easing_stack_shift = theme.easing_token("easing.motion.stack.shift");
    serde_json::json!({
        "schema_version": 1,
        "revision": theme.revision(),
        "color_scheme": theme_color_scheme_json(theme.color_scheme),
        "motion_tokens": {
            "duration_presence_enter_ms": theme.duration_ms_token("duration.motion.presence.enter"),
            "duration_presence_exit_ms": theme.duration_ms_token("duration.motion.presence.exit"),
            "duration_stack_shift_ms": theme.duration_ms_token("duration.motion.stack.shift"),
            "duration_stack_shift_stagger_ms": theme.duration_ms_token("duration.motion.stack.shift.stagger"),
            "duration_drag_release_settle_ms": theme.duration_ms_token("duration.motion.spring.drag_release_settle"),
            "drag_release_settle_bounce": rounded_f32(drag_release_settle_bounce),
            "drag_release_settle_bounce_milli": scaled_f32(drag_release_settle_bounce, 1000.0),
            "easing_standard": cubic_bezier_json(easing_standard),
            "easing_standard_milli": cubic_bezier_milli_json(easing_standard),
            "easing_stack_shift": cubic_bezier_json(easing_stack_shift),
            "easing_stack_shift_milli": cubic_bezier_milli_json(easing_stack_shift),
        },
    })
}

#[cfg(feature = "gallery-dev")]
fn rounded_f64_json(value: f64) -> serde_json::Value {
    if value.is_finite() && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
        serde_json::json!(value.round() as i64)
    } else {
        serde_json::Value::Null
    }
}

#[cfg(feature = "gallery-dev")]
const CHART_TORTURE_X_BASE_MS: f64 = 1_735_689_600_000.0;
#[cfg(feature = "gallery-dev")]
const CHART_TORTURE_X_INTERVAL_MS: f64 = 60_000.0;
#[cfg(feature = "gallery-dev")]
const CHART_TORTURE_POINT_COUNT: u64 = 200_000;
#[cfg(feature = "gallery-dev")]
const CHART_TORTURE_WINDOW_EPSILON_MS: f64 = 1.0;

#[cfg(feature = "gallery-dev")]
fn chart_torture_full_x_pair() -> (f64, f64) {
    (
        CHART_TORTURE_X_BASE_MS,
        CHART_TORTURE_X_BASE_MS
            + CHART_TORTURE_X_INTERVAL_MS * (CHART_TORTURE_POINT_COUNT.saturating_sub(1) as f64),
    )
}

#[cfg(feature = "gallery-dev")]
fn chart_windows_approx_eq(left: Option<(f64, f64)>, right: Option<(f64, f64)>) -> bool {
    match (left, right) {
        (Some((left_min, left_max)), Some((right_min, right_max))) => {
            (left_min - right_min).abs() <= CHART_TORTURE_WINDOW_EPSILON_MS
                && (left_max - right_max).abs() <= CHART_TORTURE_WINDOW_EPSILON_MS
                && ((left_max - left_min) - (right_max - right_min)).abs()
                    <= CHART_TORTURE_WINDOW_EPSILON_MS
        }
        _ => false,
    }
}

#[cfg(feature = "gallery-dev")]
fn chart_window_changed_from(window: Option<(f64, f64)>, base: (f64, f64)) -> bool {
    window
        .map(|(min, max)| {
            (min - base.0).abs() > CHART_TORTURE_WINDOW_EPSILON_MS
                || (max - base.1).abs() > CHART_TORTURE_WINDOW_EPSILON_MS
                || ((max - min) - (base.1 - base.0)).abs() > CHART_TORTURE_WINDOW_EPSILON_MS
        })
        .unwrap_or(false)
}

#[cfg(feature = "gallery-dev")]
fn chart_window_json(pair: Option<(f64, f64)>) -> serde_json::Value {
    pair.map(|(min, max)| {
        let span = max - min;
        serde_json::json!({
            "present": true,
            "min_ms_rounded": rounded_f64_json(min),
            "max_ms_rounded": rounded_f64_json(max),
            "span_ms_rounded": rounded_f64_json(span),
        })
    })
    .unwrap_or_else(|| {
        serde_json::json!({
            "present": false,
        })
    })
}

#[cfg(feature = "gallery-dev")]
fn chart_numeric_window_json(pair: Option<(f64, f64)>) -> serde_json::Value {
    pair.map(|(min, max)| {
        let span = max - min;
        serde_json::json!({
            "present": true,
            "min_milli": scaled_f32(min as f32, 1000.0),
            "max_milli": scaled_f32(max as f32, 1000.0),
            "span_milli": scaled_f32(span as f32, 1000.0),
        })
    })
    .unwrap_or_else(|| {
        serde_json::json!({
            "present": false,
        })
    })
}

#[cfg(feature = "gallery-dev")]
fn chart_torture_snapshot_json(app: &App, window: AppWindowId) -> Option<serde_json::Value> {
    let handle = app
        .global::<UiGalleryChartTortureOutputStore>()?
        .per_window
        .get(&window)?;
    let output = app.models().get_cloned(&handle.output)?;
    let x_axis = delinea::ids::AxisId::new(1);
    let x_domain_key = fret_chart::LinkAxisKey {
        kind: delinea::AxisKind::X,
        dataset: delinea::ids::DatasetId::new(1),
        field: delinea::FieldId::new(1),
    };
    let y_explicit_domain_key = fret_chart::LinkAxisKey {
        kind: delinea::AxisKind::Y,
        dataset: delinea::ids::DatasetId::new(1),
        field: delinea::FieldId::new(2),
    };
    let full_x_pair = chart_torture_full_x_pair();
    let x_output_model_domain_pair = output
        .snapshot
        .domain_windows_by_key
        .get(&x_domain_key)
        .copied()
        .flatten()
        .filter(|window| window.is_valid())
        .map(|window| (window.min, window.max));
    let y_output_model_domain_pair = output
        .snapshot
        .domain_windows_by_key
        .get(&y_explicit_domain_key)
        .copied()
        .flatten()
        .filter(|window| window.is_valid())
        .map(|window| (window.min, window.max));
    let (engine_state_revision, x_data_zoom_pair, x_axis_output_pair) = {
        let engine = handle.shared_engine.borrow();
        let x_data_zoom_pair = engine
            .state()
            .data_zoom_x
            .get(&x_axis)
            .and_then(|state| state.window)
            .filter(|window| window.is_valid())
            .map(|window| (window.min, window.max));
        let x_axis_output_pair = engine
            .output()
            .axis_windows
            .get(&x_axis)
            .copied()
            .filter(|window| window.is_valid())
            .map(|window| (window.min, window.max));
        (
            engine.state().revision.0,
            x_data_zoom_pair,
            x_axis_output_pair,
        )
    };
    let x_axis_output_matches_data_zoom =
        chart_windows_approx_eq(x_axis_output_pair, x_data_zoom_pair);
    let x_output_model_domain_matches_data_zoom =
        chart_windows_approx_eq(x_output_model_domain_pair, x_data_zoom_pair);
    let x_axis_output_changed_from_full_domain =
        chart_window_changed_from(x_axis_output_pair, full_x_pair);
    let x_output_model_domain_changed_from_full_domain =
        chart_window_changed_from(x_output_model_domain_pair, full_x_pair);
    let y_output_model_domain_matches_explicit_fixture =
        chart_windows_approx_eq(y_output_model_domain_pair, Some((-0.25, 0.75)));
    let tooltip_lines = &output.snapshot.tooltip_lines;
    let tooltip_axis_header_count = tooltip_lines
        .iter()
        .filter(|line| line.kind == fret_chart::TooltipTextLineKind::AxisHeader)
        .count() as u64;
    let tooltip_series_labels = tooltip_lines
        .iter()
        .filter(|line| line.kind == fret_chart::TooltipTextLineKind::SeriesRow)
        .filter_map(|line| {
            line.columns
                .as_ref()
                .map(|(label, _)| label.clone())
                .or_else(|| {
                    line.text
                        .split_once(':')
                        .map(|(label, _)| label.to_string())
                })
        })
        .collect::<Vec<_>>();
    let tooltip_series_rows_count = tooltip_series_labels.len() as u64;
    let tooltip_source_series_rows_count = tooltip_lines
        .iter()
        .filter(|line| {
            line.kind == fret_chart::TooltipTextLineKind::SeriesRow && line.source_series.is_some()
        })
        .count() as u64;
    let tooltip_missing_rows_count =
        tooltip_lines.iter().filter(|line| line.is_missing).count() as u64;
    let tooltip_has_series_a = tooltip_series_labels.iter().any(|label| label == "A");
    let tooltip_has_series_b = tooltip_series_labels.iter().any(|label| label == "B");

    Some(serde_json::json!({
        "schema_version": 2,
        "engine_present": true,
        "engine_state_revision": engine_state_revision,
        "x_data_zoom": {
            "active": x_data_zoom_pair.is_some(),
            "window": chart_window_json(x_data_zoom_pair),
        },
        "x_full_domain_window": chart_window_json(Some(full_x_pair)),
        "x_axis_output_window": chart_window_json(x_axis_output_pair),
        "output_model": {
            "revision": output.revision,
            "link_events_revision": output.link_events_revision,
            "domain_windows_count": output.snapshot.domain_windows_by_key.len() as u64,
            "x_domain_window": chart_window_json(x_output_model_domain_pair),
            "y_explicit_domain_window": chart_numeric_window_json(y_output_model_domain_pair),
            "tooltip_lines_count": output.snapshot.tooltip_lines.len() as u64,
            "tooltip": {
                "lines_count": tooltip_lines.len() as u64,
                "axis_header_count": tooltip_axis_header_count,
                "series_rows_count": tooltip_series_rows_count,
                "source_series_rows_count": tooltip_source_series_rows_count,
                "missing_rows_count": tooltip_missing_rows_count,
                "series_labels": tooltip_series_labels,
                "has_series_a": tooltip_has_series_a,
                "has_series_b": tooltip_has_series_b,
            },
        },
        "runtime_oracles": {
            "x_axis_output_matches_data_zoom": x_axis_output_matches_data_zoom,
            "x_output_model_domain_matches_data_zoom": x_output_model_domain_matches_data_zoom,
            "x_axis_output_changed_from_full_domain": x_axis_output_changed_from_full_domain,
            "x_output_model_domain_changed_from_full_domain": x_output_model_domain_changed_from_full_domain,
            "y_output_model_domain_matches_explicit_fixture": y_output_model_domain_matches_explicit_fixture,
        },
    }))
}

fn command_registry_string_bytes_estimate(app: &App) -> serde_json::Value {
    let mut entries = 0u64;
    let mut keywords = 0u64;
    let mut string_bytes = 0u64;
    for (id, meta) in app.commands().iter() {
        entries = entries.saturating_add(1);
        string_bytes = string_bytes
            .saturating_add(id.0.len() as u64)
            .saturating_add(meta.title.len() as u64)
            .saturating_add(
                meta.description
                    .as_ref()
                    .map(|v| v.len() as u64)
                    .unwrap_or(0),
            )
            .saturating_add(meta.category.as_ref().map(|v| v.len() as u64).unwrap_or(0));
        for keyword in &meta.keywords {
            keywords = keywords.saturating_add(1);
            string_bytes = string_bytes.saturating_add(keyword.len() as u64);
        }
    }
    serde_json::json!({
        "command_registry_entries": entries,
        "command_registry_keywords": keywords,
        "command_registry_string_bytes_estimate_total": string_bytes,
    })
}

#[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
fn code_editor_paint_perf_json(
    frame: fret_code_editor::CodeEditorPaintPerfFrame,
) -> serde_json::Value {
    let mut out = serde_json::Map::with_capacity(123);
    macro_rules! insert_u64 {
        ($key:literal, $value:expr) => {
            out.insert($key.to_string(), serde_json::Value::from($value));
        };
    }

    insert_u64!("schema_version", 12);
    insert_u64!("frame_seq", frame.frame_seq);
    insert_u64!("visible_start", frame.visible_start);
    insert_u64!("visible_end", frame.visible_end);
    insert_u64!("visible_rows", frame.visible_rows);
    insert_u64!("cache_base_entries", frame.cache_base_entries);
    insert_u64!("cache_frame_min_entries", frame.cache_frame_min_entries);
    insert_u64!("cache_effective_entries", frame.cache_effective_entries);
    insert_u64!("rows_painted", frame.rows_painted);
    insert_u64!("rows_drew_rich", frame.rows_drew_rich);
    insert_u64!("rows_scene_replayed", frame.rows_scene_replayed);
    insert_u64!(
        "rows_scene_prepaint_planned",
        frame.rows_scene_prepaint_planned
    );
    insert_u64!(
        "rows_scene_prepaint_plan_used",
        frame.rows_scene_prepaint_plan_used
    );
    insert_u64!("rows_scene_stored", frame.rows_scene_stored);
    insert_u64!(
        "rows_scene_stored_at_visible_start",
        frame.rows_scene_stored_at_visible_start
    );
    insert_u64!(
        "rows_scene_stored_at_visible_end",
        frame.rows_scene_stored_at_visible_end
    );
    insert_u64!("row_scene_ops_stored", frame.row_scene_ops_stored);
    insert_u64!(
        "rows_scene_prepaint_edge_stored",
        frame.rows_scene_prepaint_edge_stored
    );
    insert_u64!(
        "row_scene_prepaint_edge_ops_stored",
        frame.row_scene_prepaint_edge_ops_stored
    );
    insert_u64!(
        "rows_scene_prepaint_candidates",
        frame.rows_scene_prepaint_candidates
    );
    insert_u64!(
        "rows_scene_prepaint_skip_no_cache",
        frame.rows_scene_prepaint_skip_no_cache
    );
    insert_u64!(
        "rows_scene_prepaint_skip_unsupported_key",
        frame.rows_scene_prepaint_skip_unsupported_key
    );
    insert_u64!(
        "rows_scene_prepaint_skip_preedit",
        frame.rows_scene_prepaint_skip_preedit
    );
    insert_u64!(
        "rows_scene_prepaint_skip_syntax_empty",
        frame.rows_scene_prepaint_skip_syntax_empty
    );
    insert_u64!(
        "rows_scene_prepaint_skip_key_mismatch",
        frame.rows_scene_prepaint_skip_key_mismatch
    );
    insert_u64!(
        "rows_scene_fast_miss_no_entry",
        frame.rows_scene_fast_miss_no_entry
    );
    insert_u64!(
        "rows_scene_fast_miss_key_mismatch",
        frame.rows_scene_fast_miss_key_mismatch
    );
    insert_u64!(
        "rows_scene_full_miss_no_entry",
        frame.rows_scene_full_miss_no_entry
    );
    insert_u64!(
        "rows_scene_full_miss_key_mismatch",
        frame.rows_scene_full_miss_key_mismatch
    );
    insert_u64!("quads_selection", frame.quads_selection);
    insert_u64!("quads_caret", frame.quads_caret);
    insert_u64!("us_total", frame.us_total);
    insert_u64!("us_row_text", frame.us_row_text);
    insert_u64!("us_baseline_measure", frame.us_baseline_measure);
    insert_u64!("us_syntax_spans", frame.us_syntax_spans);
    insert_u64!("us_rich_materialize", frame.us_rich_materialize);
    insert_u64!("us_text_draw", frame.us_text_draw);
    insert_u64!("us_row_rich_cache_compare", frame.us_row_rich_cache_compare);
    insert_u64!("us_row_geom_key", frame.us_row_geom_key);
    insert_u64!("us_row_scene_key", frame.us_row_scene_key);
    insert_u64!("us_row_scene_fast_probe", frame.us_row_scene_fast_probe);
    insert_u64!("us_row_scene_full_probe", frame.us_row_scene_full_probe);
    insert_u64!(
        "us_row_scene_fast_key_compare",
        frame.us_row_scene_fast_key_compare
    );
    insert_u64!(
        "us_row_scene_full_key_compare",
        frame.us_row_scene_full_key_compare
    );
    insert_u64!("us_row_scene_replay_touch", frame.us_row_scene_replay_touch);
    insert_u64!("us_row_scene_replay_ops", frame.us_row_scene_replay_ops);
    insert_u64!(
        "us_row_scene_prepaint_plan",
        frame.us_row_scene_prepaint_plan
    );
    insert_u64!("us_row_scene_capture_ops", frame.us_row_scene_capture_ops);
    insert_u64!("us_row_scene_store", frame.us_row_scene_store);
    insert_u64!(
        "us_row_scene_prepaint_edge_store",
        frame.us_row_scene_prepaint_edge_store
    );
    insert_u64!("us_row_scene_fast_path", frame.us_row_scene_fast_path);
    insert_u64!("us_row_scene_full_path", frame.us_row_scene_full_path);
    insert_u64!("us_selection_rects", frame.us_selection_rects);
    insert_u64!("us_caret_x", frame.us_caret_x);
    insert_u64!("us_caret_stops", frame.us_caret_stops);
    insert_u64!("us_caret_rect", frame.us_caret_rect);
    insert_u64!("us_row_geom_cache", frame.us_row_geom_cache);
    insert_u64!("us_row_content_resolve", frame.us_row_content_resolve);
    insert_u64!("us_row_geom_resolve", frame.us_row_geom_resolve);
    insert_u64!("us_row_overlay", frame.us_row_overlay);
    insert_u64!("us_frame_overlay_prepare", frame.us_frame_overlay_prepare);
    insert_u64!("surface_rows_iterated", frame.surface_rows_iterated);
    insert_u64!("surface_rows_with_rect", frame.surface_rows_with_rect);
    insert_u64!(
        "us_windowed_surface_paint_callback",
        frame.us_windowed_surface_paint_callback
    );
    insert_u64!(
        "us_windowed_surface_frame_lookup",
        frame.us_windowed_surface_frame_lookup
    );
    insert_u64!("us_windowed_surface_hook", frame.us_windowed_surface_hook);
    insert_u64!(
        "us_windowed_surface_row_loop",
        frame.us_windowed_surface_row_loop
    );
    insert_u64!(
        "us_windowed_surface_row_rect",
        frame.us_windowed_surface_row_rect
    );
    insert_u64!(
        "us_windowed_surface_row_paint",
        frame.us_windowed_surface_row_paint
    );
    insert_u64!(
        "us_windowed_surface_non_row",
        frame.us_windowed_surface_non_row
    );
    insert_u64!(
        "us_windowed_surface_row_callback_gap",
        frame.us_windowed_surface_row_callback_gap
    );
    insert_u64!("us_torture_autoscroll", frame.us_torture_autoscroll);
    insert_u64!("us_torture_overlay", frame.us_torture_overlay);
    insert_u64!("syntax_rows_stored", frame.syntax_rows_stored);
    insert_u64!("us_syntax_slice", frame.us_syntax_slice);
    insert_u64!("us_syntax_highlight", frame.us_syntax_highlight);
    insert_u64!("us_syntax_distribute", frame.us_syntax_distribute);
    insert_u64!("us_syntax_store", frame.us_syntax_store);
    insert_u64!("ns_total", frame.ns_total);
    insert_u64!("ns_row_text", frame.ns_row_text);
    insert_u64!("ns_baseline_measure", frame.ns_baseline_measure);
    insert_u64!("ns_syntax_spans", frame.ns_syntax_spans);
    insert_u64!("ns_rich_materialize", frame.ns_rich_materialize);
    insert_u64!("ns_text_draw", frame.ns_text_draw);
    insert_u64!("ns_row_rich_cache_compare", frame.ns_row_rich_cache_compare);
    insert_u64!("ns_row_geom_key", frame.ns_row_geom_key);
    insert_u64!("ns_row_scene_key", frame.ns_row_scene_key);
    insert_u64!("ns_row_scene_fast_probe", frame.ns_row_scene_fast_probe);
    insert_u64!("ns_row_scene_full_probe", frame.ns_row_scene_full_probe);
    insert_u64!(
        "ns_row_scene_fast_key_compare",
        frame.ns_row_scene_fast_key_compare
    );
    insert_u64!(
        "ns_row_scene_full_key_compare",
        frame.ns_row_scene_full_key_compare
    );
    insert_u64!("ns_row_scene_replay_touch", frame.ns_row_scene_replay_touch);
    insert_u64!("ns_row_scene_replay_ops", frame.ns_row_scene_replay_ops);
    insert_u64!(
        "ns_row_scene_prepaint_plan",
        frame.ns_row_scene_prepaint_plan
    );
    insert_u64!("ns_row_scene_capture_ops", frame.ns_row_scene_capture_ops);
    insert_u64!("ns_row_scene_store", frame.ns_row_scene_store);
    insert_u64!(
        "ns_row_scene_prepaint_edge_store",
        frame.ns_row_scene_prepaint_edge_store
    );
    insert_u64!("ns_row_scene_fast_path", frame.ns_row_scene_fast_path);
    insert_u64!("ns_row_scene_full_path", frame.ns_row_scene_full_path);
    insert_u64!("ns_selection_rects", frame.ns_selection_rects);
    insert_u64!("ns_caret_x", frame.ns_caret_x);
    insert_u64!("ns_caret_stops", frame.ns_caret_stops);
    insert_u64!("ns_caret_rect", frame.ns_caret_rect);
    insert_u64!("ns_row_geom_cache", frame.ns_row_geom_cache);
    insert_u64!("ns_row_content_resolve", frame.ns_row_content_resolve);
    insert_u64!("ns_row_geom_resolve", frame.ns_row_geom_resolve);
    insert_u64!("ns_row_overlay", frame.ns_row_overlay);
    insert_u64!("ns_frame_overlay_prepare", frame.ns_frame_overlay_prepare);
    insert_u64!(
        "ns_windowed_surface_paint_callback",
        frame.ns_windowed_surface_paint_callback
    );
    insert_u64!(
        "ns_windowed_surface_frame_lookup",
        frame.ns_windowed_surface_frame_lookup
    );
    insert_u64!("ns_windowed_surface_hook", frame.ns_windowed_surface_hook);
    insert_u64!(
        "ns_windowed_surface_row_loop",
        frame.ns_windowed_surface_row_loop
    );
    insert_u64!(
        "ns_windowed_surface_row_rect",
        frame.ns_windowed_surface_row_rect
    );
    insert_u64!(
        "ns_windowed_surface_row_paint",
        frame.ns_windowed_surface_row_paint
    );
    insert_u64!(
        "ns_windowed_surface_non_row",
        frame.ns_windowed_surface_non_row
    );
    insert_u64!(
        "ns_windowed_surface_row_callback_gap",
        frame.ns_windowed_surface_row_callback_gap
    );
    insert_u64!("ns_torture_autoscroll", frame.ns_torture_autoscroll);
    insert_u64!("ns_torture_overlay", frame.ns_torture_overlay);
    insert_u64!("ns_syntax_slice", frame.ns_syntax_slice);
    insert_u64!("ns_syntax_highlight", frame.ns_syntax_highlight);
    insert_u64!("ns_syntax_distribute", frame.ns_syntax_distribute);
    insert_u64!("ns_syntax_store", frame.ns_syntax_store);

    serde_json::Value::Object(out)
}

fn command_palette_entries_bytes_estimate(app: &App) -> serde_json::Value {
    let mut entries = 0u64;
    let mut groups = 0u64;
    let mut string_bytes = 0u64;

    let mut commands: Vec<_> = app
        .commands()
        .iter()
        .filter_map(|(id, meta)| (!meta.hidden).then_some((id, meta)))
        .collect();
    commands.sort_by(|(a_id, a_meta), (b_id, b_meta)| {
        match (&a_meta.category, &b_meta.category) {
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => a.as_ref().cmp(b.as_ref()),
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a_meta.title.as_ref().cmp(b_meta.title.as_ref()))
        .then_with(|| a_id.as_str().cmp(b_id.as_str()))
    });

    let mut seen_categories = std::collections::BTreeSet::<Arc<str>>::new();
    for (id, meta) in commands {
        entries = entries.saturating_add(1);
        string_bytes = string_bytes
            .saturating_add(meta.title.len() as u64)
            .saturating_add(id.as_str().len() as u64);
        if let Some(category) = meta.category.as_ref() {
            if seen_categories.insert(category.clone()) {
                groups = groups.saturating_add(1);
                string_bytes = string_bytes.saturating_add(category.len() as u64);
            }
            string_bytes = string_bytes.saturating_add(category.len() as u64);
        }
        if let Some(description) = meta.description.as_ref() {
            string_bytes = string_bytes.saturating_add(description.len() as u64);
        }
        for keyword in &meta.keywords {
            string_bytes = string_bytes.saturating_add(keyword.len() as u64);
        }
    }

    serde_json::json!({
        "command_palette_entries_count": entries,
        "command_palette_groups_count": groups,
        "command_palette_entries_string_bytes_estimate_total": string_bytes,
    })
}

fn settings_sheet_static_string_bytes_estimate() -> serde_json::Value {
    let strings = [
        "Settings",
        "Menu bar presentation, text fallback policy, and chrome/debug state.",
        "Menu bar surfaces",
        "Text",
        "Common fallback injection",
        "Chrome",
        "Command availability (debug)",
        "Auto (Windows/macOS on; Linux/Web off)",
        "On",
        "Off",
        "Auto (Linux/Web on; Windows/macOS off)",
        "On",
        "Off",
        "Platform default (desktop system fallback; wasm/bundled common fallback)",
        "None (never inject common fallback)",
        "Common fallback (always inject the curated fallback stack)",
        "Workspace tabs in the top bar",
        "edit.can_undo (enables OS/in-window Undo)",
        "edit.can_redo (enables OS/in-window Redo)",
        "Apply (in memory)",
        "Write project .fret/settings.json",
        "Close",
    ];
    let total = strings.iter().map(|s| s.len() as u64).sum::<u64>();
    serde_json::json!({
        "settings_sheet_static_strings_count": strings.len() as u64,
        "settings_sheet_static_string_bytes_estimate_total": total,
    })
}

fn page_specs_string_bytes_estimate() -> serde_json::Value {
    let mut entries = 0u64;
    let mut tags = 0u64;
    let mut string_bytes = 0u64;
    for group in PAGE_GROUPS {
        string_bytes = string_bytes.saturating_add(group.title.len() as u64);
        for item in group.items {
            entries = entries.saturating_add(1);
            string_bytes = string_bytes
                .saturating_add(item.id.len() as u64)
                .saturating_add(item.label.len() as u64)
                .saturating_add(item.title.len() as u64)
                .saturating_add(item.origin.len() as u64)
                .saturating_add(item.command.len() as u64);
            for tag in item.tags {
                tags = tags.saturating_add(1);
                string_bytes = string_bytes.saturating_add(tag.len() as u64);
            }
        }
    }
    serde_json::json!({
        "page_specs_entries": entries,
        "page_specs_tags": tags,
        "page_specs_string_bytes_estimate_total": string_bytes,
    })
}

pub(super) fn install_ui_gallery_snapshot_provider(app: &mut App) {
    app.with_global_mut_untracked(
        UiDiagnosticsService::default,
        |svc: &mut UiDiagnosticsService, _app| {
            svc.set_app_snapshot_provider(Some(Arc::new(|app, window| {
                let store = app.global::<UiGalleryHarnessDiagnosticsStore>()?;
                let ids = store.per_window.get(&window)?;

                let selected_page = app.models().get_cloned(&ids.selected_page)?;
                let workspace_tabs = app.models().get_cloned(&ids.workspace_tabs)?;
                let workspace_dirty_tabs = app.models().get_cloned(&ids.workspace_dirty_tabs)?;
                let nav_query = app.models().get_cloned(&ids.nav_query)?;
                let theme_preset = app.models().get_cloned(&ids.theme_preset)?;
                let theme_preset_open = app.models().get_cloned(&ids.theme_preset_open)?;
                let motion_preset = app.models().get_cloned(&ids.motion_preset)?;
                let motion_preset_open = app.models().get_cloned(&ids.motion_preset_open)?;
                let view_cache_enabled = app.models().get_cloned(&ids.view_cache_enabled)?;
                let view_cache_cache_shell =
                    app.models().get_cloned(&ids.view_cache_cache_shell)?;
                let view_cache_cache_content =
                    app.models().get_cloned(&ids.view_cache_cache_content)?;
                let view_cache_inner_enabled =
                    app.models().get_cloned(&ids.view_cache_inner_enabled)?;
                let view_cache_popover_open =
                    app.models().get_cloned(&ids.view_cache_popover_open)?;
                let view_cache_continuous = app.models().get_cloned(&ids.view_cache_continuous)?;
                let view_cache_counter = app.models().get_cloned(&ids.view_cache_counter)?;
                let settings_open = app.models().get_cloned(&ids.settings_open)?;
                let settings_menu_bar_os = app.models().get_cloned(&ids.settings_menu_bar_os)?;
                let settings_menu_bar_in_window = app.models().get_cloned(&ids.settings_menu_bar_in_window)?;
                let settings_text_common_fallback_injection = app
                    .models()
                    .get_cloned(&ids.settings_text_common_fallback_injection)?;
                let chrome_show_workspace_tab_strip = app
                    .models()
                    .get_cloned(&ids.chrome_show_workspace_tab_strip)?;
                let cmdk_query = app.models().get_cloned(&ids.cmdk_query)?;
                let last_action = app.models().get_cloned(&ids.last_action)?;
                let input_file_value = app.models().get_cloned(&ids.input_file_value)?;
                #[cfg(feature = "gallery-dev")]
                let syntax_rust = app.models().get_cloned(&ids.code_editor_syntax_rust)?;
                #[cfg(feature = "gallery-dev")]
                let boundary_identifier = app
                    .models()
                    .get_cloned(&ids.code_editor_boundary_identifier)?;
                #[cfg(feature = "gallery-dev")]
                let soft_wrap = app.models().get_cloned(&ids.code_editor_soft_wrap)?;
                #[cfg(feature = "gallery-dev")]
                let folds = app.models().get_cloned(&ids.code_editor_folds)?;
                #[cfg(feature = "gallery-dev")]
                let inlays = app.models().get_cloned(&ids.code_editor_inlays)?;
                #[cfg(not(feature = "gallery-dev"))]
                let (syntax_rust, boundary_identifier, soft_wrap, folds, inlays) =
                    (false, false, false, false, false);
                let text_input = app.models().get_cloned(&ids.text_input)?;
                let text_area = app.models().get_cloned(&ids.text_area)?;

                let (torture, markdown_editor_source): (
                    Option<serde_json::Value>,
                    Option<serde_json::Value>,
                ) = {
                    #[cfg(all(feature = "gallery-dev", not(target_arch = "wasm32")))]
                    {
                        let torture = app
                            .global::<UiGalleryCodeEditorHandlesStore>()
                            .and_then(|store| store.per_window.get(&window))
                            .map(|handle| {
                                let text_len_bytes = handle.with_buffer(|b| b.len_bytes());
                                let marker_present = handle.diag_buffer_contains_str_cached(
                                    UI_GALLERY_CODE_EDITOR_TORTURE_SOFT_WRAP_MARKER,
                                );
                                let selection = handle.selection();
                                let anchor = selection.anchor.min(text_len_bytes) as u64;
                                let caret = selection.caret().min(text_len_bytes) as u64;
                                let stats = handle.cache_stats();
                                let sizes = handle.cache_size_snapshot();
                                let mem = handle.memory_snapshot();
                                let paint_perf =
                                    handle.paint_perf_frame().map(code_editor_paint_perf_json);
                                let feature_payloads = handle.feature_payload_snapshot();
                                let preedit_active = handle.preedit_active();
                                let allow_decorations_under_inline_preedit =
                                    handle.debug_allow_decorations_under_inline_preedit();
                                let compose_inline_preedit = handle.debug_compose_inline_preedit();
                                let interaction = handle.interaction();
                                let buffer_revision = handle.buffer_revision().0;
                                let fold_placeholder_present = handle
                                    .diag_decorated_line_text(0)
                                    .is_some_and(|t| t.contains('…'));
                                let inlay_present = handle
                                    .diag_decorated_line_text(0)
                                    .is_some_and(|t| t.contains("<inlay>"));
                                serde_json::json!({
                                    "schema_version": 1,
                                    "marker_present": marker_present,
                                    "preedit_active": preedit_active,
                                    "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                                    "compose_inline_preedit": compose_inline_preedit,
                                    "interaction": {
                                        "enabled": interaction.enabled,
                                        "focusable": interaction.focusable,
                                        "selectable": interaction.selectable,
                                        "editable": interaction.editable,
                                    },
                                    "buffer_revision": buffer_revision,
                                    "folds": { "enabled": folds, "line0_placeholder_present": fold_placeholder_present },
                                    "inlays": { "enabled": inlays, "line0_inlay_present": inlay_present },
                                    "text_len_bytes": text_len_bytes as u64,
                                    "selection": { "anchor": anchor, "caret": caret },
                                    "cache_stats": {
                                        "row_text_get_calls": stats.row_text_get_calls,
                                        "row_text_hits": stats.row_text_hits,
                                        "row_text_misses": stats.row_text_misses,
                                        "row_text_evictions": stats.row_text_evictions,
                                        "row_text_resets": stats.row_text_resets,
                                        "row_scene_get_calls": stats.row_scene_get_calls(),
                                        "row_scene_hits": stats.row_scene_hits(),
                                        "row_scene_misses": stats.row_scene_misses(),
                                        "row_scene_evictions": stats.row_scene_evictions(),
                                        "row_scene_resets": stats.row_scene_resets(),
                                        "row_scene_fast_get_calls": stats.row_scene_fast_get_calls(),
                                        "row_scene_fast_hits": stats.row_scene_fast_hits(),
                                        "row_scene_fast_misses": stats.row_scene_fast_misses(),
                                        "row_rich_get_calls": stats.row_rich_get_calls(),
                                        "row_rich_hits": stats.row_rich_hits(),
                                        "row_rich_misses": stats.row_rich_misses(),
                                        "row_rich_evictions": stats.row_rich_evictions(),
                                        "row_rich_resets": stats.row_rich_resets(),
                                        "geom_pointer_hit_test_fallbacks": stats.geom_pointer_hit_test_fallbacks,
                                        "geom_caret_rect_fallbacks": stats.geom_caret_rect_fallbacks,
                                        "geom_vertical_move_fallbacks": stats.geom_vertical_move_fallbacks,
                                        "syntax_get_calls": stats.syntax_get_calls,
                                        "syntax_hits": stats.syntax_hits,
                                        "syntax_misses": stats.syntax_misses,
                                        "syntax_evictions": stats.syntax_evictions,
                                        "syntax_resets": stats.syntax_resets,
                                    },
                                    "cache_sizes": {
                                        "schema_version": sizes.schema_version,
                                        "row_text_cache_entries": sizes.row_text_cache_entries,
                                        "row_text_cache_queue_len": sizes.row_text_cache_queue_len,
                                        "row_text_cache_text_bytes_estimate_total": sizes.row_text_cache_text_bytes_estimate_total,
                                        "row_text_cache_row_spans_len_total": sizes.row_text_cache_row_spans_len_total,
                                        "row_geom_cache_entries": sizes.row_geom_cache_entries,
                                        "row_geom_cache_queue_len": sizes.row_geom_cache_queue_len,
                                        "row_geom_cache_caret_stops_len_total": sizes.row_geom_cache_caret_stops_len_total,
                                        "row_scene_cache_entries": sizes.row_scene_cache_entries,
                                        "row_scene_cache_queue_len": sizes.row_scene_cache_queue_len,
                                        "row_scene_cache_scene_ops_len_total": sizes.row_scene_cache_scene_ops_len_total,
                                        "syntax_row_cache_entries": sizes.syntax_row_cache_entries,
                                        "syntax_row_cache_queue_len": sizes.syntax_row_cache_queue_len,
                                        "syntax_row_cache_spans_len_total": sizes.syntax_row_cache_spans_len_total,
                                        "row_rich_cache_entries": sizes.row_rich_cache_entries,
                                        "row_rich_cache_queue_len": sizes.row_rich_cache_queue_len,
                                        "row_rich_cache_line_bytes_estimate_total": sizes.row_rich_cache_line_bytes_estimate_total,
                                        "row_rich_cache_row_spans_len_total": sizes.row_rich_cache_row_spans_len_total,
                                        "row_rich_cache_syntax_spans_len_total": sizes.row_rich_cache_syntax_spans_len_total,
                                        "row_rich_cache_rich_spans_len_total": sizes.row_rich_cache_rich_spans_len_total,
                                        "selection_rect_scratch_capacity": sizes.selection_rect_scratch_capacity,
                                    },
                                    "memory": {
                                        "schema_version": mem.schema_version,
                                        "buffer_revision": mem.buffer_revision,
                                        "buffer_len_bytes": mem.buffer_len_bytes,
                                        "buffer_line_count": mem.buffer_line_count,
                                        "undo_limit": mem.undo_limit,
                                        "undo_len": mem.undo_len,
                                        "redo_len": mem.redo_len,
                                        "undo_text_bytes_estimate_total": mem.undo_text_bytes_estimate_total,
                                        "redo_text_bytes_estimate_total": mem.redo_text_bytes_estimate_total,
                                        "undo_edit_count_total": mem.undo_edit_count_total,
                                        "redo_edit_count_total": mem.redo_edit_count_total,
                                    },
                                    "feature_payloads": {
                                        "schema_version": feature_payloads.schema_version,
                                        "epoch": feature_payloads.epoch,
                                        "buffer_revision": feature_payloads.buffer_revision,
                                        "display_map_epoch": feature_payloads.display_map_epoch,
                                        "diagnostic_spans_count": feature_payloads.diagnostic_spans_count,
                                        "diagnostic_line_summaries_count": feature_payloads.diagnostic_line_summaries_count,
                                        "range_decorations_count": feature_payloads.range_decorations_count,
                                        "gutter_markers_count": feature_payloads.gutter_markers_count,
                                        "semantic_tokens_count": feature_payloads.semantic_tokens_count,
                                    },
                                    "paint_perf": paint_perf,
                                })
                            })
                            ;

                        let markdown_editor_source = app
                            .global::<UiGalleryMarkdownEditorHandlesStore>()
                            .and_then(|store| store.per_window.get(&window))
                            .map(|handle| {
                                let text_len_bytes = handle.with_buffer(|b| b.len_bytes());
                                let selection = handle.selection();
                                let anchor = selection.anchor.min(text_len_bytes) as u64;
                                let caret = selection.caret().min(text_len_bytes) as u64;
                                let preedit_active = handle.preedit_active();
                                let interaction = handle.interaction();
                                let buffer_revision = handle.buffer_revision().0 as u64;
                                let fold_placeholder_present = handle
                                    .diag_decorated_line_text(0)
                                    .is_some_and(|t| t.contains('…'));
                                let fold_fixture_span_line0 = handle
                                    .with_buffer(|b| b.line_text(0))
                                    .and_then(|line| {
                                        let start =
                                            line.find("Editor").unwrap_or(2).min(line.len());
                                        let end = line.len();
                                        (start < end).then_some(serde_json::json!({
                                            "start": start as u64,
                                            "end": end as u64,
                                        }))
                                    });
                                let inlay_present = handle
                                    .diag_decorated_line_text(0)
                                    .is_some_and(|t| t.contains("<inlay>"));
                                let inlay_fixture_byte_line0 = handle
                                    .with_buffer(|b| b.line_text(0))
                                    .map(|line| 2usize.min(line.len()) as u64)
                                    .unwrap_or(0);
                                serde_json::json!({
                                    "schema_version": 1,
                                    "preedit_active": preedit_active,
                                    "interaction": {
                                        "enabled": interaction.enabled,
                                        "focusable": interaction.focusable,
                                        "selectable": interaction.selectable,
                                        "editable": interaction.editable,
                                    },
                                    "buffer_revision": buffer_revision,
                                    "folds": {
                                        "enabled": folds,
                                        "line0_placeholder_present": fold_placeholder_present,
                                        "fixture_span_line0": fold_fixture_span_line0,
                                    },
                                    "inlays": {
                                        "enabled": inlays,
                                        "line0_present": inlay_present,
                                        "fixture_byte_line0": inlay_fixture_byte_line0,
                                    },
                                    "text_len_bytes": text_len_bytes as u64,
                                    "selection": { "anchor": anchor, "caret": caret },
                                })
                            })
                            ;

                        (torture, markdown_editor_source)
                    }

                    #[cfg(any(target_arch = "wasm32", not(feature = "gallery-dev")))]
                    {
                        (None, None)
                    }
                };

                let bisect = ui_gallery_bisect_flags();
                let nav_visibility = nav_visibility_summary(nav_query.as_str());
                let mut shell = serde_json::Map::new();
                shell.insert("schema_version".to_string(), serde_json::json!(1));
                shell.insert("bisect_flags".to_string(), serde_json::json!(bisect));
                shell.insert("minimal_root".to_string(), serde_json::json!((bisect & BISECT_MINIMAL_ROOT) != 0));
                shell.insert("simple_sidebar".to_string(), serde_json::json!((bisect & BISECT_SIMPLE_SIDEBAR) != 0));
                shell.insert("simple_content".to_string(), serde_json::json!((bisect & BISECT_SIMPLE_CONTENT) != 0));
                shell.insert("card_section_demo_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_DEMO) != 0));
                shell.insert("card_section_usage_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_USAGE) != 0));
                shell.insert("card_section_size_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_SIZE) != 0));
                shell.insert("card_section_card_content_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_CARD_CONTENT) != 0));
                shell.insert("card_section_meeting_notes_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_MEETING_NOTES) != 0));
                shell.insert("card_section_image_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_IMAGE) != 0));
                shell.insert("card_section_rtl_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_RTL) != 0));
                shell.insert("card_section_compositions_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_COMPOSITIONS) != 0));
                shell.insert("card_section_notes_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_SECTION_NOTES) != 0));
                shell.insert("card_code_tabs_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_CODE_TABS) != 0));
                shell.insert("card_page_intro_disabled".to_string(), serde_json::json!((bisect & BISECT_DISABLE_CARD_PAGE_INTRO) != 0));
                if selected_page.as_ref() == "card" {
                    if let Some(obj) = card_doc_scaffold_metrics_json(bisect).as_object() {
                        for (k, v) in obj {
                            shell.insert(k.clone(), v.clone());
                        }
                    }
                }
                shell.insert(
                    "card_sections_hidden_count".to_string(),
                    serde_json::json!(
                        ((bisect & BISECT_DISABLE_CARD_SECTION_DEMO) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_USAGE) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_SIZE) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_CARD_CONTENT) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_MEETING_NOTES) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_IMAGE) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_RTL) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_COMPOSITIONS) != 0) as u64
                            + ((bisect & BISECT_DISABLE_CARD_SECTION_NOTES) != 0) as u64
                    ),
                );
                shell.insert("workspace_tabs_count".to_string(), serde_json::json!(workspace_tabs.len() as u64));
                shell.insert("workspace_tabs_bytes_estimate_total".to_string(), serde_json::json!(vec_arc_str_len(&workspace_tabs)));
                shell.insert("workspace_dirty_tabs_count".to_string(), serde_json::json!(workspace_dirty_tabs.len() as u64));
                shell.insert("workspace_dirty_tabs_bytes_estimate_total".to_string(), serde_json::json!(vec_arc_str_len(&workspace_dirty_tabs)));
                shell.insert("nav_query_len_bytes".to_string(), serde_json::json!(nav_query.len() as u64));
                shell.insert("nav_visible_groups_count".to_string(), serde_json::json!(nav_visibility.visible_groups_count));
                shell.insert("nav_visible_items_count".to_string(), serde_json::json!(nav_visibility.visible_items_count));
                shell.insert("nav_visible_ai_items_count".to_string(), serde_json::json!(nav_visibility.visible_ai_items_count));
                shell.insert("nav_visible_tags_count".to_string(), serde_json::json!(nav_visibility.visible_tags_count));
                shell.insert("nav_max_group_items_count".to_string(), serde_json::json!(nav_visibility.max_group_items_count));
                shell.insert("nav_visible_string_bytes_estimate_total".to_string(), serde_json::json!(nav_visibility.visible_string_bytes_estimate_total));
                shell.insert(
                    "theme_preset".to_string(),
                    opt_arc_str_json(theme_preset.as_ref()),
                );
                shell.insert("theme_preset_open".to_string(), serde_json::json!(theme_preset_open));
                shell.insert(
                    "motion_preset".to_string(),
                    opt_arc_str_json(motion_preset.as_ref()),
                );
                shell.insert(
                    "motion_preset_open".to_string(),
                    serde_json::json!(motion_preset_open),
                );
                shell.insert("theme_runtime".to_string(), theme_runtime_snapshot_json(app));
                shell.insert(
                    "window_metrics_preferences".to_string(),
                    window_metrics_preferences_snapshot_json(app, window),
                );
                shell.insert("cmdk_query_len_bytes".to_string(), serde_json::json!(cmdk_query.len() as u64));
                shell.insert("last_action_len_bytes".to_string(), serde_json::json!(last_action.len() as u64));
                shell.insert("last_action".to_string(), serde_json::json!(last_action.to_string()));
                shell.insert("text_input_len_bytes".to_string(), serde_json::json!(text_input.len() as u64));
                shell.insert("text_area_len_bytes".to_string(), serde_json::json!(text_area.len() as u64));
                shell.insert("input_file_value_len_bytes".to_string(), serde_json::json!(input_file_value.len() as u64));
                shell.insert("settings_open".to_string(), serde_json::json!(settings_open));
                shell.insert("settings_menu_bar_os_len_bytes".to_string(), serde_json::json!(opt_arc_str_len(settings_menu_bar_os.as_ref())));
                shell.insert("settings_menu_bar_in_window_len_bytes".to_string(), serde_json::json!(opt_arc_str_len(settings_menu_bar_in_window.as_ref())));
                shell.insert(
                    "settings_text_common_fallback_injection_len_bytes".to_string(),
                    serde_json::json!(opt_arc_str_len(
                        settings_text_common_fallback_injection.as_ref()
                    )),
                );
                shell.insert(
                    "settings_text_common_fallback_injection".to_string(),
                    settings_text_common_fallback_injection
                        .as_ref()
                        .map(|value| serde_json::Value::String(value.to_string()))
                        .unwrap_or(serde_json::Value::Null),
                );
                shell.insert("chrome_show_workspace_tab_strip".to_string(), serde_json::json!(chrome_show_workspace_tab_strip));
                if let Some(obj) = command_registry_string_bytes_estimate(app).as_object() {
                    for (k, v) in obj {
                        shell.insert(k.clone(), v.clone());
                    }
                }
                if let Some(obj) = page_specs_string_bytes_estimate().as_object() {
                    for (k, v) in obj {
                        shell.insert(k.clone(), v.clone());
                    }
                }
                if let Some(obj) = command_palette_entries_bytes_estimate(app).as_object() {
                    for (k, v) in obj {
                        shell.insert(k.clone(), v.clone());
                    }
                }
                if let Some(obj) = settings_sheet_static_string_bytes_estimate().as_object() {
                    for (k, v) in obj {
                        shell.insert(k.clone(), v.clone());
                    }
                }

                let mut out = serde_json::Map::new();
                out.insert("schema_version".to_string(), serde_json::json!(1));
                out.insert("kind".to_string(), serde_json::json!("fret_ui_gallery"));
                out.insert(
                    "selected_page".to_string(),
                    serde_json::Value::String(selected_page.to_string()),
                );
                #[cfg(feature = "gallery-dev")]
                if let Some(chart_torture) = chart_torture_snapshot_json(app, window) {
                    out.insert("chart_torture".to_string(), chart_torture);
                }
                out.insert(
                    "code_editor".to_string(),
                    serde_json::json!({
                        "syntax_rust": syntax_rust,
                        "text_boundary_mode": if boundary_identifier { "identifier" } else { "unicode_word" },
                        "soft_wrap_cols": if soft_wrap { Some(80u32) } else { None },
                        "folds_fixture": folds,
                        "inlays_fixture": inlays,
                        "torture": torture,
                        "markdown_editor_source": markdown_editor_source,
                    }),
                );
                out.insert(
                    "text_widgets".to_string(),
                    serde_json::json!({
                        "text_input_chars": text_input.chars().count(),
                        "text_area_chars": text_area.chars().count(),
                    }),
                );
                out.insert(
                    "view_cache".to_string(),
                    serde_json::json!({
                        "schema_version": 1,
                        "enabled": view_cache_enabled,
                        "cache_shell": view_cache_cache_shell,
                        "cache_content": view_cache_cache_content,
                        "inner_enabled": view_cache_inner_enabled,
                        "popover_open": view_cache_popover_open,
                        "continuous": view_cache_continuous,
                        "counter": view_cache_counter,
                    }),
                );
                out.insert("shell".to_string(), serde_json::Value::Object(shell));

                Some(serde_json::Value::Object(out))
            })));
        },
    );
}
