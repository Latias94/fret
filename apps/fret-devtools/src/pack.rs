use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;

use fret_app::App;
use fret_diag::artifacts;

use crate::{State, is_abs_path, now_unix_ms, push_log, repo_root_from_script_paths};

#[derive(Debug, Clone)]
pub(crate) struct PackJobResult {
    pub out_path: PathBuf,
    pub result: Result<(), String>,
}

pub(crate) fn poll_pack_jobs(app: &mut App, st: &mut State) {
    while let Ok(msg) = st.pack_rx.try_recv() {
        let _ = app.models_mut().update(&st.pack_in_flight, |v| *v = false);

        match msg.result {
            Ok(()) => {
                let _ = app.models_mut().update(&st.last_pack_path, |v| {
                    *v = Some(Arc::<str>::from(msg.out_path.to_string_lossy().to_string()))
                });
                let _ = app.models_mut().update(&st.pack_last_error, |v| *v = None);
                push_log(
                    app,
                    &st.log_lines,
                    &format!("pack ok: {}", msg.out_path.to_string_lossy()),
                );
            }
            Err(err) => {
                let _ = app.models_mut().update(&st.pack_last_error, |v| {
                    *v = Some(Arc::<str>::from(err.clone()))
                });
                push_log(app, &st.log_lines, &format!("pack failed: {err}"));
            }
        }
    }
}

pub(crate) fn start_pack_last_bundle(app: &mut App, st: &mut State) -> Result<(), String> {
    let in_flight = app
        .models()
        .read(&st.pack_in_flight, |v| *v)
        .unwrap_or(false);
    if in_flight {
        return Err("pack already in progress".to_string());
    }

    let repo_root = repo_root_from_script_paths(&st.script_paths);
    let (out_dir, bundle_dir_abs) = ensure_bundle_dir_materialized(app, st, &repo_root)?;

    let pack_dir = repo_root.join(".fret").join("diag").join("packs");
    let _ = std::fs::create_dir_all(&pack_dir);

    let bundle_name = bundle_dir_abs
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("bundle");
    let out_path = pack_dir.join(format!("{bundle_name}-{}.zip", now_unix_ms()));

    let args = vec![
        "--dir".to_string(),
        out_dir,
        "--pack-out".to_string(),
        out_path.to_string_lossy().to_string(),
        "--include-all".to_string(),
        "pack".to_string(),
        bundle_dir_abs.to_string_lossy().to_string(),
    ];

    let tx = st.pack_tx.clone();
    std::thread::spawn(move || {
        let result = fret_diag::diag_cmd(args);
        let _ = tx.send(PackJobResult { out_path, result });
    });

    let _ = app.models_mut().update(&st.pack_in_flight, |v| *v = true);
    let _ = app.models_mut().update(&st.pack_last_error, |v| *v = None);

    Ok(())
}

fn ensure_bundle_dir_materialized(
    app: &mut App,
    st: &mut State,
    repo_root: &PathBuf,
) -> Result<(String, PathBuf), String> {
    let out_dir_arg = app
        .models()
        .read(&st.target_out_dir, |v| v.clone())
        .ok()
        .flatten()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "target/fret-diag".to_string());

    if let Some(dir) = app
        .models()
        .read(&st.last_bundle_dir_abs, |v| v.clone())
        .ok()
        .flatten()
    {
        let dir_abs = if is_abs_path(dir.as_ref()) {
            PathBuf::from(dir.as_ref())
        } else {
            repo_root.join(dir.as_ref())
        };
        if dir_abs.join("bundle.json").is_file() {
            return Ok((out_dir_arg, dir_abs));
        }
    }

    let Some(bundle_json) = app
        .models()
        .read(&st.last_bundle_dump_bundle_json, |v| v.clone())
        .ok()
        .flatten()
    else {
        return Err("no in-memory bundle payload to materialize yet".to_string());
    };

    let exported_unix_ms = app
        .models()
        .read(&st.last_bundle_dump_exported_unix_ms, |v| *v)
        .ok()
        .flatten()
        .unwrap_or_else(now_unix_ms);

    let mat = artifacts::materialize_bundle_json_to_exports(
        repo_root,
        exported_unix_ms,
        bundle_json.as_ref(),
    )?;

    let _ = app.models_mut().update(&st.target_out_dir, |v| {
        *v = Some(Arc::<str>::from(
            mat.exports_root.to_string_lossy().to_string(),
        ));
    });
    let _ = app.models_mut().update(&st.last_bundle_dir_abs, |v| {
        *v = Some(Arc::<str>::from(
            mat.export_dir.to_string_lossy().to_string(),
        ));
    });

    Ok((
        mat.exports_root.to_string_lossy().to_string(),
        mat.export_dir,
    ))
}

pub(crate) fn new_pack_channel() -> (mpsc::Sender<PackJobResult>, mpsc::Receiver<PackJobResult>) {
    mpsc::channel()
}
