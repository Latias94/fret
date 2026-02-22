use fret_diag_protocol::UiScriptResultV1;

use super::{UiDiagnosticsService, touch_file, write_json};

impl UiDiagnosticsService {
    pub(super) fn write_script_result(&mut self, result: UiScriptResultV1) {
        if !self.is_enabled() {
            return;
        }

        if !cfg!(target_arch = "wasm32") {
            let _ = write_json(self.cfg.script_result_path.clone(), &result);
            let _ = touch_file(&self.cfg.script_result_trigger_path);
        }

        #[cfg(feature = "diagnostics-ws")]
        {
            self.ws_send_script_result_v1(&result);
        }
    }
}
