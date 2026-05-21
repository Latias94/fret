use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;

use fret_app::App;
use fret_diag::regression_summary::RegressionBundleFollowupCommandV1;
use serde::{Deserialize, Serialize};

use crate::{State, now_unix_ms, push_log, repo_root_from_script_paths};

#[derive(Debug, Clone)]
pub(crate) struct FollowupJobResult {
    pub id: String,
    pub label: String,
    pub command_line: String,
    pub result_path: PathBuf,
    pub result_json: String,
    pub bundle_dir: Option<String>,
    pub result: Result<(), String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FollowupResultHistoryEntry {
    pub id: String,
    pub label: String,
    pub command_line: String,
    pub result_path: String,
    pub result_json: String,
    pub bundle_dir: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct FollowupResultRecordV1 {
    schema_version: u32,
    kind: &'static str,
    id: String,
    label: String,
    command_line: String,
    diag_args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bundle_dir: Option<String>,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    output_artifacts: Vec<FollowupOutputArtifactV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_report: Option<FollowupTraceReportV1>,
    started_unix_ms: u64,
    finished_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
struct FollowupOutputArtifactV1 {
    kind: &'static str,
    path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FollowupTraceReportV1 {
    trace_chrome_json_path: String,
    trace_kind: Option<String>,
    trace_schema_version: Option<u64>,
    trace_source: Option<String>,
    real_spans_included: Option<bool>,
    real_span_event_count: Option<u64>,
    real_span_extension_keys: Vec<String>,
    trace_event_count: u64,
}

pub(crate) fn runnable_diag_args_for_followup_command(
    command: &RegressionBundleFollowupCommandV1,
) -> Result<Vec<String>, String> {
    if command.requires_baseline {
        return Err(format!(
            "follow-up command requires a baseline input: {}",
            command.label
        ));
    }
    if command.diag_args.is_empty() {
        return Err(format!(
            "follow-up command has no runnable diag args: {}",
            command.label
        ));
    }
    Ok(command.diag_args.clone())
}

pub(crate) fn followup_result_summary_lines(result_json: &str) -> Vec<String> {
    if result_json.trim().is_empty() {
        return vec!["follow-up result: <none>".to_string()];
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(result_json) else {
        return vec!["follow-up result: <invalid json>".to_string()];
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
    if let Some(bundle_dir) = followup_bundle_dir_from_result_json(&value) {
        lines.push(format!("bundle_dir: {bundle_dir}"));
    }
    if let Some(error) = value
        .get("error")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
    {
        lines.push(format!("error: {error}"));
    }
    lines.extend(followup_output_artifact_lines_from_result_json(&value));
    let command_line = field("command_line");
    if command_line != "-" {
        lines.push(format!("command: {command_line}"));
    }
    lines
}

pub(crate) fn followup_trace_artifact_path_from_result_json(result_json: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(result_json).ok()?;
    followup_trace_artifact_path_from_result_value(&value)
}

pub(crate) fn load_recent_followup_result_history(
    repo_root: &Path,
    limit: usize,
) -> Vec<FollowupResultHistoryEntry> {
    let result_dir = repo_root.join(".fret").join("diag").join("followups");
    load_recent_followup_result_history_from_dir(&result_dir, limit)
}

pub(crate) fn followup_result_history_summary_lines<'a>(
    entries: &[FollowupResultHistoryEntry],
    selected_bundle_dirs: impl IntoIterator<Item = &'a str>,
) -> Vec<String> {
    let selected_bundle_keys = followup_selected_bundle_keys(selected_bundle_dirs);
    if selected_bundle_keys.is_empty() {
        return vec!["follow-up history: <no selected bundle>".to_string()];
    }

    let matching = entries
        .iter()
        .filter(|entry| followup_history_entry_matches_selected_bundle(entry, &selected_bundle_keys))
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return vec!["follow-up history: <none for selected bundle>".to_string()];
    }

    let mut lines = vec![format!(
        "follow-up history: {} matching result(s)",
        matching.len()
    )];
    for entry in matching.into_iter().take(8) {
        lines.push(format!(
            "{} | {} | {}",
            entry.status,
            entry.id,
            entry.bundle_dir.as_deref().unwrap_or("-")
        ));
        lines.push(format!("result: {}", entry.result_path));
        lines.push(format!("command: {}", entry.command_line));
        if let Some(error) = entry.error.as_deref().filter(|value| !value.trim().is_empty()) {
            lines.push(format!("error: {error}"));
        }
    }
    lines
}

pub(crate) fn followup_result_history_entries_for_selected_bundle<'a>(
    entries: &[FollowupResultHistoryEntry],
    selected_bundle_dirs: impl IntoIterator<Item = &'a str>,
) -> Vec<FollowupResultHistoryEntry> {
    let selected_bundle_keys = followup_selected_bundle_keys(selected_bundle_dirs);
    if selected_bundle_keys.is_empty() {
        return Vec::new();
    }
    entries
        .iter()
        .filter(|entry| followup_history_entry_matches_selected_bundle(entry, &selected_bundle_keys))
        .cloned()
        .collect()
}

pub(crate) fn followup_result_history_selected_or_latest_entry<'a>(
    entries: &[FollowupResultHistoryEntry],
    selected_bundle_dirs: impl IntoIterator<Item = &'a str>,
    selected_result_path: Option<&str>,
) -> Option<FollowupResultHistoryEntry> {
    let selected_bundle_keys = followup_selected_bundle_keys(selected_bundle_dirs);
    if selected_bundle_keys.is_empty() {
        return None;
    }
    if let Some(selected_result_path) =
        selected_result_path.map(str::trim).filter(|value| !value.is_empty())
        && let Some(entry) = entries.iter().find(|entry| {
            entry.result_path == selected_result_path
                && followup_history_entry_matches_selected_bundle(entry, &selected_bundle_keys)
        })
    {
        return Some(entry.clone());
    }
    entries
        .iter()
        .find(|entry| followup_history_entry_matches_selected_bundle(entry, &selected_bundle_keys))
        .cloned()
}

pub(crate) fn followup_result_history_entry_detail_lines(
    entry: Option<&FollowupResultHistoryEntry>,
) -> Vec<String> {
    let Some(entry) = entry else {
        return vec!["selected follow-up result: <none>".to_string()];
    };
    let mut lines = vec![
        format!("status: {}", entry.status),
        format!("id: {}", entry.id),
        format!("label: {}", entry.label),
        format!("result_path: {}", entry.result_path),
    ];
    if let Some(bundle_dir) = entry
        .bundle_dir
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        lines.push(format!("bundle_dir: {bundle_dir}"));
    }
    lines.push(format!("command: {}", entry.command_line));
    if let Some(error) = entry.error.as_deref().filter(|value| !value.trim().is_empty()) {
        lines.push(format!("error: {error}"));
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&entry.result_json) {
        lines.extend(followup_output_artifact_lines_from_result_json(&value));
    }
    lines
}

pub(crate) fn poll_followup_jobs(app: &mut App, st: &mut State) {
    while let Ok(msg) = st.followup_rx.try_recv() {
        let result_path_text = msg.result_path.to_string_lossy().to_string();
        let _ = app
            .models_mut()
            .update(&st.followup_in_flight, |v| *v = false);
        let _ = app.models_mut().update(&st.followup_last_command_line, |v| {
            *v = Some(Arc::<str>::from(msg.command_line.clone()))
        });
        let _ = app.models_mut().update(&st.followup_last_result_path, |v| {
            *v = Some(Arc::<str>::from(result_path_text.clone()))
        });
        let _ = app
            .models_mut()
            .update(&st.followup_selected_result_path, |v| {
                *v = Some(Arc::<str>::from(result_path_text.clone()))
            });
        let _ = app
            .models_mut()
            .update(&st.followup_last_result_json, |v| *v = msg.result_json.clone());
        let _ = app.models_mut().update(&st.followup_result_history, |v| {
            v.insert(0, FollowupResultHistoryEntry::from_job_result(&msg));
            v.truncate(32);
        });

        match msg.result {
            Ok(()) => {
                let _ = app
                    .models_mut()
                    .update(&st.followup_last_error, |v| *v = None);
                push_log(
                    app,
                    &st.log_lines,
                    &format!(
                        "follow-up ok: {} ({}) result={}",
                        msg.label,
                        msg.id,
                        msg.result_path.to_string_lossy()
                    ),
                );
            }
            Err(err) => {
                let _ = app.models_mut().update(&st.followup_last_error, |v| {
                    *v = Some(Arc::<str>::from(err.clone()))
                });
                push_log(
                    app,
                    &st.log_lines,
                    &format!(
                        "follow-up failed: {} ({}) result={}: {err}",
                        msg.label,
                        msg.id,
                        msg.result_path.to_string_lossy()
                    ),
                );
            }
        }
    }
}

fn build_followup_result_record(
    command: &RegressionBundleFollowupCommandV1,
    diag_args: Vec<String>,
    started_unix_ms: u64,
    finished_unix_ms: u64,
    result: &Result<(), String>,
    repo_root: &Path,
) -> FollowupResultRecordV1 {
    let bundle_dir = followup_bundle_dir_from_diag_args(&diag_args);
    let output_artifacts = followup_output_artifacts_for_command(command, &diag_args);
    let trace_report = if result.is_ok() {
        followup_trace_report_for_artifacts(&output_artifacts, repo_root)
    } else {
        None
    };
    FollowupResultRecordV1 {
        schema_version: 1,
        kind: "fret_devtools_regression_followup_result",
        id: command.id.clone(),
        label: command.label.clone(),
        command_line: command.command_line.clone(),
        diag_args,
        bundle_dir,
        status: if result.is_ok() { "passed" } else { "failed" },
        error: result.as_ref().err().cloned(),
        output_artifacts,
        trace_report,
        started_unix_ms,
        finished_unix_ms,
    }
}

fn followup_result_record_json(record: &FollowupResultRecordV1) -> Result<String, String> {
    serde_json::to_string_pretty(record)
        .map_err(|err| format!("failed to serialize follow-up result: {err}"))
}

fn fallback_followup_result_json(error: &str) -> String {
    serde_json::json!({
        "schema_version": 1,
        "kind": "fret_devtools_regression_followup_result",
        "status": "failed",
        "error": error,
    })
    .to_string()
}

fn load_recent_followup_result_history_from_dir(
    result_dir: &Path,
    limit: usize,
) -> Vec<FollowupResultHistoryEntry> {
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
            let history_entry = FollowupResultHistoryEntry::from_result_record(&path, result_json)?;
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

fn followup_output_artifacts_for_command(
    command: &RegressionBundleFollowupCommandV1,
    diag_args: &[String],
) -> Vec<FollowupOutputArtifactV1> {
    if !is_trace_followup_command_id(&command.id) {
        return Vec::new();
    }
    let Some(bundle_arg) = diag_args.get(1).map(String::as_str) else {
        return Vec::new();
    };
    trace_followup_output_path(bundle_arg)
        .map(|path| {
            vec![FollowupOutputArtifactV1 {
                kind: "trace.chrome.json",
                path,
            }]
        })
        .unwrap_or_default()
}

fn is_trace_followup_command_id(id: &str) -> bool {
    id == "trace"
        || id
            .strip_prefix("trace-")
            .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
}

fn trace_followup_output_path(bundle_arg: &str) -> Option<String> {
    let trimmed = bundle_arg.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = PathBuf::from(trimmed);
    let dir = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.ends_with(".json"))
        .and_then(|_| path.parent().map(PathBuf::from))
        .unwrap_or(path);
    Some(
        dir.join("trace.chrome.json")
            .to_string_lossy()
            .replace('\\', "/"),
    )
}

fn followup_output_artifact_lines_from_result_json(value: &serde_json::Value) -> Vec<String> {
    let mut lines = value
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
        .collect::<Vec<_>>();
    lines.extend(followup_trace_report_lines_from_result_json(value));
    lines
}

fn followup_trace_artifact_path_from_result_value(value: &serde_json::Value) -> Option<String> {
    let trace_report_path = value
        .get("trace_report")
        .and_then(|report| report.get("trace_chrome_json_path"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let output_artifact_path = || {
        value
            .get("output_artifacts")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
            .find(|artifact| {
                artifact
                    .get("kind")
                    .and_then(|value| value.as_str())
                    .is_some_and(|kind| kind == "trace.chrome.json")
            })
            .and_then(|artifact| artifact.get("path"))
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
    };
    trace_report_path
        .or_else(|| {
            output_artifact_path()
        })
        .map(|value| value.replace('\\', "/"))
}

fn followup_trace_report_for_artifacts(
    artifacts: &[FollowupOutputArtifactV1],
    repo_root: &Path,
) -> Option<FollowupTraceReportV1> {
    let trace_artifact = artifacts
        .iter()
        .find(|artifact| artifact.kind == "trace.chrome.json")?;
    followup_trace_report_from_trace_path(&trace_artifact.path, repo_root)
}

fn followup_trace_report_from_trace_path(
    path: &str,
    repo_root: &Path,
) -> Option<FollowupTraceReportV1> {
    let trace_path = PathBuf::from(path);
    let read_path = if trace_path.is_absolute() {
        trace_path
    } else {
        repo_root.join(&trace_path)
    };
    let trace = std::fs::read_to_string(read_path).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&trace).ok()?;
    let trace_chrome_json_path = path.replace('\\', "/");
    let trace_kind = value
        .get("kind")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    let trace_schema_version = value.get("schema_version").and_then(|value| value.as_u64());
    let trace_source = value
        .get("trace_source")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    let real_spans_included = value
        .get("real_spans_included")
        .and_then(|value| value.as_bool());
    let real_span_event_count = value
        .get("real_span_event_count")
        .and_then(|value| value.as_u64());
    let real_span_extension_keys = value
        .get("real_span_extension_keys")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str())
        .map(ToOwned::to_owned)
        .collect();
    let trace_event_count = value
        .get("traceEvents")
        .and_then(|value| value.as_array())
        .map(|events| events.len() as u64)
        .unwrap_or(0);
    Some(FollowupTraceReportV1 {
        trace_chrome_json_path,
        trace_kind,
        trace_schema_version,
        trace_source,
        real_spans_included,
        real_span_event_count,
        real_span_extension_keys,
        trace_event_count,
    })
}

fn followup_trace_report_lines_from_result_json(value: &serde_json::Value) -> Vec<String> {
    let Some(report) = value.get("trace_report") else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    if let Some(path) = report
        .get("trace_chrome_json_path")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
    {
        lines.push(format!("trace_path: {path}"));
    }
    if let Some(source) = report
        .get("trace_source")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
    {
        lines.push(format!("trace_source: {source}"));
    }
    if let Some(included) = report
        .get("real_spans_included")
        .and_then(|value| value.as_bool())
    {
        lines.push(format!("real_spans_included: {included}"));
    }
    if let Some(count) = report
        .get("real_span_event_count")
        .and_then(|value| value.as_u64())
    {
        lines.push(format!("real_span_event_count: {count}"));
    }
    if let Some(count) = report
        .get("trace_event_count")
        .and_then(|value| value.as_u64())
    {
        lines.push(format!("trace_event_count: {count}"));
    }
    if let Some(keys) = report
        .get("real_span_extension_keys")
        .and_then(|value| value.as_array())
        .filter(|keys| !keys.is_empty())
    {
        let keys = keys
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>()
            .join(",");
        if !keys.is_empty() {
            lines.push(format!("real_span_extension_keys: {keys}"));
        }
    }
    lines
}

fn write_followup_result_record(out_path: &PathBuf, result_json: &str) -> Result<(), String> {
    std::fs::write(out_path, result_json.as_bytes()).map_err(|err| {
        format!(
            "failed to write follow-up result {}: {err}",
            out_path.to_string_lossy()
        )
    })
}

impl FollowupResultHistoryEntry {
    fn from_job_result(result: &FollowupJobResult) -> Self {
        Self {
            id: result.id.clone(),
            label: result.label.clone(),
            command_line: result.command_line.clone(),
            result_path: result.result_path.to_string_lossy().to_string(),
            result_json: result.result_json.clone(),
            bundle_dir: result.bundle_dir.clone(),
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
            != Some("fret_devtools_regression_followup_result")
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
            bundle_dir: followup_bundle_dir_from_result_json(&value),
            status,
            error,
        })
    }
}

fn followup_bundle_dir_from_diag_args(args: &[String]) -> Option<String> {
    let bundle_index = if args.first().is_some_and(|value| value == "compare") {
        2
    } else {
        1
    };
    args.get(bundle_index)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn followup_bundle_dir_from_result_json(value: &serde_json::Value) -> Option<String> {
    value
        .get("bundle_dir")
        .and_then(|value| value.as_str())
        .or_else(|| {
            value
                .get("diag_args")
                .and_then(|value| value.as_array())
                .and_then(|args| {
                    let command = args.first().and_then(|value| value.as_str());
                    let bundle_index = if command == Some("compare") { 2 } else { 1 };
                    args.get(bundle_index)
                })
                .and_then(|value| value.as_str())
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn normalize_followup_bundle_key(value: &str) -> String {
    value
        .trim()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string()
}

fn followup_selected_bundle_keys<'a>(
    selected_bundle_dirs: impl IntoIterator<Item = &'a str>,
) -> Vec<String> {
    selected_bundle_dirs
        .into_iter()
        .map(normalize_followup_bundle_key)
        .filter(|value| !value.is_empty())
        .collect()
}

fn followup_history_entry_matches_selected_bundle(
    entry: &FollowupResultHistoryEntry,
    selected_bundle_keys: &[String],
) -> bool {
    entry
        .bundle_dir
        .as_deref()
        .map(normalize_followup_bundle_key)
        .is_some_and(|bundle_key| selected_bundle_keys.iter().any(|v| v == &bundle_key))
}

pub(crate) fn start_regression_followup_command(
    app: &mut App,
    st: &mut State,
    command: RegressionBundleFollowupCommandV1,
) -> Result<(), String> {
    let in_flight = app
        .models()
        .read(&st.followup_in_flight, |v| *v)
        .unwrap_or(false);
    if in_flight {
        return Err("follow-up command already in progress".to_string());
    }

    let args = runnable_diag_args_for_followup_command(&command)?;
    let bundle_dir = followup_bundle_dir_from_diag_args(&args);
    let repo_root = repo_root_from_script_paths(&st.script_paths);
    let result_dir = repo_root.join(".fret").join("diag").join("followups");
    std::fs::create_dir_all(&result_dir).map_err(|err| {
        format!(
            "failed to create follow-up result dir {}: {err}",
            result_dir.to_string_lossy()
        )
    })?;
    let started_unix_ms = now_unix_ms();
    let id = command.id.clone();
    let label = command.label.clone();
    let command_line = command.command_line.clone();
    let result_path = result_dir.join(format!("{started_unix_ms}-{id}.json"));
    let tx = st.followup_tx.clone();
    std::thread::spawn({
        let id = id.clone();
        let label = label.clone();
        let command_line = command_line.clone();
        let command = command.clone();
        let bundle_dir = bundle_dir.clone();
        let result_path = result_path.clone();
        move || {
            let diag_args = args.clone();
            let result = fret_diag::diag_cmd(args);
            let finished_unix_ms = now_unix_ms();
            let record = build_followup_result_record(
                &command,
                diag_args,
                started_unix_ms,
                finished_unix_ms,
                &result,
                &repo_root,
            );
            let result_json = followup_result_record_json(&record)
                .unwrap_or_else(|err| fallback_followup_result_json(&err));
            let write_result = write_followup_result_record(&result_path, &result_json);
            let result = match (result, write_result) {
                (Ok(()), Ok(())) => Ok(()),
                (Err(err), Ok(())) => Err(err),
                (Ok(()), Err(write_err)) => Err(write_err),
                (Err(err), Err(write_err)) => {
                    Err(format!("{err}; follow-up result write failed: {write_err}"))
                }
            };
            let _ = tx.send(FollowupJobResult {
                id,
                label,
                command_line,
                result_path,
                result_json,
                bundle_dir,
                result,
            });
        }
    });

    let _ = app
        .models_mut()
        .update(&st.followup_in_flight, |v| *v = true);
    let _ = app.models_mut().update(&st.followup_last_command_line, |v| {
        *v = Some(Arc::<str>::from(command_line))
    });
    let _ = app.models_mut().update(&st.followup_last_result_path, |v| {
        *v = Some(Arc::<str>::from(
            result_path.to_string_lossy().to_string(),
        ))
    });
    let _ = app.models_mut().update(&st.followup_last_result_json, |v| {
        *v = serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_regression_followup_result",
            "status": "running",
            "id": id.as_str(),
        }))
        .unwrap_or_else(|err| fallback_followup_result_json(&err.to_string()))
    });
    let _ = app
        .models_mut()
        .update(&st.followup_last_error, |v| *v = None);
    push_log(
        app,
        &st.log_lines,
        &format!("follow-up started: {label} ({id})"),
    );

    Ok(())
}

pub(crate) fn new_followup_channel(
) -> (mpsc::Sender<FollowupJobResult>, mpsc::Receiver<FollowupJobResult>) {
    mpsc::channel()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_diag::regression_summary::regression_bundle_followup_commands;
    use std::fs;

    fn history_entry(
        id: &str,
        command_line: &str,
        result_path: &str,
        bundle_dir: &str,
        status: &str,
        error: Option<&str>,
    ) -> FollowupResultHistoryEntry {
        FollowupResultHistoryEntry {
            id: id.to_string(),
            label: id.to_string(),
            command_line: command_line.to_string(),
            result_path: result_path.to_string(),
            result_json: "{}".to_string(),
            bundle_dir: Some(bundle_dir.to_string()),
            status: status.to_string(),
            error: error.map(ToOwned::to_owned),
        }
    }

    fn followup_test_dir(label: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("fret-devtools-followup-{label}-{}", now_unix_ms()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create followup test dir");
        dir
    }

    #[test]
    fn regression_followup_command_rejects_baseline_required_commands() {
        let commands = regression_bundle_followup_commands(["target/fret-diag/run-a"]);
        let compare = commands
            .iter()
            .find(|command| command.id == "visual-compare")
            .expect("visual compare command");

        let err = runnable_diag_args_for_followup_command(compare).unwrap_err();
        assert!(err.contains("requires a baseline input"));
    }

    #[test]
    fn regression_followup_command_returns_direct_diag_args() {
        let commands = regression_bundle_followup_commands(["target/fret-diag/run-a"]);
        let stats = commands
            .iter()
            .find(|command| command.id == "stats")
            .expect("stats command");

        let args = runnable_diag_args_for_followup_command(stats).expect("direct args");
        assert_eq!(
            args,
            vec![
                "stats".to_string(),
                "target/fret-diag/run-a".to_string(),
                "--json".to_string()
            ]
        );

        let trace = commands
            .iter()
            .find(|command| command.id == "trace")
            .expect("trace command");
        let args = runnable_diag_args_for_followup_command(trace).expect("direct args");
        assert_eq!(
            args,
            vec![
                "trace".to_string(),
                "target/fret-diag/run-a".to_string(),
                "--json".to_string()
            ]
        );
    }

    #[test]
    fn regression_followup_compare_result_uses_candidate_bundle_dir() {
        let commands = regression_bundle_followup_commands(["target/fret-diag/run-candidate"]);
        let mut compare = commands
            .into_iter()
            .find(|command| command.id == "visual-compare")
            .expect("visual compare command");
        compare.requires_baseline = false;
        compare.diag_args = vec![
            "compare".to_string(),
            "target/fret-diag/run-baseline".to_string(),
            "target/fret-diag/run-candidate".to_string(),
            "--json".to_string(),
        ];

        let record = build_followup_result_record(
            &compare,
            compare.diag_args.clone(),
            10,
            30,
            &Ok(()),
            Path::new("F:/repo"),
        );
        assert_eq!(
            record.bundle_dir.as_deref(),
            Some("target/fret-diag/run-candidate")
        );

        let json = serde_json::to_value(record).expect("record json");
        assert_eq!(
            followup_bundle_dir_from_result_json(&json).as_deref(),
            Some("target/fret-diag/run-candidate")
        );
    }

    #[test]
    fn regression_followup_result_record_has_stable_shape() {
        let command = regression_bundle_followup_commands(["target/fret-diag/run-a"])
            .into_iter()
            .find(|command| command.id == "stats")
            .expect("stats command");
        let record = build_followup_result_record(
            &command,
            command.diag_args.clone(),
            10,
            20,
            &Err("boom".to_string()),
            Path::new("."),
        );
        let value = serde_json::to_value(record).expect("record json");

        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["kind"], "fret_devtools_regression_followup_result");
        assert_eq!(value["id"], "stats");
        assert_eq!(value["status"], "failed");
        assert_eq!(value["error"], "boom");
        assert_eq!(value["bundle_dir"], "target/fret-diag/run-a");
        assert_eq!(value["started_unix_ms"], 10);
        assert_eq!(value["finished_unix_ms"], 20);
        assert!(value.get("output_artifacts").is_none());

        let command = regression_bundle_followup_commands(["target/fret-diag/run-a"])
            .into_iter()
            .find(|command| command.id == "stats")
            .expect("stats command");
        let record = build_followup_result_record(
            &command,
            command.diag_args.clone(),
            10,
            20,
            &Ok(()),
            Path::new("."),
        );
        let json = followup_result_record_json(&record).expect("record json text");
        assert!(json.contains("\"status\": \"passed\""));
    }

    #[test]
    fn regression_followup_trace_result_record_projects_output_artifact() {
        let root = std::env::temp_dir().join(format!(
            "fret-devtools-followup-trace-{}",
            std::process::id()
        ));
        let run_dir = root.join("run-a");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&run_dir).expect("create run dir");
        fs::write(
            run_dir.join("trace.chrome.json"),
            serde_json::json!({
                "kind": "perf_trace_chrome",
                "schema_version": 1,
                "trace_source": "bundle_synthetic_phases_with_extension_spans",
                "real_spans_included": true,
                "real_span_event_count": 2,
                "real_span_extension_keys": ["fret.perf.spans.v1"],
                "traceEvents": [{ "name": "fret.ui.view" }, { "name": "fret.ui.paint" }]
            })
            .to_string(),
        )
        .expect("write trace");
        let bundle_dir = run_dir.to_string_lossy().replace('\\', "/");
        let command = regression_bundle_followup_commands([bundle_dir.as_str()])
            .into_iter()
            .find(|command| command.id == "trace")
            .expect("trace command");
        let record = build_followup_result_record(
            &command,
            command.diag_args.clone(),
            10,
            20,
            &Ok(()),
            Path::new("."),
        );
        let value = serde_json::to_value(record).expect("record json");

        assert_eq!(value["id"], "trace");
        assert_eq!(value["output_artifacts"][0]["kind"], "trace.chrome.json");
        assert_eq!(
            value["output_artifacts"][0]["path"],
            format!("{bundle_dir}/trace.chrome.json")
        );
        assert_eq!(
            value["trace_report"]["trace_chrome_json_path"],
            format!("{bundle_dir}/trace.chrome.json")
        );
        assert_eq!(
            value["trace_report"]["trace_source"],
            "bundle_synthetic_phases_with_extension_spans"
        );
        assert_eq!(value["trace_report"]["real_spans_included"], true);
        assert_eq!(value["trace_report"]["real_span_event_count"], 2);
        assert_eq!(value["trace_report"]["trace_event_count"], 2);
        assert_eq!(
            value["trace_report"]["real_span_extension_keys"][0],
            "fret.perf.spans.v1"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn regression_followup_result_summary_lines_project_status_and_duration() {
        let json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_regression_followup_result",
            "id": "stats",
            "label": "diag stats",
            "command_line": "cargo run -p fretboard-dev -- diag stats target/fret-diag/run-a --json",
            "diag_args": ["stats", "target/fret-diag/run-a", "--json"],
            "status": "failed",
            "error": "boom",
            "started_unix_ms": 10,
            "finished_unix_ms": 25
        })
        .to_string();

        let lines = followup_result_summary_lines(&json);
        let text = lines.join("\n");
        assert!(text.contains("status: failed"));
        assert!(text.contains("id: stats"));
        assert!(text.contains("label: diag stats"));
        assert!(text.contains("duration_ms: 15"));
        assert!(text.contains("diag_args_count: 3"));
        assert!(text.contains("bundle_dir: target/fret-diag/run-a"));
        assert!(text.contains("error: boom"));
    }

    #[test]
    fn regression_followup_result_summary_lines_project_output_artifacts() {
        let json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_regression_followup_result",
            "id": "trace",
            "label": "trace",
            "command_line": "cargo run -p fretboard-dev -- diag trace target/fret-diag/run-a --json",
            "diag_args": ["trace", "target/fret-diag/run-a", "--json"],
            "status": "passed",
            "output_artifacts": [
                {
                    "kind": "trace.chrome.json",
                    "path": "target/fret-diag/run-a/trace.chrome.json"
                }
            ],
            "trace_report": {
                "trace_chrome_json_path": "target/fret-diag/run-a/trace.chrome.json",
                "trace_kind": "perf_trace_chrome",
                "trace_schema_version": 1,
                "trace_source": "bundle_synthetic_phases_with_extension_spans",
                "real_spans_included": true,
                "real_span_event_count": 2,
                "real_span_extension_keys": ["fret.perf.spans.v1"],
                "trace_event_count": 42
            },
            "started_unix_ms": 10,
            "finished_unix_ms": 25
        })
        .to_string();

        let text = followup_result_summary_lines(&json).join("\n");

        assert!(
            text.contains("artifact trace.chrome.json: target/fret-diag/run-a/trace.chrome.json")
        );
        assert!(text.contains("trace_source: bundle_synthetic_phases_with_extension_spans"));
        assert!(text.contains("real_spans_included: true"));
        assert!(text.contains("real_span_event_count: 2"));
        assert!(text.contains("trace_event_count: 42"));
        assert!(text.contains("real_span_extension_keys: fret.perf.spans.v1"));
    }

    #[test]
    fn regression_followup_trace_artifact_path_prefers_trace_report() {
        let json = serde_json::json!({
            "output_artifacts": [
                {
                    "kind": "trace.chrome.json",
                    "path": "target\\fret-diag\\run-a\\trace.chrome.json"
                }
            ],
            "trace_report": {
                "trace_chrome_json_path": "target/fret-diag/run-a/report-trace.chrome.json"
            }
        })
        .to_string();

        assert_eq!(
            followup_trace_artifact_path_from_result_json(&json),
            Some("target/fret-diag/run-a/report-trace.chrome.json".to_string())
        );
    }

    #[test]
    fn regression_followup_trace_artifact_path_falls_back_to_output_artifacts() {
        let json = serde_json::json!({
            "output_artifacts": [
                {
                    "kind": "stats.json",
                    "path": "target/fret-diag/run-a/stats.json"
                },
                {
                    "kind": "trace.chrome.json",
                    "path": "target\\fret-diag\\run-a\\trace.chrome.json"
                }
            ]
        })
        .to_string();

        assert_eq!(
            followup_trace_artifact_path_from_result_json(&json),
            Some("target/fret-diag/run-a/trace.chrome.json".to_string())
        );
        assert_eq!(followup_trace_artifact_path_from_result_json("{}"), None);
        assert_eq!(followup_trace_artifact_path_from_result_json("not json"), None);
    }

    #[test]
    fn load_recent_followup_result_history_reads_latest_valid_records() {
        let dir = followup_test_dir("history");
        let older = dir.join("10-stats.json");
        let newer = dir.join("20-trace.json");
        let ignored_kind = dir.join("30-other.json");
        let bad_json = dir.join("40-bad.json");

        let older_json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_regression_followup_result",
            "id": "stats",
            "label": "diag stats",
            "command_line": "cargo run -p fretboard-dev -- diag stats target/fret-diag/run-a --json",
            "diag_args": ["stats", "target/fret-diag/run-a", "--json"],
            "status": "passed",
            "started_unix_ms": 10,
            "finished_unix_ms": 20
        })
        .to_string();
        let newer_json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_regression_followup_result",
            "id": "trace",
            "label": "trace",
            "command_line": "cargo run -p fretboard-dev -- diag trace target/fret-diag/run-b --json",
            "diag_args": ["trace", "target/fret-diag/run-b", "--json"],
            "bundle_dir": "target\\fret-diag\\run-b",
            "status": "failed",
            "error": "boom",
            "started_unix_ms": 30,
            "finished_unix_ms": 35,
            "output_artifacts": [
                {
                    "kind": "trace.chrome.json",
                    "path": "target/fret-diag/run-b/trace.chrome.json"
                }
            ]
        })
        .to_string();

        std::fs::write(&older, older_json).expect("write older");
        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(&newer, newer_json).expect("write newer");
        std::fs::write(
            &ignored_kind,
            serde_json::json!({"kind": "not_followup", "status": "passed"}).to_string(),
        )
        .expect("write ignored");
        std::fs::write(&bad_json, "{").expect("write bad");

        let entries = load_recent_followup_result_history_from_dir(&dir, 8);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "trace");
        assert_eq!(entries[0].status, "failed");
        assert_eq!(entries[0].error.as_deref(), Some("boom"));
        assert_eq!(entries[0].bundle_dir.as_deref(), Some("target\\fret-diag\\run-b"));
        assert_eq!(entries[0].result_path, newer.to_string_lossy());
        assert_eq!(entries[1].id, "stats");
        assert_eq!(entries[1].bundle_dir.as_deref(), Some("target/fret-diag/run-a"));
        assert_eq!(
            followup_trace_artifact_path_from_result_json(&entries[0].result_json),
            Some("target/fret-diag/run-b/trace.chrome.json".to_string())
        );

        let matching =
            followup_result_history_entries_for_selected_bundle(&entries, ["target/fret-diag/run-b"]);
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].id, "trace");

        let limited = load_recent_followup_result_history_from_dir(&dir, 1);
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].id, "trace");
        assert!(load_recent_followup_result_history_from_dir(&dir, 0).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_recent_followup_result_history_prefers_record_time_over_file_mtime() {
        let dir = followup_test_dir("history-record-time");
        let older_mtime = dir.join("10-record-newer.json");
        let newer_mtime = dir.join("20-record-older.json");

        std::fs::write(
            &older_mtime,
            serde_json::json!({
                "schema_version": 1,
                "kind": "fret_devtools_regression_followup_result",
                "id": "record-newer",
                "label": "record newer",
                "command_line": "follow-up record newer",
                "diag_args": ["trace", "target/fret-diag/run-a", "--json"],
                "bundle_dir": "target/fret-diag/run-a",
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
                "kind": "fret_devtools_regression_followup_result",
                "id": "record-older",
                "label": "record older",
                "command_line": "follow-up record older",
                "diag_args": ["trace", "target/fret-diag/run-b", "--json"],
                "bundle_dir": "target/fret-diag/run-b",
                "status": "failed",
                "started_unix_ms": 500,
                "finished_unix_ms": 600
            })
            .to_string(),
        )
        .expect("write record-older");

        let entries = load_recent_followup_result_history_from_dir(&dir, 8);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "record-newer");
        assert_eq!(entries[0].result_path, older_mtime.to_string_lossy());
        assert_eq!(entries[1].id, "record-older");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn regression_followup_result_history_summary_filters_to_selected_bundle() {
        let entries = vec![
            history_entry(
                "stats",
                "cargo run -p fretboard-dev -- diag stats target/fret-diag/run-a --json",
                ".fret/diag/followups/10-stats.json",
                "target\\fret-diag\\run-a",
                "passed",
                None,
            ),
            history_entry(
                "triage",
                "cargo run -p fretboard-dev -- diag triage target/fret-diag/run-b --json",
                ".fret/diag/followups/20-triage.json",
                "target/fret-diag/run-b",
                "failed",
                Some("boom"),
            ),
        ];

        let lines = followup_result_history_summary_lines(&entries, ["target/fret-diag/run-a"]);
        let text = lines.join("\n");

        assert!(text.contains("follow-up history: 1 matching result(s)"));
        assert!(text.contains("passed | stats | target\\fret-diag\\run-a"));
        assert!(text.contains(".fret/diag/followups/10-stats.json"));
        assert!(!text.contains("run-b"));
    }

    #[test]
    fn regression_followup_result_history_latest_path_prefers_selected_bundle() {
        let entries = vec![
            history_entry(
                "triage",
                "cargo run -p fretboard-dev -- diag triage target/fret-diag/run-b --json",
                ".fret/diag/followups/30-triage.json",
                "target/fret-diag/run-b",
                "passed",
                None,
            ),
            {
                let mut entry = history_entry(
                    "stats",
                    "cargo run -p fretboard-dev -- diag stats target/fret-diag/run-a --json",
                    ".fret/diag/followups/20-stats.json",
                    "target/fret-diag/run-a",
                    "passed",
                    None,
                );
                entry.result_json = "{\"status\":\"passed\"}".to_string();
                entry
            },
            history_entry(
                "stats",
                "cargo run -p fretboard-dev -- diag stats target/fret-diag/run-b --json",
                ".fret/diag/followups/10-stats.json",
                "target/fret-diag/run-b",
                "failed",
                Some("boom"),
            ),
        ];

        assert_eq!(
            followup_result_history_selected_or_latest_entry(
                &entries,
                ["target\\fret-diag\\run-a"],
                None,
            )
            .map(|entry| (entry.result_path, entry.result_json)),
            Some((
                ".fret/diag/followups/20-stats.json".to_string(),
                "{\"status\":\"passed\"}".to_string(),
            ))
        );
        assert_eq!(
            followup_result_history_selected_or_latest_entry(
                &entries,
                ["target/fret-diag/missing"],
                None,
            ),
            None
        );
    }

    #[test]
    fn regression_followup_result_history_selected_entry_overrides_latest_when_matching() {
        let entries = vec![
            history_entry(
                "triage",
                "cargo run -p fretboard-dev -- diag triage target/fret-diag/run-a --json",
                ".fret/diag/followups/30-triage.json",
                "target/fret-diag/run-a",
                "passed",
                None,
            ),
            history_entry(
                "stats",
                "cargo run -p fretboard-dev -- diag stats target/fret-diag/run-a --json",
                ".fret/diag/followups/20-stats.json",
                "target/fret-diag/run-a",
                "failed",
                Some("boom"),
            ),
            history_entry(
                "stats",
                "cargo run -p fretboard-dev -- diag stats target/fret-diag/run-b --json",
                ".fret/diag/followups/10-stats.json",
                "target/fret-diag/run-b",
                "passed",
                None,
            ),
        ];

        assert_eq!(
            followup_result_history_selected_or_latest_entry(
                &entries,
                ["target/fret-diag/run-a"],
                Some(".fret/diag/followups/20-stats.json"),
            )
            .map(|entry| entry.result_path),
            Some(".fret/diag/followups/20-stats.json".to_string())
        );
        assert_eq!(
            followup_result_history_selected_or_latest_entry(
                &entries,
                ["target/fret-diag/run-a"],
                Some(".fret/diag/followups/10-stats.json"),
            )
            .map(|entry| entry.result_path),
            Some(".fret/diag/followups/30-triage.json".to_string())
        );
    }

    #[test]
    fn regression_followup_result_history_entry_detail_lines_surface_repro_fields() {
        let mut entry = history_entry(
            "triage",
            "cargo run -p fretboard-dev -- diag triage target/fret-diag/run-a --json",
            ".fret/diag/followups/30-triage.json",
            "target/fret-diag/run-a",
            "failed",
            Some("boom"),
        );
        entry.result_json = serde_json::json!({
            "output_artifacts": [
                {
                    "kind": "trace.chrome.json",
                    "path": "target/fret-diag/run-a/trace.chrome.json"
                }
            ],
            "trace_report": {
                "trace_chrome_json_path": "target/fret-diag/run-a/trace.chrome.json",
                "trace_source": "bundle_synthetic_phases_with_extension_spans",
                "real_spans_included": true,
                "real_span_event_count": 2,
                "real_span_extension_keys": ["fret.perf.spans.v1"],
                "trace_event_count": 42
            }
        })
        .to_string();

        let text = followup_result_history_entry_detail_lines(Some(&entry)).join("\n");

        assert!(text.contains("status: failed"));
        assert!(text.contains("id: triage"));
        assert!(text.contains("result_path: .fret/diag/followups/30-triage.json"));
        assert!(text.contains("bundle_dir: target/fret-diag/run-a"));
        assert!(text.contains("command: cargo run -p fretboard-dev -- diag triage"));
        assert!(text.contains("error: boom"));
        assert!(
            text.contains("artifact trace.chrome.json: target/fret-diag/run-a/trace.chrome.json")
        );
        assert!(text.contains("trace_source: bundle_synthetic_phases_with_extension_spans"));
        assert!(text.contains("real_span_event_count: 2"));
        assert_eq!(
            followup_result_history_entry_detail_lines(None),
            vec!["selected follow-up result: <none>".to_string()]
        );
    }
}
