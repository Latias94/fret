use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;

use fret_app::App;
use fret_diag::regression_summary::RegressionBundleFollowupCommandV1;
use serde::Serialize;

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
    started_unix_ms: u64,
    finished_unix_ms: u64,
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
    let command_line = field("command_line");
    if command_line != "-" {
        lines.push(format!("command: {command_line}"));
    }
    lines
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

pub(crate) fn followup_result_history_latest_path<'a>(
    entries: &[FollowupResultHistoryEntry],
    selected_bundle_dirs: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    let selected_bundle_keys = followup_selected_bundle_keys(selected_bundle_dirs);
    followup_result_history_latest_entry(entries, &selected_bundle_keys)
        .map(|entry| entry.result_path.clone())
}

pub(crate) fn followup_result_history_latest_json<'a>(
    entries: &[FollowupResultHistoryEntry],
    selected_bundle_dirs: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    let selected_bundle_keys = followup_selected_bundle_keys(selected_bundle_dirs);
    followup_result_history_latest_entry(entries, &selected_bundle_keys)
        .map(|entry| entry.result_json.clone())
}

pub(crate) fn poll_followup_jobs(app: &mut App, st: &mut State) {
    while let Ok(msg) = st.followup_rx.try_recv() {
        let _ = app
            .models_mut()
            .update(&st.followup_in_flight, |v| *v = false);
        let _ = app.models_mut().update(&st.followup_last_command_line, |v| {
            *v = Some(Arc::<str>::from(msg.command_line.clone()))
        });
        let _ = app.models_mut().update(&st.followup_last_result_path, |v| {
            *v = Some(Arc::<str>::from(
                msg.result_path.to_string_lossy().to_string(),
            ))
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
) -> FollowupResultRecordV1 {
    let bundle_dir = followup_bundle_dir_from_diag_args(&diag_args);
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
}

fn followup_bundle_dir_from_diag_args(args: &[String]) -> Option<String> {
    args.get(1)
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
                .and_then(|args| args.get(1))
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

fn followup_result_history_latest_entry<'a>(
    entries: &'a [FollowupResultHistoryEntry],
    selected_bundle_keys: &[String],
) -> Option<&'a FollowupResultHistoryEntry> {
    if selected_bundle_keys.is_empty() {
        return None;
    }
    entries
        .iter()
        .find(|entry| followup_history_entry_matches_selected_bundle(entry, selected_bundle_keys))
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

        let command = regression_bundle_followup_commands(["target/fret-diag/run-a"])
            .into_iter()
            .find(|command| command.id == "stats")
            .expect("stats command");
        let record = build_followup_result_record(&command, command.diag_args.clone(), 10, 20, &Ok(()));
        let json = followup_result_record_json(&record).expect("record json text");
        assert!(json.contains("\"status\": \"passed\""));
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
            followup_result_history_latest_path(&entries, ["target\\fret-diag\\run-a"]),
            Some(".fret/diag/followups/20-stats.json".to_string())
        );
        assert_eq!(
            followup_result_history_latest_json(&entries, ["target/fret-diag/run-a"]),
            Some("{\"status\":\"passed\"}".to_string())
        );
        assert_eq!(
            followup_result_history_latest_path(&entries, ["target/fret-diag/missing"]),
            None
        );
    }
}
