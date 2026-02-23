#[derive(Clone, Copy, Debug)]
struct DockDragRuntimeState {
    dragging: bool,
    source_window: AppWindowId,
    current_window: AppWindowId,
    moving_window: Option<AppWindowId>,
    window_under_moving_window: Option<AppWindowId>,
    window_under_moving_window_source: fret_runtime::WindowUnderCursorSource,
    transparent_payload_applied: bool,
    transparent_payload_mouse_passthrough_applied: bool,
    window_under_cursor_source: fret_runtime::WindowUnderCursorSource,
}

fn dock_drag_pointer_id_best_effort(
    app: &fret_app::App,
    known_windows: &[AppWindowId],
) -> Option<PointerId> {
    if let Some(pointer_id) = app.find_drag_pointer_id(|d| {
        (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && d.dragging
    }) {
        return Some(pointer_id);
    }

    let store = app.global::<fret_runtime::WindowInteractionDiagnosticsStore>()?;
    for window in known_windows.iter().rev().copied() {
        let docking = store.docking_latest_for_window(window)?;
        if let Some(drag) = docking.dock_drag
            && drag.dragging
        {
            // `docking_latest_for_window` is intentionally stable across frames, which makes it
            // useful for debugging but also means it can be stale. Only treat it as authoritative
            // when the drag session is still present in the live `App` drag registry.
            if app.drag(drag.pointer_id).is_some() {
                return Some(drag.pointer_id);
            }
        }
    }

    None
}

fn dock_drag_runtime_state(
    app: &fret_app::App,
    known_windows: &[AppWindowId],
) -> Option<DockDragRuntimeState> {
    if let Some(pointer_id) = dock_drag_pointer_id_best_effort(app, known_windows)
        && let Some(drag) = app.drag(pointer_id)
    {
        return Some(DockDragRuntimeState {
            dragging: drag.dragging,
            source_window: drag.source_window,
            current_window: drag.current_window,
            moving_window: drag.moving_window,
            window_under_moving_window: drag.window_under_moving_window,
            window_under_moving_window_source: drag.window_under_moving_window_source,
            transparent_payload_applied: drag.transparent_payload_applied,
            transparent_payload_mouse_passthrough_applied: drag
                .transparent_payload_mouse_passthrough_applied,
            window_under_cursor_source: drag.window_under_cursor_source,
        });
    }

    // If the drag session cannot be found in `App`, treat it as inactive. The per-window docking
    // diagnostics store may retain stale "latest" snapshots across frames (by design), which is
    // useful for debugging but unsuitable as a source of truth for scripted gates.
    None
}

fn dock_drag_window_under_cursor_source_is(
    have: fret_runtime::WindowUnderCursorSource,
    want: &str,
) -> bool {
    use fret_runtime::WindowUnderCursorSource as Src;
    match want {
        "platform" => matches!(have, Src::PlatformWin32 | Src::PlatformMacos),
        "platform_win32" => matches!(have, Src::PlatformWin32),
        "platform_macos" => matches!(have, Src::PlatformMacos),
        "latched" => matches!(have, Src::Latched),
        "heuristic" => matches!(have, Src::HeuristicZOrder | Src::HeuristicRects),
        "heuristic_z_order" => matches!(have, Src::HeuristicZOrder),
        "heuristic_rects" => matches!(have, Src::HeuristicRects),
        "unknown" => matches!(have, Src::Unknown),
        _ => false,
    }
}

fn eval_predicate_without_semantics(
    window: AppWindowId,
    known_windows: &[AppWindowId],
    platform_caps: Option<&fret_runtime::PlatformCapabilities>,
    docking: Option<&fret_runtime::DockingInteractionDiagnostics>,
    dock_drag_runtime: Option<&DockDragRuntimeState>,
    pred: &UiPredicateV1,
) -> Option<bool> {
    match pred {
        UiPredicateV1::KnownWindowCountGe { n } => Some((known_windows.len() as u32) >= *n),
        UiPredicateV1::KnownWindowCountIs { n } => Some((known_windows.len() as u32) == *n),
        UiPredicateV1::PlatformUiWindowHoverDetectionIs { quality } => Some(
            platform_caps.is_some_and(|c| c.ui.window_hover_detection.as_str() == quality.as_str()),
        ),
        UiPredicateV1::DockDragCurrentWindowIs {
            window: target_window,
        } => {
            let target_window =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)?;
            Some(
                dock_drag_runtime
                    .is_some_and(|drag| drag.dragging && drag.current_window == target_window),
            )
        }
        UiPredicateV1::DockDragMovingWindowIs {
            window: target_window,
        } => {
            let target_window =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)?;
            Some(
                dock_drag_runtime
                    .is_some_and(|drag| drag.dragging && drag.moving_window == Some(target_window)),
            )
        }
        UiPredicateV1::DockDragWindowUnderMovingWindowIs {
            window: target_window,
        } => {
            let target_window =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)?;
            Some(dock_drag_runtime.is_some_and(|drag| {
                drag.dragging && drag.window_under_moving_window == Some(target_window)
            }))
        }
        UiPredicateV1::DockDragActiveIs { active } => {
            Some(dock_drag_runtime.is_some_and(|drag| drag.dragging) == *active)
        }
        UiPredicateV1::DockDragTransparentPayloadAppliedIs { applied } => Some(
            dock_drag_runtime
                .is_some_and(|drag| drag.dragging && drag.transparent_payload_applied == *applied)
                || (!*applied && dock_drag_runtime.is_none()),
        ),
        UiPredicateV1::DockDragTransparentPayloadMousePassthroughAppliedIs { applied } => Some(
            dock_drag_runtime.is_some_and(|drag| {
                drag.dragging && drag.transparent_payload_mouse_passthrough_applied == *applied
            }) || (!*applied && dock_drag_runtime.is_none()),
        ),
        UiPredicateV1::DockDragWindowUnderCursorSourceIs { source } => {
            Some(dock_drag_runtime.is_some_and(|drag| {
                dock_drag_window_under_cursor_source_is(drag.window_under_cursor_source, source)
            }))
        }
        UiPredicateV1::DockDragWindowUnderMovingWindowSourceIs { source } => {
            Some(dock_drag_runtime.is_some_and(|drag| {
                dock_drag_window_under_cursor_source_is(
                    drag.window_under_moving_window_source,
                    source,
                )
            }))
        }
        UiPredicateV1::DockFloatingDragActiveIs { active } => {
            Some(match docking.and_then(|d| d.floating_drag) {
                Some(drag) => drag.activated == *active,
                None => !*active,
            })
        }
        UiPredicateV1::DockDropPreviewKindIs { preview_kind } => {
            let preview = docking
                .and_then(|d| d.dock_drop_resolve.as_ref())
                .and_then(|d| d.preview.as_ref())?;
            let have = match preview.kind {
                fret_runtime::DockDropPreviewKindDiagnostics::WrapBinary => "wrap_binary",
                fret_runtime::DockDropPreviewKindDiagnostics::InsertIntoSplit { .. } => {
                    "insert_into_split"
                }
            };
            Some(have == preview_kind.as_str())
        }
        UiPredicateV1::DockDropResolveSourceIs { source } => {
            let resolve = docking.and_then(|d| d.dock_drop_resolve.as_ref())?;
            let have = match resolve.source {
                fret_runtime::DockDropResolveSource::InvertDocking => "invert_docking",
                fret_runtime::DockDropResolveSource::OutsideWindow => "outside_window",
                fret_runtime::DockDropResolveSource::FloatZone => "float_zone",
                fret_runtime::DockDropResolveSource::EmptyDockSpace => "empty_dock_space",
                fret_runtime::DockDropResolveSource::LayoutBoundsMiss => "layout_bounds_miss",
                fret_runtime::DockDropResolveSource::LatchedPreviousHover => {
                    "latched_previous_hover"
                }
                fret_runtime::DockDropResolveSource::TabBar => "tab_bar",
                fret_runtime::DockDropResolveSource::FloatingTitleBar => "floating_title_bar",
                fret_runtime::DockDropResolveSource::OuterHintRect => "outer_hint_rect",
                fret_runtime::DockDropResolveSource::InnerHintRect => "inner_hint_rect",
                fret_runtime::DockDropResolveSource::None => "none",
            };
            Some(have == source.as_str())
        }
        UiPredicateV1::DockDropResolvedIsSome { some } => Some(
            docking
                .and_then(|d| d.dock_drop_resolve.as_ref())
                .is_some_and(|d| d.resolved.is_some() == *some),
        ),
        UiPredicateV1::DockGraphCanonicalIs { canonical } => Some(
            docking
                .and_then(|d| d.dock_graph_stats)
                .is_some_and(|s| s.canonical_ok == *canonical),
        ),
        UiPredicateV1::DockGraphHasNestedSameAxisSplitsIs { has_nested } => Some(
            docking
                .and_then(|d| d.dock_graph_stats)
                .is_some_and(|s| s.has_nested_same_axis_splits == *has_nested),
        ),
        UiPredicateV1::DockGraphNodeCountLe { max } => Some(
            docking
                .and_then(|d| d.dock_graph_stats)
                .is_some_and(|s| s.node_count <= *max),
        ),
        UiPredicateV1::DockGraphMaxSplitDepthLe { max } => Some(
            docking
                .and_then(|d| d.dock_graph_stats)
                .is_some_and(|s| s.max_split_depth <= *max),
        ),
        UiPredicateV1::DockGraphSignatureIs { signature } => Some(
            docking
                .and_then(|d| d.dock_graph_signature.as_ref())
                .is_some_and(|s| s.signature == *signature),
        ),
        UiPredicateV1::DockGraphSignatureContains { needle } => Some(
            docking
                .and_then(|d| d.dock_graph_signature.as_ref())
                .is_some_and(|s| s.signature.contains(needle)),
        ),
        UiPredicateV1::DockGraphSignatureFingerprint64Is { fingerprint64 } => Some(
            docking
                .and_then(|d| d.dock_graph_signature.as_ref())
                .is_some_and(|s| s.fingerprint64 == *fingerprint64),
        ),
        _ => None,
    }
}

fn eval_predicate(
    snapshot: &fret_core::SemanticsSnapshot,
    window_bounds: Rect,
    window: AppWindowId,
    input_ctx: Option<&fret_runtime::InputContext>,
    element_runtime: Option<&ElementRuntime>,
    text_input_snapshot: Option<&fret_runtime::WindowTextInputSnapshot>,
    render_text: Option<fret_core::RendererTextPerfSnapshot>,
    render_text_font_trace: Option<&fret_core::RendererTextFontTraceSnapshot>,
    known_windows: &[AppWindowId],
    platform_caps: Option<&fret_runtime::PlatformCapabilities>,
    docking: Option<&fret_runtime::DockingInteractionDiagnostics>,
    dock_drag_runtime: Option<&DockDragRuntimeState>,
    text_font_stack_key_stable_frames: u32,
    font_catalog_populated: bool,
    system_font_rescan_idle: bool,
    pred: &UiPredicateV1,
) -> bool {
    match pred {
        UiPredicateV1::Exists { target } => {
            select_semantics_node(snapshot, window, element_runtime, target).is_some()
        }
        UiPredicateV1::NotExists { target } => {
            select_semantics_node(snapshot, window, element_runtime, target).is_none()
        }
        UiPredicateV1::FocusIs { target } => {
            let Some(focus) = snapshot.focus else {
                return false;
            };
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.id == focus
        }
        UiPredicateV1::RoleIs { target, role } => {
            let Some(want) = parse_semantics_role(role) else {
                return false;
            };
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.role == want
        }
        UiPredicateV1::CheckedIs { target, checked } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.flags.checked == Some(*checked)
        }
        UiPredicateV1::SelectedIs { target, selected } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.flags.selected == *selected
        }
        UiPredicateV1::TextCompositionIs { target, composing } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.text_composition.is_some() == *composing
        }
        UiPredicateV1::ImeCursorAreaIsSome { is_some } => {
            text_input_snapshot
                .and_then(|snapshot| snapshot.ime_cursor_area)
                .is_some()
                == *is_some
        }
        UiPredicateV1::ImeCursorAreaWithinWindow {
            padding_px,
            padding_insets_px,
            eps_px,
        } => {
            let Some(area) = text_input_snapshot.and_then(|snapshot| snapshot.ime_cursor_area)
            else {
                return false;
            };

            let pad = padding_px.max(0.0);
            let pad_insets = padding_insets_px.unwrap_or_else(|| UiPaddingInsetsV1::uniform(0.0));
            let eps = eps_px.max(0.0);

            let window_left = window_bounds.origin.x.0 + pad + pad_insets.left_px.max(0.0);
            let window_top = window_bounds.origin.y.0 + pad + pad_insets.top_px.max(0.0);
            let window_right = window_bounds.origin.x.0 + window_bounds.size.width.0
                - pad
                - pad_insets.right_px.max(0.0);
            let window_bottom = window_bounds.origin.y.0 + window_bounds.size.height.0
                - pad
                - pad_insets.bottom_px.max(0.0);

            let area_left = area.origin.x.0;
            let area_top = area.origin.y.0;
            let area_right = area.origin.x.0 + area.size.width.0.max(0.0);
            let area_bottom = area.origin.y.0 + area.size.height.0.max(0.0);

            area_left >= window_left - eps
                && area_top >= window_top - eps
                && area_right <= window_right + eps
                && area_bottom <= window_bottom + eps
        }
        UiPredicateV1::ImeCursorAreaMinSize {
            min_w_px,
            min_h_px,
            eps_px,
        } => {
            let Some(area) = text_input_snapshot.and_then(|snapshot| snapshot.ime_cursor_area)
            else {
                return false;
            };

            let eps = eps_px.max(0.0);
            let min_w = min_w_px.max(0.0);
            let min_h = min_h_px.max(0.0);

            area.size.width.0.max(0.0) + eps >= min_w && area.size.height.0.max(0.0) + eps >= min_h
        }
        UiPredicateV1::CheckedIsNone { target } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            node.flags.checked.is_none()
        }
        UiPredicateV1::ActiveItemIs { container, item } => {
            let Some(item_node) = select_semantics_node(snapshot, window, element_runtime, item)
            else {
                return false;
            };

            if snapshot.focus == Some(item_node.id) {
                return true;
            }

            let Some(container_node) =
                select_semantics_node(snapshot, window, element_runtime, container)
            else {
                return false;
            };

            container_node.active_descendant == Some(item_node.id)
        }
        UiPredicateV1::ActiveItemIsNone { container } => {
            let Some(container_node) =
                select_semantics_node(snapshot, window, element_runtime, container)
            else {
                return false;
            };

            if container_node.active_descendant.is_some() {
                return false;
            }

            let Some(focus_id) = snapshot.focus else {
                return true;
            };
            let Some(focus_node) = snapshot.nodes.iter().find(|n| n.id == focus_id) else {
                return true;
            };

            focus_node.role != SemanticsRole::ListBoxOption
        }
        UiPredicateV1::BarrierRoots {
            barrier_root,
            focus_barrier_root,
            require_equal,
        } => {
            let barrier = snapshot.barrier_root.map(|n| n.data().as_ffi());
            let focus_barrier = snapshot.focus_barrier_root.map(|n| n.data().as_ffi());

            let matches_root_state = |state: UiOptionalRootStateV1, value: Option<u64>| match state
            {
                UiOptionalRootStateV1::Any => true,
                UiOptionalRootStateV1::None => value.is_none(),
                UiOptionalRootStateV1::Some => value.is_some(),
            };

            if !matches_root_state(*barrier_root, barrier) {
                return false;
            }
            if !matches_root_state(*focus_barrier_root, focus_barrier) {
                return false;
            }

            match require_equal {
                None => true,
                Some(true) => barrier == focus_barrier,
                Some(false) => barrier != focus_barrier,
            }
        }
        UiPredicateV1::RenderTextMissingGlyphsIs { missing_glyphs } => {
            render_text.is_some_and(|snapshot| snapshot.frame_missing_glyphs == *missing_glyphs)
        }
        UiPredicateV1::RenderTextFontTraceCapturedWhenMissingGlyphs => {
            let Some(perf) = render_text else {
                return false;
            };
            if perf.frame_missing_glyphs == 0 {
                return true;
            }

            let Some(trace) = render_text_font_trace else {
                return false;
            };
            trace
                .entries
                .iter()
                .any(|e| e.missing_glyphs > 0 && !e.families.is_empty())
        }
        UiPredicateV1::TextFontStackKeyStable { stable_frames } => {
            text_font_stack_key_stable_frames >= *stable_frames
        }
        UiPredicateV1::FontCatalogPopulated => font_catalog_populated,
        UiPredicateV1::SystemFontRescanIdle => system_font_rescan_idle,
        UiPredicateV1::RunnerAccessibilityActivated => false,
        UiPredicateV1::VisibleInWindow { target } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            rects_intersect(node.bounds, window_bounds)
        }
        UiPredicateV1::BoundsWithinWindow {
            target,
            padding_px,
            padding_insets_px,
            eps_px,
        } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };
            let bounds = node.bounds;
            let pad = padding_px.max(0.0);
            let pad_insets = padding_insets_px.unwrap_or_else(|| UiPaddingInsetsV1::uniform(0.0));
            let eps = eps_px.max(0.0);

            let window_left = window_bounds.origin.x.0 + pad + pad_insets.left_px.max(0.0);
            let window_top = window_bounds.origin.y.0 + pad + pad_insets.top_px.max(0.0);
            let window_right = window_bounds.origin.x.0 + window_bounds.size.width.0
                - pad
                - pad_insets.right_px.max(0.0);
            let window_bottom = window_bounds.origin.y.0 + window_bounds.size.height.0
                - pad
                - pad_insets.bottom_px.max(0.0);

            let node_left = bounds.origin.x.0;
            let node_top = bounds.origin.y.0;
            let node_right = bounds.origin.x.0 + bounds.size.width.0;
            let node_bottom = bounds.origin.y.0 + bounds.size.height.0;

            node_left >= window_left - eps
                && node_top >= window_top - eps
                && node_right <= window_right + eps
                && node_bottom <= window_bottom + eps
        }
        UiPredicateV1::TextInputImeCursorAreaWithinWindow {
            padding_px,
            padding_insets_px,
            eps_px,
        } => {
            let Some(text_input_snapshot) = text_input_snapshot else {
                return false;
            };
            let Some(cursor_area) = text_input_snapshot.ime_cursor_area else {
                return false;
            };
            let pad = padding_px.max(0.0);
            let pad_insets = padding_insets_px.unwrap_or_else(|| UiPaddingInsetsV1::uniform(0.0));
            let eps = eps_px.max(0.0);

            let window_left = window_bounds.origin.x.0 + pad + pad_insets.left_px.max(0.0);
            let window_top = window_bounds.origin.y.0 + pad + pad_insets.top_px.max(0.0);
            let window_right = window_bounds.origin.x.0 + window_bounds.size.width.0
                - pad
                - pad_insets.right_px.max(0.0);
            let window_bottom = window_bounds.origin.y.0 + window_bounds.size.height.0
                - pad
                - pad_insets.bottom_px.max(0.0);

            let area_left = cursor_area.origin.x.0;
            let area_top = cursor_area.origin.y.0;
            let area_right = cursor_area.origin.x.0 + cursor_area.size.width.0;
            let area_bottom = cursor_area.origin.y.0 + cursor_area.size.height.0;

            area_left >= window_left - eps
                && area_top >= window_top - eps
                && area_right <= window_right + eps
                && area_bottom <= window_bottom + eps
        }
        UiPredicateV1::BoundsMinSize {
            target,
            min_w_px,
            min_h_px,
            eps_px,
        } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };

            let w = node.bounds.size.width.0.max(0.0);
            let h = node.bounds.size.height.0.max(0.0);

            let min_w = min_w_px.max(0.0);
            let min_h = min_h_px.max(0.0);
            let eps = eps_px.max(0.0);

            w + eps >= min_w && h + eps >= min_h
        }
        UiPredicateV1::BoundsMaxSize {
            target,
            max_w_px,
            max_h_px,
            eps_px,
        } => {
            let Some(node) = select_semantics_node(snapshot, window, element_runtime, target)
            else {
                return false;
            };

            let w = node.bounds.size.width.0.max(0.0);
            let h = node.bounds.size.height.0.max(0.0);

            let max_w = max_w_px.max(0.0);
            let max_h = max_h_px.max(0.0);
            let eps = eps_px.max(0.0);

            w <= max_w + eps && h <= max_h + eps
        }
        UiPredicateV1::BoundsApproxEqual { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax = a.bounds.origin.x.0;
            let ay = a.bounds.origin.y.0;
            let aw = a.bounds.size.width.0.max(0.0);
            let ah = a.bounds.size.height.0.max(0.0);

            let bx = b.bounds.origin.x.0;
            let by = b.bounds.origin.y.0;
            let bw = b.bounds.size.width.0.max(0.0);
            let bh = b.bounds.size.height.0.max(0.0);

            (ax - bx).abs() <= eps
                && (ay - by).abs() <= eps
                && (aw - bw).abs() <= eps
                && (ah - bh).abs() <= eps
        }
        UiPredicateV1::BoundsCenterApproxEqual { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax = a.bounds.origin.x.0;
            let ay = a.bounds.origin.y.0;
            let aw = a.bounds.size.width.0.max(0.0);
            let ah = a.bounds.size.height.0.max(0.0);
            let acx = ax + aw * 0.5;
            let acy = ay + ah * 0.5;

            let bx = b.bounds.origin.x.0;
            let by = b.bounds.origin.y.0;
            let bw = b.bounds.size.width.0.max(0.0);
            let bh = b.bounds.size.height.0.max(0.0);
            let bcx = bx + bw * 0.5;
            let bcy = by + bh * 0.5;

            (acx - bcx).abs() <= eps && (acy - bcy).abs() <= eps
        }
        UiPredicateV1::BoundsNonOverlapping { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax0 = a.bounds.origin.x.0;
            let ay0 = a.bounds.origin.y.0;
            let ax1 = ax0 + a.bounds.size.width.0.max(0.0);
            let ay1 = ay0 + a.bounds.size.height.0.max(0.0);

            let bx0 = b.bounds.origin.x.0;
            let by0 = b.bounds.origin.y.0;
            let bx1 = bx0 + b.bounds.size.width.0.max(0.0);
            let by1 = by0 + b.bounds.size.height.0.max(0.0);

            let overlap_w = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
            let overlap_h = (ay1.min(by1) - ay0.max(by0)).max(0.0);

            !(overlap_w > eps && overlap_h > eps)
        }
        UiPredicateV1::BoundsOverlapping { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax0 = a.bounds.origin.x.0;
            let ay0 = a.bounds.origin.y.0;
            let ax1 = ax0 + a.bounds.size.width.0.max(0.0);
            let ay1 = ay0 + a.bounds.size.height.0.max(0.0);

            let bx0 = b.bounds.origin.x.0;
            let by0 = b.bounds.origin.y.0;
            let bx1 = bx0 + b.bounds.size.width.0.max(0.0);
            let by1 = by0 + b.bounds.size.height.0.max(0.0);

            let overlap_w = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
            let overlap_h = (ay1.min(by1) - ay0.max(by0)).max(0.0);

            overlap_w > eps && overlap_h > eps
        }
        UiPredicateV1::BoundsOverlappingX { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ax0 = a.bounds.origin.x.0;
            let ax1 = ax0 + a.bounds.size.width.0.max(0.0);

            let bx0 = b.bounds.origin.x.0;
            let bx1 = bx0 + b.bounds.size.width.0.max(0.0);

            let overlap_w = (ax1.min(bx1) - ax0.max(bx0)).max(0.0);
            overlap_w > eps
        }
        UiPredicateV1::BoundsOverlappingY { a, b, eps_px } => {
            let Some(a) = select_semantics_node(snapshot, window, element_runtime, a) else {
                return false;
            };
            let Some(b) = select_semantics_node(snapshot, window, element_runtime, b) else {
                return false;
            };

            let eps = eps_px.max(0.0);

            let ay0 = a.bounds.origin.y.0;
            let ay1 = ay0 + a.bounds.size.height.0.max(0.0);

            let by0 = b.bounds.origin.y.0;
            let by1 = by0 + b.bounds.size.height.0.max(0.0);

            let overlap_h = (ay1.min(by1) - ay0.max(by0)).max(0.0);
            overlap_h > eps
        }
        UiPredicateV1::KnownWindowCountGe { n } => (known_windows.len() as u32) >= *n,
        UiPredicateV1::KnownWindowCountIs { n } => (known_windows.len() as u32) == *n,
        UiPredicateV1::PlatformUiWindowHoverDetectionIs { quality } => {
            if let Some(input_ctx) = input_ctx {
                input_ctx.caps.ui.window_hover_detection.as_str() == quality.as_str()
            } else {
                platform_caps
                    .is_some_and(|c| c.ui.window_hover_detection.as_str() == quality.as_str())
            }
        }
        UiPredicateV1::DockDragCurrentWindowIs {
            window: target_window,
        } => {
            let Some(target_window) =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)
            else {
                return false;
            };
            dock_drag_runtime
                .is_some_and(|drag| drag.dragging && drag.current_window == target_window)
        }
        UiPredicateV1::DockDragMovingWindowIs {
            window: target_window,
        } => {
            let Some(target_window) =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)
            else {
                return false;
            };
            dock_drag_runtime
                .is_some_and(|drag| drag.dragging && drag.moving_window == Some(target_window))
        }
        UiPredicateV1::DockDragWindowUnderMovingWindowIs {
            window: target_window,
        } => {
            let Some(target_window) =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)
            else {
                return false;
            };
            dock_drag_runtime.is_some_and(|drag| {
                drag.dragging && drag.window_under_moving_window == Some(target_window)
            })
        }
        UiPredicateV1::DockDragActiveIs { active } => {
            let dragging = dock_drag_runtime.is_some_and(|drag| drag.dragging);
            dragging == *active
        }
        UiPredicateV1::DockDragTransparentPayloadAppliedIs { applied } => {
            if let Some(drag) = dock_drag_runtime {
                return drag.dragging && drag.transparent_payload_applied == *applied;
            }
            !*applied
        }
        UiPredicateV1::DockDragTransparentPayloadMousePassthroughAppliedIs { applied } => {
            if let Some(drag) = dock_drag_runtime {
                return drag.dragging
                    && drag.transparent_payload_mouse_passthrough_applied == *applied;
            }
            !*applied
        }
        UiPredicateV1::DockDragWindowUnderCursorSourceIs { source } => {
            let Some(drag) = dock_drag_runtime else {
                return false;
            };
            dock_drag_window_under_cursor_source_is(
                drag.window_under_cursor_source,
                source.as_str(),
            )
        }
        UiPredicateV1::DockDragWindowUnderMovingWindowSourceIs { source } => {
            let Some(drag) = dock_drag_runtime else {
                return false;
            };
            dock_drag_window_under_cursor_source_is(
                drag.window_under_moving_window_source,
                source.as_str(),
            )
        }
        UiPredicateV1::DockFloatingDragActiveIs { active } => {
            match docking.and_then(|d| d.floating_drag) {
                Some(drag) => drag.activated == *active,
                None => !*active,
            }
        }
        UiPredicateV1::DockDropPreviewKindIs { preview_kind } => {
            let Some(preview) = docking
                .and_then(|d| d.dock_drop_resolve.as_ref())
                .and_then(|d| d.preview.as_ref())
            else {
                return false;
            };
            let have = match preview.kind {
                fret_runtime::DockDropPreviewKindDiagnostics::WrapBinary => "wrap_binary",
                fret_runtime::DockDropPreviewKindDiagnostics::InsertIntoSplit { .. } => {
                    "insert_into_split"
                }
            };
            have == preview_kind.as_str()
        }
        UiPredicateV1::DockDropResolveSourceIs { source } => {
            let Some(resolve) = docking.and_then(|d| d.dock_drop_resolve.as_ref()) else {
                return false;
            };
            let have = match resolve.source {
                fret_runtime::DockDropResolveSource::InvertDocking => "invert_docking",
                fret_runtime::DockDropResolveSource::OutsideWindow => "outside_window",
                fret_runtime::DockDropResolveSource::FloatZone => "float_zone",
                fret_runtime::DockDropResolveSource::EmptyDockSpace => "empty_dock_space",
                fret_runtime::DockDropResolveSource::LayoutBoundsMiss => "layout_bounds_miss",
                fret_runtime::DockDropResolveSource::LatchedPreviousHover => {
                    "latched_previous_hover"
                }
                fret_runtime::DockDropResolveSource::TabBar => "tab_bar",
                fret_runtime::DockDropResolveSource::FloatingTitleBar => "floating_title_bar",
                fret_runtime::DockDropResolveSource::OuterHintRect => "outer_hint_rect",
                fret_runtime::DockDropResolveSource::InnerHintRect => "inner_hint_rect",
                fret_runtime::DockDropResolveSource::None => "none",
            };
            have == source.as_str()
        }
        UiPredicateV1::DockDropResolvedIsSome { some } => docking
            .and_then(|d| d.dock_drop_resolve.as_ref())
            .is_some_and(|d| d.resolved.is_some() == *some),
        UiPredicateV1::DockGraphCanonicalIs { canonical } => docking
            .and_then(|d| d.dock_graph_stats)
            .is_some_and(|s| s.canonical_ok == *canonical),
        UiPredicateV1::DockGraphHasNestedSameAxisSplitsIs { has_nested } => docking
            .and_then(|d| d.dock_graph_stats)
            .is_some_and(|s| s.has_nested_same_axis_splits == *has_nested),
        UiPredicateV1::DockGraphNodeCountLe { max } => docking
            .and_then(|d| d.dock_graph_stats)
            .is_some_and(|s| s.node_count <= *max),
        UiPredicateV1::DockGraphMaxSplitDepthLe { max } => docking
            .and_then(|d| d.dock_graph_stats)
            .is_some_and(|s| s.max_split_depth <= *max),
        UiPredicateV1::DockGraphSignatureIs { signature } => docking
            .and_then(|d| d.dock_graph_signature.as_ref())
            .is_some_and(|s| s.signature == *signature),
        UiPredicateV1::DockGraphSignatureContains { needle } => docking
            .and_then(|d| d.dock_graph_signature.as_ref())
            .is_some_and(|s| s.signature.contains(needle)),
        UiPredicateV1::DockGraphSignatureFingerprint64Is { fingerprint64 } => docking
            .and_then(|d| d.dock_graph_signature.as_ref())
            .is_some_and(|s| s.fingerprint64 == *fingerprint64),
        UiPredicateV1::EventKindSeen { event_kind: _ } => false,
    }
}
