use std::path::PathBuf;
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
        let _ = app
            .models_mut()
            .update(&st.gate_run_in_flight, |v| *v = false);
        let _ = app.models_mut().update(&st.gate_run_last_command_line, |v| {
            *v = Some(Arc::<str>::from(msg.command_line.clone()))
        });
        let _ = app.models_mut().update(&st.gate_run_last_result_path, |v| {
            *v = Some(Arc::<str>::from(
                msg.result_path.to_string_lossy().to_string(),
            ))
        });
        let _ = app
            .models_mut()
            .update(&st.gate_run_last_result_json, |v| *v = msg.result_json.clone());

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

fn gate_run_result_record_json(record: &GateRunResultRecordV1) -> Result<String, String> {
    serde_json::to_string_pretty(record)
        .map_err(|err| format!("failed to serialize gate run result: {err}"))
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
}
