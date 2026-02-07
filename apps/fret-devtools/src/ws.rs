use std::path::PathBuf;
use std::sync::Arc;

use fret_app::App;
use fret_diag_protocol::{
    DevtoolsSessionAddedV1, DevtoolsSessionListV1, DevtoolsSessionRemovedV1,
    DiagTransportMessageV1, UiScriptResultV1, UiScriptStageV1,
};

use crate::{State, is_abs_path, pack, push_log};

pub(crate) fn require_session_selected(app: &mut App, st: &State) -> bool {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    if selected.is_some() {
        return true;
    }
    push_log(
        app,
        &st.log_lines,
        "no session selected (connect an app or pick a session)",
    );
    false
}

pub(crate) fn drain_ws_messages(app: &mut App, st: &mut State) {
    while let Some(msg) = st.client.try_recv() {
        let ty = msg.r#type.clone();
        let compact = match msg.session_id.as_deref() {
            Some(s) => format!("type={ty} session_id={s}"),
            None => format!("type={ty}"),
        };
        push_log(app, &st.log_lines, &compact);

        match ty.as_str() {
            "session.list" => {
                if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionListV1>(msg.payload) {
                    let sessions = parsed.sessions;
                    let _ = app.models_mut().update(&st.sessions, |v| *v = sessions);
                    ensure_session_selection_is_valid(app, st);
                }
            }
            "session.added" => {
                if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionAddedV1>(msg.payload) {
                    let _ = app.models_mut().update(&st.sessions, |v| {
                        if let Some(pos) = v
                            .iter()
                            .position(|s| s.session_id == parsed.session.session_id)
                        {
                            v[pos] = parsed.session;
                        } else {
                            v.push(parsed.session);
                        }
                    });
                    ensure_session_selection_is_valid(app, st);
                }
            }
            "session.removed" => {
                if let Ok(parsed) = serde_json::from_value::<DevtoolsSessionRemovedV1>(msg.payload)
                {
                    let _ = app.models_mut().update(&st.sessions, |v| {
                        v.retain(|s| s.session_id != parsed.session_id);
                    });
                    ensure_session_selection_is_valid(app, st);
                }
            }
            "pick.result" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app.models_mut().update(&st.last_pick_json, |v| *v = text);
                }
            }
            "script.result" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(parsed) = serde_json::from_value::<UiScriptResultV1>(msg.payload.clone())
                {
                    let _ = app
                        .models_mut()
                        .update(&st.script_last_stage, |v| *v = Some(parsed.stage.clone()));
                    let _ = app
                        .models_mut()
                        .update(&st.script_last_step_index, |v| *v = parsed.step_index);
                    let _ = app.models_mut().update(&st.script_last_reason, |v| {
                        *v = parsed.reason.map(Into::into);
                    });
                    let _ = app.models_mut().update(&st.script_last_bundle_dir, |v| {
                        *v = parsed.last_bundle_dir.clone().map(Into::into);
                    });

                    if let Some(out_dir) = app
                        .models()
                        .read(&st.target_out_dir, |v| v.clone())
                        .ok()
                        .flatten()
                        .map(|s| s.to_string())
                    {
                        if let Some(rel) = parsed.last_bundle_dir.as_deref() {
                            if let Some(abs) = resolve_bundle_dir_abs(&out_dir, rel) {
                                let _ = app.models_mut().update(&st.last_bundle_dir_abs, |v| {
                                    *v = Some(abs.into());
                                });
                            }
                        }
                    }
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_script_result_json, |v| *v = text);
                }
                maybe_start_pack_after_run(app, st);
            }
            "bundle.dumped" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Some(ts) = msg.payload.get("exported_unix_ms").and_then(|v| v.as_u64()) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_bundle_dump_exported_unix_ms, |v| *v = Some(ts));
                }
                if let Some(bundle) = msg.payload.get("bundle") {
                    if let Ok(text) = serde_json::to_string_pretty(bundle) {
                        let _ = app
                            .models_mut()
                            .update(&st.last_bundle_dump_bundle_json, |v| {
                                *v = Some(Arc::<str>::from(text));
                            });
                    }
                }
                if let Some(out_dir) = msg.payload.get("out_dir").and_then(|v| v.as_str()) {
                    let _ = app.models_mut().update(&st.target_out_dir, |v| {
                        *v = Some(Arc::<str>::from(out_dir.to_string()));
                    });
                }
                if let (Some(out_dir), Some(dir)) = (
                    msg.payload.get("out_dir").and_then(|v| v.as_str()),
                    msg.payload.get("dir").and_then(|v| v.as_str()),
                ) {
                    if let Some(abs) = resolve_bundle_dir_abs(out_dir, dir) {
                        let _ = app.models_mut().update(&st.last_bundle_dir_abs, |v| {
                            *v = Some(Arc::<str>::from(abs));
                        });
                    }
                }
                if msg.payload.get("bundle").is_none() {
                    let loaded = msg
                        .payload
                        .get("out_dir")
                        .and_then(|v| v.as_str())
                        .zip(msg.payload.get("dir").and_then(|v| v.as_str()))
                        .and_then(|(out_dir, dir)| resolve_bundle_dir_abs(out_dir, dir))
                        .and_then(|abs_dir| {
                            let path = PathBuf::from(abs_dir).join("bundle.json");
                            std::fs::read_to_string(path).ok()
                        })
                        .map(Arc::<str>::from);
                    let _ = app
                        .models_mut()
                        .update(&st.last_bundle_dump_bundle_json, |v| *v = loaded);
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app.models_mut().update(&st.last_bundle_json, |v| *v = text);
                }
                maybe_start_pack_after_run(app, st);
            }
            "screenshot.result" => {
                if !message_matches_selected_session(app, st, &msg) {
                    continue;
                }
                if let Ok(text) = serde_json::to_string_pretty(&msg.payload) {
                    let _ = app
                        .models_mut()
                        .update(&st.last_screenshot_json, |v| *v = text);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn sync_selected_session_to_client(app: &mut App, st: &mut State) {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();

    if selected.as_deref() == st.applied_session_id.as_deref() {
        return;
    }

    st.client
        .set_default_session_id(selected.as_ref().map(|s| s.to_string()));
    st.applied_session_id = selected;
}

fn ensure_session_selection_is_valid(app: &mut App, st: &mut State) {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    let sessions = app
        .models()
        .read(&st.sessions, |v| v.clone())
        .unwrap_or_default();

    if let Some(selected) = selected.as_deref() {
        if sessions.iter().any(|s| s.session_id == selected) {
            return;
        }
    }

    let new_selected = sessions
        .first()
        .map(|s| Arc::<str>::from(s.session_id.clone()));
    let _ = app
        .models_mut()
        .update(&st.selected_session_id, |v| *v = new_selected);
}

fn message_matches_selected_session(
    app: &mut App,
    st: &State,
    msg: &DiagTransportMessageV1,
) -> bool {
    let selected = app
        .models()
        .read(&st.selected_session_id, |v| v.clone())
        .ok()
        .flatten();
    let Some(selected) = selected else {
        return true;
    };
    msg.session_id.as_deref() == Some(selected.as_ref())
}

fn maybe_start_pack_after_run(app: &mut App, st: &mut State) {
    let pack_after = app
        .models()
        .read(&st.script_pack_after_run, |v| *v)
        .unwrap_or(false);
    if !pack_after {
        return;
    }

    let stage = app
        .models()
        .read(&st.script_last_stage, |v| v.clone())
        .ok()
        .flatten();
    if !matches!(
        stage,
        Some(UiScriptStageV1::Passed) | Some(UiScriptStageV1::Failed)
    ) {
        return;
    }

    let has_bundle_dir = app
        .models()
        .read(&st.last_bundle_dir_abs, |v| v.is_some())
        .unwrap_or(false);
    let has_bundle_payload = app
        .models()
        .read(&st.last_bundle_dump_bundle_json, |v| v.is_some())
        .unwrap_or(false);
    if !(has_bundle_dir || has_bundle_payload) {
        return;
    }

    if let Err(err) = pack::start_pack_last_bundle(app, st) {
        push_log(app, &st.log_lines, &format!("pack refused: {err}"));
    }
    let _ = app
        .models_mut()
        .update(&st.script_pack_after_run, |v| *v = false);
}

fn resolve_bundle_dir_abs(out_dir: &str, dir: &str) -> Option<String> {
    let dir = dir.trim();
    if dir.is_empty() {
        return None;
    }
    if is_abs_path(dir) {
        return Some(dir.to_string());
    }

    let out_dir = out_dir.trim();
    if out_dir.is_empty() {
        return Some(dir.to_string());
    }
    let base = PathBuf::from(out_dir);
    Some(base.join(dir).to_string_lossy().to_string())
}
