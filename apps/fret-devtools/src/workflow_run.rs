use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;

use fret_app::App;
use serde::Serialize;

use crate::{State, now_unix_ms, push_log, repo_root_from_script_paths};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevtoolsWorkflowRunCommandV1 {
    pub id: String,
    pub label: String,
    pub command_line: String,
    pub diag_args: Vec<String>,
    pub missing_inputs: Vec<String>,
}

impl DevtoolsWorkflowRunCommandV1 {
    pub(crate) fn is_runnable(&self) -> bool {
        self.missing_inputs.is_empty() && !self.diag_args.is_empty()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WorkflowRunJobResult {
    pub id: String,
    pub label: String,
    pub command_line: String,
    pub result_path: PathBuf,
    pub result_json: String,
    pub result: Result<(), String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkflowRunResultHistoryEntry {
    pub id: String,
    pub label: String,
    pub command_line: String,
    pub result_path: String,
    pub result_json: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct WorkflowRunResultRecordV1 {
    schema_version: u32,
    kind: &'static str,
    id: String,
    label: String,
    command_line: String,
    diag_args: Vec<String>,
    missing_inputs: Vec<String>,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    output_artifacts: Vec<WorkflowRunOutputArtifactV1>,
    started_unix_ms: u64,
    finished_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
struct WorkflowRunOutputArtifactV1 {
    kind: &'static str,
    path: String,
}

pub(crate) fn poll_workflow_run_jobs(app: &mut App, st: &mut State) {
    while let Ok(msg) = st.workflow_run_rx.try_recv() {
        let result_path_text = msg.result_path.to_string_lossy().to_string();
        let _ = app
            .models_mut()
            .update(&st.workflow_run_in_flight, |v| *v = false);
        let _ = app
            .models_mut()
            .update(&st.workflow_run_last_command_line, |v| {
                *v = Some(Arc::<str>::from(msg.command_line.clone()))
            });
        let _ = app
            .models_mut()
            .update(&st.workflow_run_last_result_path, |v| {
                *v = Some(Arc::<str>::from(result_path_text.clone()))
            });
        let _ = app
            .models_mut()
            .update(&st.workflow_run_selected_result_path, |v| {
                *v = Some(Arc::<str>::from(result_path_text.clone()))
            });
        let _ = app
            .models_mut()
            .update(&st.workflow_run_last_result_json, |v| {
                *v = msg.result_json.clone()
            });
        let _ = app
            .models_mut()
            .update(&st.workflow_run_result_history, |v| {
                v.insert(0, WorkflowRunResultHistoryEntry::from_job_result(&msg));
                v.truncate(32);
            });

        match msg.result {
            Ok(()) => {
                let _ = app
                    .models_mut()
                    .update(&st.workflow_run_last_error, |v| *v = None);
                push_log(
                    app,
                    &st.log_lines,
                    &format!(
                        "workflow run ok: {} ({}) result={}",
                        msg.label,
                        msg.id,
                        msg.result_path.to_string_lossy()
                    ),
                );
            }
            Err(err) => {
                let _ = app
                    .models_mut()
                    .update(&st.workflow_run_last_error, |v| {
                        *v = Some(Arc::<str>::from(err.clone()))
                    });
                push_log(
                    app,
                    &st.log_lines,
                    &format!(
                        "workflow run failed: {} ({}) result={}: {err}",
                        msg.label,
                        msg.id,
                        msg.result_path.to_string_lossy()
                    ),
                );
            }
        }
    }
}

pub(crate) fn start_workflow_run(
    app: &mut App,
    st: &mut State,
    command: DevtoolsWorkflowRunCommandV1,
) -> Result<(), String> {
    let in_flight = app
        .models()
        .read(&st.workflow_run_in_flight, |v| *v)
        .unwrap_or(false);
    if in_flight {
        return Err("workflow run already in progress".to_string());
    }
    if !command.is_runnable() {
        return Err(format!(
            "workflow run requires inputs: {}",
            command.missing_inputs.join(", ")
        ));
    }

    let repo_root = repo_root_from_script_paths(&st.script_paths);
    let result_dir = repo_root.join(".fret").join("diag").join("workflow-runs");
    std::fs::create_dir_all(&result_dir).map_err(|err| {
        format!(
            "failed to create workflow run result dir {}: {err}",
            result_dir.to_string_lossy()
        )
    })?;
    let started_unix_ms = now_unix_ms();
    let id = command.id.clone();
    let label = command.label.clone();
    let command_line = command.command_line.clone();
    let result_path = result_dir.join(format!("{started_unix_ms}-{id}.json"));
    let tx = st.workflow_run_tx.clone();

    std::thread::spawn({
        let id = id.clone();
        let label = label.clone();
        let command_line = command_line.clone();
        let command = command.clone();
        let result_path = result_path.clone();
        move || {
            let diag_args = command.diag_args.clone();
            let result = fret_diag::diag_cmd(diag_args.clone());
            let finished_unix_ms = now_unix_ms();
            let record = build_workflow_run_result_record(
                &command,
                diag_args,
                started_unix_ms,
                finished_unix_ms,
                &result,
            );
            let result_json = workflow_run_result_record_json(&record)
                .unwrap_or_else(|err| fallback_workflow_run_result_json(&err));
            let write_result = write_workflow_run_result_record(&result_path, &result_json);
            let result = match (result, write_result) {
                (Ok(()), Ok(())) => Ok(()),
                (Err(err), Ok(())) => Err(err),
                (Ok(()), Err(write_err)) => Err(write_err),
                (Err(err), Err(write_err)) => {
                    Err(format!("{err}; workflow run result write failed: {write_err}"))
                }
            };
            let _ = tx.send(WorkflowRunJobResult {
                id,
                label,
                command_line,
                result_path,
                result_json,
                result,
            });
        }
    });

    let _ = app
        .models_mut()
        .update(&st.workflow_run_in_flight, |v| *v = true);
    let _ = app
        .models_mut()
        .update(&st.workflow_run_last_command_line, |v| {
            *v = Some(Arc::<str>::from(command_line))
        });
    let _ = app
        .models_mut()
        .update(&st.workflow_run_last_result_path, |v| {
            *v = Some(Arc::<str>::from(
                result_path.to_string_lossy().to_string(),
            ))
        });
    let _ = app
        .models_mut()
        .update(&st.workflow_run_last_result_json, |v| {
            *v = serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "kind": "fret_devtools_workflow_run_result",
                "status": "running",
                "id": id.as_str(),
            }))
            .unwrap_or_else(|err| fallback_workflow_run_result_json(&err.to_string()))
        });
    let _ = app
        .models_mut()
        .update(&st.workflow_run_last_error, |v| *v = None);
    push_log(
        app,
        &st.log_lines,
        &format!("workflow run started: {label} ({id})"),
    );

    Ok(())
}

pub(crate) fn new_workflow_run_channel() -> (
    mpsc::Sender<WorkflowRunJobResult>,
    mpsc::Receiver<WorkflowRunJobResult>,
) {
    mpsc::channel()
}

pub(crate) fn workflow_run_result_summary_lines(result_json: &str) -> Vec<String> {
    if result_json.trim().is_empty() {
        return vec!["workflow run result: <none>".to_string()];
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(result_json) else {
        return vec!["workflow run result: <invalid json>".to_string()];
    };

    let field = |key: &str| {
        value
            .get(key)
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("-")
            .to_string()
    };
    let mut lines = vec![
        format!("status: {}", field("status")),
        format!("id: {}", field("id")),
        format!("label: {}", field("label")),
    ];
    if let Some(duration_ms) = value
        .get("finished_unix_ms")
        .and_then(|value| value.as_u64())
        .zip(value.get("started_unix_ms").and_then(|value| value.as_u64()))
        .map(|(finished, started)| finished.saturating_sub(started))
    {
        lines.push(format!("duration_ms: {duration_ms}"));
    }
    if let Some(args) = value.get("diag_args").and_then(|value| value.as_array()) {
        lines.push(format!("diag_args_count: {}", args.len()));
    }
    if let Some(error) = value
        .get("error")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
    {
        lines.push(format!("error: {error}"));
    }
    lines.extend(workflow_run_output_artifact_lines_from_result_json(&value));
    let command_line = field("command_line");
    if command_line != "-" {
        lines.push(format!("command: {command_line}"));
    }
    lines
}

pub(crate) fn workflow_run_regression_summary_artifact_path_from_result_json(
    result_json: &str,
) -> Option<String> {
    workflow_run_output_artifact_path_from_result_json(result_json, "regression.summary.json")
}

pub(crate) fn workflow_run_regression_index_artifact_path_from_result_json(
    result_json: &str,
) -> Option<String> {
    workflow_run_output_artifact_path_from_result_json(result_json, "regression.index.json")
}

pub(crate) fn workflow_run_output_artifact_path_from_result_json(
    result_json: &str,
    kind: &str,
) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(result_json).ok()?;
    workflow_run_output_artifact_path_from_result_value(&value, kind)
}

pub(crate) fn workflow_run_result_history_summary_lines(
    entries: &[WorkflowRunResultHistoryEntry],
) -> Vec<String> {
    if entries.is_empty() {
        return vec!["workflow run history: <none>".to_string()];
    }
    let mut lines = vec![format!("workflow run history: {} result(s)", entries.len())];
    for entry in entries.iter().take(8) {
        lines.push(format!("{} | {} | {}", entry.status, entry.id, entry.result_path));
        lines.push(format!("command: {}", entry.command_line));
        if let Some(error) = entry.error.as_deref().filter(|value| !value.trim().is_empty()) {
            lines.push(format!("error: {error}"));
        }
    }
    lines
}

pub(crate) fn workflow_run_result_history_selected_or_latest_entry(
    entries: &[WorkflowRunResultHistoryEntry],
    selected_result_path: Option<&str>,
) -> Option<WorkflowRunResultHistoryEntry> {
    if let Some(selected_result_path) =
        selected_result_path.map(str::trim).filter(|value| !value.is_empty())
        && let Some(entry) = entries
            .iter()
            .find(|entry| entry.result_path == selected_result_path)
    {
        return Some(entry.clone());
    }
    entries.first().cloned()
}

pub(crate) fn workflow_run_result_history_entry_detail_lines(
    entry: Option<&WorkflowRunResultHistoryEntry>,
) -> Vec<String> {
    let Some(entry) = entry else {
        return vec!["selected workflow run result: <none>".to_string()];
    };
    let mut lines = vec![
        format!("status: {}", entry.status),
        format!("id: {}", entry.id),
        format!("label: {}", entry.label),
        format!("result_path: {}", entry.result_path),
        format!("command: {}", entry.command_line),
    ];
    if let Some(error) = entry.error.as_deref().filter(|value| !value.trim().is_empty()) {
        lines.push(format!("error: {error}"));
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&entry.result_json) {
        lines.extend(workflow_run_output_artifact_lines_from_result_json(&value));
    }
    lines
}

pub(crate) fn load_recent_workflow_run_result_history(
    repo_root: &Path,
    limit: usize,
) -> Vec<WorkflowRunResultHistoryEntry> {
    let result_dir = repo_root.join(".fret").join("diag").join("workflow-runs");
    load_recent_workflow_run_result_history_from_dir(&result_dir, limit)
}

pub(crate) fn redact_workflow_diag_args(args: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len());
    let mut redact_next = false;
    for arg in args {
        if redact_next {
            out.push("<redacted>".to_string());
            redact_next = false;
            continue;
        }
        if arg == "--devtools-token" {
            redact_next = true;
        }
        out.push(arg.clone());
    }
    out
}

fn build_workflow_run_result_record(
    command: &DevtoolsWorkflowRunCommandV1,
    diag_args: Vec<String>,
    started_unix_ms: u64,
    finished_unix_ms: u64,
    result: &Result<(), String>,
) -> WorkflowRunResultRecordV1 {
    let output_artifacts = workflow_run_output_artifacts_for_diag_args(&diag_args);
    WorkflowRunResultRecordV1 {
        schema_version: 1,
        kind: "fret_devtools_workflow_run_result",
        id: command.id.clone(),
        label: command.label.clone(),
        command_line: command.command_line.clone(),
        diag_args: redact_workflow_diag_args(&diag_args),
        missing_inputs: command.missing_inputs.clone(),
        status: if result.is_ok() { "passed" } else { "failed" },
        error: result.as_ref().err().cloned(),
        output_artifacts,
        started_unix_ms,
        finished_unix_ms,
    }
}

impl WorkflowRunResultHistoryEntry {
    fn from_job_result(result: &WorkflowRunJobResult) -> Self {
        Self {
            id: result.id.clone(),
            label: result.label.clone(),
            command_line: result.command_line.clone(),
            result_path: result.result_path.to_string_lossy().to_string(),
            result_json: result.result_json.clone(),
            status: if result.result.is_ok() {
                "passed".to_string()
            } else {
                "failed".to_string()
            },
            error: result.result.as_ref().err().cloned(),
        }
    }

    fn from_result_record(result_path: &Path, result_json: String) -> Option<Self> {
        let value = serde_json::from_str::<serde_json::Value>(&result_json).ok()?;
        if value.get("kind").and_then(|value| value.as_str())
            != Some("fret_devtools_workflow_run_result")
        {
            return None;
        }
        let field = |key: &str| -> String {
            value
                .get(key)
                .and_then(|value| value.as_str())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("-")
                .to_string()
        };
        let status = field("status");
        let error = value
            .get("error")
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned);
        Some(Self {
            id: field("id"),
            label: field("label"),
            command_line: field("command_line"),
            result_path: result_path.to_string_lossy().to_string(),
            result_json,
            status,
            error,
        })
    }
}

fn workflow_run_result_record_json(record: &WorkflowRunResultRecordV1) -> Result<String, String> {
    serde_json::to_string_pretty(record)
        .map_err(|err| format!("failed to serialize workflow run result: {err}"))
}

fn load_recent_workflow_run_result_history_from_dir(
    result_dir: &Path,
    limit: usize,
) -> Vec<WorkflowRunResultHistoryEntry> {
    if limit == 0 {
        return Vec::new();
    }
    let Ok(read_dir) = std::fs::read_dir(result_dir) else {
        return Vec::new();
    };
    let mut candidates = read_dir
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                return None;
            }
            let modified_unix_ms = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(system_time_unix_ms);
            let result_json = std::fs::read_to_string(&path).ok()?;
            let record_unix_ms = result_record_sort_unix_ms(&result_json);
            let history_entry =
                WorkflowRunResultHistoryEntry::from_result_record(&path, result_json)?;
            Some((history_entry, record_unix_ms, modified_unix_ms, path))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(
        |(left_entry, left_record, left_modified, left_path),
         (right_entry, right_record, right_modified, right_path)| {
            (right_record, right_modified, right_path, &right_entry.result_path).cmp(&(
                left_record,
                left_modified,
                left_path,
                &left_entry.result_path,
            ))
        },
    );
    candidates
        .into_iter()
        .map(|(entry, _record_unix_ms, _modified_unix_ms, _path)| entry)
        .take(limit)
        .collect()
}

fn result_record_sort_unix_ms(result_json: &str) -> Option<u64> {
    let value = serde_json::from_str::<serde_json::Value>(result_json).ok()?;
    value
        .get("finished_unix_ms")
        .and_then(|value| value.as_u64())
        .or_else(|| {
            value
                .get("started_unix_ms")
                .and_then(|value| value.as_u64())
        })
}

fn system_time_unix_ms(value: std::time::SystemTime) -> Option<u128> {
    value
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis())
}

fn fallback_workflow_run_result_json(error: &str) -> String {
    serde_json::json!({
        "schema_version": 1,
        "kind": "fret_devtools_workflow_run_result",
        "status": "failed",
        "error": error,
    })
    .to_string()
}

fn workflow_run_output_artifacts_for_diag_args(
    diag_args: &[String],
) -> Vec<WorkflowRunOutputArtifactV1> {
    match diag_args.first().map(String::as_str) {
        Some("suite") => workflow_run_suite_output_artifacts(diag_arg_value(diag_args, "--dir")),
        Some("summarize") => {
            workflow_run_summarize_output_artifacts(diag_arg_value(diag_args, "--dir"))
        }
        _ => Vec::new(),
    }
}

fn workflow_run_suite_output_artifacts(out_dir: Option<&str>) -> Vec<WorkflowRunOutputArtifactV1> {
    let Some(out_dir) = out_dir.map(str::trim).filter(|value| !value.is_empty()) else {
        return Vec::new();
    };
    let out_dir = PathBuf::from(out_dir);
    vec![
        WorkflowRunOutputArtifactV1 {
            kind: "suite.summary.json",
            path: normalized_artifact_path(out_dir.join("suite.summary.json")),
        },
        WorkflowRunOutputArtifactV1 {
            kind: "regression.summary.json",
            path: normalized_artifact_path(out_dir.join("regression.summary.json")),
        },
    ]
}

fn workflow_run_summarize_output_artifacts(
    out_dir: Option<&str>,
) -> Vec<WorkflowRunOutputArtifactV1> {
    let Some(out_dir) = out_dir.map(str::trim).filter(|value| !value.is_empty()) else {
        return Vec::new();
    };
    let out_dir = PathBuf::from(out_dir);
    vec![
        WorkflowRunOutputArtifactV1 {
            kind: "regression.summary.json",
            path: normalized_artifact_path(out_dir.join("regression.summary.json")),
        },
        WorkflowRunOutputArtifactV1 {
            kind: "regression.index.json",
            path: normalized_artifact_path(out_dir.join("regression.index.json")),
        },
    ]
}

fn diag_arg_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    for (index, arg) in args.iter().enumerate() {
        if arg == flag {
            return args
                .get(index + 1)
                .map(String::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty());
        }
        if let Some(value) = arg.strip_prefix(flag).and_then(|rest| rest.strip_prefix('=')) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn normalized_artifact_path(path: PathBuf) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn workflow_run_output_artifact_lines_from_result_json(
    value: &serde_json::Value,
) -> Vec<String> {
    value
        .get("output_artifacts")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|artifact| {
            let kind = artifact
                .get("kind")
                .and_then(|value| value.as_str())
                .filter(|value| !value.trim().is_empty())?;
            let path = artifact
                .get("path")
                .and_then(|value| value.as_str())
                .filter(|value| !value.trim().is_empty())?;
            Some(format!("artifact {kind}: {path}"))
        })
        .take(8)
        .collect()
}

fn workflow_run_output_artifact_path_from_result_value(
    value: &serde_json::Value,
    expected_kind: &str,
) -> Option<String> {
    let expected_kind = expected_kind.trim();
    if expected_kind.is_empty() {
        return None;
    }
    value
        .get("output_artifacts")
        .and_then(|value| value.as_array())?
        .iter()
        .find_map(|artifact| {
            let kind = artifact
                .get("kind")
                .and_then(|value| value.as_str())
                .map(str::trim)?;
            if kind != expected_kind {
                return None;
            }
            artifact
                .get("path")
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
}

fn write_workflow_run_result_record(out_path: &PathBuf, result_json: &str) -> Result<(), String> {
    std::fs::write(out_path, result_json.as_bytes()).map_err(|err| {
        format!(
            "failed to write workflow run result {}: {err}",
            out_path.to_string_lossy()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ws_suite_command() -> DevtoolsWorkflowRunCommandV1 {
        DevtoolsWorkflowRunCommandV1 {
            id: "perf-docking-suite-ws".to_string(),
            label: "Perf docking suite over selected session".to_string(),
            command_line: "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --dir target/fret-diag/devtools-workflows/perf-docking --devtools-ws-url ws://127.0.0.1:7331/ --devtools-token <redacted> --devtools-session-id session-1 --json".to_string(),
            diag_args: vec![
                "suite".to_string(),
                "perf-docking-arbitration-steady".to_string(),
                "--dir".to_string(),
                "target/fret-diag/devtools-workflows/perf-docking".to_string(),
                "--devtools-ws-url".to_string(),
                "ws://127.0.0.1:7331/".to_string(),
                "--devtools-token".to_string(),
                "secret-token".to_string(),
                "--devtools-session-id".to_string(),
                "session-1".to_string(),
                "--json".to_string(),
            ],
            missing_inputs: Vec::new(),
        }
    }

    fn summarize_command() -> DevtoolsWorkflowRunCommandV1 {
        DevtoolsWorkflowRunCommandV1 {
            id: "summarize-workflow-regression-index".to_string(),
            label: "Generate workflow regression index".to_string(),
            command_line: "cargo run -p fretboard-dev -- diag summarize target/fret-diag/devtools-workflows/perf-docking/regression.summary.json --dir target/fret-diag/devtools-workflows/perf-docking --json".to_string(),
            diag_args: vec![
                "summarize".to_string(),
                "target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
                    .to_string(),
                "--dir".to_string(),
                "target/fret-diag/devtools-workflows/perf-docking".to_string(),
                "--json".to_string(),
            ],
            missing_inputs: Vec::new(),
        }
    }

    fn workflow_run_test_dir(label: &str) -> PathBuf {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "fret-devtools-workflow-run-{label}-{}-{now}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create test dir");
        dir
    }

    #[test]
    fn workflow_run_result_record_has_stable_shape_and_redacts_token() {
        let command = ws_suite_command();
        let record = build_workflow_run_result_record(
            &command,
            command.diag_args.clone(),
            10,
            35,
            &Err("boom".to_string()),
        );
        let value = serde_json::to_value(record).expect("record json");

        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["kind"], "fret_devtools_workflow_run_result");
        assert_eq!(value["id"], "perf-docking-suite-ws");
        assert_eq!(value["label"], "Perf docking suite over selected session");
        assert_eq!(value["status"], "failed");
        assert_eq!(value["error"], "boom");
        assert_eq!(value["started_unix_ms"], 10);
        assert_eq!(value["finished_unix_ms"], 35);
        assert_eq!(value["diag_args"][6], "--devtools-token");
        assert_eq!(value["diag_args"][7], "<redacted>");
        assert_ne!(value["diag_args"][7], "secret-token");
        assert!(
            value["command_line"]
                .as_str()
                .expect("command line")
                .contains("--devtools-token <redacted>")
        );
        assert_eq!(value["output_artifacts"][0]["kind"], "suite.summary.json");
        assert_eq!(
            value["output_artifacts"][0]["path"],
            "target/fret-diag/devtools-workflows/perf-docking/suite.summary.json"
        );
        assert_eq!(
            value["output_artifacts"][1]["kind"],
            "regression.summary.json"
        );
        assert_eq!(
            value["output_artifacts"][1]["path"],
            "target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
        );
    }

    #[test]
    fn workflow_run_result_summary_lines_project_status_and_duration() {
        let json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_workflow_run_result",
            "id": "campaign-validate-devtools-first-open",
            "label": "Validate devtools first-open campaign",
            "command_line": "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/devtools-first-open-smoke.json --json",
            "diag_args": ["campaign", "validate", "tools/diag-campaigns/devtools-first-open-smoke.json", "--json"],
            "status": "failed",
            "error": "boom",
            "started_unix_ms": 10,
            "finished_unix_ms": 45
        })
        .to_string();

        let text = workflow_run_result_summary_lines(&json).join("\n");

        assert!(text.contains("status: failed"));
        assert!(text.contains("id: campaign-validate-devtools-first-open"));
        assert!(text.contains("label: Validate devtools first-open campaign"));
        assert!(text.contains("duration_ms: 35"));
        assert!(text.contains("diag_args_count: 4"));
        assert!(text.contains("error: boom"));
        assert!(text.contains("command: cargo run -p fretboard-dev -- diag campaign validate"));
    }

    #[test]
    fn workflow_run_result_summary_lines_project_output_artifacts() {
        let command = ws_suite_command();
        let record = build_workflow_run_result_record(
            &command,
            command.diag_args.clone(),
            10,
            45,
            &Ok(()),
        );
        let json = workflow_run_result_record_json(&record).expect("record json text");
        let text = workflow_run_result_summary_lines(&json).join("\n");

        assert!(text.contains("status: passed"));
        assert!(text.contains(
            "artifact suite.summary.json: target/fret-diag/devtools-workflows/perf-docking/suite.summary.json"
        ));
        assert!(text.contains(
            "artifact regression.summary.json: target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
        ));
    }

    #[test]
    fn workflow_run_result_summary_lines_project_summarize_output_artifacts() {
        let command = summarize_command();
        let record = build_workflow_run_result_record(
            &command,
            command.diag_args.clone(),
            10,
            45,
            &Ok(()),
        );
        let json = workflow_run_result_record_json(&record).expect("record json text");
        let text = workflow_run_result_summary_lines(&json).join("\n");

        assert!(text.contains("status: passed"));
        assert!(text.contains(
            "artifact regression.summary.json: target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
        ));
        assert!(text.contains(
            "artifact regression.index.json: target/fret-diag/devtools-workflows/perf-docking/regression.index.json"
        ));
        assert_eq!(
            workflow_run_regression_summary_artifact_path_from_result_json(&json),
            Some(
                "target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
                    .to_string()
            )
        );
        assert_eq!(
            workflow_run_regression_index_artifact_path_from_result_json(&json),
            Some(
                "target/fret-diag/devtools-workflows/perf-docking/regression.index.json"
                    .to_string()
            )
        );
    }

    #[test]
    fn workflow_run_regression_summary_artifact_path_extracts_output_artifact() {
        let json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_workflow_run_result",
            "status": "passed",
            "output_artifacts": [
                {
                    "kind": "suite.summary.json",
                    "path": "target/fret-diag/devtools-workflows/perf-docking/suite.summary.json"
                },
                {
                    "kind": "regression.summary.json",
                    "path": "target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
                }
            ]
        })
        .to_string();

        assert_eq!(
            workflow_run_regression_summary_artifact_path_from_result_json(&json),
            Some(
                "target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
                    .to_string()
            )
        );
        assert_eq!(
            workflow_run_output_artifact_path_from_result_json(&json, "suite.summary.json"),
            Some("target/fret-diag/devtools-workflows/perf-docking/suite.summary.json".to_string())
        );
        assert_eq!(
            workflow_run_output_artifact_path_from_result_json(&json, "trace.chrome.json"),
            None
        );
        assert_eq!(
            workflow_run_regression_index_artifact_path_from_result_json(&json),
            None
        );
        assert_eq!(
            workflow_run_regression_summary_artifact_path_from_result_json("not json"),
            None
        );
    }

    #[test]
    fn load_recent_workflow_run_result_history_reads_latest_valid_records() {
        let dir = workflow_run_test_dir("history");
        let older = dir.join("10-suite.json");
        let newer = dir.join("20-summarize.json");
        let ignored_kind = dir.join("30-other.json");
        let bad_json = dir.join("40-bad.json");

        let older_json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_workflow_run_result",
            "id": "perf-docking-suite-ws",
            "label": "Run perf docking suite over selected session",
            "command_line": "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --json",
            "status": "passed",
            "started_unix_ms": 10,
            "finished_unix_ms": 20,
            "output_artifacts": [
                {
                    "kind": "regression.summary.json",
                    "path": "target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
                }
            ]
        })
        .to_string();
        let newer_json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_workflow_run_result",
            "id": "summarize-workflow-regression-index",
            "label": "Generate workflow regression index",
            "command_line": "cargo run -p fretboard-dev -- diag summarize regression.summary.json --json",
            "status": "failed",
            "error": "boom",
            "started_unix_ms": 30,
            "finished_unix_ms": 35,
            "output_artifacts": [
                {
                    "kind": "regression.index.json",
                    "path": "target/fret-diag/devtools-workflows/perf-docking/regression.index.json"
                }
            ]
        })
        .to_string();

        std::fs::write(&older, older_json).expect("write older");
        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(&newer, newer_json).expect("write newer");
        std::fs::write(
            &ignored_kind,
            serde_json::json!({"kind": "not_workflow", "status": "passed"}).to_string(),
        )
        .expect("write ignored");
        std::fs::write(&bad_json, "{").expect("write bad");

        let entries = load_recent_workflow_run_result_history_from_dir(&dir, 8);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "summarize-workflow-regression-index");
        assert_eq!(entries[0].status, "failed");
        assert_eq!(entries[0].error.as_deref(), Some("boom"));
        assert_eq!(entries[0].result_path, newer.to_string_lossy());
        assert_eq!(entries[1].id, "perf-docking-suite-ws");
        assert_eq!(
            workflow_run_regression_index_artifact_path_from_result_json(&entries[0].result_json),
            Some("target/fret-diag/devtools-workflows/perf-docking/regression.index.json".to_string())
        );

        let limited = load_recent_workflow_run_result_history_from_dir(&dir, 1);
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].id, "summarize-workflow-regression-index");
        assert!(load_recent_workflow_run_result_history_from_dir(&dir, 0).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_recent_workflow_run_result_history_prefers_record_time_over_file_mtime() {
        let dir = workflow_run_test_dir("history-record-time");
        let older_mtime = dir.join("10-record-newer.json");
        let newer_mtime = dir.join("20-record-older.json");

        std::fs::write(
            &older_mtime,
            serde_json::json!({
                "schema_version": 1,
                "kind": "fret_devtools_workflow_run_result",
                "id": "record-newer",
                "label": "record newer",
                "command_line": "workflow record newer",
                "status": "failed",
                "started_unix_ms": 100,
                "finished_unix_ms": 900
            })
            .to_string(),
        )
        .expect("write record-newer");
        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(
            &newer_mtime,
            serde_json::json!({
                "schema_version": 1,
                "kind": "fret_devtools_workflow_run_result",
                "id": "record-older",
                "label": "record older",
                "command_line": "workflow record older",
                "status": "failed",
                "started_unix_ms": 500,
                "finished_unix_ms": 600
            })
            .to_string(),
        )
        .expect("write record-older");

        let entries = load_recent_workflow_run_result_history_from_dir(&dir, 8);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "record-newer");
        assert_eq!(entries[0].result_path, older_mtime.to_string_lossy());
        assert_eq!(entries[1].id, "record-older");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn workflow_run_result_history_selects_explicit_path_or_latest() {
        let entries = vec![
            WorkflowRunResultHistoryEntry {
                id: "campaign-validate-devtools-first-open".to_string(),
                label: "Validate devtools first-open campaign".to_string(),
                command_line: "cargo run -p fretboard-dev -- diag campaign validate a.json --json"
                    .to_string(),
                result_path: ".fret/diag/workflow-runs/20-campaign.json".to_string(),
                result_json: "{\"status\":\"passed\"}".to_string(),
                status: "passed".to_string(),
                error: None,
            },
            WorkflowRunResultHistoryEntry {
                id: "perf-docking-suite-ws".to_string(),
                label: "Perf docking suite over selected session".to_string(),
                command_line: "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --json"
                    .to_string(),
                result_path: ".fret/diag/workflow-runs/10-suite.json".to_string(),
                result_json: "{\"status\":\"failed\"}".to_string(),
                status: "failed".to_string(),
                error: Some("boom".to_string()),
            },
        ];

        assert_eq!(
            workflow_run_result_history_selected_or_latest_entry(&entries, None)
                .map(|entry| entry.result_path),
            Some(".fret/diag/workflow-runs/20-campaign.json".to_string())
        );
        assert_eq!(
            workflow_run_result_history_selected_or_latest_entry(
                &entries,
                Some(".fret/diag/workflow-runs/10-suite.json")
            )
            .map(|entry| (entry.result_path, entry.result_json)),
            Some((
                ".fret/diag/workflow-runs/10-suite.json".to_string(),
                "{\"status\":\"failed\"}".to_string(),
            ))
        );
        assert_eq!(
            workflow_run_result_history_selected_or_latest_entry(
                &entries,
                Some(".fret/diag/workflow-runs/missing.json")
            )
            .map(|entry| entry.result_path),
            Some(".fret/diag/workflow-runs/20-campaign.json".to_string())
        );

        let summary = workflow_run_result_history_summary_lines(&entries).join("\n");
        assert!(summary.contains("workflow run history: 2 result(s)"));
        assert!(summary.contains("passed | campaign-validate-devtools-first-open"));

        let details = workflow_run_result_history_entry_detail_lines(entries.get(1)).join("\n");
        assert!(details.contains("status: failed"));
        assert!(details.contains("result_path: .fret/diag/workflow-runs/10-suite.json"));
        assert!(details.contains("error: boom"));
    }

    #[test]
    fn workflow_run_result_history_entry_detail_lines_surface_output_artifacts() {
        let entry = WorkflowRunResultHistoryEntry {
            id: "perf-docking-suite-ws".to_string(),
            label: "Perf docking suite over selected session".to_string(),
            command_line: "cargo run -p fretboard-dev -- diag suite perf-docking-arbitration-steady --json"
                .to_string(),
            result_path: ".fret/diag/workflow-runs/10-suite.json".to_string(),
            result_json: serde_json::json!({
                "schema_version": 1,
                "kind": "fret_devtools_workflow_run_result",
                "status": "passed",
                "id": "perf-docking-suite-ws",
                "label": "Perf docking suite over selected session",
                "output_artifacts": [
                    {
                        "kind": "suite.summary.json",
                        "path": "target/fret-diag/devtools-workflows/perf-docking/suite.summary.json"
                    },
                    {
                        "kind": "regression.summary.json",
                        "path": "target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
                    }
                ]
            })
            .to_string(),
            status: "passed".to_string(),
            error: None,
        };

        let text = workflow_run_result_history_entry_detail_lines(Some(&entry)).join("\n");
        assert!(text.contains(
            "artifact suite.summary.json: target/fret-diag/devtools-workflows/perf-docking/suite.summary.json"
        ));
        assert!(text.contains(
            "artifact regression.summary.json: target/fret-diag/devtools-workflows/perf-docking/regression.summary.json"
        ));
    }

    #[test]
    fn workflow_run_command_reports_runnable_from_missing_inputs_and_args() {
        let mut command = ws_suite_command();
        assert!(command.is_runnable());

        command.missing_inputs.push("selected-session".to_string());
        assert!(!command.is_runnable());

        command.missing_inputs.clear();
        command.diag_args.clear();
        assert!(!command.is_runnable());
    }
}
