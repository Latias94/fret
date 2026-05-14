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
    pub result: Result<(), String>,
}

#[derive(Debug, Clone, Serialize)]
struct FollowupResultRecordV1 {
    schema_version: u32,
    kind: &'static str,
    id: String,
    label: String,
    command_line: String,
    diag_args: Vec<String>,
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
    FollowupResultRecordV1 {
        schema_version: 1,
        kind: "fret_devtools_regression_followup_result",
        id: command.id.clone(),
        label: command.label.clone(),
        command_line: command.command_line.clone(),
        diag_args,
        status: if result.is_ok() { "passed" } else { "failed" },
        error: result.as_ref().err().cloned(),
        started_unix_ms,
        finished_unix_ms,
    }
}

fn write_followup_result_record(
    out_path: &PathBuf,
    record: &FollowupResultRecordV1,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(record)
        .map_err(|err| format!("failed to serialize follow-up result: {err}"))?;
    std::fs::write(out_path, bytes).map_err(|err| {
        format!(
            "failed to write follow-up result {}: {err}",
            out_path.to_string_lossy()
        )
    })
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
            let write_result = write_followup_result_record(&result_path, &record);
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
        assert_eq!(value["started_unix_ms"], 10);
        assert_eq!(value["finished_unix_ms"], 20);
    }
}
