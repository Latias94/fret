use std::time::{Duration, Instant};

use fret_diag_protocol::DevtoolsBundleDumpedV1;

use crate::devtools::DevtoolsOps;

use super::DiagTransportKind;

/// Tooling-side transport seam that isolates filesystem vs DevTools WS behavior differences.
///
/// Design goals:
/// - Keep transport-specific quirks out of higher-level tooling flows (`diag run/suite/repro/perf`).
/// - Make differences explicit and removable (fearless refactor friendly).
/// - Prefer small, testable helpers over scattered `if kind == ...` checks.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ToolingTransportSeamV1 {
    kind: DiagTransportKind,
}

impl ToolingTransportSeamV1 {
    pub(crate) fn new(kind: DiagTransportKind) -> Self {
        Self { kind }
    }

    pub(crate) fn timeout_hint_for_waiting_script_result(&self) -> Option<&'static str> {
        match self.kind {
            DiagTransportKind::WebSocket => Some(
                "devtools_ws_hint=keep the app actively rendering (web: tab must be visible; background tabs may throttle rAF) and ensure the page URL includes fret_devtools_ws + fret_devtools_token",
            ),
            _ => None,
        }
    }

    pub(crate) fn new_script_run_retoucher(
        &self,
        timeout_ms: u64,
        poll_ms: u64,
    ) -> Option<ScriptRunRetoucherV1> {
        match self.kind {
            DiagTransportKind::FileSystem => Some(ScriptRunRetoucherV1::new(timeout_ms, poll_ms)),
            _ => None,
        }
    }

    pub(crate) fn bundle_dump_request_id(
        &self,
        devtools: &DevtoolsOps,
        session_id: Option<&str>,
        label: Option<&str>,
        max_snapshots: Option<u32>,
    ) -> Option<u64> {
        match (self.kind, max_snapshots) {
            (DiagTransportKind::WebSocket, Some(max)) => {
                Some(devtools.bundle_dump_with_max_snapshots(session_id, label, max))
            }
            (DiagTransportKind::WebSocket, None) => Some(devtools.bundle_dump(session_id, label)),
            (_, Some(max)) => {
                devtools.bundle_dump_with_max_snapshots(session_id, label, max);
                None
            }
            (_, None) => {
                devtools.bundle_dump(session_id, label);
                None
            }
        }
    }

    pub(crate) fn wait_for_bundle_dumped_with_baseline_mitigation(
        &self,
        devtools: &DevtoolsOps,
        selected_session_id: &str,
        expected_request_id: Option<u64>,
        label: Option<&str>,
        timeout_ms: u64,
        poll_ms: u64,
    ) -> Result<DevtoolsBundleDumpedV1, String> {
        // Filesystem transport can miss the first trigger touch edge if the app has not yet
        // established its baseline stamp. Mitigate by doing a short initial wait and re-touching
        // once before consuming the full timeout budget.
        if matches!(self.kind, DiagTransportKind::FileSystem) && expected_request_id.is_none() {
            let short_ms = timeout_ms.min(2_000);
            match crate::devtools::wait_for_bundle_dumped(
                devtools,
                selected_session_id,
                None,
                short_ms,
                poll_ms,
            ) {
                Ok(v) => return Ok(v),
                Err(err) if err.contains("timed out waiting") => {
                    devtools.bundle_dump(None, label);
                }
                Err(err) => return Err(err),
            }
        }

        crate::devtools::wait_for_bundle_dumped(
            devtools,
            selected_session_id,
            expected_request_id,
            timeout_ms,
            poll_ms,
        )
    }
}

/// Filesystem-trigger baseline race mitigation for `script.run` (`script.touch`).
///
/// When using filesystem transport, the app may not have established a baseline stamp for the
/// trigger file yet. Retouching the script payload after a grace period mitigates the race without
/// requiring in-app changes.
#[derive(Debug, Clone)]
pub(crate) struct ScriptRunRetoucherV1 {
    next_retouch_at: Instant,
    interval_ms: u64,
}

impl ScriptRunRetoucherV1 {
    pub(crate) fn new(timeout_ms: u64, poll_ms: u64) -> Self {
        fn start_grace_ms(timeout_ms: u64, poll_ms: u64) -> u64 {
            let baseline_race_ms = poll_ms.saturating_mul(4).clamp(250, 5_000);
            baseline_race_ms.min(timeout_ms.saturating_div(2).max(250))
        }

        Self {
            next_retouch_at: Instant::now()
                + Duration::from_millis(start_grace_ms(timeout_ms, poll_ms)),
            interval_ms: 2_000,
        }
    }

    pub(crate) fn maybe_retouch_at(
        &mut self,
        devtools: &DevtoolsOps,
        session_id: Option<&str>,
        script_json: &serde_json::Value,
        run_id_seen: bool,
        now: Instant,
    ) {
        if run_id_seen {
            return;
        }
        if now < self.next_retouch_at {
            return;
        }

        devtools.script_run_value(session_id, script_json.clone());

        self.interval_ms = (self.interval_ms.saturating_mul(2)).min(10_000);
        self.next_retouch_at = now + Duration::from_millis(self.interval_ms);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use fret_diag_protocol::DiagTransportMessageV1;

    use super::*;
    use crate::transport::{DiagTransport, ToolingDiagClient};

    struct TestTransport {
        kind: DiagTransportKind,
        sent: Mutex<Vec<DiagTransportMessageV1>>,
    }

    impl TestTransport {
        fn new(kind: DiagTransportKind) -> Self {
            Self {
                kind,
                sent: Mutex::new(Vec::new()),
            }
        }
    }

    impl DiagTransport for TestTransport {
        fn kind(&self) -> DiagTransportKind {
            self.kind
        }

        fn send(&self, msg: DiagTransportMessageV1) {
            self.sent.lock().unwrap().push(msg);
        }

        fn try_recv(&self) -> Option<DiagTransportMessageV1> {
            None
        }

        fn set_default_session_id(&self, _session_id: Option<String>) {}
    }

    #[test]
    fn bundle_dump_request_id_is_only_meaningful_for_ws() {
        for kind in [DiagTransportKind::WebSocket, DiagTransportKind::FileSystem] {
            let transport = Arc::new(TestTransport::new(kind));
            let client = ToolingDiagClient::new_for_test(transport.clone());
            let devtools = DevtoolsOps::with_request_id_seed(client, 7);
            let seam = ToolingTransportSeamV1::new(kind);

            let expected = seam.bundle_dump_request_id(&devtools, Some("s1"), Some("lbl"), None);
            match kind {
                DiagTransportKind::WebSocket => assert_eq!(expected, Some(7)),
                DiagTransportKind::FileSystem => assert_eq!(expected, None),
            }

            let sent = transport.sent.lock().unwrap();
            assert_eq!(sent.len(), 1);
            assert_eq!(sent[0].r#type, "bundle.dump");
        }
    }

    #[test]
    fn timeout_hint_is_only_set_for_ws() {
        assert!(
            ToolingTransportSeamV1::new(DiagTransportKind::WebSocket)
                .timeout_hint_for_waiting_script_result()
                .is_some()
        );
        assert!(
            ToolingTransportSeamV1::new(DiagTransportKind::FileSystem)
                .timeout_hint_for_waiting_script_result()
                .is_none()
        );
    }
}
