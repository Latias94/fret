use std::sync::Arc;
use std::sync::mpsc;

use fret_app::App;
use fret_diag::regression_summary::RegressionBundleFollowupCommandV1;

use crate::{State, push_log};

#[derive(Debug, Clone)]
pub(crate) struct FollowupJobResult {
    pub id: String,
    pub label: String,
    pub command_line: String,
    pub result: Result<(), String>,
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

        match msg.result {
            Ok(()) => {
                let _ = app
                    .models_mut()
                    .update(&st.followup_last_error, |v| *v = None);
                push_log(
                    app,
                    &st.log_lines,
                    &format!("follow-up ok: {} ({})", msg.label, msg.id),
                );
            }
            Err(err) => {
                let _ = app.models_mut().update(&st.followup_last_error, |v| {
                    *v = Some(Arc::<str>::from(err.clone()))
                });
                push_log(
                    app,
                    &st.log_lines,
                    &format!("follow-up failed: {} ({}): {err}", msg.label, msg.id),
                );
            }
        }
    }
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
    let id = command.id.clone();
    let label = command.label.clone();
    let command_line = command.command_line.clone();
    let tx = st.followup_tx.clone();
    std::thread::spawn({
        let id = id.clone();
        let label = label.clone();
        let command_line = command_line.clone();
        move || {
            let result = fret_diag::diag_cmd(args);
            let _ = tx.send(FollowupJobResult {
                id,
                label,
                command_line,
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
}
