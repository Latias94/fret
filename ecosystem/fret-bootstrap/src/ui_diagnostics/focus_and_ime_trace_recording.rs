fn record_focus_trace(
    trace: &mut Vec<UiFocusTraceEntryV1>,
    app: &App,
    window: AppWindowId,
    element_runtime: Option<&ElementRuntime>,
    semantics_snapshot: Option<&fret_core::SemanticsSnapshot>,
    ui: Option<&UiTree<App>>,
    step_index: u32,
    expected_node_id: Option<u64>,
    expected_test_id: Option<&str>,
    note: &str,
) {
    let snapshot = element_runtime.and_then(|rt| rt.diagnostics_snapshot(window));
    let focused_element = snapshot
        .as_ref()
        .and_then(|s| s.focused_element)
        .map(|id| id.0);
    let focused_element_path = snapshot
        .as_ref()
        .and_then(|s| s.focused_element)
        .and_then(|id| element_runtime.and_then(|rt| rt.debug_path_for_element(window, id)));
    let focused_node_id = snapshot
        .as_ref()
        .and_then(|s| s.focused_element_node)
        .map(key_to_u64);

    let (focused_test_id, focused_role) =
        if let (Some(snapshot), Some(focused_node_id)) = (semantics_snapshot, focused_node_id) {
            let node = snapshot
                .nodes
                .iter()
                .find(|n| n.id.data().as_ffi() == focused_node_id);
            let test_id = node.and_then(|n| n.test_id.clone());
            let role = node.map(|n| semantics_role_label(n.role).to_string());
            (test_id, role)
        } else {
            (None, None)
        };

    let matches_expected = match (expected_node_id, expected_test_id) {
        (Some(expected_node_id), _) => focused_node_id.map(|id| id == expected_node_id),
        (None, Some(expected_test_id)) => {
            focused_test_id.as_deref().map(|id| id == expected_test_id)
        }
        _ => None,
    };

    let (
        modal_barrier_root,
        focus_barrier_root,
        pointer_occlusion,
        pointer_occlusion_layer_id,
        pointer_capture_active,
        pointer_capture_layer_id,
        pointer_capture_multiple_layers,
    ) = if let Some(ui) = ui {
        let arbitration = ui.input_arbitration_snapshot();
        (
            arbitration.modal_barrier_root.map(key_to_u64),
            arbitration.focus_barrier_root.map(key_to_u64),
            Some(pointer_occlusion_label(arbitration.pointer_occlusion)),
            arbitration
                .pointer_occlusion_layer
                .map(|l| l.data().as_ffi()),
            Some(arbitration.pointer_capture_active),
            arbitration.pointer_capture_layer.map(|l| l.data().as_ffi()),
            Some(arbitration.pointer_capture_multiple_layers),
        )
    } else {
        (None, None, None, None, None, None, None)
    };

    let reason_code = {
        if matches_expected == Some(true) {
            Some("focus.matches_expected".to_string())
        } else if let (Some(expected), Some(barrier), Some(snapshot)) =
            (expected_node_id, focus_barrier_root, semantics_snapshot)
        {
            let index = SemanticsIndex::new(snapshot);
            if !index.is_descendant_of_or_self(expected, barrier) {
                Some("focus.blocked_by_focus_barrier".to_string())
            } else {
                Some("focus.mismatch".to_string())
            }
        } else if let (Some(expected), Some(barrier), Some(snapshot)) =
            (expected_node_id, modal_barrier_root, semantics_snapshot)
        {
            let index = SemanticsIndex::new(snapshot);
            if !index.is_descendant_of_or_self(expected, barrier) {
                Some("focus.blocked_by_modal_barrier".to_string())
            } else {
                Some("focus.mismatch".to_string())
            }
        } else {
            Some("focus.mismatch".to_string())
        }
    };

    push_focus_trace(
        trace,
        UiFocusTraceEntryV1 {
            step_index,
            note: Some(note.to_string()),
            reason_code,
            text_input_snapshot: app
                .global::<fret_runtime::WindowTextInputSnapshotService>()
                .and_then(|svc| svc.snapshot(window).cloned())
                .map(|snapshot| UiTextInputSnapshotV1 {
                    focus_is_text_input: snapshot.focus_is_text_input,
                    is_composing: snapshot.is_composing,
                    text_len_utf16: snapshot.text_len_utf16,
                    selection_utf16: snapshot.selection_utf16,
                    marked_utf16: snapshot.marked_utf16,
                    ime_cursor_area: snapshot.ime_cursor_area.map(|r| UiRectV1 {
                        x_px: r.origin.x.0,
                        y_px: r.origin.y.0,
                        w_px: r.size.width.0,
                        h_px: r.size.height.0,
                    }),
                }),
            expected_node_id,
            expected_test_id: expected_test_id.map(|s| s.to_string()),
            modal_barrier_root,
            focus_barrier_root,
            pointer_occlusion,
            pointer_occlusion_layer_id,
            pointer_capture_active,
            pointer_capture_layer_id,
            pointer_capture_multiple_layers,
            focused_element,
            focused_element_path,
            focused_node_id,
            focused_test_id,
            focused_role,
            matches_expected,
        },
    );
}

fn record_web_ime_trace(
    trace: &mut Vec<UiWebImeTraceEntryV1>,
    app: &App,
    step_index: u32,
    note: &str,
) {
    let snapshot = app
        .global::<fret_core::input::WebImeBridgeDebugSnapshot>()
        .filter(|snapshot| **snapshot != fret_core::input::WebImeBridgeDebugSnapshot::default());
    let Some(snapshot) = snapshot else {
        return;
    };

    let last_preedit_len = snapshot
        .last_preedit_text
        .as_deref()
        .map(|s| s.len().min(u32::MAX as usize) as u32);
    let last_commit_len = snapshot
        .last_commit_text
        .as_deref()
        .map(|s| s.len().min(u32::MAX as usize) as u32);

    push_web_ime_trace(
        trace,
        UiWebImeTraceEntryV1 {
            step_index,
            note: Some(note.to_string()),
            enabled: snapshot.enabled,
            composing: snapshot.composing,
            suppress_next_input: snapshot.suppress_next_input,
            textarea_has_focus: snapshot.textarea_has_focus,
            active_element_tag: snapshot.active_element_tag.clone(),
            position_mode: snapshot.position_mode.clone(),
            mount_kind: snapshot.mount_kind.clone(),
            device_pixel_ratio: snapshot.device_pixel_ratio,
            textarea_selection_start_utf16: snapshot.textarea_selection_start_utf16,
            textarea_selection_end_utf16: snapshot.textarea_selection_end_utf16,
            last_cursor_area: snapshot.last_cursor_area.map(|r| UiRectV1 {
                x_px: r.origin.x.0,
                y_px: r.origin.y.0,
                w_px: r.size.width.0,
                h_px: r.size.height.0,
            }),
            last_cursor_anchor_px: snapshot.last_cursor_anchor_px,
            last_input_type: snapshot.last_input_type.clone(),
            last_preedit_len,
            last_preedit_cursor_utf16: snapshot.last_preedit_cursor_utf16,
            last_commit_len,
            beforeinput_seen: snapshot.beforeinput_seen,
            input_seen: snapshot.input_seen,
            suppressed_input_seen: snapshot.suppressed_input_seen,
            composition_start_seen: snapshot.composition_start_seen,
            composition_update_seen: snapshot.composition_update_seen,
            composition_end_seen: snapshot.composition_end_seen,
            cursor_area_set_seen: snapshot.cursor_area_set_seen,
        },
    );
}

fn record_ime_event_trace(
    trace: &mut Vec<UiImeEventTraceEntryV1>,
    step_index: u32,
    note: &str,
    event: &fret_core::input::ImeEvent,
) {
    let mut preedit_len: Option<u32> = None;
    let mut preedit_cursor: Option<(u32, u32)> = None;
    let mut commit_len: Option<u32> = None;
    let mut delete_surrounding: Option<(u32, u32)> = None;

    let kind: &'static str = match event {
        fret_core::input::ImeEvent::Enabled => "enabled",
        fret_core::input::ImeEvent::Disabled => "disabled",
        fret_core::input::ImeEvent::Commit(text) => {
            commit_len = Some(text.len().min(u32::MAX as usize) as u32);
            "commit"
        }
        fret_core::input::ImeEvent::Preedit { text, cursor } => {
            preedit_len = Some(text.len().min(u32::MAX as usize) as u32);
            preedit_cursor = cursor.map(|(a, b)| {
                (
                    a.min(u32::MAX as usize) as u32,
                    b.min(u32::MAX as usize) as u32,
                )
            });
            "preedit"
        }
        fret_core::input::ImeEvent::DeleteSurrounding {
            before_bytes,
            after_bytes,
        } => {
            delete_surrounding = Some((
                (*before_bytes).min(u32::MAX as usize) as u32,
                (*after_bytes).min(u32::MAX as usize) as u32,
            ));
            "delete_surrounding"
        }
    };

    push_ime_event_trace(
        trace,
        UiImeEventTraceEntryV1 {
            step_index,
            note: Some(note.to_string()),
            kind: kind.to_string(),
            preedit_len,
            preedit_cursor,
            commit_len,
            delete_surrounding,
        },
    );
}

fn hit_test_scope_roots_evidence(
    position: Point,
    ui: &mut UiTree<App>,
) -> (
    Option<NodeId>,
    Option<u64>,
    Option<u64>,
    Vec<UiHitTestScopeRootEvidenceV1>,
    fret_ui::tree::UiInputArbitrationSnapshot,
) {
    let snap = UiHitTestSnapshotV1::from_tree(position, ui);
    let arbitration = ui.input_arbitration_snapshot();
    let scope_roots = snap
        .scope_roots
        .into_iter()
        .map(|r| UiHitTestScopeRootEvidenceV1 {
            kind: r.kind,
            root: r.root,
            layer_id: r.layer_id,
            pointer_occlusion: r.pointer_occlusion,
            blocks_underlay_input: r.blocks_underlay_input,
            hit_testable: r.hit_testable,
        })
        .collect();
    (
        snap.hit
            .map(|id| NodeId::from(slotmap::KeyData::from_ffi(id))),
        snap.barrier_root,
        snap.focus_barrier_root,
        scope_roots,
        arbitration,
    )
}
