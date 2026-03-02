use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use fret_diag_protocol::{
    DevtoolsBundleDumpV1, DevtoolsBundleDumpedV1, DiagTransportMessageV1, FilesystemCapabilitiesV1,
};

use crate::util::{now_unix_ms, read_json_value, touch, write_json_value};

use super::{DiagInbox, DiagTransport, DiagTransportKind, fs_single_session_list};

#[derive(Debug, Clone)]
pub struct FsDiagTransportConfig {
    pub out_dir: PathBuf,

    pub trigger_path: PathBuf,

    pub script_path: PathBuf,
    pub script_trigger_path: PathBuf,
    pub script_result_path: PathBuf,
    pub script_result_trigger_path: PathBuf,

    pub pick_trigger_path: PathBuf,
    pub pick_result_path: PathBuf,
    pub pick_result_trigger_path: PathBuf,

    pub inspect_path: PathBuf,
    pub inspect_trigger_path: PathBuf,

    pub screenshots_request_path: PathBuf,
    pub screenshots_trigger_path: PathBuf,
    pub screenshots_result_path: PathBuf,
    pub screenshots_result_trigger_path: PathBuf,
}

impl FsDiagTransportConfig {
    pub fn from_out_dir(out_dir: impl Into<PathBuf>) -> Self {
        let out_dir = out_dir.into();
        Self {
            trigger_path: out_dir.join("trigger.touch"),
            script_path: out_dir.join("script.json"),
            script_trigger_path: out_dir.join("script.touch"),
            script_result_path: out_dir.join("script.result.json"),
            script_result_trigger_path: out_dir.join("script.result.touch"),
            pick_trigger_path: out_dir.join("pick.touch"),
            pick_result_path: out_dir.join("pick.result.json"),
            pick_result_trigger_path: out_dir.join("pick.result.touch"),
            inspect_path: out_dir.join("inspect.json"),
            inspect_trigger_path: out_dir.join("inspect.touch"),
            screenshots_request_path: out_dir.join("screenshots.request.json"),
            screenshots_trigger_path: out_dir.join("screenshots.touch"),
            screenshots_result_path: out_dir.join("screenshots.result.json"),
            screenshots_result_trigger_path: out_dir.join("screenshots.result.touch"),
            out_dir,
        }
    }
}

pub struct FsDiagTransport {
    state: Mutex<State>,
    inbox: DiagInbox,
}

#[derive(Debug)]
struct State {
    cfg: FsDiagTransportConfig,
    emitted_sessions: bool,
    default_session_id: Option<String>,
    last_emitted_capabilities: Vec<String>,

    last_pick_result_stamp: Option<u64>,
    last_script_result_stamp: Option<u64>,
    last_screenshots_result_stamp: Option<u64>,
    last_latest_rel_dir: Option<String>,

    pending_outbox: VecDeque<DiagTransportMessageV1>,
}

impl FsDiagTransport {
    pub fn new(cfg: FsDiagTransportConfig) -> Self {
        Self {
            state: Mutex::new(State {
                cfg,
                emitted_sessions: false,
                default_session_id: Some("fs".to_string()),
                last_emitted_capabilities: Vec::new(),
                last_pick_result_stamp: None,
                last_script_result_stamp: None,
                last_screenshots_result_stamp: None,
                last_latest_rel_dir: None,
                pending_outbox: VecDeque::new(),
            }),
            inbox: DiagInbox::default(),
        }
    }
}

impl DiagTransport for FsDiagTransport {
    fn kind(&self) -> DiagTransportKind {
        DiagTransportKind::FileSystem
    }

    fn send(&self, msg: DiagTransportMessageV1) {
        let mut st = match self.state.lock() {
            Ok(st) => st,
            Err(_) => return,
        };

        match msg.r#type.as_str() {
            // Map WS-ish command messages to the filesystem trigger surface used by UiDiagnosticsService.
            "inspect.set" => {
                let enabled = msg.payload.get("enabled").and_then(|v| v.as_bool());
                let consume_clicks = msg.payload.get("consume_clicks").and_then(|v| v.as_bool());
                if let (Some(enabled), Some(consume_clicks)) = (enabled, consume_clicks) {
                    let cfg = serde_json::json!({
                        "schema_version": 1,
                        "enabled": enabled,
                        "consume_clicks": consume_clicks,
                    });
                    let _ = write_json_value(&st.cfg.inspect_path, &cfg);
                    let _ = touch(&st.cfg.inspect_trigger_path);
                }
            }
            "pick.arm" => {
                let _ = touch(&st.cfg.pick_trigger_path);
            }
            "bundle.dump" => {
                // Extend the filesystem trigger surface with an optional request envelope so the
                // runtime can mirror WS semantics (label/max_snapshots/request_id).
                //
                // This is best-effort: if the runtime ignores it, we still fall back to the
                // legacy trigger touch.
                if let Ok(parsed) = serde_json::from_value::<DevtoolsBundleDumpV1>(msg.payload) {
                    let request_path = st.cfg.out_dir.join("dump.request.json");
                    let payload = serde_json::json!({
                        "schema_version": 1,
                        "label": parsed.label,
                        "max_snapshots": parsed.max_snapshots,
                        "request_id": msg.request_id,
                    });
                    let _ = write_json_value(&request_path, &payload);
                }
                let _ = touch(&st.cfg.trigger_path);
            }
            "script.push" | "script.run" => {
                let script_value = msg
                    .payload
                    .get("script")
                    .cloned()
                    .unwrap_or_else(|| msg.payload.clone());
                let _ = write_json_value(&st.cfg.script_path, &script_value);
                let _ = touch(&st.cfg.script_trigger_path);
            }
            _ => {
                let session_id = st.default_session_id.clone();
                st.pending_outbox.push_back(DiagTransportMessageV1 {
                    schema_version: 1,
                    r#type: "error.unsupported_type".to_string(),
                    session_id,
                    request_id: msg.request_id,
                    payload: serde_json::json!({
                        "schema_version": 1,
                        "request_type": msg.r#type,
                        "message": "unsupported transport message for filesystem mode",
                    }),
                });
            }
        }
    }

    fn try_recv(&self) -> Option<DiagTransportMessageV1> {
        if let Some(msg) = self.inbox.pop() {
            return Some(msg);
        }

        {
            let mut st = self.state.lock().ok()?;
            st.poll(&self.inbox);
            if let Some(msg) = st.pending_outbox.pop_front() {
                return Some(msg);
            }
        }

        self.inbox.pop()
    }

    fn set_default_session_id(&self, session_id: Option<String>) {
        if let Ok(mut st) = self.state.lock() {
            st.default_session_id = session_id;
        }
    }
}

impl State {
    fn poll(&mut self, inbox: &DiagInbox) {
        self.poll_session_list(inbox);

        self.poll_pick_result(inbox);
        self.poll_script_result(inbox);
        self.poll_screenshots_result(inbox);
        self.poll_latest_pointer(inbox);
    }

    fn poll_session_list(&mut self, inbox: &DiagInbox) {
        let session_id = self
            .default_session_id
            .as_deref()
            .unwrap_or("fs")
            .to_string();

        let mut caps: Vec<String> = vec![
            // Backwards-compatible (legacy, un-namespaced) control plane capabilities.
            "inspect".to_string(),
            "pick".to_string(),
            "scripts".to_string(),
            "bundles".to_string(),
            "sessions".to_string(),
            // Namespaced control plane capabilities (recommended).
            "devtools.inspect".to_string(),
            "devtools.pick".to_string(),
            "devtools.scripts".to_string(),
            "devtools.bundles".to_string(),
            "devtools.sessions".to_string(),
        ];

        let path = self.cfg.out_dir.join("capabilities.json");
        if let Some(v) = read_json_value(&path)
            && let Ok(parsed) = serde_json::from_value::<FilesystemCapabilitiesV1>(v)
        {
            for c in parsed.capabilities {
                let c = normalize_capability_string(&c);
                if !c.is_empty() {
                    caps.push(c);
                }
            }
        }

        caps.sort();
        caps.dedup();

        if self.emitted_sessions && self.last_emitted_capabilities == caps {
            return;
        }
        self.last_emitted_capabilities = caps.clone();

        inbox.push(fs_single_session_list(&session_id, caps));
        self.emitted_sessions = true;
    }

    fn poll_pick_result(&mut self, inbox: &DiagInbox) {
        let stamp = read_touch_stamp(&self.cfg.pick_result_trigger_path);
        if !stamp_is_newer(&mut self.last_pick_result_stamp, stamp) {
            return;
        }

        let Some(payload) = read_json_value(&self.cfg.pick_result_path) else {
            return;
        };
        inbox.push(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "pick.result".to_string(),
            session_id: self.default_session_id.clone(),
            request_id: None,
            payload,
        });
    }

    fn poll_script_result(&mut self, inbox: &DiagInbox) {
        let stamp = read_touch_stamp(&self.cfg.script_result_trigger_path);
        if !stamp_is_newer(&mut self.last_script_result_stamp, stamp) {
            return;
        }

        let Some(payload) = read_json_value(&self.cfg.script_result_path) else {
            return;
        };
        inbox.push(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "script.result".to_string(),
            session_id: self.default_session_id.clone(),
            request_id: None,
            payload,
        });
    }

    fn poll_screenshots_result(&mut self, inbox: &DiagInbox) {
        let stamp = read_touch_stamp(&self.cfg.screenshots_result_trigger_path);
        if !stamp_is_newer(&mut self.last_screenshots_result_stamp, stamp) {
            return;
        }

        let Some(payload) = read_json_value(&self.cfg.screenshots_result_path) else {
            return;
        };
        inbox.push(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "screenshot.result".to_string(),
            session_id: self.default_session_id.clone(),
            request_id: None,
            payload,
        });
    }

    fn poll_latest_pointer(&mut self, inbox: &DiagInbox) {
        let latest_path = self.cfg.out_dir.join("latest.txt");
        let latest = std::fs::read_to_string(&latest_path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let Some(latest) = latest else {
            return;
        };
        if self.last_latest_rel_dir.as_deref() == Some(latest.as_str()) {
            return;
        }
        self.last_latest_rel_dir = Some(latest.clone());

        inbox.push(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "bundle.dumped".to_string(),
            session_id: self.default_session_id.clone(),
            request_id: None,
            payload: serde_json::to_value(DevtoolsBundleDumpedV1 {
                schema_version: 1,
                exported_unix_ms: now_unix_ms(),
                out_dir: self.cfg.out_dir.to_string_lossy().to_string(),
                dir: latest,
                bundle: None,
                bundle_json_chunk: None,
                bundle_json_chunk_index: None,
                bundle_json_chunk_count: None,
            })
            .unwrap_or_else(|_| serde_json::json!({})),
        });
    }
}

fn normalize_capability_string(raw: &str) -> String {
    crate::compat::normalize_capability_lossy(raw)
}

fn stamp_is_newer(slot: &mut Option<u64>, stamp: Option<u64>) -> bool {
    let Some(stamp) = stamp else {
        return false;
    };
    match slot {
        None => {
            *slot = Some(stamp);
            true
        }
        Some(prev) if stamp > *prev => {
            *slot = Some(stamp);
            true
        }
        _ => false,
    }
}

fn read_touch_stamp(path: &Path) -> Option<u64> {
    let bytes = crate::util::read_file_bytes_shared(path)?;
    let s = std::str::from_utf8(&bytes).ok()?;
    s.lines()
        .rev()
        .find_map(|line| line.trim().parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_diag_protocol::DevtoolsSessionListV1;

    #[test]
    fn fs_transport_emits_session_list_and_pick_result() {
        let dir = std::env::temp_dir().join(format!(
            "fret-diag-fs-{}-{}",
            now_unix_ms(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");

        let caps = FilesystemCapabilitiesV1 {
            schema_version: 1,
            capabilities: vec![
                "script_v2".to_string(),
                "foo.bar".to_string(),
                "diag.screenshot_png".to_string(),
            ],
            runner_kind: None,
            runner_version: None,
            hints: None,
        };
        write_json_value(
            &dir.join("capabilities.json"),
            &serde_json::to_value(caps).unwrap(),
        )
        .expect("write capabilities.json");

        let cfg = FsDiagTransportConfig::from_out_dir(&dir);
        let client = crate::transport::ToolingDiagClient::connect_fs(cfg).expect("connect_fs");

        let msg = client.try_recv().expect("session.list");
        assert_eq!(msg.r#type, "session.list");
        let list: DevtoolsSessionListV1 =
            serde_json::from_value(msg.payload).expect("parse session.list payload");
        assert_eq!(list.sessions.len(), 1);
        assert_eq!(list.sessions[0].session_id, "fs");
        let caps = &list.sessions[0].capabilities;
        assert!(caps.iter().any(|c| c == "devtools.sessions"));
        assert!(caps.iter().any(|c| c == "diag.script_v2"));
        assert!(caps.iter().any(|c| c == "diag.screenshot_png"));
        assert!(caps.iter().any(|c| c == "foo.bar"));

        let pick_payload = serde_json::json!({
            "schema_version": 1,
            "run_id": 1,
            "updated_unix_ms": now_unix_ms(),
            "selection": { "selector": { "kind": "test_id", "test_id": "ok" } }
        });
        write_json_value(&dir.join("pick.result.json"), &pick_payload).expect("write pick.result");
        touch(&dir.join("pick.result.touch")).expect("touch pick.result.touch");

        let msg = client.try_recv().expect("pick.result");
        assert_eq!(msg.r#type, "pick.result");
        assert!(msg.payload.get("selection").is_some());
    }
}
