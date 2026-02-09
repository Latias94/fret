use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use fret_diag_protocol::{
    DevtoolsBundleDumpV1, DevtoolsScreenshotRequestV1, DiagTransportMessageV1, UiInspectConfigV1,
    UiSemanticsNodeGetV1,
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

    pub fn bundle_dump(&self, session_id: Option<&str>, label: Option<&str>) {
        self.send(DiagTransportMessageV1 {
            schema_version: 1,
            r#type: "bundle.dump".to_string(),
            session_id: session_id.map(|s| s.to_string()),
            request_id: None,
            payload: serde_json::to_value(DevtoolsBundleDumpV1 {
                schema_version: 1,
                label: label.map(|s| s.to_string()),
            })
            .unwrap_or(serde_json::Value::Null),
        });
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
}
