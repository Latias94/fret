use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;

use fret_app::App;
use fret_diag::DevtoolsGateScriptTargetCommandV1;
use serde::Serialize;

use crate::{State, now_unix_ms, push_log, repo_root_from_script_paths};

#[derive(Debug, Clone)]
pub(crate) struct GateRunJobResult {
    pub id: String,
    pub label: String,
    pub command_line: String,
    pub result_path: PathBuf,
    pub result_json: String,
    pub result: Result<(), String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GateRunResultHistoryEntry {
    pub id: String,
    pub label: String,
    pub command_line: String,
    pub result_path: String,
    pub result_json: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GateRunResultRecordV1 {
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
    started_unix_ms: u64,
    finished_unix_ms: u64,
}

pub(crate) fn poll_gate_run_jobs(app: &mut App, st: &mut State) {
    while let Ok(msg) = st.gate_run_rx.try_recv() {
        let result_path_text = msg.result_path.to_string_lossy().to_string();
        let _ = app
            .models_mut()
            .update(&st.gate_run_in_flight, |v| *v = false);
        let _ = app.models_mut().update(&st.gate_run_last_command_line, |v| {
            *v = Some(Arc::<str>::from(msg.command_line.clone()))
        });
        let _ = app.models_mut().update(&st.gate_run_last_result_path, |v| {
            *v = Some(Arc::<str>::from(result_path_text.clone()))
        });
        let _ = app
            .models_mut()
            .update(&st.gate_run_selected_result_path, |v| {
                *v = Some(Arc::<str>::from(result_path_text.clone()))
            });
        let _ = app
            .models_mut()
            .update(&st.gate_run_last_result_json, |v| *v = msg.result_json.clone());
        let _ = app.models_mut().update(&st.gate_run_result_history, |v| {
            v.insert(0, GateRunResultHistoryEntry::from_job_result(&msg));
            v.truncate(32);
        });

        match msg.result {
            Ok(()) => {
                let _ = app.models_mut().update(&st.gate_run_last_error, |v| *v = None);
                push_log(
                    app,
                    &st.log_lines,
                    &format!(
                        "gate run ok: {} ({}) result={}",
                        msg.label,
                        msg.id,
                        msg.result_path.to_string_lossy()
                    ),
                );
            }
            Err(err) => {
                let _ = app.models_mut().update(&st.gate_run_last_error, |v| {
                    *v = Some(Arc::<str>::from(err.clone()))
                });
                push_log(
                    app,
                    &st.log_lines,
                    &format!(
                        "gate run failed: {} ({}) result={}: {err}",
                        msg.label,
                        msg.id,
                        msg.result_path.to_string_lossy()
                    ),
                );
            }
        }
    }
}

pub(crate) fn start_gate_run(
    app: &mut App,
    st: &mut State,
    command: DevtoolsGateScriptTargetCommandV1,
) -> Result<(), String> {
    let in_flight = app
        .models()
        .read(&st.gate_run_in_flight, |v| *v)
        .unwrap_or(false);
    if in_flight {
        return Err("gate run already in progress".to_string());
    }
    if !command.is_runnable() {
        return Err(format!(
            "gate run requires inputs: {}",
            command.missing_inputs.join(", ")
        ));
    }

    let repo_root = repo_root_from_script_paths(&st.script_paths);
    let result_dir = repo_root.join(".fret").join("diag").join("gate-runs");
    std::fs::create_dir_all(&result_dir).map_err(|err| {
        format!(
            "failed to create gate run result dir {}: {err}",
            result_dir.to_string_lossy()
        )
    })?;
    let started_unix_ms = now_unix_ms();
    let id = command.id.clone();
    let label = command.label.clone();
    let command_line = command.command_line.clone();
    let result_path = result_dir.join(format!("{started_unix_ms}-{id}.json"));
    let tx = st.gate_run_tx.clone();

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
            let record = build_gate_run_result_record(
                &command,
                diag_args,
                started_unix_ms,
                finished_unix_ms,
                &result,
            );
            let result_json = gate_run_result_record_json(&record)
                .unwrap_or_else(|err| fallback_gate_run_result_json(&err));
            let write_result = write_gate_run_result_record(&result_path, &result_json);
            let result = match (result, write_result) {
                (Ok(()), Ok(())) => Ok(()),
                (Err(err), Ok(())) => Err(err),
                (Ok(()), Err(write_err)) => Err(write_err),
                (Err(err), Err(write_err)) => {
                    Err(format!("{err}; gate run result write failed: {write_err}"))
                }
            };
            let _ = tx.send(GateRunJobResult {
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
        .update(&st.gate_run_in_flight, |v| *v = true);
    let _ = app.models_mut().update(&st.gate_run_last_command_line, |v| {
        *v = Some(Arc::<str>::from(command_line))
    });
    let _ = app.models_mut().update(&st.gate_run_last_result_path, |v| {
        *v = Some(Arc::<str>::from(
            result_path.to_string_lossy().to_string(),
        ))
    });
    let _ = app.models_mut().update(&st.gate_run_last_result_json, |v| {
        *v = serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_gate_run_result",
            "status": "running",
            "id": id.as_str(),
        }))
        .unwrap_or_else(|err| fallback_gate_run_result_json(&err.to_string()))
    });
    let _ = app
        .models_mut()
        .update(&st.gate_run_last_error, |v| *v = None);
    push_log(app, &st.log_lines, &format!("gate run started: {label} ({id})"));

    Ok(())
}

pub(crate) fn new_gate_run_channel() -> (
    mpsc::Sender<GateRunJobResult>,
    mpsc::Receiver<GateRunJobResult>,
) {
    mpsc::channel()
}

pub(crate) fn load_recent_gate_run_result_history(
    repo_root: &Path,
    limit: usize,
) -> Vec<GateRunResultHistoryEntry> {
    let result_dir = repo_root.join(".fret").join("diag").join("gate-runs");
    load_recent_gate_run_result_history_from_dir(&result_dir, limit)
}

pub(crate) fn gate_run_result_summary_lines(result_json: &str) -> Vec<String> {
    if result_json.trim().is_empty() {
        return vec!["gate run result: <none>".to_string()];
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(result_json) else {
        return vec!["gate run result: <invalid json>".to_string()];
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
    let command_line = field("command_line");
    if command_line != "-" {
        lines.push(format!("command: {command_line}"));
    }
    lines
}

pub(crate) fn gate_run_result_history_summary_lines(
    entries: &[GateRunResultHistoryEntry],
) -> Vec<String> {
    if entries.is_empty() {
        return vec!["gate run history: <none>".to_string()];
    }
    let mut lines = vec![format!("gate run history: {} result(s)", entries.len())];
    for entry in entries.iter().take(8) {
        lines.push(format!("{} | {} | {}", entry.status, entry.id, entry.result_path));
        lines.push(format!("command: {}", entry.command_line));
        if let Some(error) = entry.error.as_deref().filter(|value| !value.trim().is_empty()) {
            lines.push(format!("error: {error}"));
        }
    }
    lines
}

pub(crate) fn gate_run_result_history_selected_or_latest_entry(
    entries: &[GateRunResultHistoryEntry],
    selected_result_path: Option<&str>,
) -> Option<GateRunResultHistoryEntry> {
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

pub(crate) fn gate_run_result_history_entry_detail_lines(
    entry: Option<&GateRunResultHistoryEntry>,
) -> Vec<String> {
    let Some(entry) = entry else {
        return vec!["selected gate run result: <none>".to_string()];
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
    lines
}

fn build_gate_run_result_record(
    command: &DevtoolsGateScriptTargetCommandV1,
    diag_args: Vec<String>,
    started_unix_ms: u64,
    finished_unix_ms: u64,
    result: &Result<(), String>,
) -> GateRunResultRecordV1 {
    GateRunResultRecordV1 {
        schema_version: 1,
        kind: "fret_devtools_gate_run_result",
        id: command.id.clone(),
        label: command.label.clone(),
        command_line: command.command_line.clone(),
        diag_args,
        missing_inputs: command
            .missing_inputs
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        status: if result.is_ok() { "passed" } else { "failed" },
        error: result.as_ref().err().cloned(),
        started_unix_ms,
        finished_unix_ms,
    }
}

impl GateRunResultHistoryEntry {
    fn from_job_result(result: &GateRunJobResult) -> Self {
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
            != Some("fret_devtools_gate_run_result")
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

fn gate_run_result_record_json(record: &GateRunResultRecordV1) -> Result<String, String> {
    serde_json::to_string_pretty(record)
        .map_err(|err| format!("failed to serialize gate run result: {err}"))
}

fn load_recent_gate_run_result_history_from_dir(
    result_dir: &Path,
    limit: usize,
) -> Vec<GateRunResultHistoryEntry> {
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
            let history_entry = GateRunResultHistoryEntry::from_result_record(&path, result_json)?;
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

fn fallback_gate_run_result_json(error: &str) -> String {
    serde_json::json!({
        "schema_version": 1,
        "kind": "fret_devtools_gate_run_result",
        "status": "failed",
        "error": error,
    })
    .to_string()
}

fn write_gate_run_result_record(out_path: &PathBuf, result_json: &str) -> Result<(), String> {
    std::fs::write(out_path, result_json.as_bytes()).map_err(|err| {
        format!(
            "failed to write gate run result {}: {err}",
            out_path.to_string_lossy()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_diag::{DevtoolsGateScriptTargetCommandInputV1, devtools_gate_script_target_command};

    fn gate_run_test_dir(label: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("fret-devtools-gate-run-{label}-{}", now_unix_ms()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create gate run test dir");
        dir
    }

    #[test]
    fn gate_run_result_record_has_stable_shape() {
        let command = devtools_gate_script_target_command(
            "pixels-changed",
            DevtoolsGateScriptTargetCommandInputV1::new(
                "tools/diag-scripts/smoke.json",
                "button.ok",
            ),
        )
        .expect("script-target command");
        let record = build_gate_run_result_record(
            &command,
            command.diag_args.clone(),
            10,
            35,
            &Err("boom".to_string()),
        );
        let value = serde_json::to_value(record).expect("record json");

        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["kind"], "fret_devtools_gate_run_result");
        assert_eq!(value["id"], "pixels-changed");
        assert_eq!(value["label"], "pixels changed");
        assert_eq!(value["status"], "failed");
        assert_eq!(value["error"], "boom");
        assert_eq!(value["started_unix_ms"], 10);
        assert_eq!(value["finished_unix_ms"], 35);
        assert_eq!(
            value["diag_args"],
            serde_json::json!([
                "run",
                "tools/diag-scripts/smoke.json",
                "--check-pixels-changed",
                "button.ok",
                "--json"
            ])
        );
    }

    #[test]
    fn gate_run_result_summary_lines_project_status_and_duration() {
        let json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_gate_run_result",
            "id": "stale-paint-scene",
            "label": "stale paint/scene",
            "command_line": "cargo run -p fretboard-dev -- diag run tools/diag-scripts/smoke.json --check-stale-paint button.ok --check-stale-scene button.ok --json",
            "diag_args": ["run", "tools/diag-scripts/smoke.json", "--check-stale-paint", "button.ok", "--check-stale-scene", "button.ok", "--json"],
            "status": "failed",
            "error": "boom",
            "started_unix_ms": 10,
            "finished_unix_ms": 45
        })
        .to_string();

        let text = gate_run_result_summary_lines(&json).join("\n");

        assert!(text.contains("status: failed"));
        assert!(text.contains("id: stale-paint-scene"));
        assert!(text.contains("label: stale paint/scene"));
        assert!(text.contains("duration_ms: 35"));
        assert!(text.contains("diag_args_count: 7"));
        assert!(text.contains("error: boom"));
        assert!(text.contains("command: cargo run -p fretboard-dev -- diag run"));
    }

    #[test]
    fn load_recent_gate_run_result_history_reads_latest_valid_records() {
        let dir = gate_run_test_dir("history");
        let older = dir.join("10-stale-paint-scene.json");
        let newer = dir.join("20-pixels-changed.json");
        let ignored_kind = dir.join("30-other.json");
        let bad_json = dir.join("40-bad.json");

        let older_json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_gate_run_result",
            "id": "stale-paint-scene",
            "label": "stale paint/scene",
            "command_line": "cargo run -p fretboard-dev -- diag run a.json --check-stale-paint button.ok --json",
            "status": "passed",
            "started_unix_ms": 10,
            "finished_unix_ms": 20
        })
        .to_string();
        let newer_json = serde_json::json!({
            "schema_version": 1,
            "kind": "fret_devtools_gate_run_result",
            "id": "pixels-changed",
            "label": "pixels changed",
            "command_line": "cargo run -p fretboard-dev -- diag run b.json --check-pixels-changed button.ok --json",
            "status": "failed",
            "error": "boom",
            "started_unix_ms": 30,
            "finished_unix_ms": 35
        })
        .to_string();

        std::fs::write(&older, older_json).expect("write older");
        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(&newer, newer_json).expect("write newer");
        std::fs::write(
            &ignored_kind,
            serde_json::json!({"kind": "not_gate_run", "status": "passed"}).to_string(),
        )
        .expect("write ignored");
        std::fs::write(&bad_json, "{").expect("write bad");

        let entries = load_recent_gate_run_result_history_from_dir(&dir, 8);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "pixels-changed");
        assert_eq!(entries[0].status, "failed");
        assert_eq!(entries[0].error.as_deref(), Some("boom"));
        assert_eq!(entries[0].result_path, newer.to_string_lossy());
        assert_eq!(entries[1].id, "stale-paint-scene");
        assert_eq!(entries[1].status, "passed");

        let limited = load_recent_gate_run_result_history_from_dir(&dir, 1);
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].id, "pixels-changed");
        assert!(load_recent_gate_run_result_history_from_dir(&dir, 0).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_recent_gate_run_result_history_prefers_record_time_over_file_mtime() {
        let dir = gate_run_test_dir("history-record-time");
        let older_mtime = dir.join("10-record-newer.json");
        let newer_mtime = dir.join("20-record-older.json");

        std::fs::write(
            &older_mtime,
            serde_json::json!({
                "schema_version": 1,
                "kind": "fret_devtools_gate_run_result",
                "id": "record-newer",
                "label": "record newer",
                "command_line": "gate record newer",
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
                "kind": "fret_devtools_gate_run_result",
                "id": "record-older",
                "label": "record older",
                "command_line": "gate record older",
                "status": "failed",
                "started_unix_ms": 500,
                "finished_unix_ms": 600
            })
            .to_string(),
        )
        .expect("write record-older");

        let entries = load_recent_gate_run_result_history_from_dir(&dir, 8);

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "record-newer");
        assert_eq!(entries[0].result_path, older_mtime.to_string_lossy());
        assert_eq!(entries[1].id, "record-older");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate_run_result_history_selects_explicit_path_or_latest() {
        let entries = vec![
            GateRunResultHistoryEntry {
                id: "pixels-changed".to_string(),
                label: "pixels changed".to_string(),
                command_line: "cargo run -p fretboard-dev -- diag run b.json".to_string(),
                result_path: ".fret/diag/gate-runs/20-pixels-changed.json".to_string(),
                result_json: "{\"status\":\"passed\"}".to_string(),
                status: "passed".to_string(),
                error: None,
            },
            GateRunResultHistoryEntry {
                id: "stale-paint-scene".to_string(),
                label: "stale paint/scene".to_string(),
                command_line: "cargo run -p fretboard-dev -- diag run a.json".to_string(),
                result_path: ".fret/diag/gate-runs/10-stale-paint-scene.json".to_string(),
                result_json: "{\"status\":\"failed\"}".to_string(),
                status: "failed".to_string(),
                error: Some("boom".to_string()),
            },
        ];

        assert_eq!(
            gate_run_result_history_selected_or_latest_entry(&entries, None)
                .map(|entry| entry.result_path),
            Some(".fret/diag/gate-runs/20-pixels-changed.json".to_string())
        );
        assert_eq!(
            gate_run_result_history_selected_or_latest_entry(
                &entries,
                Some(".fret/diag/gate-runs/10-stale-paint-scene.json")
            )
            .map(|entry| (entry.result_path, entry.result_json)),
            Some((
                ".fret/diag/gate-runs/10-stale-paint-scene.json".to_string(),
                "{\"status\":\"failed\"}".to_string(),
            ))
        );
        assert_eq!(
            gate_run_result_history_selected_or_latest_entry(
                &entries,
                Some(".fret/diag/gate-runs/missing.json")
            )
            .map(|entry| entry.result_path),
            Some(".fret/diag/gate-runs/20-pixels-changed.json".to_string())
        );

        let summary = gate_run_result_history_summary_lines(&entries).join("\n");
        assert!(summary.contains("gate run history: 2 result(s)"));
        assert!(summary.contains("passed | pixels-changed"));

        let details = gate_run_result_history_entry_detail_lines(entries.get(1)).join("\n");
        assert!(details.contains("status: failed"));
        assert!(details.contains("result_path: .fret/diag/gate-runs/10-stale-paint-scene.json"));
        assert!(details.contains("error: boom"));
    }
}
