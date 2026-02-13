use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use fret_diag_protocol::{
    DevtoolsAppExitRequestV1, DevtoolsBundleDumpV1, DevtoolsScreenshotRequestV1,
    DiagTransportMessageV1, UiInspectConfigV1, UiSemanticsNodeGetV1,
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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::transport::{DiagTransport, DiagTransportKind};

    #[derive(Default)]
    struct TestTransport {
        sent: Mutex<Vec<DiagTransportMessageV1>>,
        default_session_id: Mutex<Option<String>>,
    }

    impl TestTransport {
        fn take_sent(&self) -> Vec<DiagTransportMessageV1> {
            std::mem::take(&mut self.sent.lock().unwrap())
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
            None
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
}
