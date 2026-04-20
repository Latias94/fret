use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use fret_diag_protocol::{
    DevtoolsAppExitRequestV1, DevtoolsBundleDumpV1, DevtoolsBundleDumpedV1,
    DevtoolsEnvironmentSourcesGetAckV1, DevtoolsEnvironmentSourcesGetV1,
    DevtoolsScreenshotRequestV1, DiagTransportMessageV1, UiHitTestExplainV1, UiInspectConfigV1,
    UiSelectorV1, UiSemanticsNodeGetV1,
};

use crate::transport::ToolingDiagClient;

#[derive(Clone)]
pub struct DevtoolsOps {
    client: ToolingDiagClient,
    next_request_id: Arc<AtomicU64>,
}

impl std::fmt::Debug for DevtoolsOps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DevtoolsOps").finish_non_exhaustive()
    }
}

impl DevtoolsOps {
    pub fn new(client: ToolingDiagClient) -> Self {
        Self::with_request_id_seed(client, 1000)
    }

    pub fn with_request_id_seed(client: ToolingDiagClient, seed: u64) -> Self {
        Self {
            client,
            next_request_id: Arc::new(AtomicU64::new(seed)),
        }
    }

    pub fn client(&self) -> &ToolingDiagClient {
        &self.client
    }

    pub fn set_default_session_id(&self, session_id: Option<String>) {
        self.client.set_default_session_id(session_id);
    }

    pub fn next_request_id(&self) -> u64 {
        self.next_request_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn send(&self, msg: DiagTransportMessageV1) {
        self.client.send(msg);
    }

    pub fn try_recv(&self) -> Option<DiagTransportMessageV1> {
        self.client.try_recv()
    }

    pub fn inspect_set(&self, session_id: Option<&str>, enabled: bool, consume_clicks: bool) {
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "inspect.set".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: None,
            payload: serde_json::to_value(UiInspectConfigV1 {
                schema_version: 1,
                enabled,
                consume_clicks,
            })
            .unwrap_or(serde_json::Value::Null),
        });
    }

    pub fn pick_arm(&self, session_id: Option<&str>) {
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "pick.arm".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: None,
            payload: serde_json::json!({}),
        });
    }

    pub fn bundle_dump(&self, session_id: Option<&str>, label: Option<&str>) -> u64 {
        let request_id = self.next_request_id();
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "bundle.dump".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: Some(request_id),
            payload: serde_json::to_value(DevtoolsBundleDumpV1 {
                schema_version: 1,
                label: label.map(|s| s.to_string()),
                max_snapshots: None,
            })
            .unwrap_or(serde_json::Value::Null),
        });
        request_id
    }

    pub fn bundle_dump_with_max_snapshots(
        &self,
        session_id: Option<&str>,
        label: Option<&str>,
        max_snapshots: u32,
    ) -> u64 {
        let request_id = self.next_request_id();
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "bundle.dump".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: Some(request_id),
            payload: serde_json::to_value(DevtoolsBundleDumpV1 {
                schema_version: 1,
                label: label.map(|s| s.to_string()),
                max_snapshots: Some(max_snapshots),
            })
            .unwrap_or(serde_json::Value::Null),
        });
        request_id
    }

    pub fn screenshot_request(
        &self,
        session_id: Option<&str>,
        label: Option<&str>,
        timeout_frames: u32,
        window: Option<u64>,
    ) -> u64 {
        let request_id = self.next_request_id();
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "screenshot.request".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: Some(request_id),
            payload: serde_json::to_value(DevtoolsScreenshotRequestV1 {
                schema_version: 1,
                label: label.map(|s| s.to_string()),
                timeout_frames,
                window,
            })
            .unwrap_or(serde_json::Value::Null),
        });
        request_id
    }

    pub fn environment_sources_get(&self, session_id: Option<&str>) -> u64 {
        let request_id = self.next_request_id();
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "environment.sources.get".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: Some(request_id),
            payload: serde_json::to_value(DevtoolsEnvironmentSourcesGetV1 { schema_version: 1 })
                .unwrap_or(serde_json::Value::Null),
        });
        request_id
    }

    pub fn script_push_value(&self, session_id: Option<&str>, script: serde_json::Value) {
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "script.push".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: None,
            payload: serde_json::json!({ "script": script }),
        });
    }

    pub fn script_run_value(&self, session_id: Option<&str>, script: serde_json::Value) {
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "script.run".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: None,
            payload: serde_json::json!({ "script": script }),
        });
    }

    pub fn semantics_node_get(&self, session_id: Option<&str>, window: u64, node_id: u64) -> u64 {
        let request_id = self.next_request_id();
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "semantics.node.get".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: Some(request_id),
            payload: serde_json::to_value(UiSemanticsNodeGetV1 {
                schema_version: 1,
                window,
                node_id,
            })
            .unwrap_or(serde_json::Value::Null),
        });
        request_id
    }

    pub fn hit_test_explain(
        &self,
        session_id: Option<&str>,
        window: u64,
        target: UiSelectorV1,
    ) -> u64 {
        let request_id = self.next_request_id();
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "hit_test.explain".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: Some(request_id),
            payload: serde_json::to_value(UiHitTestExplainV1 {
                schema_version: 1,
                window,
                target,
            })
            .unwrap_or(serde_json::Value::Null),
        });
        request_id
    }

    pub fn app_exit_request(
        &self,
        session_id: Option<&str>,
        reason: Option<&str>,
        delay_ms: Option<u64>,
    ) {
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "app.exit.request".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: None,
            payload: serde_json::to_value(DevtoolsAppExitRequestV1 {
                schema_version: 1,
                reason: reason.map(|s| s.to_string()),
                delay_ms,
            })
            .unwrap_or(serde_json::Value::Null),
        });
    }
}

pub(crate) fn wait_for_message<T>(
    devtools: &DevtoolsOps,
    timeout_ms: u64,
    poll_ms: u64,
    mut decode: impl FnMut(fret_diag_protocol::DiagTransportMessageV1) -> Option<T>,
) -> Result<T, String> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(1));
    loop {
        while let Some(msg) = devtools.try_recv() {
            if let Some(v) = decode(msg) {
                return Ok(v);
            }
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for DevTools message".to_string());
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

pub(crate) fn wait_for_bundle_dumped(
    devtools: &DevtoolsOps,
    selected_session_id: &str,
    expected_request_id: Option<u64>,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<DevtoolsBundleDumpedV1, String> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.max(1));

    let mut chunk_exported_unix_ms: Option<u64> = None;
    let mut chunk_out_dir: Option<String> = None;
    let mut chunk_dir: Option<String> = None;
    let mut chunks: Vec<Option<String>> = Vec::new();

    loop {
        while let Some(msg) = devtools.try_recv() {
            if msg.r#type != "bundle.dumped"
                || msg.session_id.as_deref() != Some(selected_session_id)
            {
                continue;
            }
            if let Some(expected) = expected_request_id
                && msg.request_id != Some(expected)
            {
                continue;
            }
            let Ok(dumped) = serde_json::from_value::<DevtoolsBundleDumpedV1>(msg.payload) else {
                continue;
            };

            if dumped.bundle.is_some() {
                return Ok(dumped);
            }

            if let (Some(chunk), Some(chunk_index), Some(chunk_count_value)) = (
                dumped.bundle_json_chunk.clone(),
                dumped.bundle_json_chunk_index,
                dumped.bundle_json_chunk_count,
            ) {
                if chunk_exported_unix_ms.is_none() {
                    chunk_exported_unix_ms = Some(dumped.exported_unix_ms);
                    chunk_out_dir = Some(dumped.out_dir.clone());
                    chunk_dir = Some(dumped.dir.clone());
                    chunks = vec![None; chunk_count_value.max(1) as usize];
                }

                if chunk_exported_unix_ms != Some(dumped.exported_unix_ms)
                    || chunk_dir.as_deref() != Some(dumped.dir.as_str())
                {
                    // A new dump started (or messages interleaved); reset to the latest seen.
                    chunk_exported_unix_ms = Some(dumped.exported_unix_ms);
                    chunk_out_dir = Some(dumped.out_dir.clone());
                    chunk_dir = Some(dumped.dir.clone());
                    chunks = vec![None; chunk_count_value.max(1) as usize];
                }

                if let Some(slot) = chunks.get_mut(chunk_index as usize) {
                    *slot = Some(chunk);
                }

                if chunks.iter().all(|c| c.is_some()) {
                    let mut json = String::new();
                    for part in chunks.iter().flatten() {
                        json.push_str(part);
                    }
                    let bundle = serde_json::from_str::<serde_json::Value>(&json).map_err(|e| {
                        format!("bundle.dumped chunked JSON was not valid JSON: {e}")
                    })?;
                    return Ok(DevtoolsBundleDumpedV1 {
                        schema_version: dumped.schema_version,
                        exported_unix_ms: chunk_exported_unix_ms.unwrap_or(dumped.exported_unix_ms),
                        out_dir: chunk_out_dir.clone().unwrap_or(dumped.out_dir),
                        dir: chunk_dir.clone().unwrap_or(dumped.dir),
                        bundle: Some(bundle),
                        bundle_json_chunk: None,
                        bundle_json_chunk_index: None,
                        bundle_json_chunk_count: None,
                    });
                }
                continue;
            }

            // Non-embedded bundle (native filesystem case): allow materialization to fall back to
            // reading the runtime's bundle artifact.
            return Ok(dumped);
        }

        if Instant::now() >= deadline {
            return Err("timed out waiting for bundle.dumped".to_string());
        }
        std::thread::sleep(Duration::from_millis(poll_ms.max(1)));
    }
}

pub(crate) fn wait_for_environment_sources_get_ack(
    devtools: &DevtoolsOps,
    selected_session_id: &str,
    expected_request_id: u64,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<DevtoolsEnvironmentSourcesGetAckV1, String> {
    wait_for_message(devtools, timeout_ms, poll_ms, |msg| {
        if msg.r#type != "environment.sources.get_ack"
            || msg.session_id.as_deref() != Some(selected_session_id)
            || msg.request_id != Some(expected_request_id)
        {
            return None;
        }
        serde_json::from_value::<DevtoolsEnvironmentSourcesGetAckV1>(msg.payload).ok()
    })
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use super::*;
    use crate::transport::{DiagTransport, DiagTransportKind};

    #[derive(Default)]
    struct TestTransport {
        sent: Mutex<Vec<DiagTransportMessageV1>>,
        recv: Mutex<VecDeque<DiagTransportMessageV1>>,
        default_session_id: Mutex<Option<String>>,
    }

    impl TestTransport {
        fn take_sent(&self) -> Vec<DiagTransportMessageV1> {
            std::mem::take(&mut self.sent.lock().unwrap())
        }

        fn push_recv(&self, msg: DiagTransportMessageV1) {
            self.recv.lock().unwrap().push_back(msg);
        }
    }

    impl DiagTransport for TestTransport {
        fn kind(&self) -> DiagTransportKind {
            DiagTransportKind::WebSocket
        }

        fn send(&self, msg: DiagTransportMessageV1) {
            self.sent.lock().unwrap().push(msg);
        }

        fn try_recv(&self) -> Option<DiagTransportMessageV1> {
            self.recv.lock().unwrap().pop_front()
        }

        fn set_default_session_id(&self, session_id: Option<String>) {
            *self.default_session_id.lock().unwrap() = session_id;
        }
    }

    #[test]
    fn bundle_dump_sets_request_id_and_returns_it() {
        let transport = Arc::new(TestTransport::default());
        let client = ToolingDiagClient::new_for_test(transport.clone());
        let ops = DevtoolsOps::with_request_id_seed(client, 42);

        let request_id = ops.bundle_dump(Some("s1"), Some("my-label"));
        assert_eq!(request_id, 42);

        let sent = transport.take_sent();
        assert_eq!(sent.len(), 1);

        let msg = &sent[0];
        assert_eq!(msg.r#type, "bundle.dump");
        assert_eq!(msg.session_id.as_deref(), Some("s1"));
        assert_eq!(msg.request_id, Some(request_id));

        let payload: DevtoolsBundleDumpV1 = serde_json::from_value(msg.payload.clone()).unwrap();
        assert_eq!(payload.label.as_deref(), Some("my-label"));
        assert_eq!(payload.max_snapshots, None);
    }

    #[test]
    fn bundle_dump_with_max_snapshots_sets_request_id_and_payload() {
        let transport = Arc::new(TestTransport::default());
        let client = ToolingDiagClient::new_for_test(transport.clone());
        let ops = DevtoolsOps::with_request_id_seed(client, 7);

        let request_id = ops.bundle_dump_with_max_snapshots(Some("s1"), None, 123);
        assert_eq!(request_id, 7);

        let sent = transport.take_sent();
        assert_eq!(sent.len(), 1);

        let msg = &sent[0];
        assert_eq!(msg.r#type, "bundle.dump");
        assert_eq!(msg.session_id.as_deref(), Some("s1"));
        assert_eq!(msg.request_id, Some(request_id));

        let payload: DevtoolsBundleDumpV1 = serde_json::from_value(msg.payload.clone()).unwrap();
        assert_eq!(payload.label, None);
        assert_eq!(payload.max_snapshots, Some(123));
    }

    #[test]
    fn environment_source_get_sets_request_id_and_payload() {
        let transport = Arc::new(TestTransport::default());
        let client = ToolingDiagClient::new_for_test(transport.clone());
        let ops = DevtoolsOps::with_request_id_seed(client, 99);

        let request_id = ops.environment_sources_get(Some("s1"));
        assert_eq!(request_id, 99);

        let sent = transport.take_sent();
        assert_eq!(sent.len(), 1);
        let msg = &sent[0];
        assert_eq!(msg.r#type, "environment.sources.get");
        assert_eq!(msg.session_id.as_deref(), Some("s1"));
        assert_eq!(msg.request_id, Some(request_id));

        let payload: DevtoolsEnvironmentSourcesGetV1 =
            serde_json::from_value(msg.payload.clone()).unwrap();
        assert_eq!(payload.schema_version, 1);
    }

    #[test]
    fn wait_for_environment_source_get_ack_filters_session_and_request_id() {
        let transport = Arc::new(TestTransport::default());
        transport.push_recv(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "environment.sources.get_ack".to_string(),
            session_id: Some("other-session".to_string()),
            request_id: Some(7),
            payload: serde_json::json!({ "schema_version": 1 }),
        });
        transport.push_recv(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "environment.sources.get_ack".to_string(),
            session_id: Some("session-1".to_string()),
            request_id: Some(7),
            payload: serde_json::json!({
                "schema_version": 1,
                "sources": [{
                    "source_id": "host.monitor_topology",
                    "availability": "preflight_transport_session"
                }]
            }),
        });

        let client = ToolingDiagClient::new_for_test(transport);
        let ops = DevtoolsOps::with_request_id_seed(client, 7);
        let ack = wait_for_environment_sources_get_ack(&ops, "session-1", 7, 100, 1).expect("ack");
        assert_eq!(ack.schema_version, 1);
        assert_eq!(ack.sources.len(), 1);
        assert_eq!(ack.sources[0].source_id, "host.monitor_topology");
    }
}
