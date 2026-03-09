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
    root.completed
        .iter()
        .any(|entry| entry.request_id.as_deref() == Some(request_id) && entry.window == window_ffi)
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

#[cfg(feature = "diagnostics-ws")]
fn build_hit_test_explain_ack_v1(
    ui: Option<&mut UiTree<App>>,
    redact_text: bool,
    max_string_bytes: usize,
    window_ffi: u64,
    target: UiSelectorV1,
) -> UiHitTestExplainAckV1 {
    let captured_unix_ms = Some(unix_ms_now());
    let Some(ui) = ui else {
        return UiHitTestExplainAckV1 {
            schema_version: 1,
            status: "no_ui_tree".to_string(),
            reason: Some("no_ui_tree".to_string()),
            window: window_ffi,
            target,
            semantics_fingerprint: None,
            hittable: None,
            hit_test: None,
            captured_unix_ms,
        };
    };

    let raw = ui.semantics_snapshot_arc();
    let Some(snapshot) = raw else {
        return UiHitTestExplainAckV1 {
            schema_version: 1,
            status: "no_semantics".to_string(),
            reason: Some("no_semantics_snapshot".to_string()),
            window: window_ffi,
            target,
            semantics_fingerprint: None,
            hittable: None,
            hit_test: None,
            captured_unix_ms,
        };
    };

    let snapshot = snapshot.as_ref();
    let window = AppWindowId::from(KeyData::from_ffi(window_ffi));
    let semantics_fingerprint = Some(semantics_fingerprint_v1(
        snapshot,
        redact_text,
        max_string_bytes,
    ));
    let mut trace = Vec::new();
    let node = select_semantics_node_with_trace(
        snapshot,
        window,
        None,
        &target,
        None,
        0,
        redact_text,
        &mut trace,
    );
    let Some(node) = node else {
        return UiHitTestExplainAckV1 {
            schema_version: 1,
            status: "not_found".to_string(),
            reason: Some("no_semantics_match".to_string()),
            window: window_ffi,
            target,
            semantics_fingerprint,
            hittable: None,
            hit_test: None,
            captured_unix_ms,
        };
    };

    let center = Point::new(
        Px(node.bounds.origin.x.0 + node.bounds.size.width.0 * 0.5),
        Px(node.bounds.origin.y.0 + node.bounds.size.height.0 * 0.5),
    );
    let hit_test = build_hit_test_trace_entry_for_selector(
        ui,
        None,
        window,
        Some(snapshot),
        &target,
        0,
        center,
        Some(node),
        Some("devtools.hit_test_explain"),
        max_string_bytes,
    );
    let hittable = Some(
        hit_test.includes_intended == Some(true)
            || hit_test.hit_path_contains_intended == Some(true),
    );
    let status = if hittable == Some(true) {
        "ok".to_string()
    } else {
        "blocked".to_string()
    };
    let reason = hit_test.blocking_reason.clone();

    UiHitTestExplainAckV1 {
        schema_version: 1,
        status,
        reason,
        window: window_ffi,
        target,
        semantics_fingerprint,
        hittable,
        hit_test: Some(hit_test),
        captured_unix_ms,
    }
}
