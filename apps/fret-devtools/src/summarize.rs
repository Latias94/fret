use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;

use fret_app::App;

use crate::{
    State, is_abs_path, push_log, refresh_regression_artifacts, repo_root_from_script_paths,
};

#[derive(Debug, Clone)]
pub(crate) struct SummarizeJobResult {
    pub out_dir: PathBuf,
    pub result: Result<(), String>,
}

pub(crate) fn poll_summarize_jobs(app: &mut App, st: &mut State) {
    while let Ok(msg) = st.summarize_rx.try_recv() {
        let _ = app
            .models_mut()
            .update(&st.summarize_in_flight, |v| *v = false);

        match msg.result {
            Ok(()) => {
                let _ = app
                    .models_mut()
                    .update(&st.summarize_last_error, |v| *v = None);
                push_log(
                    app,
                    &st.log_lines,
                    &format!("regression summarize ok: {}", msg.out_dir.to_string_lossy()),
                );
                refresh_regression_artifacts(app, st);
            }
            Err(err) => {
                let _ = app.models_mut().update(&st.summarize_last_error, |v| {
                    *v = Some(Arc::<str>::from(err.clone()))
                });
                push_log(
                    app,
                    &st.log_lines,
                    &format!("regression summarize failed: {err}"),
                );
            }
        }
    }
}

pub(crate) fn start_regression_summarize(app: &mut App, st: &mut State) -> Result<(), String> {
    let in_flight = app
        .models()
        .read(&st.summarize_in_flight, |v| *v)
        .unwrap_or(false);
    if in_flight {
        return Err("regression summarize already in progress".to_string());
    }

    let repo_root = repo_root_from_script_paths(&st.script_paths);
    let out_dir_raw = app
        .models()
        .read(&st.target_out_dir, |v| v.clone())
        .ok()
        .flatten()
        .map(|s| s.to_string())
        .ok_or_else(|| "no target out_dir available for regression summarize".to_string())?;

    let out_dir = if is_abs_path(&out_dir_raw) {
        PathBuf::from(&out_dir_raw)
    } else {
        repo_root.join(&out_dir_raw)
    };

    let args = vec!["--dir".to_string(), out_dir_raw, "summarize".to_string()];

    let tx = st.summarize_tx.clone();
    std::thread::spawn(move || {
        let result = fret_diag::diag_cmd(args);
        let _ = tx.send(SummarizeJobResult { out_dir, result });
    });

    let _ = app
        .models_mut()
        .update(&st.summarize_in_flight, |v| *v = true);
    let _ = app
        .models_mut()
        .update(&st.summarize_last_error, |v| *v = None);

    Ok(())
}

pub(crate) fn new_summarize_channel() -> (
    mpsc::Sender<SummarizeJobResult>,
    mpsc::Receiver<SummarizeJobResult>,
) {
    mpsc::channel()
}
