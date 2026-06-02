use super::*;
use super::header_state::{
    HeaderDiagnosticsState, header_next_action_lines_for_artifacts_root,
};
use super::recent_evidence::recent_evidence_next_action;

#[test]
fn resolve_repo_or_abs_path_resolves_relative_input() {
    let resolved = resolve_repo_or_abs_path(
        Path::new("F:/repo"),
        "target/fret-diag/campaigns/ui-gallery-pr",
    );
    assert_eq!(
        resolved,
        PathBuf::from("F:/repo").join("target/fret-diag/campaigns/ui-gallery-pr")
    );
}

#[test]
fn file_url_from_path_projects_native_artifact_paths() {
    assert_eq!(
        file_url_from_path("F:\\repo\\.fret\\diag\\followups\\10-stats.json"),
        "file:///F:/repo/.fret/diag/followups/10-stats.json"
    );
    assert_eq!(
        file_url_from_path("/tmp/fret/diag/followups/10-stats.json"),
        "file:///tmp/fret/diag/followups/10-stats.json"
    );
    assert_eq!(
        file_url_from_path("F:\\repo\\.fret\\diag\\followups\\10 stats#failed.json"),
        "file:///F:/repo/.fret/diag/followups/10%20stats%23failed.json"
    );
    assert_eq!(
        file_url_from_path("/tmp/fret/diag/followups/结果.json"),
        "file:///tmp/fret/diag/followups/%E7%BB%93%E6%9E%9C.json"
    );
}

#[test]
fn file_url_from_path_projects_trace_artifact_paths() {
    assert_eq!(
        file_url_from_path("F:\\repo\\target\\fret-diag\\run-a\\trace.chrome.json"),
        "file:///F:/repo/target/fret-diag/run-a/trace.chrome.json"
    );
    assert_eq!(
        file_url_from_path("/tmp/fret/target/fret-diag/run a/trace.chrome.json"),
        "file:///tmp/fret/target/fret-diag/run%20a/trace.chrome.json"
    );
}

#[test]
fn file_url_from_path_projects_workflow_artifact_paths() {
    assert_eq!(
        file_url_from_path(
            "F:\\repo\\target\\fret-diag\\devtools-workflows\\perf-docking\\regression.summary.json"
        ),
        "file:///F:/repo/target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
    );
    assert_eq!(
        file_url_from_path(
            "/tmp/fret/target/fret-diag/devtools-workflows/perf docking/regression.summary.json"
        ),
        "file:///tmp/fret/target/fret-diag/devtools-workflows/perf%20docking/regression.summary.json"
    );
}

#[test]
fn inspect_hover_bounds_lines_project_bounds_and_selector() {
    let payload = serde_json::json!({
        "schema_version": 1,
        "window": 7,
        "viewport_bounds": { "x_px": 0.0, "y_px": 0.0, "w_px": 800.0, "h_px": 600.0 },
        "scale_factor": 1.0,
        "hovered": {
            "node_id": 42,
            "role": "button",
            "test_id": "save",
            "selector_json": "{\"kind\":\"test_id\",\"id\":\"save\"}",
            "bounds": { "x_px": 12.0, "y_px": 24.0, "w_px": 96.0, "h_px": 32.0 },
            "root": 1,
            "root_z_index": 3
        },
        "overlay_hook": {
            "kind": "hovered-node-bounds",
            "coordinate_space": "window_logical_px",
            "viewport_bounds": { "x_px": 0.0, "y_px": 0.0, "w_px": 800.0, "h_px": 600.0 },
            "target_bounds": { "x_px": 12.0, "y_px": 24.0, "w_px": 96.0, "h_px": 32.0 },
            "target_node_id": 42
        }
    })
    .to_string();

    let lines = inspect_hover_bounds_lines(&payload).join("\n");
    assert!(lines.contains("hover window=7"));
    assert!(lines.contains("hovered node: node=42 role=button test_id=save"));
    assert!(lines.contains("hover bounds: x=12.0 y=24.0 w=96.0 h=32.0"));
    assert!(lines.contains("selector_json: {\"kind\":\"test_id\",\"id\":\"save\"}"));
}

#[test]
fn inspect_hover_bounds_lines_missing_bounds_returns_none() {
    let payload = serde_json::json!({
        "schema_version": 1,
        "window": 7,
        "viewport_bounds": { "x_px": 0.0, "y_px": 0.0, "w_px": 800.0, "h_px": 600.0 },
        "scale_factor": 1.0,
        "hovered": {
            "node_id": 42,
            "role": "button",
            "selector_json": "{\"kind\":\"node_id\",\"node\":42}"
        },
        "overlay_hook": {
            "kind": "hovered-node-bounds",
            "coordinate_space": "window_logical_px",
            "viewport_bounds": { "x_px": 0.0, "y_px": 0.0, "w_px": 800.0, "h_px": 600.0 }
        }
    })
    .to_string();

    let lines = inspect_hover_bounds_lines(&payload).join("\n");
    assert_eq!(lines, "hover: <none>\nhover bounds: <none>");
}

#[test]
fn inspect_overlay_hook_lines_project_overlay_summary() {
    let hover = serde_json::json!({
        "schema_version": 1,
        "window": 7,
        "viewport_bounds": { "x_px": 0.0, "y_px": 0.0, "w_px": 800.0, "h_px": 600.0 },
        "scale_factor": 1.0,
        "hovered": null,
        "overlay_hook": {
            "kind": "hovered-node-bounds",
            "coordinate_space": "window_logical_px",
            "viewport_bounds": { "x_px": 0.0, "y_px": 0.0, "w_px": 800.0, "h_px": 600.0 }
        }
    })
    .to_string();
    let focus = serde_json::json!({
        "schema_version": 1,
        "window": 7,
        "viewport_bounds": { "x_px": 0.0, "y_px": 0.0, "w_px": 800.0, "h_px": 600.0 },
        "scale_factor": 1.0,
        "focused": null,
        "summary": "focus: button node=42 test_id=save",
        "path": "window > button(save)",
        "overlay_hook": {
            "kind": "focused-node-bounds",
            "coordinate_space": "window_logical_px",
            "viewport_bounds": { "x_px": 0.0, "y_px": 0.0, "w_px": 800.0, "h_px": 600.0 }
        }
    })
    .to_string();
    let overlay = serde_json::json!({
        "schema_version": 1,
        "window": 7,
        "barrier_root": 9,
        "focus_barrier_root": 10,
        "blocking_roots": [
            { "root": 9, "visible": true, "blocks_underlay_input": true, "hit_testable": true, "z_index": 12 }
        ],
        "topmost_interactive_root": {
            "root": 9,
            "visible": true,
            "blocks_underlay_input": true,
            "hit_testable": true,
            "z_index": 12
        }
    })
    .to_string();

    let lines = inspect_overlay_hook_lines(&hover, &focus, &overlay).join("\n");
    assert!(lines.contains("hover overlay hook: kind=hovered-node-bounds"));
    assert!(lines.contains("focus summary: focus: button node=42 test_id=save"));
    assert!(lines.contains("overlay barrier root: 9"));
    assert!(lines.contains("overlay blocking roots: 1"));
    assert!(lines.contains("topmost interactive root=9 z=12"));
}

#[test]
fn devtools_first_open_lines_surface_canonical_paths() {
    let lines = devtools_first_open_lines("target/fret-diag");
    let text = lines.join("\n");
    assert!(text.contains("first-open: docs/diagnostics-first-open.md"));
    assert!(text.contains(
        "gui branch: docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md"
    ));
    assert!(text.contains("repo preflight: cargo run -p fretboard-dev -- diag doctor campaigns"));
    assert!(text.contains(
        "repo preflight json: cargo run -p fretboard-dev -- diag doctor campaigns --json"
    ));
    assert!(text.contains("artifacts root: target/fret-diag"));
    assert!(text.contains("direct loop: diag run -> diag latest -> diag compare"));
    assert!(text.contains(
        "campaign loop: diag campaign run devtools-first-open-smoke -> diag summarize -> diag dashboard"
    ));
    assert!(text.contains("gate: python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke"));
    assert!(text.contains("product workflow: imui-product-chain"));
    assert!(text.contains("product workflow command: python tools/diag_gate_imui_product_chain.py"));
    assert!(text.contains(
        "product workflow focused: python tools/diag_gate_imui_product_chain.py --only discovery"
    ));
    assert!(text.contains(
        "product workflow launched: python tools/diag_gate_imui_product_chain.py --reuse-built --launched --only perf-docking --release"
    ));
    assert!(text.contains(
        "product workflow suite: tools/diag-scripts/suites/perf-docking-arbitration-steady/suite.json"
    ));
    assert!(text.contains(
        "product workflow docs: docs/workstreams/imui-editor-grade-product-closure-v1/EVIDENCE_AND_GATES.md"
    ));
    assert!(text.contains(
        "product workflow artifacts: perf-docking/regression.summary.json, perf-docking/check.perf_thresholds.json, perf-docking/*/trace.chrome.json"
    ));
}

#[test]
fn devtools_first_open_next_action_lines_prioritize_stateful_workflow() {
    let empty =
        devtools_first_open_next_action_lines(
            false,
            0,
            None,
            12,
            false,
            false,
            false,
            0,
            "target/fret-diag",
            None,
            None,
            None,
            "run a workflow or generated gate",
        )
        .join("\n");
    assert!(empty.contains("target: no session yet"));
    assert!(empty.contains("session scope: waiting for the first diagnostics session"));
    assert!(empty.contains("scripts: 12 available"));
    assert!(empty.contains("regression: no aggregate loaded"));
    assert!(empty.contains("recent evidence: no failed restored GUI-launched evidence"));
    assert!(
        empty.contains("recent evidence command: <none; run a workflow or generated gate>")
    );
    assert!(empty.contains("recent evidence rerun: <none>"));
    assert!(empty.contains("artifacts root: target/fret-diag"));
    assert!(empty.contains("Evidence & Results -> Guide"));
    assert!(empty.contains("workflow runs"));

    let ready =
        devtools_first_open_next_action_lines(
            true,
            2,
            Some("session-b"),
            8,
            true,
            false,
            true,
            3,
            "target/fret-diag",
            None,
            None,
            None,
            "continue from latest passing evidence",
        )
        .join("\n");
    assert!(ready.contains("target: session selected"));
    assert!(ready.contains(
        "session scope: selected session-b; 2 sessions connected, use the Session selector to retarget inspect, bundle, screenshot, and selected-session suite actions"
    ));
    assert!(ready.contains("regression: aggregate loaded with 3 non-passing"));

    let awaiting_selection = devtools_first_open_next_action_lines(
        false,
        2,
        None,
        8,
        false,
        false,
        false,
        0,
        "target/fret-diag",
        None,
        None,
        None,
        "run a workflow or generated gate",
    )
    .join("\n");
    assert!(awaiting_selection.contains(
        "session scope: choose one available session before sending inspect, bundle, screenshot, or selected-session suite actions"
    ));

    let selected_summary = devtools_first_open_next_action_lines(
        true,
        1,
        Some("session-a"),
        8,
        false,
        true,
        false,
        0,
        "target/fret-diag",
        None,
        None,
        None,
        "continue from latest passing evidence",
    )
    .join("\n");
    assert!(selected_summary.contains(
        "session scope: selected session-a; actions target the current diagnostics session"
    ));
    assert!(selected_summary.contains("regression: selected summary loaded"));
    assert!(selected_summary.contains("follow-up actions can use selected bundle evidence"));

    let selected_followup = devtools_first_open_next_action_lines(
        true,
        1,
        Some("session-a"),
        8,
        false,
        true,
        true,
        0,
        "target/fret-diag",
        None,
        None,
        None,
        "continue from selected follow-up result",
    )
    .join("\n");
    assert!(selected_followup.contains("regression: selected follow-up result loaded"));
    assert!(selected_followup.contains("Follow-up Result Summary/History"));

    let failed_evidence = RecentEvidenceTarget {
        kind: "workflow",
        id: "perf-docking-suite-ws".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/400-fresh.json".to_string(),
        result_json: "{\"status\":\"failed\"}".to_string(),
        command_line: "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --json".to_string(),
        bundle_dir: None,
    };
    let with_failed_evidence = devtools_first_open_next_action_lines(
        true,
        1,
        Some("session-a"),
        8,
        false,
        false,
        false,
        0,
        "target/fret-diag",
        Some(&failed_evidence),
        None,
        Some("missing current selected-session"),
        "select a diagnostics session, then rerun failed workflow evidence",
    )
    .join("\n");
    assert!(with_failed_evidence.contains(
        "recent evidence: failed workflow perf-docking-suite-ws (400-fresh.json)"
    ));
    assert!(with_failed_evidence.contains(
        "recent evidence command: cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --json"
    ));
    assert!(with_failed_evidence
        .contains("recent evidence rerun: unavailable (missing current selected-session)"));
    assert!(with_failed_evidence.contains(
        "recent evidence next: select a diagnostics session, then rerun failed workflow evidence"
    ));

    let with_rerunnable_failed_evidence = devtools_first_open_next_action_lines(
        true,
        1,
        Some("session-a"),
        8,
        false,
        false,
        false,
        0,
        "target/fret-diag",
        Some(&failed_evidence),
        Some("workflow"),
        None,
        "rerun failed workflow evidence",
    )
    .join("\n");
    assert!(
        with_rerunnable_failed_evidence
            .contains("recent evidence rerun: available (workflow)")
    );
    assert!(with_rerunnable_failed_evidence
        .contains("recent evidence next: rerun failed workflow evidence"));
}

#[test]
fn first_open_recent_evidence_action_specs_gate_disabled_states() {
    let empty = first_open_recent_evidence_action_specs(false, false);
    assert_eq!(empty.len(), 3);
    assert_eq!(empty[0].label, "Copy recent evidence report");
    assert_eq!(empty[0].command, CMD_COPY_RECENT_EVIDENCE_REPORT);
    assert!(!empty[0].disabled);
    assert_eq!(empty[1].label, "Select failed evidence");
    assert_eq!(empty[1].command, CMD_SELECT_RECENT_FAILED_EVIDENCE);
    assert!(empty[1].disabled);
    assert_eq!(empty[2].label, "Rerun failed evidence");
    assert_eq!(empty[2].command, CMD_RERUN_RECENT_FAILED_EVIDENCE);
    assert!(empty[2].disabled);

    let has_failure = first_open_recent_evidence_action_specs(true, false);
    assert!(!has_failure[1].disabled);
    assert!(has_failure[2].disabled);

    let rerunnable = first_open_recent_evidence_action_specs(true, true);
    assert!(!rerunnable[1].disabled);
    assert!(!rerunnable[2].disabled);
}

#[test]
fn recent_evidence_status_failed_ignores_empty_placeholder_and_passed_case() {
    assert!(!recent_evidence_status_failed(""));
    assert!(!recent_evidence_status_failed("   "));
    assert!(!recent_evidence_status_failed("-"));
    assert!(!recent_evidence_status_failed("passed"));
    assert!(!recent_evidence_status_failed("Passed"));
    assert!(!recent_evidence_status_failed("PASSED"));
    assert!(recent_evidence_status_failed("failed"));
    assert!(recent_evidence_status_failed("error"));
}

#[test]
fn devtools_recent_evidence_lines_surface_restored_histories() {
    let gate_entries = vec![
        gate_run::GateRunResultHistoryEntry {
            id: "pixels-changed".to_string(),
            label: "Pixels changed".to_string(),
            command_line: "cargo run -p fretboard-dev -- diag run smoke.json --json"
                .to_string(),
            result_path: "F:\\repo\\.fret\\diag\\gate-runs\\100-pixels.json".to_string(),
            result_json: "{}".to_string(),
            status: "passed".to_string(),
            error: None,
        },
        gate_run::GateRunResultHistoryEntry {
            id: "stale-paint-scene".to_string(),
            label: "Stale paint/scene".to_string(),
            command_line: "cargo run -p fretboard-dev -- diag run stale.json --json"
                .to_string(),
            result_path: "F:\\repo\\.fret\\diag\\gate-runs\\090-stale.json".to_string(),
            result_json: "{}".to_string(),
            status: "failed".to_string(),
            error: Some("stale scene".to_string()),
        },
    ];
    let workflow_entries = vec![workflow_run::WorkflowRunResultHistoryEntry {
        id: "perf-docking-suite-ws".to_string(),
        label: "Run perf docking suite over selected session".to_string(),
        command_line: "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --json"
            .to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/200-workflow.json".to_string(),
        result_json: "{}".to_string(),
        status: "failed".to_string(),
        error: Some("threshold failed".to_string()),
    }];
    let followup_entries = vec![followup::FollowupResultHistoryEntry {
        id: "trace".to_string(),
        label: "Trace".to_string(),
        command_line: "cargo run -p fretboard-dev -- diag trace bundle --json".to_string(),
        result_path: "F:/repo/.fret/diag/followups/300-trace.json".to_string(),
        result_json: "{}".to_string(),
        bundle_dir: Some("target/fret-diag/run/bundle".to_string()),
        status: "passed".to_string(),
        error: None,
    }];

    let text =
        devtools_recent_evidence_lines(&gate_entries, &workflow_entries, &followup_entries)
            .join("\n");
    assert!(text.contains("recent evidence: gates=2 workflows=1 followups=1"));
    assert!(text.contains("latest gate: passed | pixels-changed | 100-pixels.json"));
    assert!(
        text.contains("latest workflow: failed | perf-docking-suite-ws | 200-workflow.json")
    );
    assert!(
        text.contains("latest follow-up: passed | trace | 300-trace.json | bundle=target/fret-diag/run/bundle")
    );
    assert!(text.contains("recent failing evidence: 2"));
    assert!(
        text.contains("failed_evidence_target: workflow | failed | perf-docking-suite-ws | 200-workflow.json")
    );
    assert!(text.contains(
        "failed_evidence_path: F:/repo/.fret/diag/workflow-runs/200-workflow.json"
    ));
    assert!(text.contains("failed_evidence_bundle_dir: <none>"));
    assert!(text.contains("failed_evidence_command: cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --json"));
    assert!(text.contains("failed_evidence_rerunnable: no"));
    assert!(
        text.contains("failed_evidence_rerun_unavailable_reason: workflow commands unavailable")
    );
    assert!(text.contains(
        "recent_evidence_next_action: refresh current workflow commands, then rerun failed workflow evidence"
    ));

    let empty = devtools_recent_evidence_lines(&[], &[], &[]).join("\n");
    assert!(empty.contains("latest gate: <none>"));
    assert!(empty.contains("latest workflow: <none>"));
    assert!(empty.contains("latest follow-up: <none>"));
    assert!(empty.contains("failed_evidence_target: <none>"));
    assert!(empty.contains("failed_evidence_path: <none>"));
    assert!(empty.contains("failed_evidence_bundle_dir: <none>"));
    assert!(empty.contains("failed_evidence_command: <none>"));
    assert!(empty.contains("failed_evidence_rerunnable: <none>"));
    assert!(empty.contains("failed_evidence_rerun_unavailable_reason: <none>"));
    assert!(empty.contains("recent_evidence_next_action: run a workflow or generated gate"));
}

#[test]
fn recent_evidence_next_action_projects_rerun_and_repair_steps() {
    let gate = RecentEvidenceTarget {
        kind: "gate",
        id: "stale".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/gate-runs/100-stale.json".to_string(),
        result_json: serde_json::json!({
            "diag_args": ["run", "tools/diag-scripts/smoke.json", "--json"]
        })
        .to_string(),
        command_line: "cargo run -p fretboard-dev -- diag run tools/diag-scripts/smoke.json --json".to_string(),
        bundle_dir: None,
    };
    assert_eq!(
        recent_evidence_next_action(1, false, Some(&gate), &[]),
        "rerun failed gate evidence"
    );

    let workflow = RecentEvidenceTarget {
        kind: "workflow",
        id: DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID.to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/200-workflow.json".to_string(),
        result_json: serde_json::json!({
            "diag_args": [
                "suite",
                DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE,
                "--devtools-token",
                "<redacted>",
                "--json"
            ]
        })
        .to_string(),
        command_line: "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --devtools-token <redacted> --json".to_string(),
        bundle_dir: None,
    };
    let missing_session_commands = devtools_workflow_commands(
        "target/fret-diag",
        "ws://127.0.0.1:7331/",
        "secret-token",
        None,
    );
    assert_eq!(
        recent_evidence_next_action(1, false, Some(&workflow), &missing_session_commands),
        "select a diagnostics session, then rerun failed workflow evidence"
    );

    let current_session_commands = devtools_workflow_commands(
        "target/fret-diag",
        "ws://127.0.0.1:7331/",
        "secret-token",
        Some("session-1"),
    );
    assert_eq!(
        recent_evidence_next_action(1, false, Some(&workflow), &current_session_commands),
        "rerun failed workflow evidence"
    );

    let old_workflow = RecentEvidenceTarget {
        id: "old-workflow".to_string(),
        ..workflow.clone()
    };
    assert_eq!(
        recent_evidence_next_action(1, false, Some(&old_workflow), &current_session_commands),
        "run a current workflow for fresh evidence"
    );

    let followup = RecentEvidenceTarget {
        kind: "follow-up",
        id: "trace".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/followups/500-trace.json".to_string(),
        result_json: "{}".to_string(),
        command_line: "cargo run -p fretboard-dev -- diag trace bundle --json".to_string(),
        bundle_dir: Some("target/fret-diag/run-bundle".to_string()),
    };
    assert_eq!(
        recent_evidence_next_action(1, false, Some(&followup), &[]),
        "select failed follow-up evidence and inspect result JSON"
    );
}

#[test]
fn devtools_recent_evidence_lines_surface_failed_followup_bundle_dir() {
    let followup_entries = vec![followup::FollowupResultHistoryEntry {
        id: "trace".to_string(),
        label: "Trace".to_string(),
        command_line: "cargo run -p fretboard-dev -- diag trace bundle --json".to_string(),
        result_path: "F:/repo/.fret/diag/followups/500-trace.json".to_string(),
        result_json: "{}".to_string(),
        bundle_dir: Some("target/fret-diag/run-bundle".to_string()),
        status: "failed".to_string(),
        error: Some("trace failed".to_string()),
    }];

    let text = devtools_recent_evidence_lines(&[], &[], &followup_entries).join("\n");
    assert!(text.contains(
        "failed_evidence_target: follow-up | failed | trace | 500-trace.json"
    ));
    assert!(
        text.contains("failed_evidence_path: F:/repo/.fret/diag/followups/500-trace.json")
    );
    assert!(text.contains("failed_evidence_bundle_dir: target/fret-diag/run-bundle"));
}

#[test]
fn recent_failed_evidence_bundle_dir_filters_empty_bundle_dir() {
    let with_bundle = RecentEvidenceTarget {
        kind: "follow-up",
        id: "trace".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/followups/500-trace.json".to_string(),
        result_json: "{}".to_string(),
        command_line: "trace fresh".to_string(),
        bundle_dir: Some("target/fret-diag/run-bundle".to_string()),
    };
    assert_eq!(
        recent_failed_evidence_bundle_dir(&with_bundle),
        Some("target/fret-diag/run-bundle")
    );

    let empty_bundle = RecentEvidenceTarget {
        bundle_dir: Some("   ".to_string()),
        ..with_bundle
    };
    assert!(recent_failed_evidence_bundle_dir(&empty_bundle).is_none());
}

#[test]
fn recent_failed_evidence_rerun_command_uses_structured_diag_args() {
    let gate = RecentEvidenceTarget {
        kind: "gate",
        id: "stale".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/gate-runs/100-stale.json".to_string(),
        result_json: serde_json::json!({
            "diag_args": ["run", "tools/diag-scripts/smoke.json", "--check-stale-paint", "button.ok", "--json"]
        })
        .to_string(),
        command_line: "cargo run -p fretboard-dev -- diag run tools/diag-scripts/smoke.json --check-stale-paint button.ok --json".to_string(),
        bundle_dir: None,
    };
    let Some(RecentEvidenceRerunCommand::Gate(command)) =
        recent_failed_evidence_rerun_command(&gate)
    else {
        panic!("expected rerunnable gate command");
    };
    assert_eq!(command.id, "stale");
    assert_eq!(
        command.diag_args,
        vec![
            "run",
            "tools/diag-scripts/smoke.json",
            "--check-stale-paint",
            "button.ok",
            "--json"
        ]
    );
    assert!(command.is_runnable());
}

#[test]
fn recent_failed_evidence_rerun_command_rejects_redacted_workflow_args() {
    let workflow = RecentEvidenceTarget {
        kind: "workflow",
        id: "perf-docking-suite-ws".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/200-workflow.json".to_string(),
        result_json: serde_json::json!({
            "diag_args": [
                "suite",
                "perf-docking-arbitration-steady",
                "--devtools-token",
                "<redacted>",
                "--json"
            ]
        })
        .to_string(),
        command_line: "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --devtools-token <redacted> --json".to_string(),
        bundle_dir: None,
    };

    assert!(recent_failed_evidence_rerun_command(&workflow).is_none());
    assert!(recent_failed_evidence_rerun_line(Some(&workflow))
        .contains("failed_evidence_rerunnable: no"));
}

#[test]
fn recent_failed_evidence_rerun_reason_reports_diag_args_issues() {
    let gate = RecentEvidenceTarget {
        kind: "gate",
        id: "stale".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/gate-runs/100-stale.json".to_string(),
        result_json: serde_json::json!({
            "diag_args": ["run", "<redacted>", "--json"]
        })
        .to_string(),
        command_line: "cargo run -p fretboard-dev -- diag run <redacted> --json".to_string(),
        bundle_dir: None,
    };
    assert_eq!(
        recent_failed_evidence_rerun_unavailable_reason_from_state(&gate, &[]).as_deref(),
        Some("diag_args missing or redacted")
    );
    assert!(recent_failed_evidence_rerun_line(Some(&gate))
        .contains("failed_evidence_rerunnable: no (diag_args missing or redacted)"));

    let missing_args = RecentEvidenceTarget {
        result_json: "{}".to_string(),
        ..gate
    };
    assert_eq!(
        recent_failed_evidence_rerun_unavailable_reason_from_state(&missing_args, &[])
            .as_deref(),
        Some("diag_args missing")
    );
}

#[test]
fn recent_failed_evidence_rerun_command_recovers_redacted_workflow_from_current_state() {
    let workflow = RecentEvidenceTarget {
        kind: "workflow",
        id: DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID.to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/200-workflow.json".to_string(),
        result_json: serde_json::json!({
            "diag_args": [
                "suite",
                DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE,
                "--devtools-token",
                "<redacted>",
                "--json"
            ]
        })
        .to_string(),
        command_line: "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --devtools-token <redacted> --json".to_string(),
        bundle_dir: None,
    };
    let workflow_commands = devtools_workflow_commands(
        "target/fret-diag",
        "ws://127.0.0.1:7331/",
        "secret-token",
        Some("session-1"),
    );
    let Some(RecentEvidenceRerunCommand::Workflow(command)) =
        recent_failed_evidence_rerun_command_from_state(&workflow, &workflow_commands)
    else {
        panic!("expected current-state workflow fallback");
    };
    assert_eq!(command.id, DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID);
    assert!(command.is_runnable());
    assert!(command.diag_args.contains(&"secret-token".to_string()));
    assert!(command.diag_args.contains(&"session-1".to_string()));
    assert!(!command.diag_args.contains(&"<redacted>".to_string()));

    let missing_session_commands = devtools_workflow_commands(
        "target/fret-diag",
        "ws://127.0.0.1:7331/",
        "secret-token",
        None,
    );
    assert!(
        recent_failed_evidence_rerun_command_from_state(&workflow, &missing_session_commands)
            .is_none()
    );
    assert_eq!(
        recent_failed_evidence_rerun_unavailable_reason_from_state(
            &workflow,
            &missing_session_commands
        )
        .as_deref(),
        Some("missing current selected-session")
    );
}

#[test]
fn recent_failed_evidence_rerun_command_uses_current_workflow_state_over_stored_args() {
    let workflow = RecentEvidenceTarget {
        kind: "workflow",
        id: DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID.to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/250-workflow.json".to_string(),
        result_json: serde_json::json!({
            "diag_args": [
                "suite",
                DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE,
                "--dir",
                "target/fret-diag/old-workflow",
                "--devtools-ws-url",
                "ws://127.0.0.1:7331/",
                "--devtools-token",
                "old-token",
                "--devtools-session-id",
                "old-session",
                "--json"
            ]
        })
        .to_string(),
        command_line: "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --devtools-token old-token --devtools-session-id old-session --json".to_string(),
        bundle_dir: None,
    };
    assert!(recent_failed_evidence_rerun_command(&workflow).is_none());

    let workflow_commands = devtools_workflow_commands(
        "target/fret-diag",
        "ws://127.0.0.1:7331/",
        "current-token",
        Some("current-session"),
    );
    let Some(RecentEvidenceRerunCommand::Workflow(command)) =
        recent_failed_evidence_rerun_command_from_state(&workflow, &workflow_commands)
    else {
        panic!("expected current-state workflow command");
    };
    assert!(command.diag_args.contains(&"current-token".to_string()));
    assert!(command.diag_args.contains(&"current-session".to_string()));
    assert!(!command.diag_args.contains(&"old-token".to_string()));
    assert!(!command.diag_args.contains(&"old-session".to_string()));
}

#[test]
fn devtools_recent_evidence_lines_use_current_workflow_state_for_rerunnable_status() {
    let workflow_entries = vec![workflow_run::WorkflowRunResultHistoryEntry {
        id: DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID.to_string(),
        label: "Run perf docking suite over selected session".to_string(),
        command_line: "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --devtools-token <redacted> --json".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/200-workflow.json".to_string(),
        result_json: serde_json::json!({
            "diag_args": [
                "suite",
                DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE,
                "--devtools-token",
                "<redacted>",
                "--json"
            ]
        })
        .to_string(),
        status: "failed".to_string(),
        error: Some("threshold failed".to_string()),
    }];
    let without_state = devtools_recent_evidence_lines(&[], &workflow_entries, &[]).join("\n");
    assert!(without_state.contains("failed_evidence_rerunnable: no"));
    assert!(without_state
        .contains("failed_evidence_rerun_unavailable_reason: workflow commands unavailable"));

    let missing_session_commands = devtools_workflow_commands(
        "target/fret-diag",
        "ws://127.0.0.1:7331/",
        "secret-token",
        None,
    );
    let missing_session = devtools_recent_evidence_lines_with_workflow_commands(
        &[],
        &workflow_entries,
        &[],
        &missing_session_commands,
    )
    .join("\n");
    assert!(missing_session
        .contains("failed_evidence_rerunnable: no (missing current selected-session)"));
    assert!(missing_session
        .contains("failed_evidence_rerun_unavailable_reason: missing current selected-session"));

    let workflow_commands = devtools_workflow_commands(
        "target/fret-diag",
        "ws://127.0.0.1:7331/",
        "secret-token",
        Some("session-1"),
    );
    let with_state = devtools_recent_evidence_lines_with_workflow_commands(
        &[],
        &workflow_entries,
        &[],
        &workflow_commands,
    )
    .join("\n");
    assert!(with_state.contains("failed_evidence_rerunnable: workflow"));
    assert!(with_state.contains("failed_evidence_rerun_unavailable_reason: <none>"));
}

#[test]
fn recent_failed_evidence_rerun_reason_reports_unregistered_workflow() {
    let workflow = RecentEvidenceTarget {
        kind: "workflow",
        id: "old-workflow".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/200-workflow.json".to_string(),
        result_json: serde_json::json!({
            "diag_args": [
                "suite",
                "old-suite",
                "--devtools-token",
                "<redacted>",
                "--json"
            ]
        })
        .to_string(),
        command_line: "cargo run -p fretboard-dev -- diag suite old-suite --json".to_string(),
        bundle_dir: None,
    };
    let workflow_commands = devtools_workflow_commands(
        "target/fret-diag",
        "ws://127.0.0.1:7331/",
        "secret-token",
        Some("session-1"),
    );

    assert_eq!(
        recent_failed_evidence_rerun_unavailable_reason_from_state(
            &workflow,
            &workflow_commands
        )
        .as_deref(),
        Some("workflow command old-workflow is no longer registered")
    );
}

#[test]
fn recent_failed_evidence_rerun_command_projects_followup_bundle() {
    let followup = RecentEvidenceTarget {
        kind: "follow-up",
        id: "trace".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/followups/500-trace.json".to_string(),
        result_json: serde_json::json!({
            "diag_args": ["trace", "target/fret-diag/run-bundle", "--json"]
        })
        .to_string(),
        command_line: "cargo run -p fretboard-dev -- diag trace target/fret-diag/run-bundle --json".to_string(),
        bundle_dir: Some("target/fret-diag/run-bundle".to_string()),
    };
    let Some(RecentEvidenceRerunCommand::Followup(command)) =
        recent_failed_evidence_rerun_command(&followup)
    else {
        panic!("expected rerunnable follow-up command");
    };
    assert_eq!(command.id, "trace");
    assert_eq!(
        command.diag_args,
        vec!["trace", "target/fret-diag/run-bundle", "--json"]
    );
    assert_eq!(
        command.target_bundle_dir.as_deref(),
        Some("target/fret-diag/run-bundle")
    );
    assert!(!command.requires_baseline);
    assert!(recent_failed_evidence_rerun_line(Some(&followup))
        .contains("failed_evidence_rerunnable: follow-up"));
}

#[test]
fn devtools_recent_failed_evidence_target_prefers_visible_latest_then_history() {
    let gate_entries = vec![
        gate_run::GateRunResultHistoryEntry {
            id: "fresh-gate".to_string(),
            label: "Fresh gate".to_string(),
            command_line: "gate fresh".to_string(),
            result_path: "F:/repo/.fret/diag/gate-runs/300-fresh-gate.json".to_string(),
            result_json: "{}".to_string(),
            status: "passed".to_string(),
            error: None,
        },
        gate_run::GateRunResultHistoryEntry {
            id: "old-gate".to_string(),
            label: "Old gate".to_string(),
            command_line: "gate old".to_string(),
            result_path: "F:/repo/.fret/diag/gate-runs/100-old-gate.json".to_string(),
            result_json: "{}".to_string(),
            status: "failed".to_string(),
            error: Some("old failure".to_string()),
        },
    ];
    let workflow_entries = vec![workflow_run::WorkflowRunResultHistoryEntry {
        id: "fresh-workflow".to_string(),
        label: "Fresh workflow".to_string(),
        command_line: "workflow fresh".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/400-fresh-workflow.json".to_string(),
        result_json: "{}".to_string(),
        status: "FAILED".to_string(),
        error: Some("fresh workflow failure".to_string()),
    }];
    let followup_entries = vec![followup::FollowupResultHistoryEntry {
        id: "fresh-followup".to_string(),
        label: "Fresh follow-up".to_string(),
        command_line: "followup fresh".to_string(),
        result_path: "F:/repo/.fret/diag/followups/500-fresh-followup.json".to_string(),
        result_json: "{}".to_string(),
        bundle_dir: None,
        status: "failed".to_string(),
        error: Some("fresh follow-up failure".to_string()),
    }];

    let visible = devtools_recent_failed_evidence_target(
        &gate_entries,
        &workflow_entries,
        &followup_entries,
    )
    .expect("latest failed follow-up target");
    assert_eq!(visible.kind, "follow-up");
    assert_eq!(visible.id, "fresh-followup");
    assert_eq!(visible.result_path, "F:/repo/.fret/diag/followups/500-fresh-followup.json");
    assert_eq!(visible.command_line, "followup fresh");
    assert_eq!(visible.result_json, "{}");

    let older = devtools_recent_failed_evidence_target(&gate_entries, &[], &[])
        .expect("older failed gate target");
    assert_eq!(older.kind, "gate");
    assert_eq!(older.id, "old-gate");
    assert_eq!(older.command_line, "gate old");

    assert!(devtools_recent_failed_evidence_target(&[], &[], &[]).is_none());
}

#[test]
fn devtools_recent_failed_evidence_target_falls_back_to_lane_order_without_timestamps() {
    let gate_entries = vec![gate_run::GateRunResultHistoryEntry {
        id: "failed-gate".to_string(),
        label: "Failed gate".to_string(),
        command_line: "gate failed".to_string(),
        result_path: "F:/repo/.fret/diag/gate-runs/failed-gate.json".to_string(),
        result_json: "{}".to_string(),
        status: "failed".to_string(),
        error: Some("gate failed".to_string()),
    }];
    let workflow_entries = vec![workflow_run::WorkflowRunResultHistoryEntry {
        id: "failed-workflow".to_string(),
        label: "Failed workflow".to_string(),
        command_line: "workflow failed".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/failed-workflow.json".to_string(),
        result_json: "{}".to_string(),
        status: "failed".to_string(),
        error: Some("workflow failed".to_string()),
    }];
    let followup_entries = vec![followup::FollowupResultHistoryEntry {
        id: "failed-followup".to_string(),
        label: "Failed follow-up".to_string(),
        command_line: "followup failed".to_string(),
        result_path: "F:/repo/.fret/diag/followups/failed-followup.json".to_string(),
        result_json: "{}".to_string(),
        bundle_dir: None,
        status: "failed".to_string(),
        error: Some("follow-up failed".to_string()),
    }];

    let target = devtools_recent_failed_evidence_target(
        &gate_entries,
        &workflow_entries,
        &followup_entries,
    )
    .expect("fallback failed evidence target");

    assert_eq!(target.kind, "gate");
    assert_eq!(target.id, "failed-gate");
}

#[test]
fn devtools_recent_failed_evidence_target_prefers_result_json_time_over_path_time() {
    let workflow_entries = vec![workflow_run::WorkflowRunResultHistoryEntry {
        id: "long-workflow".to_string(),
        label: "Long workflow".to_string(),
        command_line: "workflow long".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/100-long-workflow.json".to_string(),
        result_json: serde_json::json!({
            "started_unix_ms": 100,
            "finished_unix_ms": 900,
            "status": "failed"
        })
        .to_string(),
        status: "failed".to_string(),
        error: Some("workflow failed after follow-up started".to_string()),
    }];
    let followup_entries = vec![followup::FollowupResultHistoryEntry {
        id: "trace".to_string(),
        label: "Trace".to_string(),
        command_line: "trace failed".to_string(),
        result_path: "F:/repo/.fret/diag/followups/500-trace.json".to_string(),
        result_json: serde_json::json!({
            "started_unix_ms": 500,
            "finished_unix_ms": 600,
            "status": "failed"
        })
        .to_string(),
        bundle_dir: None,
        status: "failed".to_string(),
        error: Some("trace failed".to_string()),
    }];

    let target =
        devtools_recent_failed_evidence_target(&[], &workflow_entries, &followup_entries)
            .expect("latest failed evidence target");

    assert_eq!(target.kind, "workflow");
    assert_eq!(target.id, "long-workflow");
}

#[test]
fn devtools_recent_failed_evidence_target_carries_result_json_payload() {
    let gate_entries = vec![gate_run::GateRunResultHistoryEntry {
        id: "failed-gate".to_string(),
        label: "Failed gate".to_string(),
        command_line: "gate failed".to_string(),
        result_path: "F:/repo/.fret/diag/gate-runs/100-failed.json".to_string(),
        result_json: "{\"status\":\"failed\",\"error\":\"stale\"}".to_string(),
        status: "failed".to_string(),
        error: Some("stale".to_string()),
    }];

    let target = devtools_recent_failed_evidence_target(&gate_entries, &[], &[])
        .expect("failed gate target");
    assert_eq!(
        target.result_json,
        "{\"status\":\"failed\",\"error\":\"stale\"}"
    );
}

#[test]
fn devtools_recent_evidence_selection_effect_routes_to_existing_history_state() {
    let gate = RecentEvidenceTarget {
        kind: "gate",
        id: "old-gate".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/gate-runs/100-old.json".to_string(),
        result_json: "{\"kind\":\"gate\"}".to_string(),
        command_line: "gate old".to_string(),
        bundle_dir: None,
    };
    let gate_effect = devtools_recent_evidence_selection_effect(&gate);
    assert_eq!(gate_effect.details_tab, "guide");
    assert_eq!(
        gate_effect.selected_path,
        "F:/repo/.fret/diag/gate-runs/100-old.json"
    );
    assert!(gate_effect.selected_bundle_dir.is_none());

    let workflow = RecentEvidenceTarget {
        kind: "workflow",
        id: "fresh-workflow".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/workflow-runs/400-fresh.json".to_string(),
        result_json: "{\"kind\":\"workflow\"}".to_string(),
        command_line: "workflow fresh".to_string(),
        bundle_dir: None,
    };
    let workflow_effect = devtools_recent_evidence_selection_effect(&workflow);
    assert_eq!(workflow_effect.details_tab, "guide");
    assert_eq!(
        workflow_effect.selected_path,
        "F:/repo/.fret/diag/workflow-runs/400-fresh.json"
    );

    let followup = RecentEvidenceTarget {
        kind: "follow-up",
        id: "trace".to_string(),
        status: "failed".to_string(),
        result_path: "F:/repo/.fret/diag/followups/500-trace.json".to_string(),
        result_json: "{\"kind\":\"follow-up\"}".to_string(),
        command_line: "trace fresh".to_string(),
        bundle_dir: Some("target/fret-diag/run-bundle".to_string()),
    };
    let followup_effect = devtools_recent_evidence_selection_effect(&followup);
    assert_eq!(followup_effect.details_tab, "regression");
    assert_eq!(
        followup_effect.selected_path,
        "F:/repo/.fret/diag/followups/500-trace.json"
    );
    assert_eq!(
        followup_effect.selected_bundle_dir.as_deref(),
        Some("target/fret-diag/run-bundle")
    );
}

#[test]
fn devtools_dogfood_workflow_lines_surface_ui_gallery_loop() {
    let lines = devtools_dogfood_workflow_lines("target/fret-diag");
    let text = lines.join("\n");
    assert!(text.contains("dogfood workflow: ui-gallery-button-dogfood"));
    assert!(text.contains(
        "dogfood docs: docs/workstreams/diag-fearless-refactor-v2/DEVTOOLS_GUI_DOGFOOD_WORKFLOW.md"
    ));
    assert!(text.contains("artifacts root: target/fret-diag"));
    assert!(text.contains("open ui gallery: cargo run -p fret-ui-gallery --release"));
    assert!(
        text.contains("pick target: enable inspect -> Pick -> click a Button page control")
    );
    assert!(
        text.contains("preferred selector: {\"kind\":\"test_id\",\"id\":\"ui-gallery-nav-button\"}")
    );
    assert!(text.contains("base script: tools/diag-scripts/ui-gallery-lite-smoke.json"));
    assert!(text.contains(
        "button script: tools/diag-scripts/ui-gallery/button/ui-gallery-button-with-icon-non-overlap.json"
    ));
    assert!(text.contains("generate script from pick: cargo run -p fretboard-dev -- diag pick-script --pick-script-out target/fret-diag/picked.script.json"));
    assert!(text.contains("apply pick to script: cargo run -p fretboard-dev -- diag pick-apply tools/diag-scripts/ui-gallery-lite-smoke.json --ptr /steps/12/target --out target/fret-diag/ui-gallery-picked.script.json"));
    assert!(text.contains("run and pack: cargo run -p fretboard-dev -- diag run target/fret-diag/ui-gallery-picked.script.json --pack --include-all --pack-schema2-only --launch -- cargo run -p fret-ui-gallery --release"));
    assert!(text.contains(
        "pack selected bundle: cargo run -p fretboard-dev -- diag pack <bundle-dir> --include-all --pack-schema2-only"
    ));
    assert!(text.contains("open viewer: pnpm -C tools/fret-bundle-viewer dev"));
    assert!(text.contains(
        "viewer input: drag bundle.json, bundle.schema2.json, or the packed zip into the offline viewer"
    ));
}

#[test]
fn devtools_demo_metrics_debug_lines_surface_canonical_routes() {
    let lines = devtools_demo_metrics_debug_lines("target/fret-diag");
    let text = lines.join("\n");
    assert!(text.contains("route: demo-metrics-debug"));
    assert!(text.contains(
        "route owner: docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json"
    ));
    assert!(text.contains(
        "action metadata owner: docs/workstreams/imui-demo-metrics-debug-action-metadata-v1/WORKSTREAM.json"
    ));
    assert!(text.contains(
        "docking owner: docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json"
    ));
    assert!(text.contains(
        "wayland acceptance: docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md"
    ));
    assert!(text.contains("artifacts root: target/fret-diag"));
    assert!(text.contains(
        "action surface: dedicated DevTools guide panel + copyable action command bundle and per-action copy commands"
    ));
    assert!(text.contains(
        "workflow handoff: validate docking campaign | workflow_id=campaign-validate-imui-p3-multiwindow | run_command=fret.devtools.demo_metrics_debug.run_docking_workflow"
    ));
    assert!(text.contains(
        "workflow handoff: run perf docking suite | workflow_id=perf-docking-suite-ws | run_command=fret.devtools.demo_metrics_debug.run_perf_workflow | requires=selected-session"
    ));
    assert!(text.contains(
        "command palette: deferred until DevTools has a shared command palette contract"
    ));
    assert!(text.contains(
        "action: open workbench -> cargo run -p fret-demo --bin imui_editor_workbench_demo"
    ));
    assert!(text.contains(
        "action: run product discovery -> python tools/diag_gate_imui_product_chain.py --only discovery"
    ));
    assert!(text.contains(
        "action: inspect metrics stats -> cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json"
    ));
    assert!(text.contains(
        "action: inspect debug trace -> cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json"
    ));
    assert!(text.contains(
        "action: validate docking campaign -> cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json"
    ));
    assert!(text.contains(
        "action metadata: open workbench | id=open_workbench | category=demo | primary=true | requires_bundle=false"
    ));
    assert!(text.contains(
        "action metadata: inspect debug trace | id=inspect_debug_trace | category=debug | primary=false | requires_bundle=true"
    ));
    assert!(text.contains(
        "action copy command: open workbench | id=open_workbench | copy_command=fret.devtools.demo_metrics_debug.copy_action.open_workbench"
    ));
    assert!(text.contains(
        "action copy command: inspect debug trace | id=inspect_debug_trace | copy_command=fret.devtools.demo_metrics_debug.copy_action.inspect_debug_trace"
    ));
    assert!(text.contains(
        "action readiness: open workbench | id=open_workbench | category=demo | runnable=true | reason=no bundle required"
    ));
    assert!(text.contains(
        "action readiness: inspect metrics stats | id=inspect_metrics_stats | category=metrics | runnable=false | reason=select a regression bundle"
    ));
    assert!(text.contains(
        "workflow readiness: validate docking campaign | workflow_id=campaign-validate-imui-p3-multiwindow | runnable=true | reason=no inputs required"
    ));
    assert!(text.contains(
        "workflow readiness: run perf docking suite | workflow_id=perf-docking-suite-ws | runnable=false | reason=select a DevTools session"
    ));
    assert!(text.contains("workflow status: in_flight=false | last_result=- | last_error=-"));
    assert!(text.contains(
        "workflow result action: copy workflow result | command=fret.devtools.workflow.copy_result_path | enabled=false | reason=wait for workflow result artifact"
    ));
    assert!(text.contains(
        "workflow result action: open workflow JSON | command=fret.devtools.workflow.open_result_json | enabled=false | reason=wait for workflow result artifact"
    ));
    assert!(text.contains(
        "workflow artifact action: load regression summary | command=fret.devtools.workflow.load_regression_summary | enabled=false | reason=wait for workflow regression summary artifact"
    ));
    assert!(text.contains(
        "workflow artifact action: load regression index | command=fret.devtools.workflow.load_regression_index | enabled=false | reason=wait for workflow regression index artifact"
    ));
    assert!(text.contains("demo editor workbench: cargo run -p fret-demo --bin imui_editor_workbench_demo"));
    assert!(text.contains(
        "demo editor proof supporting: cargo run -p fret-demo --bin imui_editor_proof_demo"
    ));
    assert!(text.contains("demo editor notes: cargo run -p fret-demo --bin editor_notes_demo"));
    assert!(text.contains(
        "demo device shell: cargo run -p fret-demo --bin editor_notes_device_shell_demo"
    ));
    assert!(
        text.contains("metrics stats: cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json")
    );
    assert!(text.contains(
        "metrics layout perf: cargo run -p fretboard-dev -- diag layout-perf-summary <bundle-or-dir> --json"
    ));
    assert!(
        text.contains("metrics memory: cargo run -p fretboard-dev -- diag memory-summary <bundle-or-dir> --json")
    );
    assert!(
        text.contains("debug triage: cargo run -p fretboard-dev -- diag triage <bundle-or-dir> --json")
    );
    assert!(
        text.contains("debug hotspots: cargo run -p fretboard-dev -- diag hotspots <bundle-or-dir> --json")
    );
    assert!(
        text.contains("debug trace: cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json")
    );
    assert!(text.contains(
        "docking arbitration supporting: cargo run -p fret-demo --bin docking_arbitration_demo"
    ));
    assert!(text.contains(
        "docking campaign validate: cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json"
    ));
    assert!(text.contains(
        "docking policy-skip local: python tools/diag_gate_docking_wayland_policy_skip.py"
    ));
}

#[test]
fn demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates() {
    let text = demo_metrics_debug_action_command_text();
    let lines = text.lines().collect::<Vec<_>>();
    assert_eq!(
        lines.first(),
        Some(&"open workbench: cargo run -p fret-demo --bin imui_editor_workbench_demo")
    );
    assert!(lines.contains(
        &"run product discovery: python tools/diag_gate_imui_product_chain.py --only discovery"
    ));
    assert!(lines.contains(
        &"inspect metrics stats: cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json"
    ));
    assert!(lines.contains(
        &"inspect debug trace: cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json"
    ));
    assert!(lines.contains(
        &"validate docking campaign: cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json"
    ));
    assert_eq!(
        CMD_COPY_DEMO_METRICS_DEBUG_ACTIONS,
        "fret.devtools.demo_metrics_debug.copy_actions"
    );
    assert_eq!(
        CMD_RUN_DEMO_METRICS_DEBUG_DOCKING_WORKFLOW,
        "fret.devtools.demo_metrics_debug.run_docking_workflow"
    );
    assert_eq!(
        CMD_RUN_DEMO_METRICS_DEBUG_PERF_WORKFLOW,
        "fret.devtools.demo_metrics_debug.run_perf_workflow"
    );
    let metadata = demo_metrics_debug_action_metadata_lines();
    assert!(metadata.contains(&"action metadata: open workbench | id=open_workbench | category=demo | primary=true | requires_bundle=false".to_string()));
    assert!(metadata.contains(&"action metadata: inspect metrics stats | id=inspect_metrics_stats | category=metrics | primary=false | requires_bundle=true".to_string()));
    let copy_commands = demo_metrics_debug_action_copy_command_lines();
    assert!(copy_commands.contains(&"action copy command: open workbench | id=open_workbench | copy_command=fret.devtools.demo_metrics_debug.copy_action.open_workbench".to_string()));
    assert!(copy_commands.contains(&"action copy command: validate docking campaign | id=validate_docking_campaign | copy_command=fret.devtools.demo_metrics_debug.copy_action.validate_docking_campaign".to_string()));
    assert_eq!(
        demo_metrics_debug_action_command_for_copy_command(
            "fret.devtools.demo_metrics_debug.copy_action.open_workbench"
        )
        .as_deref(),
        Some(DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND)
    );
    assert!(
        demo_metrics_debug_action_command_for_copy_command(
            "fret.devtools.demo_metrics_debug.copy_action.unknown"
        )
        .is_none()
    );
    let no_bundle = demo_metrics_debug_action_readiness_lines(0);
    assert!(no_bundle.contains(&"action readiness: inspect metrics stats | id=inspect_metrics_stats | category=metrics | runnable=false | reason=select a regression bundle".to_string()));
    let with_bundle = demo_metrics_debug_action_readiness_lines(1);
    assert!(with_bundle.contains(&"action readiness: inspect metrics stats | id=inspect_metrics_stats | category=metrics | runnable=true | reason=selected bundle evidence available".to_string()));
}

#[test]
fn demo_metrics_debug_workflow_lines_surface_runtime_readiness_and_status() {
    let missing_session = demo_metrics_debug_workflow_readiness_lines(false, false);
    assert!(missing_session.contains(&"workflow readiness: validate docking campaign | workflow_id=campaign-validate-imui-p3-multiwindow | runnable=true | reason=no inputs required".to_string()));
    assert!(missing_session.contains(&"workflow readiness: run perf docking suite | workflow_id=perf-docking-suite-ws | runnable=false | reason=select a DevTools session".to_string()));

    let selected_session = demo_metrics_debug_workflow_readiness_lines(false, true);
    assert!(selected_session.contains(&"workflow readiness: run perf docking suite | workflow_id=perf-docking-suite-ws | runnable=true | reason=selected session available".to_string()));

    let in_flight = demo_metrics_debug_workflow_readiness_lines(true, true);
    assert!(in_flight.contains(&"workflow readiness: validate docking campaign | workflow_id=campaign-validate-imui-p3-multiwindow | runnable=false | reason=workflow run already in flight".to_string()));
    assert!(in_flight.contains(&"workflow readiness: run perf docking suite | workflow_id=perf-docking-suite-ws | runnable=false | reason=workflow run already in flight".to_string()));

    assert!(demo_metrics_debug_workflow_status_lines(false, None, Some("")).contains(
        &"workflow status: in_flight=false | last_result=- | last_error=-".to_string()
    ));
    assert!(demo_metrics_debug_workflow_status_lines(
        true,
        Some("target/fret-diag/devtools-workflows/perf-docking/result.json"),
        Some("suite failed")
    )
    .contains(
        &"workflow status: in_flight=true | last_result=target/fret-diag/devtools-workflows/perf-docking/result.json | last_error=suite failed".to_string()
    ));

    let missing_result = demo_metrics_debug_workflow_result_action_lines(false);
    assert!(missing_result.contains(&"workflow result action: copy workflow result | command=fret.devtools.workflow.copy_result_path | enabled=false | reason=wait for workflow result artifact".to_string()));
    assert!(missing_result.contains(&"workflow result action: open workflow JSON | command=fret.devtools.workflow.open_result_json | enabled=false | reason=wait for workflow result artifact".to_string()));

    let available_result = demo_metrics_debug_workflow_result_action_lines(true);
    assert!(available_result.contains(&"workflow result action: copy workflow result | command=fret.devtools.workflow.copy_result_path | enabled=true | reason=workflow result available".to_string()));
    assert!(available_result.contains(&"workflow result action: open workflow JSON | command=fret.devtools.workflow.open_result_json | enabled=true | reason=workflow result available".to_string()));

    let missing_artifacts = demo_metrics_debug_workflow_artifact_action_lines(false, false);
    assert!(missing_artifacts.contains(&"workflow artifact action: load regression summary | command=fret.devtools.workflow.load_regression_summary | enabled=false | reason=wait for workflow regression summary artifact".to_string()));
    assert!(missing_artifacts.contains(&"workflow artifact action: load regression index | command=fret.devtools.workflow.load_regression_index | enabled=false | reason=wait for workflow regression index artifact".to_string()));

    let available_artifacts = demo_metrics_debug_workflow_artifact_action_lines(true, true);
    assert!(available_artifacts.contains(&"workflow artifact action: load regression summary | command=fret.devtools.workflow.load_regression_summary | enabled=true | reason=workflow regression summary available".to_string()));
    assert!(available_artifacts.contains(&"workflow artifact action: load regression index | command=fret.devtools.workflow.load_regression_index | enabled=true | reason=workflow regression index available".to_string()));
}

#[test]
fn demo_metrics_debug_lines_mark_bundle_actions_runnable_with_selected_bundle() {
    let text = devtools_demo_metrics_debug_lines_with_state("target/fret-diag", 2).join("\n");
    assert!(text.contains(
        "action readiness: inspect metrics stats | id=inspect_metrics_stats | category=metrics | runnable=true | reason=selected bundle evidence available"
    ));
    assert!(text.contains(
        "action readiness: inspect debug trace | id=inspect_debug_trace | category=debug | runnable=true | reason=selected bundle evidence available"
    ));
}

#[test]
fn devtools_workflow_run_lines_surface_campaign_and_suite_entrypoints() {
    let lines = devtools_workflow_run_lines("target/fret-diag");
    let text = lines.join("\n");
    assert!(text.contains("workflow route: workflow-runs"));
    assert!(text.contains("artifacts root: target/fret-diag"));
    assert!(text.contains("result artifacts: .fret/diag/workflow-runs/*.json"));
    assert!(text.contains(
        "handoff: load suite regression.summary.json into Regression Workspace"
    ));
    assert!(text.contains(
        "handoff: run workflow summarize to create regression.index.json when missing"
    ));
    assert!(text.contains(
        "campaign validate: cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/devtools-first-open-smoke.json --json"
    ));
    assert!(text.contains(
        "imui p3 validate: cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json"
    ));
    assert!(text.contains(
        "suite ws: cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --dir target/fret-diag/devtools-workflows/perf-docking --devtools-ws-url <devtools-ws-url> --devtools-token <redacted> --devtools-session-id <selected-session> --json"
    ));
}

#[test]
fn devtools_workflow_commands_mark_suite_ws_missing_without_session() {
    let commands = devtools_workflow_commands(
        "target/fret-diag",
        "ws://127.0.0.1:7331/",
        "secret-token",
        None,
    );
    let campaign = commands
        .iter()
        .find(|command| command.id == DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID)
        .expect("first-open campaign command");
    assert!(campaign.is_runnable());
    assert_eq!(
        campaign.diag_args,
        vec![
            "campaign",
            "validate",
            "tools/diag-campaigns/devtools-first-open-smoke.json",
            "--json",
        ]
    );

    let suite = commands
        .iter()
        .find(|command| command.id == DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID)
        .expect("suite command");
    assert!(!suite.is_runnable());
    assert_eq!(suite.missing_inputs, vec!["selected-session"]);
    assert!(suite.diag_args.is_empty());
    assert!(suite.command_line.contains("--devtools-token <redacted>"));
    assert!(suite.command_line.contains("--devtools-session-id <selected-session>"));
    assert!(!suite.command_line.contains("secret-token"));
}

#[test]
fn devtools_workflow_commands_include_selected_session_for_suite_ws() {
    let commands = devtools_workflow_commands(
        "target/fret-diag",
        "ws://127.0.0.1:7331/",
        "secret-token",
        Some("session-1"),
    );
    let suite = commands
        .iter()
        .find(|command| command.id == DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID)
        .expect("suite command");
    assert!(suite.is_runnable());
    assert!(suite.missing_inputs.is_empty());
    assert_eq!(
        suite.diag_args,
        vec![
            "suite",
            "perf-docking-arbitration-steady",
            "--dir",
            "target/fret-diag/devtools-workflows/perf-docking",
            "--devtools-ws-url",
            "ws://127.0.0.1:7331/",
            "--devtools-token",
            "secret-token",
            "--devtools-session-id",
            "session-1",
            "--json",
        ]
    );
    assert!(suite.command_line.contains("--devtools-token <redacted>"));
    assert!(suite.command_line.contains("--devtools-session-id session-1"));
    assert!(!suite.command_line.contains("secret-token"));
}

#[test]
fn workflow_summarize_command_from_summary_path_targets_same_dir() {
    let command = workflow_summarize_command_from_summary_path(
        "target/fret-diag/devtools workflows/perf-docking/regression.summary.json",
    )
    .expect("summarize command");

    assert_eq!(command.id, "summarize-workflow-regression-index");
    assert!(command.is_runnable());
    assert_eq!(
        command.diag_args,
        vec![
            "summarize",
            "target/fret-diag/devtools workflows/perf-docking/regression.summary.json",
            "--dir",
            "target/fret-diag/devtools workflows/perf-docking",
            "--json",
        ]
    );
    assert!(command.command_line.contains(
        "diag summarize 'target/fret-diag/devtools workflows/perf-docking/regression.summary.json' --dir 'target/fret-diag/devtools workflows/perf-docking' --json"
    ));
    assert!(workflow_summarize_command_from_summary_path("").is_none());
    assert!(workflow_summarize_command_from_summary_path("regression.summary.json").is_none());
}

#[test]
fn workflow_regression_index_parent_dir_targets_artifact_root() {
    assert_eq!(
        workflow_regression_index_parent_dir(
            "target/fret-diag/devtools-workflows/perf-docking/regression.index.json",
        ),
        Some("target/fret-diag/devtools-workflows/perf-docking".to_string())
    );
    assert_eq!(
        workflow_regression_index_parent_dir(
            "F:\\repo\\target\\fret-diag\\devtools-workflows\\perf-docking\\regression.index.json",
        ),
        Some("F:\\repo\\target\\fret-diag\\devtools-workflows\\perf-docking".to_string())
    );
    assert!(workflow_regression_index_parent_dir("").is_none());
    assert!(workflow_regression_index_parent_dir("regression.index.json").is_none());
}

#[test]
fn workflow_aggregate_index_loaded_matches_loaded_artifact_root() {
    assert!(workflow_aggregate_index_loaded(
        Some("F:/repo/target/fret-diag/devtools-workflows/perf-docking/regression.index.json"),
        Some("F:\\repo\\target\\fret-diag\\devtools-workflows\\perf-docking\\"),
        true,
    ));
    assert!(!workflow_aggregate_index_loaded(
        Some("target/fret-diag/devtools-workflows/perf-docking/regression.index.json"),
        Some("target/fret-diag/devtools-workflows/other"),
        true,
    ));
    assert!(!workflow_aggregate_index_loaded(
        Some("target/fret-diag/devtools-workflows/perf-docking/regression.index.json"),
        Some("target/fret-diag/devtools-workflows/perf-docking"),
        false,
    ));
}

#[test]
fn workflow_regression_index_action_ids_cover_copy_open_load() {
    assert_eq!(
        CMD_COPY_WORKFLOW_SUITE_SUMMARY_PATH,
        "fret.devtools.workflow.copy_suite_summary_path"
    );
    assert_eq!(
        CMD_OPEN_WORKFLOW_SUITE_SUMMARY,
        "fret.devtools.workflow.open_suite_summary"
    );
    assert_eq!(
        CMD_COPY_WORKFLOW_REGRESSION_INDEX_PATH,
        "fret.devtools.workflow.copy_regression_index_path"
    );
    assert_eq!(
        CMD_OPEN_WORKFLOW_REGRESSION_INDEX,
        "fret.devtools.workflow.open_regression_index"
    );
    assert_eq!(
        CMD_LOAD_WORKFLOW_REGRESSION_INDEX,
        "fret.devtools.workflow.load_regression_index"
    );
}

#[test]
fn workflow_handoff_readiness_lines_project_next_action() {
    let empty =
        workflow_handoff_readiness_lines(false, false, None, None, false, false).join("\n");
    assert!(empty.contains("selected_workflow_result: none"));
    assert!(empty.contains("next_action: run selected workflow"));

    let ready = workflow_handoff_readiness_lines(
        false,
        true,
        Some("F:\\repo\\target\\fret-diag\\devtools-workflows\\perf-docking\\regression.summary.json"),
        None,
        false,
        false,
    )
    .join("\n");
    assert!(ready.contains("selected_workflow_result: loaded"));
    assert!(ready.contains("aggregate_index_ready: false"));
    assert!(ready.contains("aggregate_index_loaded: false"));
    assert!(ready.contains("next_action: Run workflow summarize"));
    assert!(ready.contains("aggregate_next_action: Run workflow summarize"));

    let indexed = workflow_handoff_readiness_lines(
        false,
        true,
        Some("F:\\repo\\target\\fret-diag\\devtools-workflows\\perf-docking\\regression.summary.json"),
        None,
        true,
        false,
    )
    .join("\n");
    assert!(indexed.contains("aggregate_index_ready: true"));
    assert!(indexed.contains("aggregate_index_loaded: false"));
    assert!(indexed.contains("regression_workspace: not loaded from workflow"));
    assert!(indexed.contains("aggregate_workspace: index ready but not loaded"));
    assert!(indexed.contains("next_action: Load workflow regression summary"));
    assert!(indexed.contains("aggregate_next_action: Load workflow regression index"));

    let loaded = workflow_handoff_readiness_lines(
        false,
        true,
        Some("F:/repo/target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"),
        Some("F:\\repo\\target\\fret-diag\\devtools-workflows\\perf-docking\\regression.summary.json"),
        true,
        false,
    )
    .join("\n");
    assert!(loaded.contains("regression_workspace: selected summary loaded from workflow"));
    assert!(loaded.contains("next_action: use Regression Workspace follow-up actions"));
    assert!(loaded.contains("aggregate_next_action: Load workflow regression index"));

    let aggregate_loaded = workflow_handoff_readiness_lines(
        false,
        true,
        Some("F:/repo/target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"),
        None,
        true,
        true,
    )
    .join("\n");
    assert!(aggregate_loaded.contains("aggregate_index_loaded: true"));
    assert!(aggregate_loaded.contains("aggregate_workspace: workflow index loaded"));
    assert!(aggregate_loaded.contains("aggregate_next_action: aggregate index already loaded"));

    let running =
        workflow_handoff_readiness_lines(true, true, None, None, false, false).join("\n");
    assert!(running.contains("selected_workflow_result: in_flight"));
    assert!(running.contains("next_action: wait for workflow result artifact"));
}

#[test]
fn devtools_gate_command_lines_surface_first_class_gates() {
    let lines = devtools_gate_command_lines("target/fret-diag");
    let text = lines.join("\n");
    assert!(text.contains("gate route: first-class-gates"));
    assert!(text.contains("artifacts root: target/fret-diag"));
    assert!(text.contains(
        "stale paint/scene: cargo run -p fretboard-dev -- diag run <script.json> --check-stale-paint <test-id> --check-stale-scene <test-id> --json"
    ));
    assert!(text.contains(
        "pixels changed: cargo run -p fretboard-dev -- diag run <script.json> --check-pixels-changed <test-id> --json"
    ));
    assert!(text.contains(
        "perf thresholds: cargo run -p fretboard-dev -- diag perf <script-or-suite> --repeat 7 --warmup-frames 5 --perf-threshold-agg p95 --max-top-total-us <us> --max-top-layout-us <us> --max-top-solve-us <us>"
    ));
    assert!(text.contains("--max-pointer-move-global-changes <count>"));
    assert!(text.contains("--max-renderer-encode-scene-text-ops <ops> --json"));
    assert!(text.contains(
        "resource footprint thresholds: cargo run -p fretboard-dev -- diag repro <script-or-suite> --max-working-set-bytes <bytes> --max-peak-working-set-bytes <bytes> --max-cpu-avg-percent-total-cores <percent> --json --launch -- <app-command>"
    ));
    assert!(text.contains(
        "resource footprint compare: cargo run -p fretboard-dev -- diag compare <baseline-session> <candidate-session> --footprint --json"
    ));
    assert!(text.contains("check.pixels_changed.json"));
    assert!(text.contains("check.perf_thresholds.json"));
    assert!(text.contains("check.resource_footprint.json"));
    assert!(text.contains("resource.footprint.json"));
}

#[test]
fn header_next_action_lines_project_recent_evidence_state() {
    let state = HeaderDiagnosticsState {
        has_session: true,
        selected_session: Some(Arc::<str>::from("session-1")),
        session_count: 2,
        scripts_count: 4,
        regression_loaded: true,
        regression_selected_summary_loaded: true,
        selected_followup_result_loaded: false,
        regression_failing_count: 3,
        recent_failed_evidence_target: None,
        recent_failed_evidence_rerunnable_kind: None,
        recent_failed_evidence_rerun_reason: Some("select failed evidence".to_string()),
        recent_evidence_next: "rerun the latest failed evidence".to_string(),
    };
    let text = header_next_action_lines_for_artifacts_root("target/fret-diag", &state).join("\n");
    assert!(text.contains("selected session: session-1"));
    assert!(text.contains("regression failing rows: 3"));
    assert!(text.contains("recent evidence next: rerun the latest failed evidence"));
}

#[test]
fn build_regression_dashboard_human_includes_totals_and_reason_codes() {
    let payload = serde_json::json!({
        "kind": "diag_regression_index",
        "out_dir": "target/fret-diag/campaigns/ui-gallery-pr",
        "summaries": [
            { "items_total": 2 },
            { "items_total": 4 }
        ],
        "counters": {
            "by_status": { "passed": 4, "failed_deterministic": 2 },
            "by_lane": { "smoke": 1 },
            "by_tool": { "suite": 1 }
        },
        "top_reason_codes": [
            { "reason_code": "pixel_diff", "count": 2 }
        ],
        "failing_summaries": [
            { "path": "runs/a/regression.summary.json", "lane": "smoke", "failures": 2, "items_total": 4 }
        ]
    });

    let human = build_regression_dashboard_human(
        Path::new("F:/repo/target/fret-diag/campaigns/ui-gallery-pr/regression.index.json"),
        &payload,
        5,
    );

    assert!(human.contains("summaries_total: 2"));
    assert!(human.contains("items_total: 6"));
    assert!(human.contains("top reason codes:"));
    assert!(human.contains("pixel_diff: 2"));
}

#[test]
fn regression_failing_summary_rows_reads_ranked_rows() {
    let rows = regression_failing_summary_rows(
        &serde_json::json!({
            "failing_summaries": [
                { "path": "a/regression.summary.json", "lane": "smoke", "failures": 2, "items_total": 4 },
                { "path": "b/regression.summary.json", "lane": "perf", "failures": 1, "items_total": 3 }
            ]
        })
        .to_string(),
        1,
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].path, "a/regression.summary.json");
    assert_eq!(rows[0].lane, "smoke");
}

#[test]
fn load_regression_summary_drilldown_collects_failed_bundle_dirs() {
    let dir = std::env::temp_dir().join(format!(
        "fret-devtools-regression-drilldown-{}-{}",
        std::process::id(),
        now_unix_ms()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("regression.summary.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "diag_regression_summary",
        "campaign": { "name": "ui-gallery-pr", "lane": "smoke" },
        "run": { "run_id": "run-1", "created_unix_ms": 1, "tool": "suite" },
        "totals": { "items_total": 2, "passed": 1, "failed_deterministic": 1, "failed_flaky": 0, "failed_tooling": 0, "failed_timeout": 0, "skipped_policy": 0, "quarantined": 0 },
        "items": [
            { "item_id": "a", "kind": "script", "name": "a", "status": "passed", "lane": "smoke" },
            {
                "item_id": "b",
                "kind": "script",
                "name": "b",
                "status": "failed_deterministic",
                "lane": "smoke",
                "evidence": { "bundle_dir": "target/fret-diag/runs/bundle-a" }
            },
            {
                "item_id": "c",
                "kind": "script",
                "name": "c",
                "status": "failed_tooling",
                "lane": "smoke",
                "evidence": { "bundle_dir": "target/fret-diag/runs/bundle-a" }
            }
        ]
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();

    let data = load_regression_summary_drilldown(&path).expect("load drilldown");
    assert!(data.summary_json.contains("failed_deterministic"));
    assert_eq!(
        data.bundle_dirs,
        vec!["target/fret-diag/runs/bundle-a".to_string()]
    );
    assert!(data.capability_sources.is_empty());
    assert!(data.capabilities_check_paths.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_regression_summary_drilldown_collects_policy_skip_capability_checks() {
    let dir = std::env::temp_dir().join(format!(
        "fret-devtools-regression-drilldown-policy-{}-{}",
        std::process::id(),
        now_unix_ms()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("regression.summary.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "diag_regression_summary",
        "campaign": { "name": "ui-gallery-pr", "lane": "smoke" },
        "run": { "run_id": "run-1", "created_unix_ms": 1, "tool": "suite" },
        "totals": { "items_total": 1, "passed": 0, "failed_deterministic": 0, "failed_flaky": 0, "failed_tooling": 0, "failed_timeout": 0, "skipped_policy": 1, "quarantined": 0 },
        "items": [
            {
                "item_id": "capability-check",
                "kind": "script",
                "name": "capability-check",
                "status": "skipped_policy",
                "lane": "smoke",
                "reason_code": "capability.missing",
                "evidence": {
                    "extra": {
                        "capability_source": {
                            "kind": "filesystem",
                            "path": "target/fret-diag/capabilities.json",
                            "label": "filesystem:target/fret-diag/capabilities.json",
                            "transport": "filesystem",
                            "session_id": null
                        },
                        "capabilities_check_path": "target/fret-diag/campaigns/ui-gallery/check.capabilities.json"
                    }
                }
            }
        ]
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();

    let data = load_regression_summary_drilldown(&path).expect("load drilldown");
    assert!(data.bundle_dirs.is_empty());
    assert_eq!(
        data.capability_sources,
        vec!["target/fret-diag/capabilities.json".to_string()]
    );
    assert_eq!(
        data.capabilities_check_paths,
        vec!["target/fret-diag/campaigns/ui-gallery/check.capabilities.json".to_string()]
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_regression_summary_drilldown_collects_perf_evidence() {
    let dir = std::env::temp_dir().join(format!(
        "fret-devtools-regression-drilldown-perf-{}-{}",
        std::process::id(),
        now_unix_ms()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("regression.summary.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "diag_regression_summary",
        "campaign": { "name": "perf-docking", "lane": "perf" },
        "run": { "run_id": "run-1", "created_unix_ms": 1, "tool": "fretboard-dev diag perf" },
        "totals": { "items_total": 1, "passed": 0, "failed_deterministic": 1, "failed_flaky": 0, "failed_tooling": 0, "failed_timeout": 0, "skipped_policy": 0, "quarantined": 0 },
        "items": [
            {
                "item_id": "perf-case",
                "kind": "perf_case",
                "name": "docking steady drag",
                "status": "failed_deterministic",
                "lane": "perf",
                "evidence": {
                    "bundle_artifact": "target/fret-diag/perf-docking/run-a/bundle.schema2.json",
                    "triage_artifact": "target/fret-diag/perf-docking/run-a/triage.json",
                    "script_result": "target/fret-diag/perf-docking/run-a/script.result.json",
                    "screenshots_manifest": "target/fret-diag/perf-docking/run-a/screenshots.manifest.json",
                    "share_artifact": "target/fret-diag/perf-docking/share/perf-case.ai.zip",
                    "perf_summary_json": "target/fret-diag/perf-docking/layout.perf.summary.v1.json",
                    "compare_json": "target/fret-diag/perf-docking/check.perf_thresholds.json",
                    "extra": {
                        "metrics": {
                            "top_total_time_us": 24000,
                            "top_renderer_encode_scene_us": 6000,
                            "top_renderer_instance_bytes": 700000,
                            "stats": {
                                "total_time_us": 24000,
                                "top_renderer_encode_scene_us": 6000
                            }
                        },
                        "threshold_failures": [
                            {
                                "metric": "top_total_time_us",
                                "observed": 24000,
                                "threshold": 20000,
                                "evidence_bundle": "target/fret-diag/perf-docking/run-threshold/bundle.schema2.json"
                            }
                        ]
                    }
                }
            }
        ]
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();

    let data = load_regression_summary_drilldown(&path).expect("load drilldown");
    assert_eq!(
        data.bundle_dirs,
        vec![
            "target/fret-diag/perf-docking/run-threshold".to_string(),
            "target/fret-diag/perf-docking/run-a".to_string(),
        ]
    );
    let text = data.perf_evidence_lines.join("\n");
    assert!(text.contains(
        "docking steady drag [failed_deterministic] perf_summary_json: target/fret-diag/perf-docking/layout.perf.summary.v1.json"
    ));
    assert!(text.contains(
        "docking steady drag [failed_deterministic] compare_json: target/fret-diag/perf-docking/check.perf_thresholds.json"
    ));
    assert!(text.contains(
        "docking steady drag [failed_deterministic] metric top_total_time_us: 24000"
    ));
    assert!(text.contains(
        "docking steady drag [failed_deterministic] metric top_renderer_encode_scene_us: 6000"
    ));
    assert!(text.contains(
        "docking steady drag [failed_deterministic] metric top_renderer_instance_bytes: 700000"
    ));
    assert!(text.contains("docking steady drag [failed_deterministic] threshold_failures: 1"));
    assert!(text.contains("threshold_failures_json"));
    let first_open_text = data.first_open_evidence_lines.join("\n");
    assert!(first_open_text.contains(
        "docking steady drag [failed_deterministic] triage_artifact: target/fret-diag/perf-docking/run-a/triage.json"
    ));
    assert!(first_open_text.contains(
        "docking steady drag [failed_deterministic] script_result: target/fret-diag/perf-docking/run-a/script.result.json"
    ));
    assert!(first_open_text.contains(
        "docking steady drag [failed_deterministic] screenshots_manifest: target/fret-diag/perf-docking/run-a/screenshots.manifest.json"
    ));
    assert!(first_open_text.contains(
        "docking steady drag [failed_deterministic] share_artifact: target/fret-diag/perf-docking/share/perf-case.ai.zip"
    ));
    assert_eq!(
        data.share_artifacts,
        vec!["target/fret-diag/perf-docking/share/perf-case.ai.zip".to_string()]
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn runnable_followup_command_action_lines_surface_indexed_bundle_commands() {
    let commands = regression_bundle_followup_commands([
        "target/fret-diag/perf-docking/run-threshold",
        "target/fret-diag/perf-docking/run-a",
    ]);

    let lines = runnable_followup_command_action_lines(&commands);
    assert!(lines.contains(&"diag stats (stats)".to_string()));
    assert!(lines.contains(&"triage (triage)".to_string()));
    assert!(lines.contains(&"trace (trace)".to_string()));
    assert!(lines.contains(&"diag stats [2] (stats-2)".to_string()));
    assert!(lines.contains(&"triage [2] (triage-2)".to_string()));
    assert!(lines.contains(&"trace [2] (trace-2)".to_string()));
    assert!(
        !lines
            .iter()
            .any(|line| line.contains("visual compare") || line.contains("footprint compare"))
    );
}

#[test]
fn selected_followup_readiness_lines_summarize_next_runnable_command() {
    let commands =
        regression_bundle_followup_commands(["target/fret-diag/perf-docking/run-threshold"]);

    let lines = selected_followup_readiness_lines(1, &commands, "", "");
    let text = lines.join("\n");

    assert!(text.contains("selected_bundle_dirs: 1"));
    assert!(text.contains("runnable_followups: 6"));
    assert!(text.contains("manual_compare_followups: 2"));
    assert!(text.contains("visual_compare_ready: false"));
    assert!(text.contains("footprint_compare_ready: false"));
    assert!(text.contains("first_runnable: diag stats (stats)"));
    assert!(text.contains("diag stats target/fret-diag/perf-docking/run-threshold --json"));

    let ready = selected_followup_readiness_lines(
        1,
        &commands,
        "target/fret-diag/baseline/bundle.schema2.json",
        "target/fret-diag/baseline-session",
    )
    .join("\n");
    assert!(ready.contains("visual_compare_ready: true"));
    assert!(ready.contains("footprint_compare_ready: true"));
}

#[test]
fn materialize_baseline_compare_followup_command_fills_diag_args() {
    let commands =
        regression_bundle_followup_commands(["target/fret-diag/perf-docking/run-candidate"]);
    let visual = commands
        .iter()
        .find(|command| command.id == "visual-compare")
        .expect("visual compare command");
    let visual = materialize_baseline_compare_followup_command(
        visual,
        "target/fret-diag/baseline/bundle schema2.json",
    )
    .expect("visual command");
    assert!(!visual.requires_baseline);
    assert_eq!(
        visual.diag_args,
        vec![
            "compare".to_string(),
            "target/fret-diag/baseline/bundle schema2.json".to_string(),
            "target/fret-diag/perf-docking/run-candidate".to_string(),
            "--json".to_string(),
        ]
    );
    assert!(visual.command_line.contains(
        "diag compare 'target/fret-diag/baseline/bundle schema2.json' target/fret-diag/perf-docking/run-candidate --json"
    ));

    let footprint = commands
        .iter()
        .find(|command| command.id == "footprint-compare")
        .expect("footprint compare command");
    let footprint = materialize_baseline_compare_followup_command(
        footprint,
        "target/fret-diag/baseline-session",
    )
    .expect("footprint command");
    assert!(!footprint.requires_baseline);
    assert_eq!(
        footprint.diag_args,
        vec![
            "compare".to_string(),
            "target/fret-diag/baseline-session".to_string(),
            "target/fret-diag/perf-docking/run-candidate".to_string(),
            "--footprint".to_string(),
            "--json".to_string(),
        ]
    );
    assert!(footprint.command_line.contains("--footprint --json"));
}

#[test]
fn load_regression_summary_drilldown_falls_back_to_capability_source_label() {
    let dir = std::env::temp_dir().join(format!(
        "fret-devtools-regression-drilldown-source-label-{}-{}",
        std::process::id(),
        now_unix_ms()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("regression.summary.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "diag_regression_summary",
        "campaign": { "name": "ui-gallery-pr", "lane": "smoke" },
        "run": { "run_id": "run-1", "created_unix_ms": 1, "tool": "suite" },
        "totals": { "items_total": 1, "passed": 0, "failed_deterministic": 0, "failed_flaky": 0, "failed_tooling": 0, "failed_timeout": 0, "skipped_policy": 1, "quarantined": 0 },
        "items": [
            {
                "item_id": "capability-check",
                "kind": "script",
                "name": "capability-check",
                "status": "skipped_policy",
                "lane": "smoke",
                "reason_code": "capability.missing",
                "source": {
                    "metadata": {
                        "capability_source": {
                            "kind": "transport_session",
                            "path": null,
                            "label": "devtools_ws:session-123",
                            "transport": "devtools_ws",
                            "session_id": "session-123"
                        }
                    }
                }
            }
        ]
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&payload).unwrap()).unwrap();

    let data = load_regression_summary_drilldown(&path).expect("load drilldown");
    assert!(data.bundle_dirs.is_empty());
    assert_eq!(
        data.capability_sources,
        vec!["devtools_ws:session-123".to_string()]
    );
    assert!(data.capabilities_check_paths.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}
