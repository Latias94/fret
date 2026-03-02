use std::path::PathBuf;

use fret_diag_protocol::{UiScriptEvidenceV1, UiScriptResultV1};

use super::{UiDiagnosticsService, touch_file};

fn write_script_result_json(
    path: PathBuf,
    result: &UiScriptResultV1,
) -> Result<(), std::io::Error> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;

    let bytes = serde_json::to_vec_pretty(result).or_else(|_err| {
        let mut sanitized = result.clone();
        sanitized.reason_code = sanitized
            .reason_code
            .take()
            .or_else(|| Some("diag.script_result.serialize_failed_evidence_dropped".to_string()));

        sanitized.evidence = sanitized.evidence.take().map(|e| UiScriptEvidenceV1 {
            event_log: e.event_log,
            event_log_dropped: e.event_log_dropped,
            ..UiScriptEvidenceV1::default()
        });

        serde_json::to_vec_pretty(&sanitized)
    });

    let bytes = bytes.unwrap_or_else(|_err| {
        br#"{"schema_version":1,"run_id":0,"updated_unix_ms":0,"window":null,"stage":"failed","step_index":null,"reason_code":"diag.script_result.serialize_failed","reason":"failed to serialize script.result.json","last_bundle_dir":null}"#.to_vec()
    });

    std::fs::write(path, bytes)
}

impl UiDiagnosticsService {
    pub(super) fn write_script_result(&mut self, result: UiScriptResultV1) {
        if !self.is_enabled() {
            return;
        }

        if !cfg!(target_arch = "wasm32") {
            let _ = write_script_result_json(self.cfg.script_result_path.clone(), &result);
            let _ = touch_file(&self.cfg.script_result_trigger_path);
        }

        #[cfg(feature = "diagnostics-ws")]
        {
            self.ws_send_script_result_v1(&result);
        }
    }
}
