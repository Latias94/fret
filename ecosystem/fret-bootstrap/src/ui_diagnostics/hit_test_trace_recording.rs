fn record_hit_test_trace_for_selector(
    trace: &mut Vec<UiHitTestTraceEntryV1>,
    ui: &mut UiTree<App>,
    element_runtime: Option<&ElementRuntime>,
    window: AppWindowId,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    selector: &UiSelectorV1,
    step_index: u32,
    position: Point,
    intended: Option<&fret_core::SemanticsNode>,
    note: Option<&str>,
    max_debug_string_bytes: usize,
) {
    let entry = build_hit_test_trace_entry_for_selector(
        ui,
        element_runtime,
        window,
        semantics_snapshot,
        selector,
        step_index,
        position,
        intended,
        note,
        max_debug_string_bytes,
    );
    push_hit_test_trace(trace, entry);
}

fn build_hit_test_trace_entry_for_selector(
    ui: &mut UiTree<App>,
    element_runtime: Option<&ElementRuntime>,
    window: AppWindowId,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    selector: &UiSelectorV1,
    step_index: u32,
    position: Point,
    intended: Option<&fret_core::SemanticsNode>,
    note: Option<&str>,
    max_debug_string_bytes: usize,
) -> UiHitTestTraceEntryV1 {
    const MAX_HIT_NODE_PATH: usize = 64;

    let (hit_node, barrier_root, focus_barrier_root, scope_roots, arbitration) =
        hit_test_scope_roots_evidence(position, ui);

    let hit_semantics =
        semantics_snapshot.and_then(|snapshot| pick_semantics_node_at(snapshot, ui, position));
    let hit_semantics_node_id = hit_semantics.map(|n| n.id.data().as_ffi());
    let hit_semantics_test_id = hit_semantics.and_then(|n| n.test_id.clone());

    let intended_node_id = intended.map(|n| n.id.data().as_ffi());
    let intended_test_id = intended.and_then(|n| n.test_id.clone());
    let intended_bounds = intended.map(|n| UiRectV1 {
        x_px: n.bounds.origin.x.0,
        y_px: n.bounds.origin.y.0,
        w_px: n.bounds.size.width.0,
        h_px: n.bounds.size.height.0,
    });

    let hit_node_id = hit_node.map(|id| id.data().as_ffi());
    let hit_node_path: Vec<u64> = hit_node
        .map(|id| {
            ui.debug_node_path(id)
                .into_iter()
                .rev()
                .take(MAX_HIT_NODE_PATH)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|n| n.data().as_ffi())
                .collect()
        })
        .unwrap_or_default();

    let includes_intended = intended.map(|target| {
        if let Some(hit_id) = hit_semantics_node_id {
            if hit_id == target.id.data().as_ffi() {
                return true;
            }
        }
        if let (Some(want), Some(got)) =
            (target.test_id.as_deref(), hit_semantics_test_id.as_deref())
        {
            return want == got;
        }
        false
    });

    let hit_path_contains_intended = intended_node_id.map(|id| hit_node_path.contains(&id));

    let pointer_occlusion = pointer_occlusion_label(arbitration.pointer_occlusion).to_string();
    let pointer_occlusion_layer_id = arbitration
        .pointer_occlusion_layer
        .map(|id| id.data().as_ffi());
    let pointer_capture_layer_id = arbitration
        .pointer_capture_layer
        .map(|id| id.data().as_ffi());

    let pointer_occlusion_root = pointer_occlusion_layer_id.and_then(|layer_id| {
        scope_roots
            .iter()
            .find(|r| r.kind == "layer_root" && r.layer_id == Some(layer_id))
            .map(|r| r.root)
    });
    let (
        pointer_occlusion_node_id,
        pointer_occlusion_test_id,
        pointer_occlusion_role,
        pointer_occlusion_bounds,
    ) = pointer_occlusion_root
        .map(|root| {
            let node = NodeId::from(KeyData::from_ffi(root));
            let mut test_id: Option<String> = None;
            let mut role: Option<String> = None;
            let mut bounds: Option<UiRectV1> = None;
            if let Some(snapshot) = semantics_snapshot {
                if let Some(n) = snapshot.nodes.iter().find(|n| n.id == node) {
                    test_id = n.test_id.clone();
                    role = Some(semantics_role_label(n.role).to_string());
                    bounds = Some(UiRectV1 {
                        x_px: n.bounds.origin.x.0,
                        y_px: n.bounds.origin.y.0,
                        w_px: n.bounds.size.width.0,
                        h_px: n.bounds.size.height.0,
                    });
                }
            }
            if bounds.is_none() {
                bounds = ui.debug_node_bounds(node).map(|r| UiRectV1 {
                    x_px: r.origin.x.0,
                    y_px: r.origin.y.0,
                    w_px: r.size.width.0,
                    h_px: r.size.height.0,
                });
            }
            (Some(root), test_id, role, bounds)
        })
        .unwrap_or((None, None, None, None));

    let is_ok = includes_intended == Some(true) || hit_path_contains_intended == Some(true);
    let (blocking_reason, blocking_root, blocking_layer_id) = if is_ok {
        (None, None, None)
    } else if barrier_root.is_some() {
        (Some("modal_barrier"), barrier_root, None)
    } else if focus_barrier_root.is_some() {
        let layer_id = scope_roots
            .iter()
            .find(|r| r.kind == "focus_barrier_root")
            .and_then(|r| r.layer_id);
        (Some("focus_barrier"), focus_barrier_root, layer_id)
    } else if arbitration.pointer_capture_active {
        let blocking_root = pointer_capture_layer_id.and_then(|layer_id| {
            scope_roots
                .iter()
                .find(|r| r.kind == "layer_root" && r.layer_id == Some(layer_id))
                .map(|r| r.root)
        });
        (
            Some("pointer_capture"),
            blocking_root,
            pointer_capture_layer_id,
        )
    } else if pointer_occlusion != "none" {
        let blocking_root = pointer_occlusion_layer_id.and_then(|layer_id| {
            scope_roots
                .iter()
                .find(|r| r.kind == "layer_root" && r.layer_id == Some(layer_id))
                .map(|r| r.root)
        });
        (
            Some("pointer_occlusion"),
            blocking_root,
            pointer_occlusion_layer_id,
        )
    } else if hit_node_id.is_none() {
        (Some("no_hit"), None, None)
    } else {
        (Some("miss"), None, None)
    };

    let (
        pointer_capture_node_id,
        pointer_capture_test_id,
        pointer_capture_role,
        pointer_capture_bounds,
        pointer_capture_element,
        pointer_capture_element_path,
    ) = ui
        .any_captured_node()
        .map(|captured| {
            let captured_id = captured.data().as_ffi();
            let mut test_id: Option<String> = None;
            let mut role: Option<String> = None;
            let mut bounds: Option<UiRectV1> = None;
            let mut element_id: Option<u64> = None;
            let mut element_path: Option<String> = None;
            if let Some(el) = ui.debug_node_element(captured) {
                element_id = Some(el.0);
                element_path = element_runtime
                    .and_then(|rt| rt.debug_path_for_element(window, el))
                    .map(|mut s| {
                        truncate_string_bytes(&mut s, max_debug_string_bytes);
                        s
                    });
            }
            if let Some(snapshot) = semantics_snapshot {
                if let Some(n) = snapshot.nodes.iter().find(|n| n.id == captured) {
                    test_id = n.test_id.clone();
                    role = Some(semantics_role_label(n.role).to_string());
                    bounds = Some(UiRectV1 {
                        x_px: n.bounds.origin.x.0,
                        y_px: n.bounds.origin.y.0,
                        w_px: n.bounds.size.width.0,
                        h_px: n.bounds.size.height.0,
                    });
                }
            }
            (
                Some(captured_id),
                test_id,
                role,
                bounds,
                element_id,
                element_path,
            )
        })
        .unwrap_or((None, None, None, None, None, None));
    let blocking_layer_hint = blocking_layer_id.and_then(|layer_id| {
        scope_roots
            .iter()
            .find(|r| r.kind == "layer_root" && r.layer_id == Some(layer_id))
            .map(|r| {
                let mut parts: Vec<String> = Vec::new();
                parts.push(format!("layer_root={}", r.root));
                if let Some(v) = r.blocks_underlay_input {
                    parts.push(format!("blocks_underlay_input={v}"));
                }
                if let Some(v) = r.hit_testable {
                    parts.push(format!("hit_testable={v}"));
                }
                if let Some(v) = r.pointer_occlusion.as_deref() {
                    parts.push(format!("pointer_occlusion={v}"));
                }
                parts.join(" ")
            })
    });

    let routing_explain = if is_ok {
        None
    } else {
        let intended = intended_test_id
            .as_deref()
            .map(|t| format!("test_id={t}"))
            .or_else(|| intended_node_id.map(|id| format!("node_id={id}")))
            .unwrap_or_else(|| "<none>".to_string());
        let hit = hit_semantics_test_id
            .as_deref()
            .map(|t| format!("test_id={t}"))
            .or_else(|| hit_node_id.map(|id| format!("node_id={id}")))
            .unwrap_or_else(|| "<none>".to_string());

        match blocking_reason {
            Some("modal_barrier") => Some(format!(
                "blocked by modal barrier (barrier_root={}) intended={intended} hit={hit}",
                barrier_root.unwrap_or(0)
            )),
            Some("focus_barrier") => Some(format!(
                "blocked by focus barrier (focus_barrier_root={}) intended={intended} hit={hit}",
                focus_barrier_root.unwrap_or(0)
            )),
            Some("pointer_capture") => Some(format!(
                "blocked by pointer capture (layer_id={}, captured_node_id={}) {}{} intended={intended} hit={hit}",
                blocking_layer_id.unwrap_or(0),
                pointer_capture_node_id.unwrap_or(0),
                blocking_layer_hint.as_deref().unwrap_or(""),
                pointer_capture_element_path
                    .as_deref()
                    .map(|p| format!(" element_path={p}"))
                    .unwrap_or_default(),
            )),
            Some("pointer_occlusion") => Some(format!(
                "blocked by pointer occlusion ({pointer_occlusion}) (layer_id={}, root={}) {} intended={intended} hit={hit}",
                blocking_layer_id.unwrap_or(0),
                blocking_root.unwrap_or(0),
                blocking_layer_hint.as_deref().unwrap_or(""),
            )),
            Some("no_hit") => Some(format!("hit-test returned no node intended={intended}")),
            Some("miss") => Some(format!("hit-test missed intended={intended} hit={hit}")),
            _ => None,
        }
    };

    UiHitTestTraceEntryV1 {
        step_index,
        selector: selector.clone(),
        position: UiPointV1 {
            x_px: position.x.0,
            y_px: position.y.0,
        },
        intended_node_id,
        intended_test_id,
        intended_bounds,
        hit_node_id,
        hit_node_path,
        hit_semantics_node_id,
        hit_semantics_test_id,
        includes_intended,
        hit_path_contains_intended,
        blocking_reason: blocking_reason.map(|s| s.to_string()),
        blocking_root,
        blocking_layer_id,
        routing_explain,
        barrier_root,
        focus_barrier_root,
        pointer_occlusion: Some(pointer_occlusion),
        pointer_occlusion_layer_id,
        pointer_occlusion_node_id,
        pointer_occlusion_test_id,
        pointer_occlusion_role,
        pointer_occlusion_bounds,
        pointer_capture_active: Some(arbitration.pointer_capture_active),
        pointer_capture_layer_id,
        pointer_capture_multiple_layers: Some(arbitration.pointer_capture_multiple_layers),
        pointer_capture_node_id,
        pointer_capture_test_id,
        pointer_capture_role,
        pointer_capture_bounds,
        pointer_capture_element,
        pointer_capture_element_path,
        scope_roots,
        note: note.map(|s| s.to_string()),
    }
}
