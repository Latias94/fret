use fret_diag_protocol::DiagScreenshotResultFileV1;

#[cfg(feature = "diagnostics-ws")]
fn build_semantics_node_get_ack_v1(
    snapshot: Option<&fret_core::SemanticsSnapshot>,
    window_ffi: u64,
    node_id: u64,
    redact_text: bool,
    max_string_bytes: usize,
) -> UiSemanticsNodeGetAckV1 {
    let captured_unix_ms = Some(unix_ms_now());

    let Some(snapshot) = snapshot else {
        return UiSemanticsNodeGetAckV1 {
            schema_version: 1,
            status: "no_semantics".to_string(),
            reason: Some("no_semantics_snapshot".to_string()),
            window: window_ffi,
            node_id,
            semantics_fingerprint: None,
            node: None,
            children: Vec::new(),
            captured_unix_ms,
        };
    };

    let semantics_fingerprint = Some(semantics_fingerprint_v1(
        snapshot,
        redact_text,
        max_string_bytes,
    ));
    let want = NodeId::from(KeyData::from_ffi(node_id));

    let Some(node) = snapshot.nodes.iter().find(|n| n.id == want) else {
        return UiSemanticsNodeGetAckV1 {
            schema_version: 1,
            status: "not_found".to_string(),
            reason: None,
            window: window_ffi,
            node_id,
            semantics_fingerprint,
            node: None,
            children: Vec::new(),
            captured_unix_ms,
        };
    };

    let exported = UiSemanticsNodeV1::from_node(node, redact_text, max_string_bytes);
    let node = serde_json::to_value(exported).ok();
    let children = snapshot
        .nodes
        .iter()
        .filter(|n| n.parent == Some(want))
        .map(|n| key_to_u64(n.id))
        .collect::<Vec<_>>();

    UiSemanticsNodeGetAckV1 {
        schema_version: 1,
        status: "ok".to_string(),
        reason: None,
        window: window_ffi,
        node_id,
        semantics_fingerprint,
        node,
        children,
        captured_unix_ms,
    }
}

fn screenshot_request_completed(path: &Path, request_id: &str, window_ffi: u64) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let Ok(root) = serde_json::from_slice::<DiagScreenshotResultFileV1>(&bytes) else {
        return false;
    };
    root.completed.iter().any(|entry| {
        entry.request_id.as_deref() == Some(request_id) && entry.window == window_ffi
    })
}

#[cfg(feature = "diagnostics-ws")]
fn read_screenshot_result_entry(
    path: &Path,
    request_id: &str,
    window_ffi: u64,
) -> Option<serde_json::Value> {
    let bytes = std::fs::read(path).ok()?;
    let root = serde_json::from_slice::<DiagScreenshotResultFileV1>(&bytes).ok()?;
    let found = root.completed.iter().find(|entry| {
        entry.request_id.as_deref() == Some(request_id) && entry.window == window_ffi
    })?;
    serde_json::to_value(found).ok()
}
