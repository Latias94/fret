use crate::recent_evidence::RecentEvidenceTarget;
use crate::short_artifact_result_path;
use crate::workflow_run;
use crate::{
    DEVTOOLS_DOGFOOD_BASE_SCRIPT, DEVTOOLS_DOGFOOD_BUTTON_SCRIPT,
    DEVTOOLS_DOGFOOD_PACK_COMMAND, DEVTOOLS_DOGFOOD_PICK_APPLY_COMMAND,
    DEVTOOLS_DOGFOOD_PICK_SCRIPT_COMMAND, DEVTOOLS_DOGFOOD_RUN_PACK_COMMAND,
    DEVTOOLS_DOGFOOD_TARGET_COMMAND, DEVTOOLS_DOGFOOD_VIEWER_COMMAND,
    DEVTOOLS_DOGFOOD_WORKFLOW_ID, DEVTOOLS_FIRST_OPEN_CAMPAIGN_ID, DEVTOOLS_FIRST_OPEN_DOC,
    DEVTOOLS_FIRST_OPEN_GATE_COMMAND, DEVTOOLS_GUI_BRANCH_DOC,
    DEVTOOLS_REPO_PREFLIGHT_COMMAND, DEVTOOLS_REPO_PREFLIGHT_JSON_COMMAND,
    DEVTOOLS_WORKFLOW_FIRST_OPEN_CAMPAIGN_MANIFEST, DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID,
    DEVTOOLS_WORKFLOW_IMUI_P3_CAMPAIGN_MANIFEST, DEVTOOLS_WORKFLOW_IMUI_P3_VALIDATE_ID,
    DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE, DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID,
    DEVTOOLS_WORKFLOW_ROUTE_ID, IMUI_PRODUCT_WORKFLOW_ARTIFACTS, IMUI_PRODUCT_WORKFLOW_COMMAND,
    IMUI_PRODUCT_WORKFLOW_DOC, IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND,
    IMUI_PRODUCT_WORKFLOW_ID, IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND,
    IMUI_PRODUCT_WORKFLOW_SUITE,
};
use fret_diag::devtools_gate_profile_lines;

#[allow(clippy::too_many_arguments)]
pub(super) fn devtools_first_open_next_action_lines(
    has_session: bool,
    session_count: usize,
    selected_session_id: Option<&str>,
    scripts_count: usize,
    regression_loaded: bool,
    selected_summary_loaded: bool,
    selected_followup_result_loaded: bool,
    failing_count: usize,
    artifacts_root: &str,
    recent_failed_evidence: Option<&RecentEvidenceTarget>,
    recent_failed_evidence_rerunnable_kind: Option<&'static str>,
    recent_failed_evidence_rerun_unavailable_reason: Option<&str>,
    recent_evidence_next_action: &str,
) -> Vec<String> {
    let artifacts_root = artifacts_root.trim();
    let artifacts_root = if artifacts_root.is_empty() {
        "<unset>"
    } else {
        artifacts_root
    };
    let target = if has_session {
        "target: session selected; inspect, pick, dump, and screenshot actions are ready".to_string()
    } else if session_count > 0 {
        format!("target: {session_count} session(s) available; select one before inspecting")
    } else {
        "target: no session yet; launch a Fret app with diagnostics enabled".to_string()
    };
    let session_scope = if has_session {
        let selected = selected_session_id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("<selected>");
        if session_count > 1 {
            format!(
                "session scope: selected {selected}; {session_count} sessions connected, use the Session selector to retarget inspect, bundle, screenshot, and selected-session suite actions"
            )
        } else {
            format!(
                "session scope: selected {selected}; actions target the current diagnostics session"
            )
        }
    } else if session_count > 0 {
        "session scope: choose one available session before sending inspect, bundle, screenshot, or selected-session suite actions".to_string()
    } else {
        "session scope: waiting for the first diagnostics session".to_string()
    };
    let regression = if regression_loaded {
        format!("regression: aggregate loaded with {failing_count} non-passing summary row(s)")
    } else if selected_summary_loaded && selected_followup_result_loaded {
        "regression: selected follow-up result loaded; inspect Follow-up Result Summary/History"
            .to_string()
    } else if selected_summary_loaded {
        "regression: selected summary loaded; follow-up actions can use selected bundle evidence"
            .to_string()
    } else {
        "regression: no aggregate loaded; run a script, then Refresh or Summarize".to_string()
    };
    let recent_evidence = recent_failed_evidence
        .map(|target| {
            format!(
                "recent evidence: failed {} {} ({})",
                target.kind,
                target.id,
                short_artifact_result_path(&target.result_path)
            )
        })
        .unwrap_or_else(|| "recent evidence: no failed restored GUI-launched evidence".to_string());
    let recent_command = recent_failed_evidence
        .map(|target| format!("recent evidence command: {}", target.command_line))
        .unwrap_or_else(|| {
            "recent evidence command: <none; run a workflow or generated gate>".to_string()
        });
    let recent_rerun = match (recent_failed_evidence, recent_failed_evidence_rerunnable_kind) {
        (Some(_), Some(kind)) => format!("recent evidence rerun: available ({kind})"),
        (Some(_), None) => recent_failed_evidence_rerun_unavailable_reason
            .map(|reason| format!("recent evidence rerun: unavailable ({reason})"))
            .unwrap_or_else(|| "recent evidence rerun: unavailable".to_string()),
        (None, _) => "recent evidence rerun: <none>".to_string(),
    };
    vec![
        target,
        session_scope,
        format!("scripts: {scripts_count} available in Script Studio"),
        regression,
        recent_evidence,
        recent_command,
        recent_rerun,
        format!("recent evidence next: {recent_evidence_next_action}"),
        format!("artifacts root: {artifacts_root}"),
        "guide: open Evidence & Results -> Guide for docs, dogfood, workflow runs, demo/metrics, and gate commands".to_string(),
    ]
}

pub(super) fn devtools_first_open_lines(artifacts_root: &str) -> Vec<String> {
    let artifacts_root = artifacts_root.trim();
    let artifacts_root = if artifacts_root.is_empty() {
        "<unset>"
    } else {
        artifacts_root
    };
    vec![
        format!("first-open: {DEVTOOLS_FIRST_OPEN_DOC}"),
        format!("gui branch: {DEVTOOLS_GUI_BRANCH_DOC}"),
        format!("repo preflight: {DEVTOOLS_REPO_PREFLIGHT_COMMAND}"),
        format!("repo preflight json: {DEVTOOLS_REPO_PREFLIGHT_JSON_COMMAND}"),
        format!("artifacts root: {artifacts_root}"),
        "direct loop: diag run -> diag latest -> diag compare".to_string(),
        format!(
            "campaign loop: diag campaign run {DEVTOOLS_FIRST_OPEN_CAMPAIGN_ID} -> diag summarize -> diag dashboard"
        ),
        format!("gate: {DEVTOOLS_FIRST_OPEN_GATE_COMMAND}"),
        format!("product workflow: {IMUI_PRODUCT_WORKFLOW_ID}"),
        format!("product workflow command: {IMUI_PRODUCT_WORKFLOW_COMMAND}"),
        format!("product workflow focused: {IMUI_PRODUCT_WORKFLOW_FOCUSED_COMMAND}"),
        format!("product workflow launched: {IMUI_PRODUCT_WORKFLOW_LAUNCHED_COMMAND}"),
        format!("product workflow suite: {IMUI_PRODUCT_WORKFLOW_SUITE}"),
        format!("product workflow docs: {IMUI_PRODUCT_WORKFLOW_DOC}"),
        format!(
            "product workflow artifacts: {}",
            IMUI_PRODUCT_WORKFLOW_ARTIFACTS.join(", ")
        ),
    ]
}

pub(super) fn devtools_dogfood_workflow_lines(artifacts_root: &str) -> Vec<String> {
    let artifacts_root = artifacts_root.trim();
    let artifacts_root = if artifacts_root.is_empty() {
        "<unset>"
    } else {
        artifacts_root
    };
    vec![
        format!("dogfood workflow: {DEVTOOLS_DOGFOOD_WORKFLOW_ID}"),
        format!("dogfood docs: {DEVTOOLS_GUI_BRANCH_DOC}"),
        format!("artifacts root: {artifacts_root}"),
        format!("open ui gallery: {DEVTOOLS_DOGFOOD_TARGET_COMMAND}"),
        "pick target: enable inspect -> Pick -> click a Button page control".to_string(),
        "preferred selector: {\"kind\":\"test_id\",\"id\":\"ui-gallery-nav-button\"}".to_string(),
        format!("base script: {DEVTOOLS_DOGFOOD_BASE_SCRIPT}"),
        format!("button script: {DEVTOOLS_DOGFOOD_BUTTON_SCRIPT}"),
        format!("generate script from pick: {DEVTOOLS_DOGFOOD_PICK_SCRIPT_COMMAND}"),
        format!("apply pick to script: {DEVTOOLS_DOGFOOD_PICK_APPLY_COMMAND}"),
        format!("run and pack: {DEVTOOLS_DOGFOOD_RUN_PACK_COMMAND}"),
        format!("pack selected bundle: {DEVTOOLS_DOGFOOD_PACK_COMMAND}"),
        format!("open viewer: {DEVTOOLS_DOGFOOD_VIEWER_COMMAND}"),
        "viewer input: drag bundle.json, bundle.schema2.json, or the packed zip into the offline viewer"
            .to_string(),
    ]
}

pub(super) fn devtools_workflow_run_lines(artifacts_root: &str) -> Vec<String> {
    let artifacts_root = artifacts_root.trim();
    let artifacts_root = if artifacts_root.is_empty() {
        "<unset>"
    } else {
        artifacts_root
    };
    vec![
        format!("workflow route: {DEVTOOLS_WORKFLOW_ROUTE_ID}"),
        format!("artifacts root: {artifacts_root}"),
        "result artifacts: .fret/diag/workflow-runs/*.json".to_string(),
        "handoff: load suite regression.summary.json into Regression Workspace".to_string(),
        "handoff: run workflow summarize to create regression.index.json when missing".to_string(),
        format!(
            "campaign validate: cargo run -p fretboard-dev -- diag campaign validate {DEVTOOLS_WORKFLOW_FIRST_OPEN_CAMPAIGN_MANIFEST} --json"
        ),
        format!(
            "imui p3 validate: cargo run -p fretboard-dev -- diag campaign validate {DEVTOOLS_WORKFLOW_IMUI_P3_CAMPAIGN_MANIFEST} --json"
        ),
        format!(
            "suite ws: cargo run -p fretboard-dev -- diag suite {DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE} --dir {artifacts_root}/devtools-workflows/perf-docking --devtools-ws-url <devtools-ws-url> --devtools-token <redacted> --devtools-session-id <selected-session> --json"
        ),
    ]
}

pub(super) fn devtools_workflow_commands(
    artifacts_root: &str,
    ws_url: &str,
    token: &str,
    selected_session_id: Option<&str>,
) -> Vec<workflow_run::DevtoolsWorkflowRunCommandV1> {
    let workflow_out_dir = workflow_run_artifacts_dir(artifacts_root);
    let selected_session = selected_session_id
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let selected_session_or_placeholder = selected_session.unwrap_or("<selected-session>");
    let mut suite_missing_inputs = Vec::new();
    if selected_session.is_none() {
        suite_missing_inputs.push("selected-session".to_string());
    }

    vec![
        workflow_campaign_validate_command(
            DEVTOOLS_WORKFLOW_FIRST_OPEN_VALIDATE_ID,
            "Validate devtools first-open campaign",
            DEVTOOLS_WORKFLOW_FIRST_OPEN_CAMPAIGN_MANIFEST,
        ),
        workflow_campaign_validate_command(
            DEVTOOLS_WORKFLOW_IMUI_P3_VALIDATE_ID,
            "Validate IMUI P3 multi-window campaign",
            DEVTOOLS_WORKFLOW_IMUI_P3_CAMPAIGN_MANIFEST,
        ),
        workflow_run::DevtoolsWorkflowRunCommandV1 {
            id: DEVTOOLS_WORKFLOW_PERF_DOCKING_WS_ID.to_string(),
            label: "Run perf docking suite over selected session".to_string(),
            command_line: format!(
                "cargo run -p fretboard-dev -- diag suite {DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE} --dir {workflow_out_dir} --devtools-ws-url {ws_url} --devtools-token <redacted> --devtools-session-id {selected_session_or_placeholder} --json"
            ),
            diag_args: selected_session
                .map(|session_id| {
                    vec![
                        "suite".to_string(),
                        DEVTOOLS_WORKFLOW_PERF_DOCKING_SUITE.to_string(),
                        "--dir".to_string(),
                        workflow_out_dir.clone(),
                        "--devtools-ws-url".to_string(),
                        ws_url.to_string(),
                        "--devtools-token".to_string(),
                        token.to_string(),
                        "--devtools-session-id".to_string(),
                        session_id.to_string(),
                        "--json".to_string(),
                    ]
                })
                .unwrap_or_default(),
            missing_inputs: suite_missing_inputs,
        },
    ]
}

fn workflow_campaign_validate_command(
    id: &str,
    label: &str,
    manifest: &str,
) -> workflow_run::DevtoolsWorkflowRunCommandV1 {
    workflow_run::DevtoolsWorkflowRunCommandV1 {
        id: id.to_string(),
        label: label.to_string(),
        command_line: format!(
            "cargo run -p fretboard-dev -- diag campaign validate {manifest} --json"
        ),
        diag_args: vec![
            "campaign".to_string(),
            "validate".to_string(),
            manifest.to_string(),
            "--json".to_string(),
        ],
        missing_inputs: Vec::new(),
    }
}

fn workflow_run_artifacts_dir(artifacts_root: &str) -> String {
    let root = artifacts_root.trim();
    let root = if root.is_empty() {
        "target/fret-diag"
    } else {
        root
    };
    format!(
        "{}/devtools-workflows/perf-docking",
        root.trim_end_matches(['/', '\\'])
    )
}

pub(super) fn workflow_handoff_readiness_lines(
    workflow_run_in_flight: bool,
    selected_result_loaded: bool,
    regression_summary_path: Option<&str>,
    loaded_regression_summary_path: Option<&str>,
    aggregate_index_ready: bool,
    aggregate_index_loaded: bool,
) -> Vec<String> {
    let selected_state = if workflow_run_in_flight {
        "in_flight"
    } else if selected_result_loaded {
        "loaded"
    } else {
        "none"
    };
    let artifact = regression_summary_path
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let loaded = loaded_regression_summary_path
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let already_loaded = artifact
        .zip(loaded)
        .is_some_and(|(artifact, loaded)| {
            normalize_workflow_artifact_path(artifact) == normalize_workflow_artifact_path(loaded)
        });

    let mut lines = vec![
        format!("selected_workflow_result: {selected_state}"),
        format!(
            "regression_summary_artifact: {}",
            artifact.unwrap_or("<none>")
        ),
        format!(
            "aggregate_index_ready: {}",
            if aggregate_index_ready { "true" } else { "false" }
        ),
        format!(
            "aggregate_index_loaded: {}",
            if aggregate_index_loaded {
                "true"
            } else {
                "false"
            }
        ),
    ];
    if already_loaded {
        lines.push("regression_workspace: selected summary loaded from workflow".to_string());
    } else if artifact.is_some() {
        lines.push("regression_workspace: not loaded from workflow".to_string());
    } else {
        lines.push("regression_workspace: <not applicable>".to_string());
    }
    if aggregate_index_loaded {
        lines.push("aggregate_workspace: workflow index loaded".to_string());
    } else if aggregate_index_ready {
        lines.push("aggregate_workspace: index ready but not loaded".to_string());
    } else {
        lines.push("aggregate_workspace: <not applicable>".to_string());
    }

    let next_action = if workflow_run_in_flight {
        "wait for workflow result artifact"
    } else if !selected_result_loaded {
        "run selected workflow"
    } else if artifact.is_some() && !aggregate_index_ready {
        "Run workflow summarize"
    } else if already_loaded {
        "use Regression Workspace follow-up actions"
    } else if artifact.is_some() {
        "Load workflow regression summary"
    } else {
        "selected workflow result has no regression.summary.json artifact"
    };
    lines.push(format!("next_action: {next_action}"));
    let aggregate_next_action = if workflow_run_in_flight {
        "wait for workflow result artifact"
    } else if !selected_result_loaded {
        "run selected workflow"
    } else if aggregate_index_loaded {
        "aggregate index already loaded"
    } else if aggregate_index_ready {
        "Load workflow regression index"
    } else if artifact.is_some() {
        "Run workflow summarize"
    } else {
        "selected workflow result has no regression.index.json artifact"
    };
    lines.push(format!("aggregate_next_action: {aggregate_next_action}"));
    lines
}

fn normalize_workflow_artifact_path(path: &str) -> String {
    path.trim().replace('\\', "/")
}

pub(super) fn devtools_gate_command_lines(artifacts_root: &str) -> Vec<String> {
    devtools_gate_profile_lines(artifacts_root)
}
