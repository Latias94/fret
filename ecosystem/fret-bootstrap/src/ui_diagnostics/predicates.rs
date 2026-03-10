// Split across small files to reduce churn in fearless refactors.
include!("predicates/dock_drag.rs");

fn redaction_aware_len_bytes(s: &str) -> usize {
    // Diagnostics redaction uses `<redacted len={}>` where the number is the UTF-8 byte length.
    // Prefer reading that value so predicates remain stable regardless of `redact_text`.
    const PREFIX: &str = "<redacted len=";
    const SUFFIX: &str = ">";

    let s = s.trim();
    if let Some(rest) = s.strip_prefix(PREFIX)
        && let Some(num) = rest.strip_suffix(SUFFIX)
        && let Ok(n) = num.parse::<usize>()
    {
        return n;
    }

    s.len()
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

fn dock_drag_kind_is(have: fret_runtime::DragKindId, want: &str) -> bool {
    match want {
        "dock_panel" => have == fret_runtime::DRAG_KIND_DOCK_PANEL,
        "dock_tabs" => have == fret_runtime::DRAG_KIND_DOCK_TABS,
        _ => false,
    }
}

fn semantics_scroll_field_value(
    node: &fret_core::SemanticsNode,
    field: fret_diag_protocol::UiSemanticsScrollFieldV1,
) -> Option<f64> {
    match field {
        fret_diag_protocol::UiSemanticsScrollFieldV1::X => node.extra.scroll.x,
        fret_diag_protocol::UiSemanticsScrollFieldV1::XMin => node.extra.scroll.x_min,
        fret_diag_protocol::UiSemanticsScrollFieldV1::XMax => node.extra.scroll.x_max,
        fret_diag_protocol::UiSemanticsScrollFieldV1::Y => node.extra.scroll.y,
        fret_diag_protocol::UiSemanticsScrollFieldV1::YMin => node.extra.scroll.y_min,
        fret_diag_protocol::UiSemanticsScrollFieldV1::YMax => node.extra.scroll.y_max,
    }
}

fn eval_predicate_without_semantics(
    window: AppWindowId,
    known_windows: &[AppWindowId],
    open_window_count: u32,
    platform_caps: Option<&fret_runtime::PlatformCapabilities>,
    window_style: Option<&fret_runtime::RunnerWindowStyleDiagnosticsStore>,
    platform_window_receiver: Option<&fret_runtime::RunnerPlatformWindowReceiverDiagnosticsStore>,
    docking: Option<&fret_runtime::DockingInteractionDiagnostics>,
    workspace: Option<&fret_runtime::WorkspaceInteractionDiagnostics>,
    dock_drag_runtime: Option<&DockDragRuntimeState>,
    pred: &UiPredicateV1,
) -> Option<bool> {
    match pred {
        UiPredicateV1::KnownWindowCountGe { n } => Some(open_window_count >= *n),
        UiPredicateV1::KnownWindowCountIs { n } => Some(open_window_count == *n),
        UiPredicateV1::PlatformUiWindowHoverDetectionIs { quality } => Some(
            platform_caps.is_some_and(|c| c.ui.window_hover_detection.as_str() == quality.as_str()),
        ),
        UiPredicateV1::PlatformWindowReceiverAtCursorIs {
            window: target_window,
        } => {
            let target_window =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)?;
            let have = platform_window_receiver?.latest_at_cursor()?;
            Some(have.receiver_window == Some(target_window))
        }
        UiPredicateV1::WindowStyleEffectiveIs {
            window: target_window,
            style,
        } => {
            let target_window =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)?;
            let have = window_style?.effective_snapshot(target_window)?;
            Some(window_style_effective_matches(&have, style))
        }
        UiPredicateV1::WindowBackgroundMaterialEffectiveIs {
            window: target_window,
            material,
        } => {
            let target_window =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)?;
            let have = window_style?.effective_snapshot(target_window)?;
            Some(window_background_material_matches(
                have.background_material,
                *material,
            ))
        }
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
        UiPredicateV1::DockDragKindIs { drag_kind } => Some(
            dock_drag_runtime
                .is_some_and(|drag| drag.dragging && dock_drag_kind_is(drag.kind, drag_kind)),
        ),
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
        UiPredicateV1::DockDragTransparentPayloadHitTestPassthroughAppliedIs { applied } => Some(
            dock_drag_runtime.is_some_and(|drag| {
                drag.dragging && drag.transparent_payload_hit_test_passthrough_applied == *applied
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
        UiPredicateV1::DockDropResolvedZoneIs { zone } => {
            let resolved = docking
                .and_then(|d| d.dock_drop_resolve.as_ref())
                .and_then(|d| d.resolved.as_ref())?;
            let have = match resolved.zone {
                fret_core::dock::DropZone::Center => "center",
                fret_core::dock::DropZone::Left => "left",
                fret_core::dock::DropZone::Right => "right",
                fret_core::dock::DropZone::Top => "top",
                fret_core::dock::DropZone::Bottom => "bottom",
            };
            Some(have == zone.as_str())
        }
        UiPredicateV1::DockDropResolvedInsertIndexIs { index } => {
            let resolved = docking
                .and_then(|d| d.dock_drop_resolve.as_ref())
                .and_then(|d| d.resolved.as_ref())?;
            Some(resolved.insert_index == Some(*index as usize))
        }
        UiPredicateV1::DockTabStripActiveOverflowIs { overflow } => Some(
            docking
                .and_then(|d| d.tab_strip_active_visibility.as_ref())
                .is_some_and(|s| s.overflow == *overflow),
        ),
        UiPredicateV1::DockTabStripActiveVisibleIs { visible } => Some(
            docking
                .and_then(|d| d.tab_strip_active_visibility.as_ref())
                .is_some_and(|s| s.active_visible == *visible),
        ),
        UiPredicateV1::DockTabStripActiveScrollPxGe { px } => Some(
            docking
                .and_then(|d| d.tab_strip_active_visibility.as_ref())
                .is_some_and(|s| s.scroll.0 >= *px),
        ),
        UiPredicateV1::DockTabStripActiveScrollPxLe { px } => Some(
            docking
                .and_then(|d| d.tab_strip_active_visibility.as_ref())
                .is_some_and(|s| s.scroll.0 <= *px),
        ),
        UiPredicateV1::WorkspaceTabStripActiveOverflowIs { overflow, pane_id } => Some(
            workspace
                .and_then(|w| {
                    w.tab_strip_active_visibility.iter().rev().find(|s| {
                        s.status
                            == fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok
                            && pane_id.as_ref().is_none_or(|id| {
                                s.pane_id
                                    .as_ref()
                                    .is_some_and(|p| p.as_ref() == id.as_str())
                            })
                    })
                })
                .is_some_and(|s| s.overflow == *overflow),
        ),
        UiPredicateV1::WorkspaceTabStripActiveVisibleIs { visible, pane_id } => Some(
            workspace
                .and_then(|w| {
                    w.tab_strip_active_visibility.iter().rev().find(|s| {
                        s.status
                            == fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok
                            && pane_id.as_ref().is_none_or(|id| {
                                s.pane_id
                                    .as_ref()
                                    .is_some_and(|p| p.as_ref() == id.as_str())
                            })
                    })
                })
                .is_some_and(|s| s.active_visible == *visible),
        ),
        UiPredicateV1::WorkspaceTabStripDragActiveIs { active, pane_id } => Some(
            workspace
                .and_then(|w| {
                    w.tab_strip_drag.iter().rev().find(|s| {
                        pane_id.as_ref().is_none_or(|id| {
                            s.pane_id
                                .as_ref()
                                .is_some_and(|p| p.as_ref() == id.as_str())
                        })
                    })
                })
                .is_some_and(|s| s.dragging == *active),
        ),
        UiPredicateV1::WorkspaceTabStripDragArmedIs { armed, pane_id } => Some(
            workspace
                .and_then(|w| {
                    w.tab_strip_drag.iter().rev().find(|s| {
                        pane_id.as_ref().is_none_or(|id| {
                            s.pane_id
                                .as_ref()
                                .is_some_and(|p| p.as_ref() == id.as_str())
                        })
                    })
                })
                .is_some_and(|s| s.pointer_id.is_some() == *armed),
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
        UiPredicateV1::DockGraphSignatureNotContains { needle } => Some(
            docking
                .and_then(|d| d.dock_graph_signature.as_ref())
                .is_some_and(|s| !s.signature.contains(needle)),
        ),
        UiPredicateV1::DockGraphSignatureFingerprint64Is { fingerprint64 } => Some(
            docking
                .and_then(|d| d.dock_graph_signature.as_ref())
                .is_some_and(|s| s.fingerprint64 == *fingerprint64),
        ),
        _ => None,
    }
}

fn window_style_effective_matches(
    have: &fret_runtime::RunnerWindowStyleEffectiveSnapshotV1,
    want: &UiWindowStyleMatchV1,
) -> bool {
    if let Some(decorations) = want.decorations
        && !window_decorations_match(have.decorations, decorations)
    {
        return false;
    }
    if let Some(resizable) = want.resizable
        && have.resizable != resizable
    {
        return false;
    }
    if let Some(transparent) = want.transparent
        && have.surface_composited_alpha != transparent
    {
        return false;
    }
    if let Some(visual_transparent) = want.visual_transparent
        && have.visual_transparent != visual_transparent
    {
        return false;
    }
    if let Some(appearance) = want.appearance
        && !window_appearance_match(have.appearance, appearance)
    {
        return false;
    }
    if let Some(taskbar) = want.taskbar
        && !taskbar_visibility_match(have.taskbar, taskbar)
    {
        return false;
    }
    if let Some(activation) = want.activation
        && !activation_policy_match(have.activation, activation)
    {
        return false;
    }
    if let Some(z_level) = want.z_level
        && !window_z_level_match(have.z_level, z_level)
    {
        return false;
    }
    if let Some(hit_test) = want.hit_test
        && !window_hit_test_match(&have.hit_test, hit_test)
    {
        return false;
    }
    if let Some(fp) = want.hit_test_regions_fingerprint64
        && have.hit_test_regions_fingerprint64 != Some(fp)
    {
        return false;
    }
    true
}

fn window_appearance_match(
    have: fret_runtime::RunnerWindowAppearanceV1,
    want: fret_diag_protocol::UiWindowAppearanceV1,
) -> bool {
    use fret_diag_protocol::UiWindowAppearanceV1 as W;
    use fret_runtime::RunnerWindowAppearanceV1 as H;
    match (have, want) {
        (H::Opaque, W::Opaque) => true,
        (H::CompositedNoBackdrop, W::CompositedNoBackdrop) => true,
        (H::CompositedBackdrop, W::CompositedBackdrop) => true,
        _ => false,
    }
}

fn window_hit_test_match(
    have: &fret_runtime::WindowHitTestRequestV1,
    want: UiWindowHitTestRequestV1,
) -> bool {
    use UiWindowHitTestRequestV1 as W;
    use fret_runtime::WindowHitTestRequestV1 as H;

    match (have, want) {
        (&H::Normal, W::Normal) => true,
        (&H::PassthroughAll, W::PassthroughAll) => true,
        (&H::PassthroughRegions { .. }, W::PassthroughRegions) => true,
        _ => false,
    }
}

fn window_background_material_matches(
    have: fret_runtime::WindowBackgroundMaterialRequest,
    want: UiWindowBackgroundMaterialRequestV1,
) -> bool {
    use UiWindowBackgroundMaterialRequestV1 as W;
    use fret_runtime::WindowBackgroundMaterialRequest as H;
    match (have, want) {
        (H::None, W::None) => true,
        (H::SystemDefault, W::SystemDefault) => true,
        (H::Mica, W::Mica) => true,
        (H::Acrylic, W::Acrylic) => true,
        (H::Vibrancy, W::Vibrancy) => true,
        _ => false,
    }
}

fn window_decorations_match(
    have: fret_runtime::WindowDecorationsRequest,
    want: UiWindowDecorationsRequestV1,
) -> bool {
    use UiWindowDecorationsRequestV1 as W;
    use fret_runtime::WindowDecorationsRequest as H;
    match (have, want) {
        (H::System, W::System) => true,
        (H::None, W::None) => true,
        (H::Server, W::Server) => true,
        (H::Client, W::Client) => true,
        _ => false,
    }
}

fn taskbar_visibility_match(
    have: fret_runtime::TaskbarVisibility,
    want: UiTaskbarVisibilityV1,
) -> bool {
    use UiTaskbarVisibilityV1 as W;
    use fret_runtime::TaskbarVisibility as H;
    match (have, want) {
        (H::Show, W::Show) => true,
        (H::Hide, W::Hide) => true,
        _ => false,
    }
}

fn activation_policy_match(
    have: fret_runtime::ActivationPolicy,
    want: UiActivationPolicyV1,
) -> bool {
    use UiActivationPolicyV1 as W;
    use fret_runtime::ActivationPolicy as H;
    match (have, want) {
        (H::Activates, W::Activates) => true,
        (H::NonActivating, W::NonActivating) => true,
        _ => false,
    }
}

fn window_z_level_match(have: fret_runtime::WindowZLevel, want: UiWindowZLevelV1) -> bool {
    use UiWindowZLevelV1 as W;
    use fret_runtime::WindowZLevel as H;
    match (have, want) {
        (H::Normal, W::Normal) => true,
        (H::AlwaysOnTop, W::AlwaysOnTop) => true,
        _ => false,
    }
}

fn eval_predicate(
    snapshot: &fret_core::SemanticsSnapshot,
    window_bounds: Rect,
    window: AppWindowId,
    scope_root: Option<u64>,
    input_ctx: Option<&fret_runtime::InputContext>,
    element_runtime: Option<&ElementRuntime>,
    text_input_snapshot: Option<&fret_runtime::WindowTextInputSnapshot>,
    render_text: Option<fret_core::RendererTextPerfSnapshot>,
    render_text_font_trace: Option<&fret_core::RendererTextFontTraceSnapshot>,
    known_windows: &[AppWindowId],
    open_window_count: u32,
    platform_caps: Option<&fret_runtime::PlatformCapabilities>,
    window_style: Option<&fret_runtime::RunnerWindowStyleDiagnosticsStore>,
    platform_window_receiver: Option<&fret_runtime::RunnerPlatformWindowReceiverDiagnosticsStore>,
    docking: Option<&fret_runtime::DockingInteractionDiagnostics>,
    workspace: Option<&fret_runtime::WorkspaceInteractionDiagnostics>,
    dock_drag_runtime: Option<&DockDragRuntimeState>,
    text_font_stack_key_stable_frames: u32,
    font_catalog_populated: bool,
    system_font_rescan_idle: bool,
    pred: &UiPredicateV1,
) -> bool {
    let select_node = |target: &UiSelectorV1| {
        select_semantics_node_scoped(snapshot, window, element_runtime, target, scope_root)
    };

    match pred {
        UiPredicateV1::Exists { target } => select_node(target).is_some(),
        UiPredicateV1::NotExists { target } => select_node(target).is_none(),
        UiPredicateV1::ExistsUnder { scope, target } => {
            let Some(scope_node) = select_node(scope) else {
                return false;
            };
            let scope_root = scope_node.id.data().as_ffi();
            select_semantics_node_scoped(
                snapshot,
                window,
                element_runtime,
                target,
                Some(scope_root),
            )
            .is_some()
        }
        UiPredicateV1::NotExistsUnder { scope, target } => {
            let Some(scope_node) = select_node(scope) else {
                return false;
            };
            let scope_root = scope_node.id.data().as_ffi();
            select_semantics_node_scoped(
                snapshot,
                window,
                element_runtime,
                target,
                Some(scope_root),
            )
            .is_none()
        }
        UiPredicateV1::FocusedDescendantIs { scope, target } => {
            let Some(focus) = snapshot.focus else {
                return false;
            };
            let Some(scope_node) = select_node(scope) else {
                return false;
            };
            let scope_root = scope_node.id.data().as_ffi();
            let Some(node) = select_semantics_node_scoped(
                snapshot,
                window,
                element_runtime,
                target,
                Some(scope_root),
            ) else {
                return false;
            };
            node.id == focus
        }
        UiPredicateV1::FocusIs { target } => {
            let Some(focus) = snapshot.focus else {
                return false;
            };
            let Some(node) = select_node(target) else {
                return false;
            };
            node.id == focus
        }
        UiPredicateV1::RoleIs { target, role } => {
            let Some(want) = parse_semantics_role(role) else {
                return false;
            };
            let Some(node) = select_node(target) else {
                return false;
            };
            node.role == want
        }
        UiPredicateV1::LabelContains { target, text } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.label
                .as_deref()
                .is_some_and(|label| label.contains(text))
        }
        UiPredicateV1::LabelLenIs { target, len_bytes } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            let got = node
                .label
                .as_deref()
                .map(redaction_aware_len_bytes)
                .unwrap_or(0);
            got == (*len_bytes as usize)
        }
        UiPredicateV1::LabelLenGe {
            target,
            min_len_bytes,
        } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            let got = node
                .label
                .as_deref()
                .map(redaction_aware_len_bytes)
                .unwrap_or(0);
            got >= (*min_len_bytes as usize)
        }
        UiPredicateV1::ValueContains { target, text } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.value
                .as_deref()
                .is_some_and(|value| value.contains(text))
        }
        UiPredicateV1::ValueEquals { target, text } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.value.as_deref() == Some(text.as_str())
        }
        UiPredicateV1::ValueLenIs { target, len_bytes } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            let got = node
                .value
                .as_deref()
                .map(redaction_aware_len_bytes)
                .unwrap_or(0);
            got == (*len_bytes as usize)
        }
        UiPredicateV1::ValueLenGe {
            target,
            min_len_bytes,
        } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            let got = node
                .value
                .as_deref()
                .map(redaction_aware_len_bytes)
                .unwrap_or(0);
            got >= (*min_len_bytes as usize)
        }
        UiPredicateV1::PosInSetIs { target, pos_in_set } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.pos_in_set == Some(*pos_in_set)
        }
        UiPredicateV1::SetSizeIs { target, set_size } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.set_size == Some(*set_size)
        }
        UiPredicateV1::CheckedIs { target, checked } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.flags.checked == Some(*checked)
        }
        UiPredicateV1::SelectedIs { target, selected } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.flags.selected == *selected
        }
        UiPredicateV1::SemanticsNumericApproxEq {
            target,
            field,
            value,
            eps,
        } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            let got = match field {
                fret_diag_protocol::UiSemanticsNumericFieldV1::Value => node.extra.numeric.value,
                fret_diag_protocol::UiSemanticsNumericFieldV1::Min => node.extra.numeric.min,
                fret_diag_protocol::UiSemanticsNumericFieldV1::Max => node.extra.numeric.max,
                fret_diag_protocol::UiSemanticsNumericFieldV1::Step => node.extra.numeric.step,
                fret_diag_protocol::UiSemanticsNumericFieldV1::Jump => node.extra.numeric.jump,
            };
            let Some(got) = got else {
                return false;
            };
            let want = *value;
            let eps = eps.abs();
            got.is_finite() && want.is_finite() && eps.is_finite() && (got - want).abs() <= eps
        }
        UiPredicateV1::SemanticsScrollIsFinite { target, field } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            let got = semantics_scroll_field_value(node, *field);
            got.is_some_and(|v| v.is_finite())
        }
        UiPredicateV1::SemanticsScrollApproxEq {
            target,
            field,
            value,
            eps,
        } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            let got = semantics_scroll_field_value(node, *field);
            let Some(got) = got else {
                return false;
            };
            let want = *value;
            let eps = eps.abs();
            got.is_finite() && want.is_finite() && eps.is_finite() && (got - want).abs() <= eps
        }
        UiPredicateV1::SemanticsScrollNotApproxEq {
            target,
            field,
            value,
            eps,
        } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            let got = semantics_scroll_field_value(node, *field);
            let Some(got) = got else {
                return false;
            };
            let want = *value;
            let eps = eps.abs();
            got.is_finite() && want.is_finite() && eps.is_finite() && (got - want).abs() > eps
        }
        UiPredicateV1::TextCompositionIs { target, composing } => {
            let Some(node) = select_node(target) else {
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
        UiPredicateV1::ImeSurroundingTextIsSome { is_some } => {
            text_input_snapshot
                .and_then(|snapshot| snapshot.surrounding_text.as_ref())
                .is_some()
                == *is_some
        }
        UiPredicateV1::ImeSurroundingTextValid => {
            let Some(surrounding) =
                text_input_snapshot.and_then(|snapshot| snapshot.surrounding_text.as_ref())
            else {
                return false;
            };

            let text = surrounding.text.as_ref();
            if text.len() > fret_runtime::WindowImeSurroundingText::MAX_TEXT_BYTES {
                return false;
            }

            let cursor = surrounding.cursor as usize;
            let anchor = surrounding.anchor as usize;
            if cursor > text.len() || anchor > text.len() {
                return false;
            }

            text.is_char_boundary(cursor) && text.is_char_boundary(anchor)
        }
        UiPredicateV1::CheckedIsNone { target } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.flags.checked.is_none()
        }
        UiPredicateV1::ActiveItemIs { container, item } => {
            let Some(item_node) = select_node(item) else {
                return false;
            };

            if snapshot.focus == Some(item_node.id) {
                return true;
            }

            let Some(container_node) = select_node(container) else {
                return false;
            };

            container_node.active_descendant == Some(item_node.id)
        }
        UiPredicateV1::ActiveItemIsNone { container } => {
            let Some(container_node) = select_node(container) else {
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
            let Some(node) = select_node(target) else {
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
            let Some(node) = select_node(target) else {
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
            let Some(node) = select_node(target) else {
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
            let Some(node) = select_node(target) else {
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
            let Some(a) = select_node(a) else {
                return false;
            };
            let Some(b) = select_node(b) else {
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
            let Some(a) = select_node(a) else {
                return false;
            };
            let Some(b) = select_node(b) else {
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
            let Some(a) = select_node(a) else {
                return false;
            };
            let Some(b) = select_node(b) else {
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
            let Some(a) = select_node(a) else {
                return false;
            };
            let Some(b) = select_node(b) else {
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
            let Some(a) = select_node(a) else {
                return false;
            };
            let Some(b) = select_node(b) else {
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
            let Some(a) = select_node(a) else {
                return false;
            };
            let Some(b) = select_node(b) else {
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
        UiPredicateV1::KnownWindowCountGe { n } => open_window_count >= *n,
        UiPredicateV1::KnownWindowCountIs { n } => open_window_count == *n,
        UiPredicateV1::PlatformUiWindowHoverDetectionIs { quality } => {
            if let Some(input_ctx) = input_ctx {
                input_ctx.caps.ui.window_hover_detection.as_str() == quality.as_str()
            } else {
                platform_caps
                    .is_some_and(|c| c.ui.window_hover_detection.as_str() == quality.as_str())
            }
        }
        UiPredicateV1::PlatformWindowReceiverAtCursorIs {
            window: target_window,
        } => {
            let Some(target_window) =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)
            else {
                return false;
            };
            let Some(have) = platform_window_receiver.and_then(|s| s.latest_at_cursor()) else {
                return false;
            };
            have.receiver_window == Some(target_window)
        }
        UiPredicateV1::WindowStyleEffectiveIs {
            window: target_window,
            style,
        } => {
            let Some(target_window) =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)
            else {
                return false;
            };
            let Some(have) = window_style.and_then(|s| s.effective_snapshot(target_window)) else {
                return false;
            };
            window_style_effective_matches(&have, style)
        }
        UiPredicateV1::WindowBackgroundMaterialEffectiveIs {
            window: target_window,
            material,
        } => {
            let Some(target_window) =
                resolve_window_target_from_known_windows(window, known_windows, *target_window)
            else {
                return false;
            };
            let Some(have) = window_style.and_then(|s| s.effective_snapshot(target_window)) else {
                return false;
            };
            window_background_material_matches(have.background_material, *material)
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
        UiPredicateV1::DockDragKindIs { drag_kind } => {
            let Some(drag) = dock_drag_runtime else {
                return false;
            };
            drag.dragging && dock_drag_kind_is(drag.kind, drag_kind.as_str())
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
        UiPredicateV1::DockDragTransparentPayloadHitTestPassthroughAppliedIs { applied } => {
            if let Some(drag) = dock_drag_runtime {
                return drag.dragging
                    && drag.transparent_payload_hit_test_passthrough_applied == *applied;
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
        UiPredicateV1::DockDropResolvedZoneIs { zone } => {
            let Some(resolved) = docking
                .and_then(|d| d.dock_drop_resolve.as_ref())
                .and_then(|d| d.resolved.as_ref())
            else {
                return false;
            };
            let have = match resolved.zone {
                fret_core::dock::DropZone::Center => "center",
                fret_core::dock::DropZone::Left => "left",
                fret_core::dock::DropZone::Right => "right",
                fret_core::dock::DropZone::Top => "top",
                fret_core::dock::DropZone::Bottom => "bottom",
            };
            have == zone.as_str()
        }
        UiPredicateV1::DockDropResolvedInsertIndexIs { index } => docking
            .and_then(|d| d.dock_drop_resolve.as_ref())
            .and_then(|d| d.resolved.as_ref())
            .is_some_and(|d| d.insert_index == Some(*index as usize)),
        UiPredicateV1::DockTabStripActiveOverflowIs { overflow } => docking
            .and_then(|d| d.tab_strip_active_visibility.as_ref())
            .is_some_and(|s| s.overflow == *overflow),
        UiPredicateV1::DockTabStripActiveVisibleIs { visible } => docking
            .and_then(|d| d.tab_strip_active_visibility.as_ref())
            .is_some_and(|s| s.active_visible == *visible),
        UiPredicateV1::DockTabStripActiveScrollPxGe { px } => docking
            .and_then(|d| d.tab_strip_active_visibility.as_ref())
            .is_some_and(|s| s.scroll.0 >= *px),
        UiPredicateV1::DockTabStripActiveScrollPxLe { px } => docking
            .and_then(|d| d.tab_strip_active_visibility.as_ref())
            .is_some_and(|s| s.scroll.0 <= *px),
        UiPredicateV1::WorkspaceTabStripActiveOverflowIs { overflow, pane_id } => workspace
            .and_then(|w| {
                w.tab_strip_active_visibility.iter().rev().find(|s| {
                    s.status == fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok
                        && pane_id.as_ref().is_none_or(|id| {
                            s.pane_id
                                .as_ref()
                                .is_some_and(|p| p.as_ref() == id.as_str())
                        })
                })
            })
            .is_some_and(|s| s.overflow == *overflow),
        UiPredicateV1::WorkspaceTabStripActiveVisibleIs { visible, pane_id } => workspace
            .and_then(|w| {
                w.tab_strip_active_visibility.iter().rev().find(|s| {
                    s.status == fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok
                        && pane_id.as_ref().is_none_or(|id| {
                            s.pane_id
                                .as_ref()
                                .is_some_and(|p| p.as_ref() == id.as_str())
                        })
                })
            })
            .is_some_and(|s| s.active_visible == *visible),
        UiPredicateV1::WorkspaceTabStripActiveScrollPxGe { px, pane_id } => workspace
            .and_then(|w| {
                w.tab_strip_active_visibility.iter().rev().find(|s| {
                    s.status == fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok
                        && pane_id.as_ref().is_none_or(|id| {
                            s.pane_id
                                .as_ref()
                                .is_some_and(|p| p.as_ref() == id.as_str())
                        })
                })
            })
            .is_some_and(|s| s.scroll_x.0 >= *px),
        UiPredicateV1::WorkspaceTabStripActiveScrollPxLe { px, pane_id } => workspace
            .and_then(|w| {
                w.tab_strip_active_visibility.iter().rev().find(|s| {
                    s.status == fret_runtime::WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok
                        && pane_id.as_ref().is_none_or(|id| {
                            s.pane_id
                                .as_ref()
                                .is_some_and(|p| p.as_ref() == id.as_str())
                        })
                })
            })
            .is_some_and(|s| s.scroll_x.0 <= *px),
        UiPredicateV1::WorkspaceTabStripDragActiveIs { active, pane_id } => workspace
            .and_then(|w| {
                w.tab_strip_drag.iter().rev().find(|s| {
                    pane_id.as_ref().is_none_or(|id| {
                        s.pane_id
                            .as_ref()
                            .is_some_and(|p| p.as_ref() == id.as_str())
                    })
                })
            })
            .is_some_and(|s| s.dragging == *active),
        UiPredicateV1::WorkspaceTabStripDragArmedIs { armed, pane_id } => workspace
            .and_then(|w| {
                w.tab_strip_drag.iter().rev().find(|s| {
                    pane_id.as_ref().is_none_or(|id| {
                        s.pane_id
                            .as_ref()
                            .is_some_and(|p| p.as_ref() == id.as_str())
                    })
                })
            })
            .is_some_and(|s| s.pointer_id.is_some() == *armed),
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
        UiPredicateV1::DockGraphSignatureNotContains { needle } => docking
            .and_then(|d| d.dock_graph_signature.as_ref())
            .is_some_and(|s| !s.signature.contains(needle)),
        UiPredicateV1::DockGraphSignatureFingerprint64Is { fingerprint64 } => docking
            .and_then(|d| d.dock_graph_signature.as_ref())
            .is_some_and(|s| s.fingerprint64 == *fingerprint64),
        UiPredicateV1::EventKindSeen { event_kind: _ } => false,
    }
}
