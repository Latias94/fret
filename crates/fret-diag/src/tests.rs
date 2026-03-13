use super::*;
use crate::compare::compare_bundles_json;
use crate::stats::{
    bundle_stats_from_json_with_options, check_bundle_for_chart_sampling_window_shifts_min,
    check_bundle_for_dock_drag_min_json, check_bundle_for_gc_sweep_liveness,
    check_bundle_for_idle_no_paint_min, check_bundle_for_layout_fast_path_min,
    check_bundle_for_node_graph_cull_window_shifts_max,
    check_bundle_for_node_graph_cull_window_shifts_min, check_bundle_for_notify_hotspot_file_max,
    check_bundle_for_overlay_synthesis_min_json, check_bundle_for_prepaint_actions_min,
    check_bundle_for_retained_vlist_attach_detach_max_json,
    check_bundle_for_retained_vlist_keep_alive_budget_json,
    check_bundle_for_retained_vlist_reconcile_no_notify_min_json,
    check_bundle_for_semantics_changed_repainted_json, check_bundle_for_stale_scene_json,
    check_bundle_for_ui_gallery_code_editor_a11y_composition_drag_json,
    check_bundle_for_ui_gallery_code_editor_a11y_composition_json,
    check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_json,
    check_bundle_for_ui_gallery_code_editor_a11y_selection_json,
    check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap_json,
    check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection_json,
    check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll_json,
    check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed,
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_json,
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_json,
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_json,
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_json,
    check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo_json,
    check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits_json,
    check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_json,
    check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap_json,
    check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits_json,
    check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds_json,
    check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable_json,
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable_json,
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable_json,
    check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click_json,
    check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json,
    check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_json,
    check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable_json,
    check_bundle_for_ui_gallery_markdown_editor_source_word_boundary_json,
    check_bundle_for_ui_gallery_web_ime_bridge_enabled_json,
    check_bundle_for_view_cache_reuse_min_json, check_bundle_for_view_cache_reuse_stable_min,
    check_bundle_for_viewport_capture_min_json, check_bundle_for_viewport_input_min_json,
    check_bundle_for_vlist_policy_key_stable, check_bundle_for_vlist_window_shifts_explainable,
    check_bundle_for_vlist_window_shifts_have_prepaint_actions,
    check_bundle_for_vlist_window_shifts_kind_max,
    check_bundle_for_vlist_window_shifts_non_retained_max,
    check_bundle_for_wheel_scroll_hit_changes_json,
    check_bundle_for_windowed_rows_offset_changes_min,
    check_bundle_for_windowed_rows_visible_start_changes_repainted_json,
    scan_semantics_changed_repainted_json,
};
use fret_diag_protocol::{DevtoolsSessionDescriptorV1, DevtoolsSessionListV1};
use serde_json::json;
use std::path::Path;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use util::json_pointer_set;

#[test]
fn resolve_bundle_artifact_path_prefers_run_id_dir_from_script_result() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-resolve-bundle-run-id-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let run_id_dir = root.join("777");
    std::fs::create_dir_all(&run_id_dir).expect("create run_id dir");
    std::fs::write(
        run_id_dir.join("bundle.json"),
        br#"{"schema_version":1,"windows":[]}"#,
    )
    .expect("write bundle.json");

    std::fs::write(root.join("script.result.json"), br#"{"run_id":777}"#)
        .expect("write script.result.json");

    let resolved = resolve_bundle_artifact_path(&root);
    assert_eq!(resolved, run_id_dir.join("bundle.json"));
}

#[test]
fn resolve_bundle_artifact_path_prefers_run_id_schema2_from_script_result() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-resolve-bundle-run-id-schema2-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let run_id_dir = root.join("777");
    std::fs::create_dir_all(&run_id_dir).expect("create run_id dir");
    std::fs::write(
        run_id_dir.join("bundle.schema2.json"),
        br#"{"schema_version":2}"#,
    )
    .expect("write bundle.schema2.json");

    std::fs::write(root.join("script.result.json"), br#"{"run_id":777}"#)
        .expect("write script.result.json");

    let resolved = resolve_bundle_artifact_path(&root);
    assert_eq!(resolved, run_id_dir.join("bundle.schema2.json"));
}

#[test]
fn resolve_bundle_artifact_path_records_integrity_failure_reason_code_on_chunk_hash_mismatch() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-resolve-bundle-chunks-integrity-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let run_id = 1u64;
    let run_id_dir = root.join(run_id.to_string());
    std::fs::create_dir_all(&run_id_dir).expect("create run_id dir");

    let initial = UiScriptResultV1 {
        schema_version: 1,
        run_id,
        updated_unix_ms: 0,
        window: None,
        stage: UiScriptStageV1::Passed,
        step_index: None,
        reason_code: None,
        reason: None,
        evidence: None,
        last_bundle_dir: None,
        last_bundle_artifact: None,
    };
    std::fs::write(
        run_id_dir.join("script.result.json"),
        serde_json::to_vec_pretty(&initial).expect("script.result json"),
    )
    .expect("write script.result.json");

    let chunks_dir = run_id_dir.join("chunks").join("bundle_json");
    std::fs::create_dir_all(&chunks_dir).expect("create chunks dir");
    let chunk_path = chunks_dir.join("chunk-000000");
    let chunk_bytes = br#"{ "schema_version": 1, "windows": [] }"#.to_vec();
    std::fs::write(&chunk_path, &chunk_bytes).expect("write chunk");

    // Intentionally wrong hash values to force an integrity failure.
    let manifest = serde_json::json!({
        "schema_version": 2,
        "generated_unix_ms": 0,
        "run_id": run_id,
        "bundle_json": {
            "mode": "chunks.v1",
            "total_bytes": chunk_bytes.len() as u64,
            "chunk_bytes": chunk_bytes.len() as u64,
            "blake3": "deadbeef",
            "chunks": [
                {
                    "index": 0,
                    "path": "chunks/bundle_json/chunk-000000",
                    "bytes": chunk_bytes.len() as u64,
                    "blake3": "deadbeef",
                }
            ]
        }
    });
    std::fs::write(
        run_id_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("manifest json"),
    )
    .expect("write manifest.json");

    let _ = resolve_bundle_artifact_path(&run_id_dir);

    let bytes =
        std::fs::read(run_id_dir.join("script.result.json")).expect("read script.result.json");
    let parsed: UiScriptResultV1 =
        serde_json::from_slice(&bytes).expect("parse script.result.json");
    assert!(matches!(parsed.stage, UiScriptStageV1::Failed));
    assert_eq!(
        parsed.reason_code.as_deref(),
        Some("tooling.artifact.integrity.failed")
    );
    assert!(
        parsed
            .evidence
            .as_ref()
            .and_then(|e| e.event_log.last())
            .map(|e| e.kind.as_str())
            == Some("tooling_artifact_integrity_failed")
    );
}

#[test]
fn materialize_devtools_bundle_dumped_embedded_writes_bundle_json_and_latest() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-devtools-dumped-embedded-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let dumped = DevtoolsBundleDumpedV1 {
        schema_version: 1,
        exported_unix_ms: 1,
        out_dir: root.to_string_lossy().to_string(),
        dir: "123-embedded".to_string(),
        bundle: Some(json!({
            "schema_version": 1,
            "windows": [],
        })),
        bundle_json_chunk: None,
        bundle_json_chunk_index: None,
        bundle_json_chunk_count: None,
    };

    let bundle_path =
        materialize_devtools_bundle_dumped(&root, &dumped).expect("materialize dumped");
    assert!(bundle_path.is_file());

    let bytes = std::fs::read(&bundle_path).expect("read bundle.json");
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("parse bundle.json");
    assert_eq!(
        parsed.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );

    let latest = std::fs::read_to_string(root.join("latest.txt"))
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    assert_eq!(latest, "123-embedded");
}

#[test]
fn materialize_devtools_bundle_dumped_falls_back_to_runtime_bundle_json() {
    let runtime_root = std::env::temp_dir().join(format!(
        "fret-diag-devtools-dumped-runtime-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    let local_root = std::env::temp_dir().join(format!(
        "fret-diag-devtools-dumped-local-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    let _ = std::fs::remove_dir_all(&runtime_root);
    let _ = std::fs::remove_dir_all(&local_root);
    std::fs::create_dir_all(&runtime_root).expect("create runtime root");
    std::fs::create_dir_all(&local_root).expect("create local root");

    let runtime_dir = runtime_root.join("456-runtime");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime dir");
    std::fs::write(
        runtime_dir.join("bundle.json"),
        br#"{ "schema_version": 1, "windows": [ { "window": 1 } ] }"#,
    )
    .expect("write runtime bundle.json");

    let dumped = DevtoolsBundleDumpedV1 {
        schema_version: 1,
        exported_unix_ms: 1,
        out_dir: runtime_root.to_string_lossy().to_string(),
        dir: "456-runtime".to_string(),
        bundle: None,
        bundle_json_chunk: None,
        bundle_json_chunk_index: None,
        bundle_json_chunk_count: None,
    };

    let bundle_path =
        materialize_devtools_bundle_dumped(&local_root, &dumped).expect("materialize dumped");
    assert!(bundle_path.is_file());

    let bytes = std::fs::read(&bundle_path).expect("read bundle.json");
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("parse bundle.json");
    assert_eq!(
        parsed.get("schema_version").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(parsed.get("windows").is_some());

    let dumped_path = local_root.join("456-runtime").join("bundle.dumped.json");
    assert!(dumped_path.is_file());
}

#[test]
fn run_script_over_transport_streams_incremental_script_result_updates() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-script-stream-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
        schema_version: 1,
        capabilities: vec!["script_v2".to_string()],
        runner_kind: None,
        runner_version: None,
        hints: None,
    };
    crate::util::write_json_value(
        &root.join("capabilities.json"),
        &serde_json::to_value(caps).expect("capabilities json"),
    )
    .expect("write capabilities.json");

    let ready_path = root.join("ready.touch");

    let cfg = crate::transport::FsDiagTransportConfig {
        out_dir: root.clone(),
        trigger_path: root.join("trigger.touch"),
        script_path: root.join("runtime.script.json"),
        script_trigger_path: root.join("runtime.script.touch"),
        script_result_path: root.join("runtime.script.result.json"),
        script_result_trigger_path: root.join("runtime.script.result.touch"),
        pick_trigger_path: root.join("pick.touch"),
        pick_result_path: root.join("pick.result.json"),
        pick_result_trigger_path: root.join("pick.result.touch"),
        inspect_path: root.join("inspect.json"),
        inspect_trigger_path: root.join("inspect.touch"),
        screenshots_request_path: root.join("screenshots.request.json"),
        screenshots_trigger_path: root.join("screenshots.touch"),
        screenshots_result_path: root.join("screenshots.result.json"),
        screenshots_result_trigger_path: root.join("screenshots.result.touch"),
    };

    let runtime_cfg = cfg.clone();
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if runtime_cfg.script_trigger_path.is_file() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        let running = fret_diag_protocol::UiScriptResultV1 {
            schema_version: 1,
            run_id: 1,
            updated_unix_ms: crate::util::now_unix_ms(),
            window: None,
            stage: fret_diag_protocol::UiScriptStageV1::Running,
            step_index: Some(0),
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        let _ = crate::util::write_json_value(
            &runtime_cfg.script_result_path,
            &serde_json::to_value(running).unwrap_or_else(|_| serde_json::json!({})),
        );
        let _ = crate::util::touch(&runtime_cfg.script_result_trigger_path);

        std::thread::sleep(Duration::from_millis(250));

        let passed = fret_diag_protocol::UiScriptResultV1 {
            schema_version: 1,
            run_id: 1,
            updated_unix_ms: crate::util::now_unix_ms(),
            window: None,
            stage: fret_diag_protocol::UiScriptStageV1::Passed,
            step_index: Some(0),
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        let _ = crate::util::write_json_value(
            &runtime_cfg.script_result_path,
            &serde_json::to_value(passed).unwrap_or_else(|_| serde_json::json!({})),
        );
        let _ = crate::util::touch(&runtime_cfg.script_result_trigger_path);
    });

    let tool_script_result_path = root.join("tool.script.result.json");
    let capabilities_check_path = root.join("check.capabilities.json");

    let runner_root = root.clone();
    let runner_cfg = cfg.clone();
    let runner_ready_path = ready_path.clone();
    let runner_tool_path = tool_script_result_path.clone();
    let runner_check_path = capabilities_check_path.clone();
    let handle = std::thread::spawn(move || {
        let connected =
            connect_filesystem_tooling(&runner_cfg, &runner_ready_path, false, 5_000, 5)
                .expect("connect fs tooling");
        let script_json = serde_json::json!({
            "schema_version": 2,
            "steps": [],
        });
        let (result, _bundle_path) = run_script_over_transport(
            &runner_root,
            &connected,
            script_json,
            false,
            false,
            None,
            None,
            5_000,
            5,
            &runner_tool_path,
            &runner_check_path,
        )
        .expect("run_script_over_transport");
        result
    });

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut saw_running = false;
    while Instant::now() < deadline {
        if let Some(v) = crate::util::read_json_value(&tool_script_result_path)
            && v.get("stage").and_then(|v| v.as_str()) == Some("running")
        {
            saw_running = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(saw_running, "expected streamed stage=running update");

    let final_result = handle.join().expect("join run thread");
    assert!(matches!(
        final_result.stage,
        fret_diag_protocol::UiScriptStageV1::Passed
    ));

    let bytes = std::fs::read(root.join("1").join("script.result.json"))
        .expect("read run_id script.result.json");
    let v: serde_json::Value =
        serde_json::from_slice(&bytes).expect("parse run_id script.result.json");
    assert_eq!(v.get("run_id").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(v.get("stage").and_then(|v| v.as_str()), Some("passed"));

    let bytes = std::fs::read(root.join("1").join("manifest.json")).expect("read manifest.json");
    let v: serde_json::Value = serde_json::from_slice(&bytes).expect("parse manifest.json");
    assert_eq!(v.get("run_id").and_then(|v| v.as_u64()), Some(1));
    assert_eq!(
        v.get("script_result")
            .and_then(|v| v.get("stage"))
            .and_then(|v| v.as_str()),
        Some("passed")
    );
}

#[test]
fn run_script_over_transport_reuses_existing_filesystem_bundle_after_pass() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-script-existing-bundle-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
        schema_version: 1,
        capabilities: vec!["script_v2".to_string()],
        runner_kind: None,
        runner_version: None,
        hints: None,
    };
    crate::util::write_json_value(
        &root.join("capabilities.json"),
        &serde_json::to_value(caps).expect("capabilities json"),
    )
    .expect("write capabilities.json");

    let cfg = crate::transport::FsDiagTransportConfig {
        out_dir: root.clone(),
        trigger_path: root.join("trigger.touch"),
        script_path: root.join("runtime.script.json"),
        script_trigger_path: root.join("runtime.script.touch"),
        script_result_path: root.join("runtime.script.result.json"),
        script_result_trigger_path: root.join("runtime.script.result.touch"),
        pick_trigger_path: root.join("pick.touch"),
        pick_result_path: root.join("pick.result.json"),
        pick_result_trigger_path: root.join("pick.result.touch"),
        inspect_path: root.join("inspect.json"),
        inspect_trigger_path: root.join("inspect.touch"),
        screenshots_request_path: root.join("screenshots.request.json"),
        screenshots_trigger_path: root.join("screenshots.touch"),
        screenshots_result_path: root.join("screenshots.result.json"),
        screenshots_result_trigger_path: root.join("screenshots.result.touch"),
    };

    let runtime_cfg = cfg.clone();
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if runtime_cfg.script_trigger_path.is_file() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        let bundle_dir_name = "captured-bundle";
        let bundle_dir = runtime_cfg.out_dir.join(bundle_dir_name);
        let _ = std::fs::create_dir_all(&bundle_dir);
        let _ = std::fs::write(
            bundle_dir.join("bundle.schema2.json"),
            br#"{ "schema_version": 2, "windows": [] }"#,
        );

        let passed = fret_diag_protocol::UiScriptResultV1 {
            schema_version: 1,
            run_id: 7,
            updated_unix_ms: crate::util::now_unix_ms(),
            window: None,
            stage: fret_diag_protocol::UiScriptStageV1::Passed,
            step_index: Some(0),
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: Some(bundle_dir_name.to_string()),
            last_bundle_artifact: Some(fret_diag_protocol::UiArtifactStatsV1 {
                schema_version: 1,
                bundle_json_bytes: Some(37),
                window_count: 0,
                event_count: 0,
                snapshot_count: 0,
                max_snapshots: 0,
                dump_max_snapshots: None,
            }),
        };
        let _ = crate::util::write_json_value(
            &runtime_cfg.script_result_path,
            &serde_json::to_value(passed).unwrap_or_else(|_| serde_json::json!({})),
        );
        let _ = crate::util::touch(&runtime_cfg.script_result_trigger_path);
    });

    let connected = connect_filesystem_tooling(&cfg, &root.join("ready.touch"), false, 5_000, 5)
        .expect("connect fs tooling");

    let tool_script_result_path = root.join("tool.script.result.json");
    let capabilities_check_path = root.join("check.capabilities.json");
    let script_json = serde_json::json!({
        "schema_version": 2,
        "steps": [],
    });

    let (result, bundle_path) = run_script_over_transport(
        &root,
        &connected,
        script_json,
        true,
        false,
        None,
        None,
        750,
        5,
        &tool_script_result_path,
        &capabilities_check_path,
    )
    .expect("run_script_over_transport");

    assert!(matches!(
        result.stage,
        fret_diag_protocol::UiScriptStageV1::Passed
    ));
    assert_eq!(result.last_bundle_dir.as_deref(), Some("captured-bundle"));
    assert_eq!(
        bundle_path,
        Some(root.join("captured-bundle").join("bundle.schema2.json"))
    );
    assert!(root.join("7").join("bundle.schema2.json").is_file());
}

#[test]
fn diag_poke_wait_record_run_writes_run_id_manifest() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let root = std::env::temp_dir().join(format!(
        "fret-diag-poke-record-run-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let out_dir = root.as_path();
    let trigger_path = root.join("trigger.touch");

    let runtime_root = root.clone();
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if trigger_path.is_file() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        let export_dir_name = "1234567890-manual-poke-test";
        let export_dir = runtime_root.join(export_dir_name);
        let _ = std::fs::create_dir_all(&export_dir);
        let _ = std::fs::write(
            export_dir.join("bundle.json"),
            br#"{ "schema_version": 1, "windows": [], "config": { "max_snapshots": 0 } }"#,
        );
        let _ = std::fs::write(runtime_root.join("latest.txt"), export_dir_name);
    });

    let rest = vec![
        "--wait".to_string(),
        "--record-run".to_string(),
        "--run-id".to_string(),
        "42".to_string(),
    ];
    crate::commands::session::cmd_poke(
        &rest,
        false,
        out_dir,
        &root.join("trigger.touch"),
        5_000,
        5,
    )
    .expect("cmd_poke");

    let run_dir = root.join("42");
    let bytes = std::fs::read(run_dir.join("manifest.json")).expect("read manifest.json");
    let v: serde_json::Value = serde_json::from_slice(&bytes).expect("parse manifest.json");
    assert_eq!(v.get("run_id").and_then(|v| v.as_u64()), Some(42));
    assert_eq!(
        v.get("script_result")
            .and_then(|v| v.get("stage"))
            .and_then(|v| v.as_str()),
        Some("passed")
    );
    assert_eq!(
        v.get("script_result")
            .and_then(|v| v.get("reason_code"))
            .and_then(|v| v.as_str()),
        Some("manual.poke.dump")
    );
    assert!(run_dir.join("bundle.json").is_file());
}

#[test]
fn run_script_over_transport_timeout_writes_failed_tool_script_result() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-script-timeout-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
        schema_version: 1,
        capabilities: vec!["script_v2".to_string()],
        runner_kind: None,
        runner_version: None,
        hints: None,
    };
    crate::util::write_json_value(
        &root.join("capabilities.json"),
        &serde_json::to_value(caps).expect("capabilities json"),
    )
    .expect("write capabilities.json");

    let cfg = crate::transport::FsDiagTransportConfig {
        out_dir: root.clone(),
        trigger_path: root.join("trigger.touch"),
        script_path: root.join("runtime.script.json"),
        script_trigger_path: root.join("runtime.script.touch"),
        script_result_path: root.join("runtime.script.result.json"),
        script_result_trigger_path: root.join("runtime.script.result.touch"),
        pick_trigger_path: root.join("pick.touch"),
        pick_result_path: root.join("pick.result.json"),
        pick_result_trigger_path: root.join("pick.result.touch"),
        inspect_path: root.join("inspect.json"),
        inspect_trigger_path: root.join("inspect.touch"),
        screenshots_request_path: root.join("screenshots.request.json"),
        screenshots_trigger_path: root.join("screenshots.touch"),
        screenshots_result_path: root.join("screenshots.result.json"),
        screenshots_result_trigger_path: root.join("screenshots.result.touch"),
    };

    let connected = connect_filesystem_tooling(&cfg, &root.join("ready.touch"), false, 5_000, 5)
        .expect("connect fs tooling");

    let tool_script_result_path = root.join("tool.script.result.json");
    let capabilities_check_path = root.join("check.capabilities.json");
    let script_json = serde_json::json!({
        "schema_version": 2,
        "steps": [],
    });

    let err = run_script_over_transport(
        &root,
        &connected,
        script_json,
        false,
        false,
        None,
        None,
        200,
        5,
        &tool_script_result_path,
        &capabilities_check_path,
    )
    .unwrap_err();
    assert!(err.contains("timeout waiting for script result"));

    let bytes = std::fs::read(&tool_script_result_path).expect("read tool script.result.json");
    let parsed: fret_diag_protocol::UiScriptResultV1 =
        serde_json::from_slice(&bytes).expect("parse tool script.result.json");
    assert!(matches!(
        parsed.stage,
        fret_diag_protocol::UiScriptStageV1::Failed
    ));
    assert_eq!(
        parsed.reason_code.as_deref(),
        Some("timeout.tooling.script_result")
    );
    assert!(
        parsed
            .evidence
            .as_ref()
            .and_then(|e| e.event_log.first())
            .map(|e| e.kind.as_str())
            == Some("tooling_timeout")
    );
}

#[test]
fn write_tooling_failure_script_result_overwrites_existing_reason_code() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-tooling-failure-overwrite-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let path = root.join("script.result.json");
    write_tooling_failure_script_result(
        &path,
        "tooling.old",
        "old failure",
        "tooling_error",
        Some("old".to_string()),
    );
    write_tooling_failure_script_result(
        &path,
        "tooling.new",
        "new failure",
        "tooling_error",
        Some("new".to_string()),
    );

    let bytes = std::fs::read(&path).expect("read script.result.json");
    let parsed: fret_diag_protocol::UiScriptResultV1 =
        serde_json::from_slice(&bytes).expect("parse script.result.json");
    assert_eq!(parsed.reason_code.as_deref(), Some("tooling.new"));
    assert_eq!(parsed.reason.as_deref(), Some("new failure"));
}

#[test]
fn run_script_over_transport_retouches_in_filesystem_mode_to_avoid_baseline_race() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-script-retouch-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
        schema_version: 1,
        capabilities: vec!["script_v2".to_string()],
        runner_kind: None,
        runner_version: None,
        hints: None,
    };
    crate::util::write_json_value(
        &root.join("capabilities.json"),
        &serde_json::to_value(caps).expect("capabilities json"),
    )
    .expect("write capabilities.json");

    let cfg = crate::transport::FsDiagTransportConfig {
        out_dir: root.clone(),
        trigger_path: root.join("trigger.touch"),
        script_path: root.join("runtime.script.json"),
        script_trigger_path: root.join("runtime.script.touch"),
        script_result_path: root.join("runtime.script.result.json"),
        script_result_trigger_path: root.join("runtime.script.result.touch"),
        pick_trigger_path: root.join("pick.touch"),
        pick_result_path: root.join("pick.result.json"),
        pick_result_trigger_path: root.join("pick.result.touch"),
        inspect_path: root.join("inspect.json"),
        inspect_trigger_path: root.join("inspect.touch"),
        screenshots_request_path: root.join("screenshots.request.json"),
        screenshots_trigger_path: root.join("screenshots.touch"),
        screenshots_result_path: root.join("screenshots.result.json"),
        screenshots_result_trigger_path: root.join("screenshots.result.touch"),
    };

    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let saw_retouch = Arc::new(AtomicBool::new(false));

    let runtime_cfg = cfg.clone();
    let runtime_saw_retouch = saw_retouch.clone();
    std::thread::spawn(move || {
        fn read_stamp(path: &Path) -> Option<u64> {
            let s = std::fs::read_to_string(path).ok()?;
            s.lines().last()?.trim().parse::<u64>().ok()
        }

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut first_stamp: Option<u64> = None;
        while Instant::now() < deadline {
            let Some(stamp) = read_stamp(&runtime_cfg.script_trigger_path) else {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            };
            match first_stamp {
                None => first_stamp = Some(stamp),
                Some(prev) if stamp > prev => {
                    runtime_saw_retouch.store(true, Ordering::Relaxed);
                    break;
                }
                _ => {}
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        if !runtime_saw_retouch.load(Ordering::Relaxed) {
            return;
        }

        let passed = fret_diag_protocol::UiScriptResultV1 {
            schema_version: 1,
            run_id: 1,
            updated_unix_ms: crate::util::now_unix_ms(),
            window: None,
            stage: fret_diag_protocol::UiScriptStageV1::Passed,
            step_index: Some(0),
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        let _ = crate::util::write_json_value(
            &runtime_cfg.script_result_path,
            &serde_json::to_value(passed).unwrap_or_else(|_| serde_json::json!({})),
        );
        let _ = crate::util::touch(&runtime_cfg.script_result_trigger_path);
    });

    let connected = connect_filesystem_tooling(&cfg, &root.join("ready.touch"), false, 5_000, 5)
        .expect("connect fs tooling");

    let tool_script_result_path = root.join("tool.script.result.json");
    let capabilities_check_path = root.join("check.capabilities.json");
    let script_json = serde_json::json!({
        "schema_version": 2,
        "steps": [],
    });

    let (result, _bundle_path) = run_script_over_transport(
        &root,
        &connected,
        script_json,
        false,
        false,
        None,
        None,
        5_000,
        5,
        &tool_script_result_path,
        &capabilities_check_path,
    )
    .expect("run_script_over_transport");

    assert!(matches!(
        result.stage,
        fret_diag_protocol::UiScriptStageV1::Passed
    ));
    assert!(
        saw_retouch.load(Ordering::Relaxed),
        "expected tooling retouch to advance script stamp"
    );
}

#[test]
fn run_script_over_transport_prefers_newer_run_id_after_retouch() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-script-retouch-supersede-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
        schema_version: 1,
        capabilities: vec!["script_v2".to_string()],
        runner_kind: None,
        runner_version: None,
        hints: None,
    };
    crate::util::write_json_value(
        &root.join("capabilities.json"),
        &serde_json::to_value(caps).expect("capabilities json"),
    )
    .expect("write capabilities.json");

    let cfg = crate::transport::FsDiagTransportConfig {
        out_dir: root.clone(),
        trigger_path: root.join("trigger.touch"),
        script_path: root.join("runtime.script.json"),
        script_trigger_path: root.join("runtime.script.touch"),
        script_result_path: root.join("runtime.script.result.json"),
        script_result_trigger_path: root.join("runtime.script.result.touch"),
        pick_trigger_path: root.join("pick.touch"),
        pick_result_path: root.join("pick.result.json"),
        pick_result_trigger_path: root.join("pick.result.touch"),
        inspect_path: root.join("inspect.json"),
        inspect_trigger_path: root.join("inspect.touch"),
        screenshots_request_path: root.join("screenshots.request.json"),
        screenshots_trigger_path: root.join("screenshots.touch"),
        screenshots_result_path: root.join("screenshots.result.json"),
        screenshots_result_trigger_path: root.join("screenshots.result.touch"),
    };

    let runtime_cfg = cfg.clone();
    std::thread::spawn(move || {
        fn read_stamp(path: &Path) -> Option<u64> {
            let s = std::fs::read_to_string(path).ok()?;
            s.lines().last()?.trim().parse::<u64>().ok()
        }

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut first_stamp: Option<u64> = None;
        while Instant::now() < deadline {
            let Some(stamp) = read_stamp(&runtime_cfg.script_trigger_path) else {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            };

            match first_stamp {
                None => first_stamp = Some(stamp),
                Some(prev) if stamp > prev => {
                    let running = fret_diag_protocol::UiScriptResultV1 {
                        schema_version: 1,
                        run_id: 1,
                        updated_unix_ms: crate::util::now_unix_ms(),
                        window: None,
                        stage: fret_diag_protocol::UiScriptStageV1::Running,
                        step_index: Some(0),
                        reason_code: None,
                        reason: None,
                        evidence: None,
                        last_bundle_dir: None,
                        last_bundle_artifact: None,
                    };
                    let _ = crate::util::write_json_value(
                        &runtime_cfg.script_result_path,
                        &serde_json::to_value(running).unwrap_or_else(|_| serde_json::json!({})),
                    );
                    let _ = crate::util::touch(&runtime_cfg.script_result_trigger_path);

                    std::thread::sleep(Duration::from_millis(25));

                    let passed = fret_diag_protocol::UiScriptResultV1 {
                        schema_version: 1,
                        run_id: 2,
                        updated_unix_ms: crate::util::now_unix_ms(),
                        window: None,
                        stage: fret_diag_protocol::UiScriptStageV1::Passed,
                        step_index: Some(0),
                        reason_code: None,
                        reason: None,
                        evidence: None,
                        last_bundle_dir: None,
                        last_bundle_artifact: None,
                    };
                    let _ = crate::util::write_json_value(
                        &runtime_cfg.script_result_path,
                        &serde_json::to_value(passed).unwrap_or_else(|_| serde_json::json!({})),
                    );
                    let _ = crate::util::touch(&runtime_cfg.script_result_trigger_path);
                    break;
                }
                _ => {}
            }

            std::thread::sleep(Duration::from_millis(5));
        }
    });

    let connected = connect_filesystem_tooling(&cfg, &root.join("ready.touch"), false, 5_000, 5)
        .expect("connect fs tooling");

    let tool_script_result_path = root.join("tool.script.result.json");
    let capabilities_check_path = root.join("check.capabilities.json");
    let script_json = serde_json::json!({
        "schema_version": 2,
        "steps": [],
    });

    let (result, _bundle_path) = run_script_over_transport(
        &root,
        &connected,
        script_json,
        false,
        false,
        None,
        None,
        5_000,
        5,
        &tool_script_result_path,
        &capabilities_check_path,
    )
    .expect("run_script_over_transport");

    assert_eq!(result.run_id, 2, "expected newer superseding run_id");
    assert!(matches!(
        result.stage,
        fret_diag_protocol::UiScriptStageV1::Passed
    ));
    assert!(root.join("2").join("script.result.json").is_file());
}

#[test]
fn run_script_and_wait_prefers_newer_run_id_after_retouch() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-script-runtime-supersede-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let src = root.join("script.json");
    std::fs::write(
        &src,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": 2,
            "steps": [],
        }))
        .expect("serialize script"),
    )
    .expect("write script");

    let script_path = root.join("runtime.script.json");
    let script_trigger_path = root.join("runtime.script.touch");
    let script_result_path = root.join("runtime.script.result.json");
    let script_result_trigger_path = root.join("runtime.script.result.touch");

    let runtime_script_trigger_path = script_trigger_path.clone();
    let runtime_script_result_path = script_result_path.clone();
    let runtime_script_result_trigger_path = script_result_trigger_path.clone();
    std::thread::spawn(move || {
        fn read_stamp(path: &Path) -> Option<u64> {
            let s = std::fs::read_to_string(path).ok()?;
            s.lines().last()?.trim().parse::<u64>().ok()
        }

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut first_stamp: Option<u64> = None;
        while Instant::now() < deadline {
            let Some(stamp) = read_stamp(&runtime_script_trigger_path) else {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            };

            match first_stamp {
                None => first_stamp = Some(stamp),
                Some(prev) if stamp > prev => {
                    let running = serde_json::json!({
                        "schema_version": 1,
                        "run_id": 1,
                        "updated_unix_ms": crate::util::now_unix_ms(),
                        "stage": "running",
                        "step_index": 0
                    });
                    let _ = crate::util::write_json_value(&runtime_script_result_path, &running);
                    let _ = crate::util::touch(&runtime_script_result_trigger_path);

                    std::thread::sleep(Duration::from_millis(25));

                    let passed = serde_json::json!({
                        "schema_version": 1,
                        "run_id": 2,
                        "updated_unix_ms": crate::util::now_unix_ms(),
                        "stage": "passed",
                        "step_index": 0
                    });
                    let _ = crate::util::write_json_value(&runtime_script_result_path, &passed);
                    let _ = crate::util::touch(&runtime_script_result_trigger_path);
                    break;
                }
                _ => {}
            }

            std::thread::sleep(Duration::from_millis(5));
        }
    });

    let result = crate::stats::run_script_and_wait(
        &src,
        &script_path,
        &script_trigger_path,
        &script_result_path,
        &script_result_trigger_path,
        5_000,
        5,
    )
    .expect("run_script_and_wait");

    assert_eq!(result.run_id, 2, "expected newer superseding run_id");
    assert_eq!(result.stage.as_deref(), Some("passed"));
}

#[test]
fn dump_bundle_over_transport_materializes_filesystem_latest_pointer() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-bundle-dump-fs-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
        schema_version: 1,
        capabilities: vec!["script_v2".to_string()],
        runner_kind: None,
        runner_version: None,
        hints: None,
    };
    crate::util::write_json_value(
        &root.join("capabilities.json"),
        &serde_json::to_value(caps).expect("capabilities json"),
    )
    .expect("write capabilities.json");

    let latest_dir = "123-latest";
    let export_dir = root.join(latest_dir);
    std::fs::create_dir_all(&export_dir).expect("create export dir");
    std::fs::write(root.join("latest.txt"), latest_dir.as_bytes()).expect("write latest.txt");
    crate::util::write_json_value(
        &export_dir.join("bundle.json"),
        &serde_json::json!({
            "schema_version": 1,
            "windows": [],
        }),
    )
    .expect("write bundle.json");

    let cfg = crate::transport::FsDiagTransportConfig::from_out_dir(&root);
    let connected = connect_filesystem_tooling(&cfg, &root.join("ready.touch"), false, 2_000, 5)
        .expect("connect fs tooling");

    let bundle_path = dump_bundle_over_transport(&root, &connected, Some("test"), None, 2_000, 5)
        .expect("dump bundle");
    assert!(bundle_path.is_file());
    assert_eq!(
        bundle_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str()),
        Some(latest_dir)
    );
}

#[test]
fn script_run_fs_transport_cfg_overrides_script_paths_but_keeps_out_dir_artifacts() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-script-run-fs-transport-cfg-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let out_dir = root.join("diag-out");
    std::fs::create_dir_all(&out_dir).expect("create out dir");

    let script_path = root.join("runtime.script.json");
    let script_trigger_path = root.join("runtime.script.touch");
    let script_result_path = root.join("runtime.script.result.json");
    let script_result_trigger_path = root.join("runtime.script.result.touch");

    let cfg = crate::script_run_fs_transport_cfg(
        &out_dir,
        &script_path,
        &script_trigger_path,
        &script_result_path,
        &script_result_trigger_path,
    );

    assert_eq!(cfg.out_dir, out_dir);
    assert_eq!(
        cfg.trigger_path,
        root.join("diag-out").join("trigger.touch")
    );
    assert_eq!(cfg.script_path, script_path);
    assert_eq!(cfg.script_trigger_path, script_trigger_path);
    assert_eq!(cfg.script_result_path, script_result_path);
    assert_eq!(cfg.script_result_trigger_path, script_result_trigger_path);
    assert_eq!(
        cfg.pick_trigger_path,
        root.join("diag-out").join("pick.touch")
    );
}

#[test]
fn script_result_fs_transport_cfg_overrides_result_paths_only() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-script-result-fs-transport-cfg-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let out_dir = root.join("diag-out");
    std::fs::create_dir_all(&out_dir).expect("create out dir");

    let script_result_path = root.join("suite.script.result.json");
    let script_result_trigger_path = root.join("suite.script.result.touch");

    let cfg = crate::script_result_fs_transport_cfg(
        &out_dir,
        &script_result_path,
        &script_result_trigger_path,
    );

    assert_eq!(cfg.out_dir, out_dir);
    assert_eq!(cfg.script_result_path, script_result_path);
    assert_eq!(cfg.script_result_trigger_path, script_result_trigger_path);
    assert_eq!(cfg.script_path, root.join("diag-out").join("script.json"));
    assert_eq!(
        cfg.script_trigger_path,
        root.join("diag-out").join("script.touch")
    );
    assert_eq!(
        cfg.pick_result_path,
        root.join("diag-out").join("pick.result.json")
    );
}

#[test]
fn run_script_over_transport_dump_bundle_writes_run_id_bundle_json() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-run-dump-runid-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
        schema_version: 1,
        capabilities: vec!["script_v2".to_string()],
        runner_kind: None,
        runner_version: None,
        hints: None,
    };
    crate::util::write_json_value(
        &root.join("capabilities.json"),
        &serde_json::to_value(caps).expect("capabilities json"),
    )
    .expect("write capabilities.json");

    let cfg = crate::transport::FsDiagTransportConfig::from_out_dir(&root);

    let runtime_cfg = cfg.clone();
    std::thread::spawn(move || {
        fn read_stamp(path: &Path) -> Option<u64> {
            let s = std::fs::read_to_string(path).ok()?;
            s.lines().last()?.trim().parse::<u64>().ok()
        }

        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if read_stamp(&runtime_cfg.script_trigger_path).is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        let passed = fret_diag_protocol::UiScriptResultV1 {
            schema_version: 1,
            run_id: 1,
            updated_unix_ms: crate::util::now_unix_ms(),
            window: None,
            stage: fret_diag_protocol::UiScriptStageV1::Passed,
            step_index: Some(0),
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        let _ = crate::util::write_json_value(
            &runtime_cfg.script_result_path,
            &serde_json::to_value(passed).unwrap_or_else(|_| serde_json::json!({})),
        );
        let _ = crate::util::touch(&runtime_cfg.script_result_trigger_path);

        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if read_stamp(&runtime_cfg.trigger_path).is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        let export_dir = runtime_cfg.out_dir.join("777-bundle");
        let _ = std::fs::create_dir_all(&export_dir);
        let _ = crate::util::write_json_value(
            &export_dir.join("bundle.json"),
            &serde_json::json!({
                "schema_version": 1,
                "windows": [],
            }),
        );
        let _ = std::fs::write(runtime_cfg.out_dir.join("latest.txt"), b"777-bundle");
    });

    let connected = connect_filesystem_tooling(&cfg, &root.join("ready.touch"), false, 5_000, 5)
        .expect("connect fs tooling");

    let tool_script_result_path = root.join("tool.script.result.json");
    let capabilities_check_path = root.join("check.capabilities.json");
    let script_json = serde_json::json!({
        "schema_version": 2,
        "steps": [],
    });

    let (result, bundle_path) = run_script_over_transport(
        &root,
        &connected,
        script_json,
        true,
        false,
        Some("dump"),
        None,
        5_000,
        5,
        &tool_script_result_path,
        &capabilities_check_path,
    )
    .expect("run_script_over_transport");

    assert!(matches!(
        result.stage,
        fret_diag_protocol::UiScriptStageV1::Passed
    ));
    assert!(bundle_path.is_some());

    let run_id_bundle = root.join("1").join("bundle.json");
    assert!(run_id_bundle.is_file(), "expected run_id bundle.json alias");
}

#[test]
fn run_script_over_transport_dump_bundle_with_trace_writes_run_id_trace_chrome_json() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-run-dump-trace-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let caps = fret_diag_protocol::FilesystemCapabilitiesV1 {
        schema_version: 1,
        capabilities: vec!["script_v2".to_string()],
        runner_kind: None,
        runner_version: None,
        hints: None,
    };
    crate::util::write_json_value(
        &root.join("capabilities.json"),
        &serde_json::to_value(caps).expect("capabilities json"),
    )
    .expect("write capabilities.json");

    let cfg = crate::transport::FsDiagTransportConfig::from_out_dir(&root);

    let runtime_cfg = cfg.clone();
    std::thread::spawn(move || {
        fn read_stamp(path: &Path) -> Option<u64> {
            let s = std::fs::read_to_string(path).ok()?;
            s.lines().last()?.trim().parse::<u64>().ok()
        }

        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if read_stamp(&runtime_cfg.script_trigger_path).is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        let passed = fret_diag_protocol::UiScriptResultV1 {
            schema_version: 1,
            run_id: 1,
            updated_unix_ms: crate::util::now_unix_ms(),
            window: None,
            stage: fret_diag_protocol::UiScriptStageV1::Passed,
            step_index: Some(0),
            reason_code: None,
            reason: None,
            evidence: None,
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        let _ = crate::util::write_json_value(
            &runtime_cfg.script_result_path,
            &serde_json::to_value(passed).unwrap_or_else(|_| serde_json::json!({})),
        );
        let _ = crate::util::touch(&runtime_cfg.script_result_trigger_path);

        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if read_stamp(&runtime_cfg.trigger_path).is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        let export_dir = runtime_cfg.out_dir.join("777-bundle");
        let _ = std::fs::create_dir_all(&export_dir);
        let _ = crate::util::write_json_value(
            &export_dir.join("bundle.json"),
            &serde_json::json!({
                "schema_version": 1,
                "windows": [],
            }),
        );
        let _ = std::fs::write(runtime_cfg.out_dir.join("latest.txt"), b"777-bundle");
    });

    let connected = connect_filesystem_tooling(&cfg, &root.join("ready.touch"), false, 5_000, 5)
        .expect("connect fs tooling");

    let tool_script_result_path = root.join("tool.script.result.json");
    let capabilities_check_path = root.join("check.capabilities.json");
    let script_json = serde_json::json!({
        "schema_version": 2,
        "steps": [],
    });

    let (result, bundle_path) = run_script_over_transport(
        &root,
        &connected,
        script_json,
        true,
        true,
        Some("dump"),
        None,
        5_000,
        5,
        &tool_script_result_path,
        &capabilities_check_path,
    )
    .expect("run_script_over_transport");

    assert!(matches!(
        result.stage,
        fret_diag_protocol::UiScriptStageV1::Passed
    ));
    assert!(bundle_path.is_some());

    let trace_path = root.join("1").join("trace.chrome.json");
    assert!(trace_path.is_file(), "expected run_id trace.chrome.json");

    let manifest_path = root.join("1").join("manifest.json");
    let bytes = std::fs::read(&manifest_path).expect("read manifest.json");
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("parse manifest");
    let ids = parsed
        .get("files")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|f| f.get("id").and_then(|v| v.as_str()))
        .collect::<Vec<_>>();
    assert!(ids.contains(&"script_result"));
    assert!(ids.contains(&"trace_chrome_json"));
}

#[test]
fn triage_includes_hints_and_unit_costs_for_worst_frame() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-triage-hints-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let bundle = serde_json::json!({
    "schema_version": 1,
    "windows": [{
        "window": 1,
        "events": [],
        "snapshots": [{
            "schema_version": 1,
            "tick_id": 1,
            "frame_id": 1,
            "window": 1,
            "timestamp_unix_ms": 123,
            "debug": { "stats": {
                "layout_time_us": 10_000,
                "prepaint_time_us": 0,
                "paint_time_us": 0,
                "layout_engine_solves": 1,
                "layout_engine_solve_time_us": 7_000,
                "layout_observation_record_time_us": 3_000,
                "layout_observation_record_models_items": 100,
                    "layout_observation_record_globals_items": 0,
                    "paint_text_prepare_time_us": 2_500,
                    "paint_text_prepare_calls": 10,
                    "paint_text_prepare_reason_text_changed": 10,
                    "renderer_upload_us": 123,
                    "renderer_record_passes_us": 45,
                    "renderer_encoder_finish_us": 67,
                    "renderer_text_atlas_upload_bytes": 2_000_000,
                } }
            }]
        }]
    });

    let bundle_path = root.join("bundle.json");
    crate::util::write_json_value(&bundle_path, &bundle).expect("write bundle.json");

    let report = crate::stats::bundle_stats_from_json_with_options(
        &bundle,
        1,
        BundleStatsSort::Time,
        crate::stats::BundleStatsOptions::default(),
    )
    .expect("bundle stats");

    let triage = triage_json_from_stats(&bundle_path, &report, BundleStatsSort::Time, 0);
    let codes = triage
        .get("hints")
        .and_then(|v| v.as_array())
        .unwrap()
        .iter()
        .filter_map(|h| h.get("code").and_then(|v| v.as_str()))
        .collect::<Vec<_>>();

    assert!(codes.contains(&"layout.observation_heavy"));
    assert!(codes.contains(&"layout.solve_heavy"));
    assert!(codes.contains(&"paint.text_prepare_churn"));
    assert!(codes.contains(&"renderer.upload_churn"));

    assert_eq!(
        triage
            .get("worst")
            .and_then(|v| v.get("renderer_upload_us"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        123
    );
    assert_eq!(
        triage
            .get("worst")
            .and_then(|v| v.get("renderer_record_passes_us"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        45
    );
    assert_eq!(
        triage
            .get("worst")
            .and_then(|v| v.get("renderer_encoder_finish_us"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        67
    );

    assert_eq!(
        triage
            .get("unit_costs")
            .and_then(|v| v.get("layout_engine_solve_us_per_solve"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        7_000
    );
}

#[test]
fn perf_hints_gate_reports_failures_for_denied_warn_hints() {
    let root = std::env::temp_dir().join(format!(
        "fret-diag-perf-hints-gate-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create temp root");

    let bundle = serde_json::json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "events": [],
            "snapshots": [{
                "schema_version": 1,
                "tick_id": 1,
                "frame_id": 1,
                "window": 1,
                "timestamp_unix_ms": 123,
                "debug": { "stats": {
                    "layout_time_us": 10_000,
                    "prepaint_time_us": 0,
                    "paint_time_us": 0,
                    "layout_engine_solves": 1,
                    "layout_engine_solve_time_us": 7_000,
                    "layout_observation_record_time_us": 3_000,
                    "layout_observation_record_models_items": 100,
                    "layout_observation_record_globals_items": 0,
                    "paint_text_prepare_time_us": 2_500,
                    "paint_text_prepare_calls": 10,
                    "paint_text_prepare_reason_text_changed": 10,
                    "renderer_text_atlas_upload_bytes": 2_000_000,
                } }
            }]
        }]
    });

    let bundle_path = root.join("bundle.json");
    crate::util::write_json_value(&bundle_path, &bundle).expect("write bundle.json");

    let report = crate::stats::bundle_stats_from_json_with_options(
        &bundle,
        1,
        BundleStatsSort::Time,
        crate::stats::BundleStatsOptions::default(),
    )
    .expect("bundle stats");

    let triage = triage_json_from_stats(&bundle_path, &report, BundleStatsSort::Time, 0);

    let deny_specs: Vec<String> = Vec::new();
    let opts = parse_perf_hint_gate_options(true, &deny_specs, None).expect("parse hint gate opts");
    let failures = perf_hint_gate_failures_for_triage_json(
        "script.json",
        &bundle_path,
        Some(0),
        &triage,
        &opts,
    );
    let codes = failures
        .iter()
        .filter_map(|f| f.get("code").and_then(|v| v.as_str()))
        .collect::<Vec<_>>();
    assert!(codes.contains(&"layout.observation_heavy"));
    assert!(codes.contains(&"layout.solve_heavy"));
    assert!(codes.contains(&"paint.text_prepare_churn"));
    assert!(!codes.contains(&"renderer.upload_churn"));

    let deny_specs = vec!["renderer.upload_churn".to_string()];
    let opts = parse_perf_hint_gate_options(true, &deny_specs, Some("info"))
        .expect("parse hint gate opts");
    let failures = perf_hint_gate_failures_for_triage_json(
        "script.json",
        &bundle_path,
        Some(0),
        &triage,
        &opts,
    );
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0].get("code").and_then(|v| v.as_str()),
        Some("renderer.upload_churn")
    );
}

#[test]
fn stale_scene_check_fails_when_label_changes_without_scene_change() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "scene_fingerprint": 7,
                        "debug": { "semantics": { "nodes": [
                            { "id": 1, "test_id": "search", "bounds": { "y": 0.0 }, "label": "hello" }
                        ]}}
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "scene_fingerprint": 7,
                        "debug": { "semantics": { "nodes": [
                            { "id": 1, "test_id": "search", "bounds": { "y": 0.0 }, "label": "world" }
                        ]}}
                    }
                ]
            }
        ]
    });

    let err = check_bundle_for_stale_scene_json(&bundle, Path::new("bundle.json"), "search", 0.5)
        .unwrap_err();
    assert!(err.contains("stale scene suspected"));
}

#[test]
fn semantics_repaint_check_fails_when_semantics_fingerprint_changes_without_scene_change() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "scene_fingerprint": 7,
                        "semantics_fingerprint": 100,
                        "debug": { "stats": { "paint_nodes_performed": 0, "paint_cache_replayed_ops": 0 } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "scene_fingerprint": 7,
                        "semantics_fingerprint": 101,
                        "debug": { "stats": { "paint_nodes_performed": 0, "paint_cache_replayed_ops": 0 } }
                    }
                ]
            }
        ]
    });

    let err =
        check_bundle_for_semantics_changed_repainted_json(&bundle, Path::new("bundle.json"), 0)
            .unwrap_err();
    assert!(err.contains("missing repaint suspected"));
}

#[test]
fn semantics_repaint_scan_includes_semantics_diff_detail_when_available() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "scene_fingerprint": 7,
                        "semantics_fingerprint": 100,
                        "debug": { "semantics": { "nodes": [
                            { "id": 1, "test_id": "search", "role": "textbox", "label": "hello", "bounds": { "x": 0.0, "y": 0.0, "w": 10.0, "h": 10.0 } }
                        ]}}
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "scene_fingerprint": 7,
                        "semantics_fingerprint": 101,
                        "debug": {
                            "stats": { "paint_nodes_performed": 0, "paint_cache_replayed_ops": 0 },
                            "semantics": { "nodes": [
                                { "id": 1, "test_id": "search", "role": "textbox", "label": "world", "bounds": { "x": 0.0, "y": 0.0, "w": 10.0, "h": 10.0 } }
                            ]}
                        }
                    }
                ]
            }
        ]
    });

    let scan = scan_semantics_changed_repainted_json(&bundle, 0);
    assert_eq!(scan.findings.len(), 1);
    assert!(scan.findings[0].get("semantics_diff").is_some());
    assert_eq!(
        scan.findings[0]
            .get("semantics_diff")
            .and_then(|v: &serde_json::Value| v.get("counts"))
            .and_then(|v: &serde_json::Value| v.get("changed"))
            .and_then(|v: &serde_json::Value| v.as_u64()),
        Some(1)
    );
}

#[test]
fn semantics_repaint_check_passes_when_scene_fingerprint_changes() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "scene_fingerprint": 7,
                        "semantics_fingerprint": 100
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "scene_fingerprint": 8,
                        "semantics_fingerprint": 101
                    }
                ]
            }
        ]
    });

    check_bundle_for_semantics_changed_repainted_json(&bundle, Path::new("bundle.json"), 0)
        .unwrap();
}

#[test]
fn ui_gallery_code_editor_torture_composed_preedit_a11y_toggle_gate_passes() {
    let out_dir = tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_a11y_toggle_pass");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");

    let snapshot = |frame_id: u64, folds: bool, inlays: bool| {
        json!({
            "tick_id": frame_id,
            "frame_id": frame_id,
            "app_snapshot": {
                "kind": "fret_ui_gallery",
                "selected_page": "code_editor_torture",
                "code_editor": {
                    "soft_wrap_cols": 80,
                    "folds_fixture": folds,
                    "inlays_fixture": inlays,
                    "torture": {
                        "preedit_active": true,
                        "allow_decorations_under_inline_preedit": true,
                        "compose_inline_preedit": true
                    }
                }
            },
            "debug": {
                "semantics": {
                    "nodes": [
                        {
                            "id": 10,
                            "role": "text_field",
                            "value": "----ab----",
                            "text_selection": [6, 6],
                            "text_composition": [4, 6]
                        },
                        {
                            "id": 11,
                            "parent": 10,
                            "test_id": "ui-gallery-code-editor-torture-viewport"
                        }
                    ]
                }
            }
        })
    };

    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    snapshot(10, true, true),
                    snapshot(11, false, true),
                    snapshot(12, false, false),
                    snapshot(13, true, true)
                ]
            }
        ]
    });

    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed(
        &bundle_path,
        0,
    )
    .unwrap();

    assert!(
        out_dir
            .join("check.ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed.json")
            .is_file()
    );
}

#[test]
fn ui_gallery_code_editor_torture_composed_preedit_a11y_toggle_gate_fails_on_mismatched_preedit_text()
 {
    let out_dir = tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_a11y_toggle_fail");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 10,
                        "frame_id": 10,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "folds_fixture": true,
                                "inlays_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "allow_decorations_under_inline_preedit": true,
                                    "compose_inline_preedit": true
                                }
                            }
                        },
                        "debug": {
                            "semantics": {
                                "nodes": [
                                    {
                                        "id": 10,
                                        "role": "text_field",
                                        "value": "----ac----",
                                        "text_selection": [6, 6],
                                        "text_composition": [4, 6]
                                    },
                                    {
                                        "id": 11,
                                        "parent": 10,
                                        "test_id": "ui-gallery-code-editor-torture-viewport"
                                    }
                                ]
                            }
                        }
                    },
                    {
                        "tick_id": 11,
                        "frame_id": 11,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "folds_fixture": false,
                                "inlays_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "allow_decorations_under_inline_preedit": true,
                                    "compose_inline_preedit": true
                                }
                            }
                        },
                        "debug": {
                            "semantics": {
                                "nodes": [
                                    {
                                        "id": 10,
                                        "role": "text_field",
                                        "value": "----ac----",
                                        "text_selection": [6, 6],
                                        "text_composition": [4, 6]
                                    },
                                    {
                                        "id": 11,
                                        "parent": 10,
                                        "test_id": "ui-gallery-code-editor-torture-viewport"
                                    }
                                ]
                            }
                        }
                    },
                    {
                        "tick_id": 12,
                        "frame_id": 12,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "folds_fixture": false,
                                "inlays_fixture": false,
                                "torture": {
                                    "preedit_active": true,
                                    "allow_decorations_under_inline_preedit": true,
                                    "compose_inline_preedit": true
                                }
                            }
                        },
                        "debug": {
                            "semantics": {
                                "nodes": [
                                    {
                                        "id": 10,
                                        "role": "text_field",
                                        "value": "----ac----",
                                        "text_selection": [6, 6],
                                        "text_composition": [4, 6]
                                    },
                                    {
                                        "id": 11,
                                        "parent": 10,
                                        "test_id": "ui-gallery-code-editor-torture-viewport"
                                    }
                                ]
                            }
                        }
                    },
                    {
                        "tick_id": 13,
                        "frame_id": 13,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "folds_fixture": true,
                                "inlays_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "allow_decorations_under_inline_preedit": true,
                                    "compose_inline_preedit": true
                                }
                            }
                        },
                        "debug": {
                            "semantics": {
                                "nodes": [
                                    {
                                        "id": 10,
                                        "role": "text_field",
                                        "value": "----ac----",
                                        "text_selection": [6, 6],
                                        "text_composition": [4, 6]
                                    },
                                    {
                                        "id": 11,
                                        "parent": 10,
                                        "test_id": "ui-gallery-code-editor-torture-viewport"
                                    }
                                ]
                            }
                        }
                    }
                ]
            }
        ]
    });

    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    assert!(
        check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed(
            &bundle_path,
            0,
        )
        .is_err()
    );
}

#[test]
fn ui_gallery_code_editor_torture_composed_preedit_wheel_gate_passes_when_stable() {
    let out_dir = tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_wheel_pass");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");

    let snapshot = |frame_id: u64, rev: u64| {
        json!({
            "tick_id": frame_id,
            "frame_id": frame_id,
            "app_snapshot": {
                "kind": "fret_ui_gallery",
                "selected_page": "code_editor_torture",
                "code_editor": {
                    "soft_wrap_cols": 80,
                    "torture": {
                        "preedit_active": true,
                        "allow_decorations_under_inline_preedit": true,
                        "compose_inline_preedit": true,
                        "buffer_revision": rev,
                        "text_len_bytes": 123,
                        "selection": { "anchor": 4, "caret": 4 }
                    }
                }
            },
            "debug": {
                "semantics": {
                    "nodes": [
                        {
                            "id": 10,
                            "role": "text_field",
                            "value": "zzab",
                            "text_selection": [4, 4],
                            "text_composition": [2, 4]
                        },
                        {
                            "id": 11,
                            "parent": 10,
                            "test_id": "ui-gallery-code-editor-torture-viewport"
                        }
                    ]
                }
            }
        })
    };

    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "events": [
                    { "kind": "pointer.wheel", "frame_id": 10 }
                ],
                "snapshots": [
                    snapshot(9, 1),
                    snapshot(10, 1)
                ]
            }
        ]
    });

    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();

    assert!(
        out_dir
            .join("check.ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll.json")
            .is_file()
    );
}

#[test]
fn ui_gallery_code_editor_torture_composed_preedit_wheel_gate_fails_when_buffer_changes() {
    let out_dir = tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_wheel_fail");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");

    let snapshot = |frame_id: u64, rev: u64| {
        json!({
            "tick_id": frame_id,
            "frame_id": frame_id,
            "app_snapshot": {
                "kind": "fret_ui_gallery",
                "selected_page": "code_editor_torture",
                "code_editor": {
                    "soft_wrap_cols": 80,
                    "torture": {
                        "preedit_active": true,
                        "allow_decorations_under_inline_preedit": true,
                        "compose_inline_preedit": true,
                        "buffer_revision": rev,
                        "text_len_bytes": 123
                    }
                }
            },
            "debug": {
                "semantics": {
                    "nodes": [
                        {
                            "id": 10,
                            "role": "text_field",
                            "value": "zzab",
                            "text_selection": [4, 4],
                            "text_composition": [2, 4]
                        },
                        {
                            "id": 11,
                            "parent": 10,
                            "test_id": "ui-gallery-code-editor-torture-viewport"
                        }
                    ]
                }
            }
        })
    };

    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "events": [
                    { "kind": "pointer.wheel", "frame_id": 10 }
                ],
                "snapshots": [
                    snapshot(9, 1),
                    snapshot(10, 2)
                ]
            }
        ]
    });

    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err =
        check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
    assert!(err.contains("wheel gate failed"));
}

#[test]
fn ui_gallery_code_editor_torture_composed_preedit_drag_select_gate_passes() {
    let out_dir = tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_drag_pass");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");

    let snapshot = |frame_id: u64,
                    preedit_active: bool,
                    rev: u64,
                    anchor: u64,
                    caret: u64,
                    text_composition: serde_json::Value| {
        json!({
            "tick_id": frame_id,
            "frame_id": frame_id,
            "app_snapshot": {
                "kind": "fret_ui_gallery",
                "selected_page": "code_editor_torture",
                "code_editor": {
                    "soft_wrap_cols": 80,
                    "torture": {
                        "preedit_active": preedit_active,
                        "allow_decorations_under_inline_preedit": true,
                        "compose_inline_preedit": true,
                        "buffer_revision": rev,
                        "text_len_bytes": 123,
                        "selection": { "anchor": anchor, "caret": caret }
                    }
                }
            },
            "debug": {
                "semantics": {
                    "nodes": [
                        {
                            "id": 10,
                            "role": "text_field",
                            "value": "zzab",
                            "text_selection": [anchor, caret],
                            "text_composition": text_composition
                        },
                        {
                            "id": 11,
                            "parent": 10,
                            "test_id": "ui-gallery-code-editor-torture-viewport"
                        }
                    ]
                }
            }
        })
    };

    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    snapshot(10, true, 1, 4, 4, json!([2,4])),
                    snapshot(11, false, 1, 0, 4, serde_json::Value::Null)
                ]
            }
        ]
    });

    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();

    assert!(
        out_dir
            .join("check.ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection.json")
            .is_file()
    );
}

#[test]
fn ui_gallery_code_editor_torture_composed_preedit_drag_select_gate_fails_when_buffer_changes() {
    let out_dir = tmp_out_dir("ui_gallery_code_editor_torture_composed_preedit_drag_fail");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 10,
                        "frame_id": 10,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "torture": {
                                    "preedit_active": true,
                                    "allow_decorations_under_inline_preedit": true,
                                    "compose_inline_preedit": true,
                                    "buffer_revision": 1,
                                    "text_len_bytes": 123,
                                    "selection": { "anchor": 4, "caret": 4 }
                                }
                            }
                        },
                        "debug": {
                            "semantics": {
                                "nodes": [
                                    {
                                        "id": 10,
                                        "role": "text_field",
                                        "value": "zzab",
                                        "text_selection": [4, 4],
                                        "text_composition": [2, 4]
                                    },
                                    {
                                        "id": 11,
                                        "parent": 10,
                                        "test_id": "ui-gallery-code-editor-torture-viewport"
                                    }
                                ]
                            }
                        }
                    },
                    {
                        "tick_id": 11,
                        "frame_id": 11,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "torture": {
                                    "preedit_active": false,
                                    "allow_decorations_under_inline_preedit": true,
                                    "compose_inline_preedit": true,
                                    "buffer_revision": 2,
                                    "text_len_bytes": 123,
                                    "selection": { "anchor": 0, "caret": 4 }
                                }
                            }
                        },
                        "debug": {
                            "semantics": {
                                "nodes": [
                                    {
                                        "id": 10,
                                        "role": "text_field",
                                        "value": "zzab",
                                        "text_selection": [0, 4],
                                        "text_composition": null
                                    },
                                    {
                                        "id": 11,
                                        "parent": 10,
                                        "test_id": "ui-gallery-code-editor-torture-viewport"
                                    }
                                ]
                            }
                        }
                    }
                ]
            }
        ]
    });

    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err =
        check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
    assert!(err.contains("drag-select gate failed"));
}

#[test]
fn bundle_stats_sums_and_sorts_top_by_invalidation_nodes() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "changed_models": [],
                        "changed_globals": [],
                        "debug": { "stats": {
                            "invalidation_walk_calls": 2,
                            "invalidation_walk_nodes": 10,
                            "model_change_invalidation_roots": 1,
                            "global_change_invalidation_roots": 0
                        } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "changed_models": [123],
                        "changed_globals": ["TypeId(0x0)"],
                        "debug": { "stats": {
                            "invalidation_walk_calls": 5,
                            "invalidation_walk_nodes": 7,
                            "model_change_invalidation_roots": 2,
                            "global_change_invalidation_roots": 1
                        } }
                    }
                ]
            }
        ]
    });

    let report = bundle_stats_from_json_with_options(
        &bundle,
        1,
        BundleStatsSort::Invalidation,
        BundleStatsOptions::default(),
    )
    .unwrap();
    assert_eq!(report.windows, 1);
    assert_eq!(report.snapshots, 2);
    assert_eq!(report.snapshots_with_model_changes, 1);
    assert_eq!(report.snapshots_with_global_changes, 1);
    assert_eq!(report.sum_invalidation_walk_calls, 7);
    assert_eq!(report.sum_invalidation_walk_nodes, 17);
    assert_eq!(report.max_invalidation_walk_calls, 5);
    assert_eq!(report.max_invalidation_walk_nodes, 10);
    assert_eq!(report.top.len(), 1);
    assert_eq!(report.top[0].invalidation_walk_nodes, 10);
    assert_eq!(report.top[0].tick_id, 1);
}

#[test]
fn bundle_stats_extracts_top_invalidation_walks_with_semantics() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "changed_models": [],
                        "changed_globals": [],
                        "debug": {
                            "stats": {
                                "invalidation_walk_calls": 1,
                                "invalidation_walk_nodes": 42,
                                "model_change_invalidation_roots": 0,
                                "global_change_invalidation_roots": 0
                            },
                            "invalidation_walks": [
                                { "root_node": 42, "kind": "paint", "source": "other", "walked_nodes": 10 },
                                { "root_node": 43, "kind": "layout", "source": "other", "walked_nodes": 20, "root_element": 9 }
                            ],
                            "semantics": {
                                "nodes": [
                                    { "id": 43, "role": "button", "test_id": "todo-add" }
                                ]
                            }
                        }
                    }
                ]
            }
        ]
    });

    let report = bundle_stats_from_json_with_options(
        &bundle,
        1,
        BundleStatsSort::Invalidation,
        BundleStatsOptions::default(),
    )
    .unwrap();
    assert_eq!(report.top.len(), 1);
    assert_eq!(report.top[0].top_invalidation_walks.len(), 2);
    assert_eq!(report.top[0].top_invalidation_walks[0].root_node, 43);
    assert_eq!(
        report.top[0].top_invalidation_walks[0]
            .root_test_id
            .as_deref(),
        Some("todo-add")
    );
    assert_eq!(
        report.top[0].top_invalidation_walks[0].root_role.as_deref(),
        Some("button")
    );
    assert_eq!(
        report.top[0].top_invalidation_walks[0].root_element,
        Some(9)
    );
}

#[test]
fn perf_percentile_nearest_rank_is_stable() {
    let values = vec![10u64, 20, 30, 40, 50, 60, 70];
    let mut sorted = values.clone();
    sorted.sort_unstable();
    assert_eq!(percentile_nearest_rank_sorted(&sorted, 0.50), 40);
    assert_eq!(percentile_nearest_rank_sorted(&sorted, 0.95), 70);
    assert_eq!(
        summarize_times_us(&values),
        json!({"min":10,"p50":40,"p95":70,"max":70})
    );
}

#[test]
fn bundle_stats_tracks_hover_declarative_layout_invalidations() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "changed_models": [],
                        "changed_globals": [],
                        "debug": {
                            "stats": {
                                "invalidation_walk_calls": 1,
                                "invalidation_walk_nodes": 1,
                                "model_change_invalidation_roots": 0,
                                "global_change_invalidation_roots": 0,
                                "hover_declarative_layout_invalidations": 0
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "changed_models": [],
                        "changed_globals": [],
                        "debug": {
                            "stats": {
                                "invalidation_walk_calls": 2,
                                "invalidation_walk_nodes": 10,
                                "model_change_invalidation_roots": 0,
                                "global_change_invalidation_roots": 0,
                                "hover_declarative_layout_invalidations": 2
                            },
                            "hover_declarative_invalidation_hotspots": [
                                { "node": 43, "layout": 2, "hit_test": 0, "paint": 0 }
                            ],
                            "semantics": {
                                "nodes": [
                                    { "id": 43, "role": "button", "test_id": "hover-offender" }
                                ]
                            }
                        }
                    }
                ]
            }
        ]
    });

    let report = bundle_stats_from_json_with_options(
        &bundle,
        1,
        BundleStatsSort::Invalidation,
        BundleStatsOptions::default(),
    )
    .unwrap();

    assert_eq!(report.sum_hover_layout_invalidations, 2);
    assert_eq!(report.max_hover_layout_invalidations, 2);
    assert_eq!(report.snapshots_with_hover_layout_invalidations, 1);
    assert_eq!(report.top.len(), 1);
    assert_eq!(report.top[0].tick_id, 2);
    assert_eq!(report.top[0].hover_declarative_layout_invalidations, 2);
    assert_eq!(report.top[0].top_hover_declarative_invalidations.len(), 1);
    assert_eq!(
        report.top[0].top_hover_declarative_invalidations[0].node,
        43
    );
    assert_eq!(
        report.top[0].top_hover_declarative_invalidations[0]
            .test_id
            .as_deref(),
        Some("hover-offender")
    );
}

#[test]
fn json_pointer_set_updates_object_field() {
    let mut v = json!({
        "steps": [
            { "type": "click", "target": { "kind": "node_id", "node": 1 } }
        ]
    });
    json_pointer_set(
        &mut v,
        "/steps/0/target",
        json!({"kind":"test_id","id":"x"}),
    )
    .unwrap();
    assert_eq!(v["steps"][0]["target"]["kind"], "test_id");
}

#[test]
fn json_pointer_set_updates_predicate_target() {
    let mut v = json!({
        "steps": [
            { "type": "wait_until", "predicate": { "kind": "exists", "target": { "kind": "node_id", "node": 1 } }, "timeout_frames": 10 }
        ]
    });
    json_pointer_set(
        &mut v,
        "/steps/0/predicate/target",
        json!({"kind":"test_id","id":"open"}),
    )
    .unwrap();
    assert_eq!(v["steps"][0]["predicate"]["target"]["id"], "open");
}

#[test]
fn check_bundle_for_view_cache_reuse_min_counts_reused_cache_roots() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "stats": { "view_cache_active": true },
                            "cache_roots": [
                                { "root": 1, "reused": true },
                                { "root": 2, "reused": false }
                            ]
                        }
                    },
                    {
                        "frame_id": 1,
                        "debug": {
                            "stats": { "view_cache_active": true },
                            "cache_roots": [
                                { "root": 3, "reused": true }
                            ]
                        }
                    }
                ]
            }
        ]
    });

    check_bundle_for_view_cache_reuse_min_json(&bundle, Path::new("bundle.json"), 2, 0)
        .expect("expected reuse>=2");
}

#[test]
fn check_bundle_for_view_cache_reuse_min_respects_warmup_frames() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "stats": { "view_cache_active": true },
                            "cache_roots": [
                                { "root": 1, "reused": true }
                            ]
                        }
                    },
                    {
                        "frame_id": 1,
                        "debug": {
                            "stats": { "view_cache_active": true },
                            "cache_roots": [
                                { "root": 2, "reused": true }
                            ]
                        }
                    }
                ]
            }
        ]
    });

    let err = check_bundle_for_view_cache_reuse_min_json(&bundle, Path::new("bundle.json"), 2, 1)
        .expect_err("expected reuse<2 due to warmup");
    assert!(err.contains("expected at least 2 view-cache reuse events"));
    assert!(err.contains("got 1"));
}

#[test]
fn view_cache_reuse_stable_check_passes_when_tail_streak_meets_min() {
    let out_dir = tmp_out_dir("view_cache_reuse_stable_pass");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 1, "reused": false }] } },
                { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 2, "reused": true }] } },
                { "tick_id": 3, "frame_id": 3, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 5 }, "cache_roots": [] } },
                { "tick_id": 4, "frame_id": 4, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 3, "reused": true }] } }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_view_cache_reuse_stable_min(&bundle_path, &out_dir, 3, 0).unwrap();
    assert!(out_dir.join("check.view_cache_reuse_stable.json").is_file());
}

#[test]
fn view_cache_reuse_stable_check_fails_when_tail_streak_is_too_small() {
    let out_dir = tmp_out_dir("view_cache_reuse_stable_fail");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 1, "reused": true }] } },
                { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "view_cache_active": true, "paint_cache_replayed_ops": 0 }, "cache_roots": [{ "root": 2, "reused": false }] } }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err =
        check_bundle_for_view_cache_reuse_stable_min(&bundle_path, &out_dir, 2, 0).unwrap_err();
    assert!(err.contains("view-cache reuse stable gate failed"));
    assert!(out_dir.join("check.view_cache_reuse_stable.json").is_file());
}

#[test]
fn check_bundle_for_overlay_synthesis_min_counts_synthesized_events() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "stats": { "view_cache_active": true },
                            "overlay_synthesis": [
                                { "kind": "popover", "id": 101, "source": "cached_declaration", "outcome": "synthesized" },
                                { "kind": "tooltip", "id": 202, "source": "cached_declaration", "outcome": "suppressed_missing_trigger" }
                            ]
                        }
                    },
                    {
                        "frame_id": 1,
                        "debug": {
                            "stats": { "view_cache_active": true },
                            "overlay_synthesis": [
                                { "kind": "tooltip", "id": 303, "source": "cached_declaration", "outcome": "synthesized" }
                            ]
                        }
                    }
                ]
            }
        ]
    });

    check_bundle_for_overlay_synthesis_min_json(&bundle, Path::new("bundle.json"), 2, 0)
        .expect("expected synthesized>=2");
}

#[test]
fn check_bundle_for_overlay_synthesis_min_respects_warmup_frames() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "stats": { "view_cache_active": true },
                            "overlay_synthesis": [
                                { "kind": "tooltip", "id": 1, "source": "cached_declaration", "outcome": "synthesized" }
                            ]
                        }
                    },
                    {
                        "frame_id": 1,
                        "debug": {
                            "stats": { "view_cache_active": true },
                            "overlay_synthesis": [
                                { "kind": "hover", "id": 2, "source": "cached_declaration", "outcome": "suppressed_trigger_not_live_in_current_frame" }
                            ]
                        }
                    }
                ]
            }
        ]
    });

    let err = check_bundle_for_overlay_synthesis_min_json(&bundle, Path::new("bundle.json"), 1, 1)
        .expect_err("expected synthesized<1 due to warmup");
    assert!(err.contains("expected at least 1 overlay synthesis events"));
    assert!(err.contains("got 0"));
    assert!(err.contains("suppressions=["));
}

#[test]
fn check_bundle_for_retained_vlist_reconcile_no_notify_min_passes() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                {
                    "frame_id": 0,
                    "debug": {
                        "stats": { "retained_virtual_list_reconciles": 1 },
                        "dirty_views": [{ "root_node": 1, "source": "notify" }]
                    }
                },
                {
                    "frame_id": 1,
                    "debug": {
                        "stats": { "retained_virtual_list_reconciles": 2 },
                        "retained_virtual_list_reconciles": [
                            { "node": 10, "element": 20, "prev_items": 1, "next_items": 2, "preserved_items": 1, "attached_items": 1, "detached_items": 0 },
                            { "node": 11, "element": 21, "prev_items": 2, "next_items": 3, "preserved_items": 2, "attached_items": 1, "detached_items": 0 }
                        ],
                        "dirty_views": []
                    }
                }
            ]
        }]
    });

    check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
        &bundle,
        Path::new("bundle.json"),
        1,
        1,
    )
    .expect("expected reconcile>=1 without notify dirtiness");
}

#[test]
fn check_bundle_for_retained_vlist_reconcile_no_notify_min_fails_on_notify_dirty_view() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "frame_id": 1,
                "debug": {
                    "stats": { "retained_virtual_list_reconciles": 1 },
                    "dirty_views": [
                        { "root_node": 123, "source": "notify", "detail": "notify_call" }
                    ]
                }
            }]
        }]
    });

    let err = check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
        &bundle,
        Path::new("bundle.json"),
        1,
        0,
    )
    .expect_err("expected notify offenders");
    assert!(
        err.contains("retained virtual-list reconcile should not require notify-based dirty views")
    );
    assert!(err.contains("source=notify"));
}

#[test]
fn check_bundle_for_retained_vlist_reconcile_no_notify_min_fails_when_missing_reconciles() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "frame_id": 10,
                "debug": { "stats": { "retained_virtual_list_reconciles": 0 } }
            }]
        }]
    });

    let err = check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
        &bundle,
        Path::new("bundle.json"),
        1,
        0,
    )
    .expect_err("expected missing reconcile events");
    assert!(err.contains("expected at least 1 retained virtual-list reconcile events"));
    assert!(err.contains("got 0"));
}

#[test]
fn check_bundle_for_retained_vlist_attach_detach_max_passes() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "frame_id": 10,
                "debug": {
                    "stats": {
                        "retained_virtual_list_reconciles": 1,
                        "retained_virtual_list_attached_items": 12,
                        "retained_virtual_list_detached_items": 13
                    },
                    "retained_virtual_list_reconciles": [
                        { "node": 10, "element": 20, "prev_items": 1, "next_items": 2, "preserved_items": 1, "attached_items": 12, "detached_items": 13 }
                    ]
                }
            }]
        }]
    });

    check_bundle_for_retained_vlist_attach_detach_max_json(
        &bundle,
        Path::new("bundle.json"),
        25,
        0,
    )
    .expect("expected delta<=25");
}

#[test]
fn check_bundle_for_retained_vlist_attach_detach_max_fails_when_exceeded() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "frame_id": 10,
                "debug": {
                    "stats": {
                        "retained_virtual_list_reconciles": 1,
                        "retained_virtual_list_attached_items": 20,
                        "retained_virtual_list_detached_items": 21
                    }
                }
            }]
        }]
    });

    let err = check_bundle_for_retained_vlist_attach_detach_max_json(
        &bundle,
        Path::new("bundle.json"),
        40,
        0,
    )
    .expect_err("expected delta>40 to fail");
    assert!(err.contains("attach/detach delta exceeded"));
    assert!(err.contains("delta=41"));
}

#[test]
fn check_bundle_for_retained_vlist_attach_detach_max_fails_when_missing_reconciles() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "frame_id": 10,
                "debug": { "stats": { "retained_virtual_list_reconciles": 0 } }
            }]
        }]
    });

    let err = check_bundle_for_retained_vlist_attach_detach_max_json(
        &bundle,
        Path::new("bundle.json"),
        10,
        0,
    )
    .expect_err("expected missing reconcile events");
    assert!(err.contains("expected at least 1 retained virtual-list reconcile event"));
}

#[test]
fn check_bundle_for_retained_vlist_keep_alive_budget_passes() {
    let out_dir = tmp_out_dir("retained_vlist_keep_alive_budget_pass");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "frame_id": 10,
                "debug": {
                    "retained_virtual_list_reconciles": [
                        { "keep_alive_pool_len_after": 128, "evicted_keep_alive_items": 0 }
                    ]
                }
            }]
        }]
    });

    check_bundle_for_retained_vlist_keep_alive_budget_json(&bundle, &bundle_path, 1, 0, 0)
        .expect("expected keep-alive budget to pass");
    assert!(
        out_dir
            .join("check.retained_vlist_keep_alive_budget.json")
            .is_file()
    );
}

#[test]
fn check_bundle_for_retained_vlist_keep_alive_budget_fails_when_evicted() {
    let out_dir = tmp_out_dir("retained_vlist_keep_alive_budget_fail_evicted");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "frame_id": 10,
                "debug": {
                    "retained_virtual_list_reconciles": [
                        { "keep_alive_pool_len_after": 64, "evicted_keep_alive_items": 1 }
                    ]
                }
            }]
        }]
    });

    let err =
        check_bundle_for_retained_vlist_keep_alive_budget_json(&bundle, &bundle_path, 1, 0, 0)
            .expect_err("expected eviction budget to fail");
    assert!(err.contains("keep-alive budget violated"));
    assert!(err.contains("total_evicted_items=1"));
}

#[test]
fn check_bundle_for_viewport_input_min_counts_events() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "viewport_input": [
                                { "target": 1, "pointer_id": 0, "pointer_type": "mouse", "cursor_px": {"x": 1.0, "y": 2.0}, "uv": [0.0, 0.0], "target_px": [0, 0], "kind": { "type": "pointer_down", "button": "left", "modifiers": {}, "click_count": 1 } }
                            ]
                        }
                    },
                    {
                        "frame_id": 1,
                        "debug": {
                            "viewport_input": [
                                { "target": 1, "pointer_id": 0, "pointer_type": "mouse", "cursor_px": {"x": 2.0, "y": 3.0}, "uv": [0.1, 0.1], "target_px": [10, 10], "kind": { "type": "pointer_move", "buttons": {"left": true, "right": false, "middle": false}, "modifiers": {} } }
                            ]
                        }
                    }
                ]
            }
        ]
    });

    check_bundle_for_viewport_input_min_json(&bundle, Path::new("bundle.json"), 2, 0)
        .expect("expected viewport_input>=2");
}

#[test]
fn check_bundle_for_viewport_input_min_respects_warmup_frames() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "viewport_input": [
                                { "target": 1, "pointer_id": 0, "pointer_type": "mouse", "cursor_px": {"x": 1.0, "y": 2.0}, "uv": [0.0, 0.0], "target_px": [0, 0], "kind": { "type": "pointer_down", "button": "left", "modifiers": {}, "click_count": 1 } }
                            ]
                        }
                    },
                    {
                        "frame_id": 1,
                        "debug": {
                            "viewport_input": []
                        }
                    }
                ]
            }
        ]
    });

    let err = check_bundle_for_viewport_input_min_json(&bundle, Path::new("bundle.json"), 1, 1)
        .expect_err("expected viewport input < 1 due to warmup");
    assert!(err.contains("expected at least 1 viewport input events"));
    assert!(err.contains("got 0"));
}

#[test]
fn check_bundle_for_dock_drag_min_counts_active_frames() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "docking_interaction": {
                                "dock_drag": { "pointer_id": 0, "source_window": 1, "current_window": 1, "dragging": true, "cross_window_hover": false },
                                "viewport_capture": null
                            }
                        }
                    }
                ]
            }
        ]
    });

    check_bundle_for_dock_drag_min_json(&bundle, Path::new("bundle.json"), 1, 0)
        .expect("expected dock_drag>=1");
}

#[test]
fn check_bundle_for_viewport_capture_min_respects_warmup_frames() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "frame_id": 0,
                        "debug": {
                            "docking_interaction": {
                                "dock_drag": null,
                                "viewport_capture": { "pointer_id": 0, "target": 2 }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let err = check_bundle_for_viewport_capture_min_json(&bundle, Path::new("bundle.json"), 1, 1)
        .expect_err("expected viewport_capture<1 due to warmup");
    assert!(err.contains("expected at least 1 snapshots with an active viewport capture"));
    assert!(err.contains("got 0"));
}

#[test]
fn compare_bundles_passes_when_test_id_semantics_match() {
    let a = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "frame_id": 10,
                "scene_fingerprint": 42,
                "debug": {
                    "semantics": {
                        "roots": [{ "root": 1, "visible": true, "blocks_underlay_input": false, "hit_testable": true, "z_index": 0 }],
                        "nodes": [{
                            "id": 1,
                            "role": "button",
                            "bounds": { "x": 1.0, "y": 2.0, "w": 3.0, "h": 4.0 },
                            "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                            "actions": { "focus": true, "invoke": true, "set_value": false, "set_text_selection": false },
                            "test_id": "ok"
                        }]
                    }
                }
            }]
        }]
    });
    let b = a.clone();
    let report = compare_bundles_json(
        &a,
        Path::new("a/bundle.json"),
        &b,
        Path::new("b/bundle.json"),
        CompareOptions {
            warmup_frames: 0,
            eps_px: 0.5,
            ignore_bounds: false,
            ignore_scene_fingerprint: false,
        },
    )
    .unwrap();
    assert!(report.ok);
    assert!(report.diffs.is_empty());
}

#[test]
fn compare_bundles_reports_role_mismatch_for_test_id() {
    let a = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "frame_id": 10,
                "scene_fingerprint": 42,
                "debug": {
                    "semantics": {
                        "roots": [{ "root": 1, "visible": true, "blocks_underlay_input": false, "hit_testable": true, "z_index": 0 }],
                        "nodes": [{
                            "id": 1,
                            "role": "button",
                            "bounds": { "x": 1.0, "y": 2.0, "w": 3.0, "h": 4.0 },
                            "flags": { "focused": false, "captured": false, "disabled": false, "selected": false, "expanded": false, "checked": null },
                            "actions": { "focus": true, "invoke": true, "set_value": false, "set_text_selection": false },
                            "test_id": "t"
                        }]
                    }
                }
            }]
        }]
    });
    let mut b = a.clone();
    b["windows"][0]["snapshots"][0]["debug"]["semantics"]["nodes"][0]["role"] =
        serde_json::Value::from("menuitem");

    let report = compare_bundles_json(
        &a,
        Path::new("a/bundle.json"),
        &b,
        Path::new("b/bundle.json"),
        CompareOptions {
            warmup_frames: 0,
            eps_px: 0.5,
            ignore_bounds: false,
            ignore_scene_fingerprint: false,
        },
    )
    .unwrap();
    assert!(!report.ok);
    assert!(report.diffs.iter().any(|d| d.kind == "node_field_mismatch"
        && d.key.as_deref() == Some("t")
        && d.field == Some("role")));
}

fn tmp_out_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fretboard_test_{label}_pid{}_{}",
        std::process::id(),
        nanos
    ))
}

#[test]
fn layout_fast_path_min_check_passes() {
    let out_dir = tmp_out_dir("layout_fast_path_min_pass");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                { "frame_id": 0, "debug": { "stats": { "layout_fast_path_taken": false } } },
                { "frame_id": 1, "debug": { "stats": { "layout_fast_path_taken": true } } }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_layout_fast_path_min(&bundle_path, &out_dir, 1, 0)
        .expect("expected layout fast-path >= 1");
    assert!(out_dir.join("check.layout_fast_path_min.json").is_file());
}

#[test]
fn layout_fast_path_min_check_fails_when_missing() {
    let out_dir = tmp_out_dir("layout_fast_path_min_fail");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                { "frame_id": 0, "debug": { "stats": { "layout_fast_path_taken": false } } },
                { "frame_id": 1, "debug": { "stats": { "layout_fast_path_taken": false } } }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_layout_fast_path_min(&bundle_path, &out_dir, 1, 0)
        .expect_err("expected fast-path < 1");
    assert!(err.contains("layout fast-path gate failed"));
    assert!(out_dir.join("check.layout_fast_path_min.json").is_file());
}

#[test]
fn vlist_policy_key_stable_check_passes() {
    let out_dir = tmp_out_dir("vlist_policy_key_stable_pass");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                {
                    "frame_id": 1,
                    "debug": { "virtual_list_windows": [{ "node": 10, "element": 20, "policy_key": 7 }] }
                },
                {
                    "frame_id": 2,
                    "debug": { "virtual_list_windows": [{ "node": 10, "element": 20, "policy_key": 7 }] }
                }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_vlist_policy_key_stable(&bundle_path, &out_dir, 0)
        .expect("expected stable vlist policy_key");
    assert!(out_dir.join("check.vlist_policy_key_stable.json").is_file());
}

#[test]
fn vlist_policy_key_stable_check_fails_when_changed() {
    let out_dir = tmp_out_dir("vlist_policy_key_stable_fail");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                {
                    "frame_id": 1,
                    "debug": { "virtual_list_windows": [{ "node": 10, "element": 20, "policy_key": 7 }] }
                },
                {
                    "frame_id": 2,
                    "debug": { "virtual_list_windows": [{ "node": 10, "element": 20, "policy_key": 9 }] }
                }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_vlist_policy_key_stable(&bundle_path, &out_dir, 0)
        .expect_err("expected unstable vlist policy_key");
    assert!(err.contains("vlist policy-key stability gate failed"));
    assert!(out_dir.join("check.vlist_policy_key_stable.json").is_file());
}

#[test]
fn windowed_rows_offset_changes_min_check_passes() {
    let out_dir = tmp_out_dir("windowed_rows_offset_changes_min_pass");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                { "frame_id": 0, "tick_id": 0, "debug": { "scroll_handle_changes": [], "windowed_rows_surfaces": [] } },
                {
                    "frame_id": 1,
                    "tick_id": 1,
                    "debug": {
                        "scroll_handle_changes": [{ "offset_changed": true }],
                        "windowed_rows_surfaces": [
                            {
                                "callsite_id": 7,
                                "offset_y": 0.0,
                                "location": { "file": "x.rs", "line": 1, "column": 1 }
                            }
                        ]
                    }
                },
                {
                    "frame_id": 2,
                    "tick_id": 2,
                    "debug": {
                        "scroll_handle_changes": [{ "offset_changed": true }],
                        "windowed_rows_surfaces": [
                            {
                                "callsite_id": 7,
                                "offset_y": 10.0,
                                "location": { "file": "x.rs", "line": 1, "column": 1 }
                            }
                        ]
                    }
                }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_windowed_rows_offset_changes_min(&bundle_path, &out_dir, 1, 0, 0.5)
        .expect("expected windowed rows offset changes >= 1");
    assert!(
        out_dir
            .join("check.windowed_rows_offset_changes_min.json")
            .is_file()
    );
}

#[test]
fn windowed_rows_offset_changes_min_check_fails_when_offset_is_stable() {
    let out_dir = tmp_out_dir("windowed_rows_offset_changes_min_fail");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                {
                    "frame_id": 1,
                    "tick_id": 1,
                    "debug": {
                        "scroll_handle_changes": [{ "offset_changed": true }],
                        "windowed_rows_surfaces": [
                            {
                                "callsite_id": 7,
                                "offset_y": 0.0,
                                "location": { "file": "x.rs", "line": 1, "column": 1 }
                            }
                        ]
                    }
                },
                {
                    "frame_id": 2,
                    "tick_id": 2,
                    "debug": {
                        "scroll_handle_changes": [{ "offset_changed": true }],
                        "windowed_rows_surfaces": [
                            {
                                "callsite_id": 7,
                                "offset_y": 0.0,
                                "location": { "file": "x.rs", "line": 1, "column": 1 }
                            }
                        ]
                    }
                }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_windowed_rows_offset_changes_min(&bundle_path, &out_dir, 1, 0, 0.5)
        .expect_err("expected offset changes < 1");
    assert!(err.contains("total_offset_changes"));
    assert!(
        out_dir
            .join("check.windowed_rows_offset_changes_min.json")
            .is_file()
    );
}

#[test]
fn windowed_rows_visible_start_repaint_gate_passes_when_scene_fingerprint_changes() {
    let out_dir = tmp_out_dir("windowed_rows_visible_start_repaint_pass");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                {
                    "frame_id": 1,
                    "tick_id": 1,
                    "scene_fingerprint": 1,
                    "debug": {
                        "scroll_handle_changes": [{ "offset_changed": true }],
                        "windowed_rows_surfaces": [
                            {
                                "callsite_id": 7,
                                "offset_y": 0.0,
                                "visible_start": 0,
                                "location": { "file": "x.rs", "line": 1, "column": 1 }
                            }
                        ]
                    }
                },
                {
                    "frame_id": 2,
                    "tick_id": 2,
                    "scene_fingerprint": 2,
                    "debug": {
                        "scroll_handle_changes": [{ "offset_changed": true }],
                        "windowed_rows_surfaces": [
                            {
                                "callsite_id": 7,
                                "offset_y": 10.0,
                                "visible_start": 10,
                                "location": { "file": "x.rs", "line": 1, "column": 1 }
                            }
                        ]
                    }
                }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_windowed_rows_visible_start_changes_repainted_json(
        &bundle,
        &bundle_path,
        &out_dir,
        0,
    )
    .expect("expected repaint on visible_start changes");
    assert!(
        out_dir
            .join("check.windowed_rows_visible_start_changes_repainted.json")
            .is_file()
    );
}

#[test]
fn windowed_rows_visible_start_repaint_gate_fails_when_scene_fingerprint_is_stale() {
    let out_dir = tmp_out_dir("windowed_rows_visible_start_repaint_fail");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                {
                    "frame_id": 1,
                    "tick_id": 1,
                    "scene_fingerprint": 1,
                    "debug": {
                        "scroll_handle_changes": [{ "offset_changed": true }],
                        "windowed_rows_surfaces": [
                            {
                                "callsite_id": 7,
                                "offset_y": 0.0,
                                "visible_start": 0,
                                "location": { "file": "x.rs", "line": 1, "column": 1 }
                            }
                        ]
                    }
                },
                {
                    "frame_id": 2,
                    "tick_id": 2,
                    "scene_fingerprint": 1,
                    "debug": {
                        "scroll_handle_changes": [{ "offset_changed": true }],
                        "windowed_rows_surfaces": [
                            {
                                "callsite_id": 7,
                                "offset_y": 10.0,
                                "visible_start": 10,
                                "location": { "file": "x.rs", "line": 1, "column": 1 }
                            }
                        ]
                    }
                }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_windowed_rows_visible_start_changes_repainted_json(
        &bundle,
        &bundle_path,
        &out_dir,
        0,
    )
    .expect_err("expected stale fingerprint failure");
    assert!(err.contains("windowed rows repaint gate failed"));
    assert!(
        out_dir
            .join("check.windowed_rows_visible_start_changes_repainted.json")
            .is_file()
    );
}

#[test]
fn wheel_scroll_hit_changes_check_passes_when_offset_changes() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "events": [{ "kind": "pointer.wheel", "frame_id": 1 }],
            "snapshots": [
                {
                    "frame_id": 0,
                    "debug": {
                        "hit_test": { "hit": 2 },
                        "semantics": { "nodes": [
                            { "id": 1, "test_id": "root" },
                            { "id": 2, "parent": 1 }
                        ]},
                        "virtual_list_windows": [{ "offset": 0.0 }]
                    }
                },
                {
                    "frame_id": 1,
                    "debug": {
                        "hit_test": { "hit": 2 },
                        "semantics": { "nodes": [
                            { "id": 1, "test_id": "root" },
                            { "id": 2, "parent": 1 }
                        ]},
                        "virtual_list_windows": [{ "offset": 12.0 }]
                    }
                }
            ]
        }]
    });

    check_bundle_for_wheel_scroll_hit_changes_json(&bundle, Path::new("bundle.json"), "root", 0)
        .expect("expected wheel scroll to change offset");
}

#[test]
fn wheel_scroll_hit_changes_check_fails_when_hit_and_offset_are_stable() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "events": [{ "kind": "pointer.wheel", "frame_id": 1 }],
            "snapshots": [
                {
                    "frame_id": 0,
                    "debug": {
                        "hit_test": { "hit": 2 },
                        "semantics": { "nodes": [
                            { "id": 1, "test_id": "root" },
                            { "id": 2, "parent": 1 }
                        ]},
                        "virtual_list_windows": [{ "offset": 0.0 }]
                    }
                },
                {
                    "frame_id": 1,
                    "debug": {
                        "hit_test": { "hit": 2 },
                        "semantics": { "nodes": [
                            { "id": 1, "test_id": "root" },
                            { "id": 2, "parent": 1 }
                        ]},
                        "virtual_list_windows": [{ "offset": 0.0 }]
                    }
                }
            ]
        }]
    });

    let err = check_bundle_for_wheel_scroll_hit_changes_json(
        &bundle,
        Path::new("bundle.json"),
        "root",
        0,
    )
    .expect_err("expected wheel scroll check to fail when stable");
    assert!(err.contains("wheel scroll hit-change check failed"));
    assert!(err.contains("error=hit_did_not_change"));
}

fn write_png_solid(path: &std::path::Path, w: u32, h: u32, rgba: [u8; 4]) {
    let _ = std::fs::create_dir_all(
        path.parent()
            .expect("png output must have a parent directory"),
    );
    let mut img = image::RgbaImage::new(w, h);
    for p in img.pixels_mut() {
        *p = image::Rgba(rgba);
    }
    img.save(path).expect("png save should succeed");
}

#[derive(Debug, Clone, Copy)]
struct RectF {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn write_bundle_with_bounds(
    out_dir: &std::path::Path,
    bundle_dir_name: &str,
    window: u64,
    tick_id: u64,
    frame_id: u64,
    test_id: &str,
    bounds: RectF,
) {
    let path = out_dir.join(bundle_dir_name).join("bundle.json");
    let _ = std::fs::create_dir_all(
        path.parent()
            .expect("bundle output must have a parent directory"),
    );

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": window,
            "snapshots": [{
                "tick_id": tick_id,
                "frame_id": frame_id,
                "debug": {
                    "semantics": { "nodes": [{
                        "id": 1,
                        "test_id": test_id,
                        "bounds": { "x": bounds.x, "y": bounds.y, "w": bounds.w, "h": bounds.h }
                    }]}
                }
            }]
        }]
    });

    std::fs::write(&path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");
}

fn write_bundle_v2_table_only_with_bounds(
    out_dir: &std::path::Path,
    bundle_dir_name: &str,
    window: u64,
    tick_id: u64,
    frame_id: u64,
    test_id: &str,
    bounds: RectF,
) {
    let path = out_dir.join(bundle_dir_name).join("bundle.json");
    let _ = std::fs::create_dir_all(
        path.parent()
            .expect("bundle output must have a parent directory"),
    );

    let semantics_fingerprint = 1u64;
    let bundle = json!({
        "schema_version": 2,
        "windows": [{
            "window": window,
            "snapshots": [{
                "window": window,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "semantics_fingerprint": semantics_fingerprint,
                "debug": {}
            }]
        }],
        "tables": {
            "semantics": {
                "schema_version": 1,
                "entries": [{
                    "window": window,
                    "semantics_fingerprint": semantics_fingerprint,
                    "semantics": {
                        "nodes": [{
                            "id": 1,
                            "test_id": test_id,
                            "bounds": { "x": bounds.x, "y": bounds.y, "w": bounds.w, "h": bounds.h }
                        }]
                    }
                }]
            }
        }
    });

    std::fs::write(&path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");
}

#[test]
fn gc_sweep_liveness_writes_evidence_json_on_failure() {
    let out_dir = tmp_out_dir("gc_sweep_liveness_evidence");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "removed_subtrees": [{
                        "root": 10,
                        "unreachable_from_liveness_roots": false,
                        "reachable_from_layer_roots": true,
                        "reachable_from_view_cache_roots": true,
                        "root_layer_visible": true,
                        "liveness_layer_roots_len": 2,
                        "view_cache_reuse_roots_len": 1,
                        "view_cache_reuse_root_nodes_len": 1,
                        "root_element_path": "root[demo].overlay",
                        "trigger_element_path": "root[demo].trigger"
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_gc_sweep_liveness(&bundle_path, 0).unwrap_err();
    assert!(err.contains("GC sweep liveness violation"));

    let evidence_path = bundle_dir.join("check.gc_sweep_liveness.json");
    assert!(
        evidence_path.is_file(),
        "expected gc sweep liveness evidence JSON to be written"
    );

    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("gc_sweep_liveness")
    );
    assert_eq!(
        evidence
            .get("removed_subtrees_offenders")
            .and_then(|v| v.as_u64()),
        Some(1)
    );
    assert!(
        evidence
            .get("offender_samples")
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty()),
        "expected offender_samples to be populated"
    );
}

#[test]
fn gc_sweep_liveness_fails_on_keep_alive_mismatch_under_reuse() {
    let out_dir = tmp_out_dir("gc_sweep_liveness_keep_alive_mismatch");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "removed_subtrees": [{
                        "root": 10,
                        "unreachable_from_liveness_roots": true,
                        "reachable_from_layer_roots": false,
                        "reachable_from_view_cache_roots": false,
                        "root_layer_visible": false,
                        "view_cache_reuse_roots_len": 1,
                        "trigger_element_in_view_cache_keep_alive": true,
                        "root_element_path": "root[demo].overlay",
                        "trigger_element_path": "root[demo].trigger"
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_gc_sweep_liveness(&bundle_path, 0).unwrap_err();
    assert!(err.contains("GC sweep liveness violation"));

    let evidence_path = bundle_dir.join("check.gc_sweep_liveness.json");
    assert!(
        evidence_path.is_file(),
        "expected gc sweep liveness evidence JSON to be written"
    );

    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("gc_sweep_liveness")
    );
    assert!(
        evidence
            .get("offender_taxonomy_counts")
            .and_then(|v| v.get("keep_alive_liveness_mismatch"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        "expected keep_alive_liveness_mismatch to be counted"
    );
}

#[test]
fn notify_hotspots_writes_evidence_json_on_failure() {
    let out_dir = tmp_out_dir("notify_hotspots_evidence");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "notify_requests": [{
                        "frame_id": 1,
                        "caller_node": 100,
                        "target_view": 200,
                        "file": "crates/fret-ui/src/declarative/host_widget/event/pressable.rs",
                        "line": 123,
                        "column": 9
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err =
        check_bundle_for_notify_hotspot_file_max(&bundle_path, "pressable.rs", 0, 0).unwrap_err();
    assert!(err.contains("notify hotspot file budget exceeded"));

    let evidence_path = bundle_dir.join("check.notify_hotspots.json");
    assert!(
        evidence_path.is_file(),
        "expected notify hotspots evidence JSON to be written"
    );
    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("notify_hotspots")
    );
}

#[test]
fn gc_sweep_liveness_fails_on_unmapped_view_cache_reuse_roots() {
    let out_dir = tmp_out_dir("gc_sweep_liveness_reuse_roots_unmapped");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "removed_subtrees": [{
                        "root": 10,
                        "unreachable_from_liveness_roots": true,
                        "reachable_from_layer_roots": false,
                        "reachable_from_view_cache_roots": false,
                        "root_layer_visible": false,
                        "view_cache_reuse_roots_len": 1,
                        "view_cache_reuse_root_nodes_len": 0,
                        "root_element_path": "root[demo].overlay",
                        "trigger_element_path": "root[demo].trigger"
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_gc_sweep_liveness(&bundle_path, 0).unwrap_err();
    assert!(err.contains("GC sweep liveness violation"));

    let evidence_path = bundle_dir.join("check.gc_sweep_liveness.json");
    assert!(
        evidence_path.is_file(),
        "expected gc sweep liveness evidence JSON to be written"
    );

    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("gc_sweep_liveness")
    );
    assert!(
        evidence
            .get("offender_taxonomy_counts")
            .and_then(|v| v.get("reuse_roots_unmapped"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        "expected reuse_roots_unmapped to be counted"
    );
}

#[test]
fn vlist_window_shifts_explainable_writes_evidence_json_on_failure() {
    let out_dir = tmp_out_dir("vlist_window_shifts_explainable_evidence");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "virtual_list_windows": [{
                        "node": 10,
                        "element": 1,
                        "window_mismatch": true,
                        "window_shift_kind": "escape"
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err =
        check_bundle_for_vlist_window_shifts_explainable(&bundle_path, &bundle_dir, 0).unwrap_err();
    assert!(err.contains("vlist window-shift explainability gate failed"));

    let evidence_path = bundle_dir.join("check.vlist_window_shifts_explainable.json");
    assert!(
        evidence_path.is_file(),
        "expected vlist window-shift evidence JSON to be written"
    );

    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("vlist_window_shifts_explainable")
    );
    assert_eq!(evidence.get("offenders").and_then(|v| v.as_u64()), Some(1));
    assert!(
        evidence
            .get("samples")
            .and_then(|v| v.as_array())
            .is_some_and(|a| !a.is_empty()),
        "expected samples to be populated"
    );
}

#[test]
fn vlist_window_shifts_non_retained_max_writes_evidence_json_on_failure() {
    let out_dir = tmp_out_dir("vlist_window_shifts_non_retained_max_evidence");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "stats": {
                        "virtual_list_window_shifts_total": 1,
                        "virtual_list_window_shifts_non_retained": 1
                    },
                    "virtual_list_window_shift_samples": [{
                        "frame_id": 1,
                        "source": "prepaint",
                        "node": 10,
                        "element": 1,
                        "window_shift_kind": "escape",
                        "window_shift_reason": "scroll_offset",
                        "window_shift_apply_mode": "non_retained_rerender",
                        "window_shift_invalidation_detail": "scroll_handle_escape_window_update"
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err =
        check_bundle_for_vlist_window_shifts_non_retained_max(&bundle_path, &bundle_dir, 0, 0)
            .unwrap_err();
    assert!(err.contains("vlist non-retained window-shift gate failed"));

    let evidence_path = bundle_dir.join("check.vlist_window_shifts_non_retained_max.json");
    assert!(
        evidence_path.is_file(),
        "expected vlist non-retained window-shift evidence JSON to be written"
    );

    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("vlist_window_shifts_non_retained_max")
    );
    assert_eq!(
        evidence
            .get("total_non_retained_shifts")
            .and_then(|v| v.as_u64()),
        Some(1)
    );
}

#[test]
fn vlist_window_shifts_kind_max_writes_evidence_json_on_failure() {
    let out_dir = tmp_out_dir("vlist_window_shifts_kind_max_evidence");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "virtual_list_windows": [{
                        "source": "prepaint",
                        "node": 10,
                        "element": 1,
                        "window_mismatch": false,
                        "window_shift_kind": "prefetch",
                        "window_shift_reason": "scroll_offset",
                        "window_shift_apply_mode": "non_retained_rerender"
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err =
        check_bundle_for_vlist_window_shifts_kind_max(&bundle_path, &bundle_dir, "prefetch", 0, 0)
            .unwrap_err();
    assert!(err.contains("vlist window-shift kind gate failed"));

    let evidence_path = bundle_dir.join("check.vlist_window_shifts_prefetch_max.json");
    assert!(
        evidence_path.is_file(),
        "expected vlist window-shift kind evidence JSON to be written"
    );

    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("vlist_window_shifts_prefetch_max")
    );
    assert_eq!(
        evidence.get("total_kind_shifts").and_then(|v| v.as_u64()),
        Some(1)
    );
}

#[test]
fn vlist_window_shifts_explainable_accepts_viewport_resize_detail() {
    let out_dir = tmp_out_dir("vlist_window_shifts_explainable_viewport_resize");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "virtual_list_windows": [{
                        "node": 10,
                        "element": 1,
                        "window_mismatch": true,
                        "window_shift_kind": "escape",
                        "window_shift_reason": "viewport_resize",
                        "window_shift_apply_mode": "non_retained_rerender",
                        "window_shift_invalidation_detail": "scroll_handle_viewport_resize_window_update"
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_vlist_window_shifts_explainable(&bundle_path, &bundle_dir, 0)
        .expect("expected gate to accept viewport resize mapping");
}

#[test]
fn vlist_window_shifts_explainable_accepts_items_revision_detail() {
    let out_dir = tmp_out_dir("vlist_window_shifts_explainable_items_revision");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "virtual_list_windows": [{
                        "node": 10,
                        "element": 1,
                        "window_mismatch": true,
                        "window_shift_kind": "escape",
                        "window_shift_reason": "items_revision",
                        "window_shift_apply_mode": "non_retained_rerender",
                        "window_shift_invalidation_detail": "scroll_handle_items_revision_window_update"
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_vlist_window_shifts_explainable(&bundle_path, &bundle_dir, 0)
        .expect("expected gate to accept items revision mapping");
}

#[test]
fn prepaint_actions_min_writes_evidence_json_on_failure() {
    let out_dir = tmp_out_dir("prepaint_actions_min_evidence");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "prepaint_actions": []
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_prepaint_actions_min(&bundle_path, &bundle_dir, 1, 0).unwrap_err();
    assert!(err.contains("prepaint actions"));

    let evidence_path = bundle_dir.join("check.prepaint_actions_min.json");
    assert!(
        evidence_path.is_file(),
        "expected prepaint actions min evidence JSON to be written"
    );
    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("prepaint_actions_min")
    );
}

#[test]
fn chart_sampling_window_shifts_min_accepts_matching_action() {
    let out_dir = tmp_out_dir("chart_sampling_window_shifts_min_ok");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "prepaint_actions": [{
                        "node": 10,
                        "kind": "chart_sampling_window_shift",
                        "chart_sampling_window_key": 123
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_chart_sampling_window_shifts_min(&bundle_path, &bundle_dir, 1, 0)
        .expect("expected gate to accept chart sampling action");

    let evidence_path = bundle_dir.join("check.chart_sampling_window_shifts_min.json");
    assert!(
        evidence_path.is_file(),
        "expected chart sampling window shifts evidence JSON to be written"
    );
}

#[test]
fn chart_sampling_window_shifts_min_writes_evidence_json_on_failure() {
    let out_dir = tmp_out_dir("chart_sampling_window_shifts_min_evidence");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "prepaint_actions": []
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_chart_sampling_window_shifts_min(&bundle_path, &bundle_dir, 1, 0)
        .unwrap_err();
    assert!(err.contains("chart sampling window shift"));

    let evidence_path = bundle_dir.join("check.chart_sampling_window_shifts_min.json");
    assert!(
        evidence_path.is_file(),
        "expected chart sampling window shifts evidence JSON to be written"
    );
    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("chart_sampling_window_shifts_min")
    );
}

#[test]
fn node_graph_cull_window_shifts_min_accepts_matching_action() {
    let out_dir = tmp_out_dir("node_graph_cull_window_shifts_min_ok");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "prepaint_actions": [{
                        "node": 10,
                        "kind": "node_graph_cull_window_shift",
                        "node_graph_cull_window_key": 456
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_node_graph_cull_window_shifts_min(&bundle_path, &bundle_dir, 1, 0)
        .expect("expected gate to accept node graph cull action");

    let evidence_path = bundle_dir.join("check.node_graph_cull_window_shifts_min.json");
    assert!(
        evidence_path.is_file(),
        "expected node graph cull window shifts evidence JSON to be written"
    );
}

#[test]
fn node_graph_cull_window_shifts_min_writes_evidence_json_on_failure() {
    let out_dir = tmp_out_dir("node_graph_cull_window_shifts_min_evidence");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "prepaint_actions": []
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_node_graph_cull_window_shifts_min(&bundle_path, &bundle_dir, 1, 0)
        .unwrap_err();
    assert!(err.contains("node graph cull window shift"));

    let evidence_path = bundle_dir.join("check.node_graph_cull_window_shifts_min.json");
    assert!(
        evidence_path.is_file(),
        "expected node graph cull window shifts evidence JSON to be written"
    );
    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("node_graph_cull_window_shifts_min")
    );
}

#[test]
fn node_graph_cull_window_shifts_max_accepts_when_under_budget() {
    let out_dir = tmp_out_dir("node_graph_cull_window_shifts_max_ok");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "prepaint_actions": [{
                        "node": 10,
                        "kind": "node_graph_cull_window_shift",
                        "node_graph_cull_window_key": 456
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_node_graph_cull_window_shifts_max(&bundle_path, &bundle_dir, 1, 0)
        .expect("expected max gate to accept actions under budget");

    let evidence_path = bundle_dir.join("check.node_graph_cull_window_shifts_max.json");
    assert!(
        evidence_path.is_file(),
        "expected node graph cull window shifts max evidence JSON to be written"
    );
}

#[test]
fn node_graph_cull_window_shifts_max_writes_evidence_json_on_failure() {
    let out_dir = tmp_out_dir("node_graph_cull_window_shifts_max_evidence");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "prepaint_actions": [{
                        "node": 10,
                        "kind": "node_graph_cull_window_shift",
                        "node_graph_cull_window_key": 456
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_node_graph_cull_window_shifts_max(&bundle_path, &bundle_dir, 0, 0)
        .unwrap_err();
    assert!(err.contains("node graph cull window shift"));

    let evidence_path = bundle_dir.join("check.node_graph_cull_window_shifts_max.json");
    assert!(
        evidence_path.is_file(),
        "expected node graph cull window shifts max evidence JSON to be written"
    );
    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("node_graph_cull_window_shifts_max")
    );
}

#[test]
fn vlist_window_shifts_have_prepaint_actions_accepts_matching_action() {
    let out_dir = tmp_out_dir("vlist_window_shifts_have_prepaint_actions_ok");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "virtual_list_windows": [{
                        "node": 10,
                        "element": 1,
                        "source": "prepaint",
                        "window_shift_kind": "escape",
                        "window_shift_reason": "viewport_resize"
                    }],
                    "prepaint_actions": [{
                        "kind": "virtual_list_window_shift",
                        "node": 10,
                        "element": 1,
                        "virtual_list_window_shift_kind": "escape",
                        "virtual_list_window_shift_reason": "viewport_resize",
                        "frame_id": 1
                    }]
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_vlist_window_shifts_have_prepaint_actions(&bundle_path, &bundle_dir, 0)
        .expect("expected vlist shift prepaint-action gate to pass");
}

#[test]
fn vlist_window_shifts_have_prepaint_actions_writes_evidence_json_on_failure() {
    let out_dir = tmp_out_dir("vlist_window_shifts_have_prepaint_actions_evidence");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_dir = out_dir.join("run");
    let _ = std::fs::create_dir_all(&bundle_dir);
    let bundle_path = bundle_dir.join("bundle.json");

    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [{
                "tick_id": 1,
                "frame_id": 1,
                "debug": {
                    "virtual_list_windows": [{
                        "node": 10,
                        "element": 1,
                        "source": "prepaint",
                        "window_shift_kind": "escape",
                        "window_shift_reason": "items_revision"
                    }],
                    "prepaint_actions": []
                }
            }]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err =
        check_bundle_for_vlist_window_shifts_have_prepaint_actions(&bundle_path, &bundle_dir, 0)
            .unwrap_err();
    assert!(err.contains("vlist window-shift prepaint-action gate failed"));

    let evidence_path = bundle_dir.join("check.vlist_window_shifts_have_prepaint_actions.json");
    assert!(
        evidence_path.is_file(),
        "expected vlist shift prepaint-action evidence JSON to be written"
    );
    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence_path).unwrap()).unwrap();
    assert_eq!(
        evidence.get("kind").and_then(|v| v.as_str()),
        Some("vlist_window_shifts_have_prepaint_actions")
    );
}

#[test]
fn idle_no_paint_check_passes_when_tail_streak_meets_min() {
    let out_dir = tmp_out_dir("idle_no_paint_pass");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "prepaint_time_us": 1, "paint_time_us": 1, "paint_nodes_performed": 1 } } },
                { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                { "tick_id": 3, "frame_id": 3, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                { "tick_id": 4, "frame_id": 4, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    check_bundle_for_idle_no_paint_min(&bundle_path, &out_dir, 3, 0).unwrap();
    assert!(out_dir.join("check.idle_no_paint.json").is_file());
}

#[test]
fn idle_no_paint_check_fails_when_tail_streak_is_too_small() {
    let out_dir = tmp_out_dir("idle_no_paint_fail");
    let _ = std::fs::create_dir_all(&out_dir);

    let bundle_path = out_dir.join("bundle.json");
    let bundle = json!({
        "schema_version": 1,
        "windows": [{
            "window": 1,
            "snapshots": [
                { "tick_id": 1, "frame_id": 1, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                { "tick_id": 2, "frame_id": 2, "debug": { "stats": { "prepaint_time_us": 0, "paint_time_us": 0, "paint_nodes_performed": 0 } } },
                { "tick_id": 3, "frame_id": 3, "debug": { "stats": { "prepaint_time_us": 1, "paint_time_us": 1, "paint_nodes_performed": 1 } } }
            ]
        }]
    });
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap())
        .expect("bundle.json write should succeed");

    let err = check_bundle_for_idle_no_paint_min(&bundle_path, &out_dir, 2, 0).unwrap_err();
    assert!(err.contains("idle no-paint gate failed"));
    assert!(out_dir.join("check.idle_no_paint.json").is_file());
}

#[test]
fn pixels_changed_check_passes_when_region_hash_changes() {
    let out_dir = tmp_out_dir("pixels_changed_pass");
    let _ = std::fs::create_dir_all(out_dir.join("screenshots"));

    let window = 1u64;
    let test_id = "root";
    let bounds = RectF {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };

    write_bundle_with_bounds(&out_dir, "b0", window, 10, 10, test_id, bounds);
    write_bundle_with_bounds(&out_dir, "b1", window, 20, 20, test_id, bounds);

    write_png_solid(
        &out_dir.join("screenshots").join("b0").join("shot0.png"),
        10,
        10,
        [0, 0, 0, 255],
    );
    write_png_solid(
        &out_dir.join("screenshots").join("b1").join("shot1.png"),
        10,
        10,
        [255, 0, 0, 255],
    );

    let result = json!({
        "schema_version": 1,
        "completed": [
            { "bundle_dir_name": "b0", "file": "shot0.png", "window": window, "tick_id": 10, "frame_id": 10, "scale_factor": 1.0 },
            { "bundle_dir_name": "b1", "file": "shot1.png", "window": window, "tick_id": 20, "frame_id": 20, "scale_factor": 1.0 }
        ]
    });
    std::fs::write(
        out_dir.join("screenshots.result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .expect("screenshots.result.json write should succeed");

    stats::check_out_dir_for_pixels_changed(&out_dir, test_id, 0).unwrap();
    assert!(out_dir.join("check.pixels_changed.json").is_file());
}

#[test]
fn pixels_changed_check_supports_schema_v2_table_only_semantics() {
    let out_dir = tmp_out_dir("pixels_changed_v2_table_only");
    let _ = std::fs::create_dir_all(out_dir.join("screenshots"));

    let window = 1u64;
    let test_id = "root";
    let bounds = RectF {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };

    write_bundle_v2_table_only_with_bounds(&out_dir, "b0", window, 10, 10, test_id, bounds);
    write_bundle_v2_table_only_with_bounds(&out_dir, "b1", window, 20, 20, test_id, bounds);

    write_png_solid(
        &out_dir.join("screenshots").join("b0").join("shot0.png"),
        10,
        10,
        [0, 0, 0, 255],
    );
    write_png_solid(
        &out_dir.join("screenshots").join("b1").join("shot1.png"),
        10,
        10,
        [255, 0, 0, 255],
    );

    let result = json!({
        "schema_version": 1,
        "completed": [
            { "bundle_dir_name": "b0", "file": "shot0.png", "window": window, "tick_id": 10, "frame_id": 10, "scale_factor": 1.0 },
            { "bundle_dir_name": "b1", "file": "shot1.png", "window": window, "tick_id": 20, "frame_id": 20, "scale_factor": 1.0 }
        ]
    });
    std::fs::write(
        out_dir.join("screenshots.result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .expect("screenshots.result.json write should succeed");

    stats::check_out_dir_for_pixels_changed(&out_dir, test_id, 0).unwrap();
    assert!(out_dir.join("check.pixels_changed.json").is_file());
}

#[test]
fn pixels_changed_check_fails_when_region_hash_is_unchanged() {
    let out_dir = tmp_out_dir("pixels_changed_fail");
    let _ = std::fs::create_dir_all(out_dir.join("screenshots"));

    let window = 1u64;
    let test_id = "root";
    let bounds = RectF {
        x: 0.0,
        y: 0.0,
        w: 10.0,
        h: 10.0,
    };

    write_bundle_with_bounds(&out_dir, "b0", window, 10, 10, test_id, bounds);
    write_bundle_with_bounds(&out_dir, "b1", window, 20, 20, test_id, bounds);

    write_png_solid(
        &out_dir.join("screenshots").join("b0").join("shot0.png"),
        10,
        10,
        [0, 0, 0, 255],
    );
    write_png_solid(
        &out_dir.join("screenshots").join("b1").join("shot1.png"),
        10,
        10,
        [0, 0, 0, 255],
    );

    let result = json!({
        "schema_version": 1,
        "completed": [
            { "bundle_dir_name": "b0", "file": "shot0.png", "window": window, "tick_id": 10, "frame_id": 10, "scale_factor": 1.0 },
            { "bundle_dir_name": "b1", "file": "shot1.png", "window": window, "tick_id": 20, "frame_id": 20, "scale_factor": 1.0 }
        ]
    });
    std::fs::write(
        out_dir.join("screenshots.result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .expect("screenshots.result.json write should succeed");

    let err = stats::check_out_dir_for_pixels_changed(&out_dir, test_id, 0).unwrap_err();
    assert!(err.contains("pixels unchanged suspected"));
    assert!(out_dir.join("check.pixels_changed.json").is_file());
}

#[test]
fn perf_threshold_scan_passes_when_under_limits() {
    let failures = scan_perf_threshold_failures(
        "script.json",
        BundleStatsSort::Time,
        compare::PerfThresholdAggregate::Max,
        PerfThresholds {
            max_top_total_us: Some(100),
            max_top_layout_us: Some(80),
            max_top_solve_us: Some(50),
            max_frame_p95_total_us: None,
            max_frame_p95_layout_us: None,
            max_frame_p95_solve_us: None,
            max_pointer_move_dispatch_us: Some(2000),
            max_pointer_move_hit_test_us: Some(1500),
            max_pointer_move_global_changes: Some(1),
            min_run_paint_cache_hit_test_only_replay_allowed_max: None,
            max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: None,
            max_renderer_encode_scene_us: None,
            max_renderer_upload_us: None,
            max_renderer_record_passes_us: None,
            max_renderer_encoder_finish_us: None,
            max_renderer_prepare_text_us: None,
            max_renderer_prepare_svg_us: None,
        },
        PerfThresholds::default(),
        99,
        99,
        99,
        79,
        79,
        79,
        49,
        49,
        49,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        true,
        1999,
        1499,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(failures.is_empty());
}

#[test]
fn perf_threshold_scan_reports_each_exceeded_metric() {
    let failures = scan_perf_threshold_failures(
        "script.json",
        BundleStatsSort::Time,
        compare::PerfThresholdAggregate::Max,
        PerfThresholds {
            max_top_total_us: Some(100),
            max_top_layout_us: Some(80),
            max_top_solve_us: Some(50),
            max_frame_p95_total_us: None,
            max_frame_p95_layout_us: None,
            max_frame_p95_solve_us: None,
            max_pointer_move_dispatch_us: Some(2000),
            max_pointer_move_hit_test_us: Some(1500),
            max_pointer_move_global_changes: Some(1),
            min_run_paint_cache_hit_test_only_replay_allowed_max: None,
            max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max: None,
            max_renderer_encode_scene_us: None,
            max_renderer_upload_us: None,
            max_renderer_record_passes_us: None,
            max_renderer_encoder_finish_us: None,
            max_renderer_prepare_text_us: None,
            max_renderer_prepare_svg_us: None,
        },
        PerfThresholds::default(),
        101,
        101,
        101,
        81,
        81,
        81,
        51,
        51,
        51,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        true,
        2001,
        1501,
        2,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        Some(Path::new("bundle.json")),
        Some(7),
        None,
        None,
        None,
        None,
    );
    assert_eq!(failures.len(), 6);
    for failure in &failures {
        assert_eq!(
            failure
                .get("evidence_bundle")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            "bundle.json"
        );
        assert_eq!(
            failure
                .get("evidence_run_index")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            7
        );
    }
    let metrics: Vec<String> = failures
        .iter()
        .filter_map(|v| {
            v.get("metric")
                .and_then(|m| m.as_str())
                .map(|s| s.to_string())
        })
        .collect();
    assert!(metrics.contains(&"top_total_time_us".to_string()));
    assert!(metrics.contains(&"top_layout_time_us".to_string()));
    assert!(metrics.contains(&"top_layout_engine_solve_time_us".to_string()));
    assert!(metrics.contains(&"pointer_move_max_dispatch_time_us".to_string()));
    assert!(metrics.contains(&"pointer_move_max_hit_test_time_us".to_string()));
    assert!(metrics.contains(&"pointer_move_snapshots_with_global_changes".to_string()));
}

#[test]
fn perf_baseline_headroom_rounds_up() {
    assert_eq!(apply_perf_baseline_headroom(100, 20), 120);
    assert_eq!(apply_perf_baseline_headroom(101, 20), 122);
    assert_eq!(apply_perf_baseline_headroom(0, 20), 0);
}

#[test]
fn perf_baseline_parse_reads_script_thresholds() {
    let out_dir = tmp_out_dir("perf_baseline_parse");
    let _ = std::fs::create_dir_all(&out_dir);
    let path = out_dir.join("perf.baseline.json");

    let v = json!({
        "schema_version": 1,
        "kind": "perf_baseline",
        "rows": [{
            "script": "tools/diag-scripts/ui-gallery-overlay-torture.json",
            "thresholds": {
                "max_top_total_us": 25000,
                "max_top_layout_us": 15000,
                "max_top_solve_us": 8000
            }
        }]
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&v).unwrap())
        .expect("baseline write should succeed");

    let baseline = read_perf_baseline_file(Path::new("."), &path).unwrap();
    let t = baseline
        .thresholds_by_script
        .get("tools/diag-scripts/ui-gallery-overlay-torture.json")
        .copied()
        .unwrap();
    assert_eq!(t.max_top_total_us, Some(25_000));
    assert_eq!(t.max_top_layout_us, Some(15_000));
    assert_eq!(t.max_top_solve_us, Some(8_000));
}

#[test]
fn redraw_hitch_gate_fails_when_log_missing() {
    let out_dir = tmp_out_dir("redraw_hitch_gate_missing");
    let _ = std::fs::create_dir_all(&out_dir);

    let r = check_redraw_hitches_max_total_ms(&out_dir, 16).unwrap();
    assert!(r.failures > 0);

    let v = read_json_value(&out_dir.join("check.redraw_hitches.json")).unwrap();
    assert_eq!(
        v.get("kind").and_then(|v| v.as_str()),
        Some("redraw_hitches_thresholds")
    );
    assert_eq!(
        v.get("observed")
            .and_then(|v| v.get("present"))
            .and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn redraw_hitch_gate_passes_under_threshold() {
    let out_dir = tmp_out_dir("redraw_hitch_gate_pass");
    let _ = std::fs::create_dir_all(&out_dir);
    let log = out_dir.join("redraw_hitches.log");
    std::fs::write(
        &log,
        "\
[1] [thread=ThreadId(1)] redraw hitch window=AppWindowId(1v1) total_ms=10 prepare_ms=Some(0) render_ms=Some(10) record_ms=Some(0) present_ms=Some(0) scene_ops=1 bounds=Rect {} scale_factor=1.0\n\
[2] [thread=ThreadId(1)] redraw hitch window=AppWindowId(1v1) total_ms=12 prepare_ms=Some(0) render_ms=Some(12) record_ms=Some(0) present_ms=Some(0) scene_ops=1 bounds=Rect {} scale_factor=1.0\n",
    )
    .unwrap();

    let r = check_redraw_hitches_max_total_ms(&out_dir, 20).unwrap();
    assert_eq!(r.failures, 0);

    let v = read_json_value(&out_dir.join("check.redraw_hitches.json")).unwrap();
    assert_eq!(
        v.get("failures")
            .and_then(|v| v.as_array())
            .map(|a| a.len()),
        Some(0)
    );
    assert_eq!(
        v.get("observed")
            .and_then(|v| v.get("records"))
            .and_then(|v| v.as_u64()),
        Some(2)
    );
}

#[test]
fn redraw_hitch_gate_fails_over_threshold() {
    let out_dir = tmp_out_dir("redraw_hitch_gate_fail");
    let _ = std::fs::create_dir_all(&out_dir);
    let log = out_dir.join("redraw_hitches.log");
    std::fs::write(
        &log,
        "\
[1] [thread=ThreadId(1)] redraw hitch window=AppWindowId(1v1) total_ms=30 prepare_ms=Some(0) render_ms=Some(29) record_ms=Some(0) present_ms=Some(1) scene_ops=1 bounds=Rect {} scale_factor=1.0\n",
    )
    .unwrap();

    let r = check_redraw_hitches_max_total_ms(&out_dir, 20).unwrap();
    assert_eq!(r.failures, 1);

    let v = read_json_value(&out_dir.join("check.redraw_hitches.json")).unwrap();
    let failures = v.get("failures").and_then(|v| v.as_array()).unwrap();
    assert!(
        failures
            .iter()
            .any(|f| f.get("kind").and_then(|v| v.as_str()) == Some("max_total_ms"))
    );
}

#[test]
fn redraw_hitch_gate_parses_tick_and_frame_ids() {
    let out_dir = tmp_out_dir("redraw_hitch_gate_tick_frame");
    let _ = std::fs::create_dir_all(&out_dir);
    let log = out_dir.join("redraw_hitches.log");
    std::fs::write(
        &log,
        "\
[1] [thread=ThreadId(1)] redraw hitch window=AppWindowId(1v1) tick_id=7 frame_id=9 total_ms=30 prepare_ms=Some(0) render_ms=Some(29) record_ms=Some(0) present_ms=Some(1) scene_ops=1 bounds=Rect {} scale_factor=1.0\n",
    )
    .unwrap();

    let _ = check_redraw_hitches_max_total_ms(&out_dir, 10).unwrap();
    let v = read_json_value(&out_dir.join("check.redraw_hitches.json")).unwrap();
    let top = v.get("top").and_then(|v| v.as_array()).unwrap();
    assert_eq!(
        top.first()
            .and_then(|t| t.get("tick_id"))
            .and_then(|v| v.as_u64()),
        Some(7)
    );
    assert_eq!(
        top.first()
            .and_then(|t| t.get("frame_id"))
            .and_then(|v| v.as_u64()),
        Some(9)
    );
}

#[test]
fn ui_gallery_code_editor_undo_redo_gate_passes_on_marker_toggle_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "code_editor": { "torture": {
                                "marker_present": false,
                                "text_len_bytes": 10,
                                "selection": { "anchor": 0, "caret": 0 }
                            }}
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "code_editor": { "torture": {
                                "marker_present": true,
                                "text_len_bytes": 30,
                                "selection": { "anchor": 0, "caret": 5 }
                            }}
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "code_editor": { "torture": {
                                "marker_present": false,
                                "text_len_bytes": 10,
                                "selection": { "anchor": 0, "caret": 5 }
                            }}
                        }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "code_editor": { "torture": {
                                "marker_present": true,
                                "text_len_bytes": 30,
                                "selection": { "anchor": 0, "caret": 5 }
                            }}
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_undo_redo_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo_json(&bundle, &bundle_path, 0)
        .unwrap();
}

#[test]
fn ui_gallery_code_editor_undo_redo_gate_fails_without_redo() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "code_editor": { "torture": {
                                "marker_present": true,
                                "text_len_bytes": 30,
                                "selection": { "anchor": 0, "caret": 5 }
                            }}
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "code_editor": { "torture": {
                                "marker_present": false,
                                "text_len_bytes": 10,
                                "selection": { "anchor": 0, "caret": 5 }
                            }}
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_undo_redo_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap_err();
    assert!(err.contains("undo/redo gate failed"));
}

#[test]
fn ui_gallery_code_editor_read_only_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": { "torture": {
                                "interaction": { "enabled": true, "editable": true },
                                "buffer_revision": 1,
                                "text_len_bytes": 5
                            }}
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": { "torture": {
                                "interaction": { "enabled": true, "editable": true },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": { "torture": {
                                "interaction": { "enabled": true, "editable": false },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": { "torture": {
                                "interaction": { "enabled": true, "editable": false },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    },
                    {
                        "tick_id": 5,
                        "frame_id": 5,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": { "torture": {
                                "interaction": { "enabled": true, "editable": false },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_read_only_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_code_editor_read_only_gate_fails_when_mutated() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": { "torture": {
                                "interaction": { "enabled": true, "editable": true },
                                "buffer_revision": 1,
                                "text_len_bytes": 5
                            }}
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": { "torture": {
                                "interaction": { "enabled": true, "editable": true },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": { "torture": {
                                "interaction": { "enabled": true, "editable": false },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": { "torture": {
                                "interaction": { "enabled": true, "editable": false },
                                "buffer_revision": 3,
                                "text_len_bytes": 7
                            }}
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_read_only_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap_err();
    assert!(err.contains("read-only gate failed"));
}

#[test]
fn ui_gallery_markdown_editor_read_only_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": true, "editable": true },
                                "buffer_revision": 1,
                                "text_len_bytes": 5
                            }}
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": true, "editable": true },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": true, "editable": false },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": true, "editable": false },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    },
                    {
                        "tick_id": 5,
                        "frame_id": 5,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": true, "editable": false },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_read_only_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_markdown_editor_disabled_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": true, "editable": true },
                                "buffer_revision": 7,
                                "text_len_bytes": 42,
                                "selection": { "caret": 3 }
                            }}
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [3,3] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": false, "editable": false },
                                "buffer_revision": 7,
                                "text_len_bytes": 42,
                                "selection": { "caret": 3 }
                            }}
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": false }, "text_selection": [3,3] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": false, "editable": false },
                                "buffer_revision": 7,
                                "text_len_bytes": 42,
                                "selection": { "caret": 3 }
                            }}
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": false }, "text_selection": [3,3] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": false, "editable": false },
                                "buffer_revision": 7,
                                "text_len_bytes": 42,
                                "selection": { "caret": 3 }
                            }}
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": false }, "text_selection": [3,3] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_disabled_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_markdown_editor_read_only_gate_fails_when_mutated() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": true, "editable": true },
                                "buffer_revision": 1,
                                "text_len_bytes": 5
                            }}
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": true, "editable": true },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": true, "editable": false },
                                "buffer_revision": 2,
                                "text_len_bytes": 6
                            }}
                        }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "markdown_editor_source": {
                                "interaction": { "enabled": true, "editable": false },
                                "buffer_revision": 3,
                                "text_len_bytes": 7
                            }}
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_read_only_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap_err();
    assert!(err.contains("read-only gate failed"));
}

#[test]
fn ui_gallery_markdown_editor_soft_wrap_toggle_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_soft_wrap_toggle_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_markdown_editor_soft_wrap_toggle_gate_fails_when_caret_moves() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 6 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 6 }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_soft_wrap_toggle_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap_err();
    assert!(err.contains("soft-wrap toggle gate failed"));
}

#[test]
fn ui_gallery_markdown_editor_folds_toggle_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": true,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_folds_toggle_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_markdown_editor_folds_toggle_gate_fails_when_rev_changes() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": true,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 3,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 3,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_folds_toggle_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap_err();
    assert!(err.contains("folds toggle gate failed"));
}

#[test]
fn ui_gallery_markdown_editor_folds_clamp_selection_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 },
                                    "folds": {
                                        "fixture_span_line0": { "start": 3, "end": 9 }
                                    }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": true,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 3 },
                                    "folds": {
                                        "fixture_span_line0": { "start": 3, "end": 9 },
                                        "line0_placeholder_present": true
                                    }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_folds_clamp_selection_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_markdown_editor_folds_clamp_selection_gate_fails_when_caret_stays_inside() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 },
                                    "folds": {
                                        "fixture_span_line0": { "start": 3, "end": 9 }
                                    }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": true,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 },
                                    "folds": {
                                        "fixture_span_line0": { "start": 3, "end": 9 },
                                        "line0_placeholder_present": true
                                    }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_folds_clamp_selection_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err =
        check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
    assert!(err.contains("clamp-selection gate failed"));
}

#[test]
fn ui_gallery_markdown_editor_inlays_toggle_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "inlays_fixture": false,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "inlays_fixture": true,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "inlays_fixture": false,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_inlays_toggle_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_markdown_editor_inlays_toggle_gate_fails_when_caret_moves() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "inlays_fixture": false,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 5 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "inlays_fixture": true,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 6 }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "inlays_fixture": false,
                                "markdown_editor_source": {
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 10,
                                    "selection": { "caret": 6 }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_inlays_toggle_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap_err();
    assert!(err.contains("inlays toggle gate failed"));
}

#[test]
fn ui_gallery_markdown_editor_inlays_caret_navigation_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "inlays_fixture": false,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 5,
                                    "selection": { "anchor": 2, "caret": 2 },
                                    "inlays": { "enabled": false, "line0_present": false }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "inlays_fixture": true,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 5,
                                    "selection": { "anchor": 2, "caret": 2 },
                                    "inlays": { "enabled": true, "line0_present": true }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "inlays_fixture": true,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 5,
                                    "selection": { "anchor": 3, "caret": 3 },
                                    "inlays": { "enabled": true, "line0_present": true }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "inlays_fixture": true,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 5,
                                    "selection": { "anchor": 2, "caret": 2 },
                                    "inlays": { "enabled": true, "line0_present": true }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_inlays_caret_navigation_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_markdown_editor_inlays_caret_navigation_gate_fails_when_caret_does_not_move() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "inlays_fixture": false,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 5,
                                    "selection": { "anchor": 2, "caret": 2 },
                                    "inlays": { "enabled": false, "line0_present": false }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "inlays_fixture": true,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 5,
                                    "selection": { "anchor": 2, "caret": 2 },
                                    "inlays": { "enabled": true, "line0_present": true }
                                }
                            }
                        }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": {
                                "soft_wrap_cols": null,
                                "folds_fixture": false,
                                "inlays_fixture": true,
                                "markdown_editor_source": {
                                    "preedit_active": false,
                                    "interaction": { "enabled": true, "editable": true },
                                    "buffer_revision": 2,
                                    "text_len_bytes": 5,
                                    "selection": { "anchor": 2, "caret": 2 },
                                    "inlays": { "enabled": true, "line0_present": true }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_inlays_caret_navigation_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err =
        check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable_json(
            &bundle,
            &bundle_path,
            0,
        )
        .unwrap_err();
    assert!(err.contains("caret-navigation gate failed"));
}

#[test]
fn ui_gallery_markdown_editor_word_boundary_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,0] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [5,5] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_word_boundary_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_word_boundary_json(&bundle, &bundle_path, 0)
        .unwrap();
}

#[test]
fn ui_gallery_web_ime_bridge_gate_passes_when_enabled_and_cursor_area_seen() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": { "kind": "fret_ui_gallery", "selected_page": "markdown_editor_source" },
                        "debug": { "web_ime_bridge": {
                            "enabled": true,
                            "mount_kind": "body",
                            "position_mode": "fixed",
                            "textarea_has_focus": true,
                            "cursor_area_set_seen": 1,
                            "last_cursor_area": { "origin": { "x": 0.0, "y": 0.0 }, "size": { "width": 1.0, "height": 1.0 } }
                        } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_web_ime_bridge_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_web_ime_bridge_enabled_json(&bundle, &bundle_path, 0).unwrap();
}

#[test]
fn ui_gallery_web_ime_bridge_gate_fails_when_disabled() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": { "kind": "fret_ui_gallery", "selected_page": "markdown_editor_source" },
                        "debug": { "web_ime_bridge": {
                            "enabled": false,
                            "mount_kind": "body",
                            "position_mode": "fixed",
                            "textarea_has_focus": false,
                            "cursor_area_set_seen": 0,
                            "last_cursor_area": null
                        } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_web_ime_bridge_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_web_ime_bridge_enabled_json(&bundle, &bundle_path, 0)
        .unwrap_err();
    assert!(err.contains("ui-gallery web-ime bridge gate failed"));
}

#[test]
fn devtools_sanitize_export_dir_name_takes_file_name() {
    assert_eq!(
        devtools_sanitize_export_dir_name("1700000-bundle"),
        "1700000-bundle"
    );
    assert_eq!(devtools_sanitize_export_dir_name("a/b/c"), "c");
    assert_eq!(devtools_sanitize_export_dir_name(""), "bundle");
}

#[test]
fn devtools_select_session_id_prefers_single_web_app_when_multiple() {
    let list = DevtoolsSessionListV1 {
        sessions: vec![
            DevtoolsSessionDescriptorV1 {
                session_id: "s-native".to_string(),
                client_kind: "native_app".to_string(),
                client_version: "1".to_string(),
                capabilities: Vec::new(),
            },
            DevtoolsSessionDescriptorV1 {
                session_id: "s-web".to_string(),
                client_kind: "web_app".to_string(),
                client_version: "1".to_string(),
                capabilities: Vec::new(),
            },
        ],
    };
    assert_eq!(
        devtools_select_session_id(&list, None).unwrap(),
        "s-web".to_string()
    );
}

#[test]
fn devtools_select_session_id_requires_explicit_when_ambiguous() {
    let list = DevtoolsSessionListV1 {
        sessions: vec![
            DevtoolsSessionDescriptorV1 {
                session_id: "s1".to_string(),
                client_kind: "native_app".to_string(),
                client_version: "1".to_string(),
                capabilities: Vec::new(),
            },
            DevtoolsSessionDescriptorV1 {
                session_id: "s2".to_string(),
                client_kind: "native_app".to_string(),
                client_version: "1".to_string(),
                capabilities: Vec::new(),
            },
        ],
    };
    let err = devtools_select_session_id(&list, None).unwrap_err();
    assert!(err.contains("multiple DevTools sessions available"));
}

#[test]
fn devtools_select_session_id_rejects_tooling_only_sessions() {
    let list = DevtoolsSessionListV1 {
        sessions: vec![DevtoolsSessionDescriptorV1 {
            session_id: "s-tooling".to_string(),
            client_kind: "tooling".to_string(),
            client_version: "1".to_string(),
            capabilities: Vec::new(),
        }],
    };
    let err = devtools_select_session_id(&list, None).unwrap_err();
    assert!(err.contains("no DevTools app sessions available"));
}

#[test]
fn ui_gallery_markdown_editor_line_boundary_triple_click_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "hello\nworld\n", "text_selection": [0,0] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "hello\nworld\n", "text_selection": [0,6] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_line_boundary_triple_click_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_markdown_editor_a11y_composition_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [4,4], "text_composition": [2,4] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_a11y_composition_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_markdown_editor_a11y_composition_soft_wrap_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "soft_wrap_cols": 80 }
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "soft_wrap_cols": 80 }
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [4,4], "text_composition": [2,4] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "soft_wrap_cols": 80 }
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_a11y_composition_soft_wrap_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_markdown_editor_soft_wrap_editing_gate_passes_on_sequence() {
    let value_a = "a".repeat(100);
    let value_b = "a".repeat(101);

    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "soft_wrap_cols": 80 }
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": value_a, "text_selection": [0,0] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "soft_wrap_cols": 80 }
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": value_a, "text_selection": [80,80] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "soft_wrap_cols": 80 }
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": value_b, "text_selection": [81,81] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "soft_wrap_cols": 80 }
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": value_b, "text_selection": [0,0] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 5,
                        "frame_id": 5,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "markdown_editor_source",
                            "code_editor": { "soft_wrap_cols": 80 }
                        },
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": value_b, "text_selection": [80,80] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-markdown-editor-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_markdown_editor_soft_wrap_editing_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_code_editor_a11y_selection_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,0] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,5] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [11,11] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [11,0] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_selection_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_a11y_selection_json(&bundle, &bundle_path, 0).unwrap();
}

#[test]
fn ui_gallery_code_editor_a11y_selection_gate_fails_without_select_all() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,0] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,5] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [11,11] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-gate-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_selection_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_code_editor_a11y_selection_json(&bundle, &bundle_path, 0)
        .unwrap_err();
    assert!(err.contains("a11y-selection gate failed"));
}

#[test]
fn ui_gallery_code_editor_a11y_composition_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [4,4], "text_composition": [2,4] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2], "text_composition": [0,2] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 5,
                        "frame_id": 5,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [0,5] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-gate-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_composition_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_a11y_composition_json(&bundle, &bundle_path, 0)
        .unwrap();
}

#[test]
fn ui_gallery_code_editor_a11y_composition_gate_fails_without_preedit() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [2,2] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-gate-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_composition_gate_fails");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err =
        check_bundle_for_ui_gallery_code_editor_a11y_composition_json(&bundle, &bundle_path, 0)
            .unwrap_err();
    assert!(err.contains("a11y-composition gate failed"));
}

#[test]
fn ui_gallery_code_editor_a11y_selection_wrap_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "<redacted len=200>", "text_selection": [0,0] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "<redacted len=200>", "text_selection": [80,80] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "<redacted len=200>", "text_selection": [200,200] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 4,
                        "frame_id": 4,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "value": "<redacted len=200>", "text_selection": [200,0] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_selection_wrap_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap_json(&bundle, &bundle_path, 0)
        .unwrap();
}

#[test]
fn ui_gallery_code_editor_a11y_composition_wrap_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [78,78] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-wrap-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [80,80], "text_composition": [78,80] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-wrap-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [78,78] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-wrap-gate-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_composition_wrap_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_json(&bundle, &bundle_path, 0)
        .unwrap();
}

#[test]
fn ui_gallery_code_editor_a11y_composition_drag_gate_passes_on_sequence() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [78,78] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-drag-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 2,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [80,80], "text_composition": [78,80] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-drag-gate-viewport", "parent": 2 }
                        ] } }
                    },
                    {
                        "tick_id": 3,
                        "frame_id": 3,
                        "debug": { "semantics": { "nodes": [
                            { "id": 2, "role": "text_field", "flags": { "focused": true }, "text_selection": [10,0] },
                            { "id": 3, "role": "viewport", "test_id": "ui-gallery-code-editor-a11y-composition-drag-gate-viewport", "parent": 2 }
                        ] } }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir("ui_gallery_code_editor_a11y_composition_drag_gate_passes");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_a11y_composition_drag_json(&bundle, &bundle_path, 0)
        .unwrap();
}

#[test]
fn ui_gallery_code_editor_folds_inline_preedit_unwrapped_gate_passes_when_placeholder_present() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "folds_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "folds": { "line0_placeholder_present": true }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir(
        "ui_gallery_code_editor_folds_inline_preedit_unwrapped_gate_passes_when_present",
    );
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_code_editor_folds_inline_preedit_unwrapped_gate_fails_when_placeholder_absent() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "folds_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "folds": { "line0_placeholder_present": false }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir =
        tmp_out_dir("ui_gallery_code_editor_folds_inline_preedit_unwrapped_gate_fails_when_absent");
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap_err();
    assert!(err.contains("gate failed"));
}

#[test]
fn ui_gallery_code_editor_folds_inline_preedit_with_decorations_gate_passes_when_placeholder_present()
 {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "folds_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "allow_decorations_under_inline_preedit": true,
                                    "folds": { "line0_placeholder_present": true }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir(
        "ui_gallery_code_editor_folds_inline_preedit_with_decorations_gate_passes_when_present",
    );
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_code_editor_folds_inline_preedit_with_decorations_gate_fails_when_placeholder_absent()
{
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "folds_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "allow_decorations_under_inline_preedit": true,
                                    "folds": { "line0_placeholder_present": false }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir(
        "ui_gallery_code_editor_folds_inline_preedit_with_decorations_gate_fails_when_absent",
    );
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap_err();
    assert!(err.contains("gate failed"));
}

#[test]
fn ui_gallery_code_editor_inlays_inline_preedit_unwrapped_gate_passes_when_inlay_present() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "inlays_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "inlays": { "line0_inlay_present": true }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir(
        "ui_gallery_code_editor_inlays_inline_preedit_unwrapped_gate_passes_when_present",
    );
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_code_editor_inlays_inline_preedit_unwrapped_gate_fails_when_inlay_absent() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "inlays_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "inlays": { "line0_inlay_present": false }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir(
        "ui_gallery_code_editor_inlays_inline_preedit_unwrapped_gate_fails_when_absent",
    );
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap_err();
    assert!(err.contains("gate failed"));
}

#[test]
fn ui_gallery_code_editor_inlays_inline_preedit_with_decorations_gate_passes_when_inlay_present() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "inlays_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "allow_decorations_under_inline_preedit": true,
                                    "inlays": { "line0_inlay_present": true }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir(
        "ui_gallery_code_editor_inlays_inline_preedit_with_decorations_gate_passes_when_present",
    );
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap();
}

#[test]
fn ui_gallery_code_editor_inlays_inline_preedit_with_decorations_gate_fails_when_inlay_absent() {
    let bundle = json!({
        "schema_version": 1,
        "windows": [
            {
                "window": 1,
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 1,
                        "app_snapshot": {
                            "kind": "fret_ui_gallery",
                            "selected_page": "code_editor_torture",
                            "code_editor": {
                                "soft_wrap_cols": 80,
                                "inlays_fixture": true,
                                "torture": {
                                    "preedit_active": true,
                                    "allow_decorations_under_inline_preedit": true,
                                    "inlays": { "line0_inlay_present": false }
                                }
                            }
                        }
                    }
                ]
            }
        ]
    });

    let out_dir = tmp_out_dir(
        "ui_gallery_code_editor_inlays_inline_preedit_with_decorations_gate_fails_when_absent",
    );
    let _ = std::fs::create_dir_all(&out_dir);
    let bundle_path = out_dir.join("bundle.json");
    let err = check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_json(
        &bundle,
        &bundle_path,
        0,
    )
    .unwrap_err();
    assert!(err.contains("gate failed"));
}
