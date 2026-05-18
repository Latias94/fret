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

fn font_trace_entry_matches(
    entry: &fret_core::RendererTextFontTraceEntry,
    text_contains: Option<&str>,
    font: Option<&str>,
    wrap: Option<&str>,
    overflow: Option<&str>,
    missing_glyphs: Option<u32>,
    family_contains: Option<&str>,
    family_class: Option<fret_diag_protocol::UiRenderTextFontTraceFamilyClassV1>,
) -> bool {
    if let Some(text_contains) = text_contains
        && !entry.text_preview.contains(text_contains)
    {
        return false;
    }
    if let Some(font) = font
        && !font_id_matches(&entry.font, font)
    {
        return false;
    }
    if let Some(wrap) = wrap
        && text_wrap_as_str(entry.wrap) != wrap
    {
        return false;
    }
    if let Some(overflow) = overflow
        && text_overflow_as_str(entry.overflow) != overflow
    {
        return false;
    }
    if let Some(missing_glyphs) = missing_glyphs
        && entry.missing_glyphs != missing_glyphs
    {
        return false;
    }

    let family_matches = |family: &fret_core::RendererTextFontTraceFamilyUsage| {
        if let Some(contains) = family_contains
            && !family.family.contains(contains)
        {
            return false;
        }
        if let Some(class) = family_class
            && !font_trace_family_class_matches(family.class, class)
        {
            return false;
        }
        true
    };

    if family_contains.is_some() || family_class.is_some() {
        entry.families.iter().any(family_matches)
    } else {
        true
    }
}

fn font_id_matches(font: &fret_core::FontId, want: &str) -> bool {
    match font {
        fret_core::FontId::Ui => want == "ui",
        fret_core::FontId::Serif => want == "serif",
        fret_core::FontId::Monospace => want == "monospace",
        fret_core::FontId::Family(name) => {
            want.strip_prefix("family:") == Some(name.as_str()) || want == name.as_str()
        }
    }
}

fn text_wrap_as_str(wrap: fret_core::TextWrap) -> &'static str {
    match wrap {
        fret_core::TextWrap::None => "none",
        fret_core::TextWrap::Word => "word",
        fret_core::TextWrap::Balance => "balance",
        fret_core::TextWrap::WordBreak => "word_break",
        fret_core::TextWrap::Grapheme => "grapheme",
    }
}

fn text_overflow_as_str(overflow: fret_core::TextOverflow) -> &'static str {
    match overflow {
        fret_core::TextOverflow::Clip => "clip",
        fret_core::TextOverflow::Ellipsis => "ellipsis",
    }
}

fn font_trace_family_class_matches(
    have: fret_core::RendererTextFontTraceFamilyClass,
    want: fret_diag_protocol::UiRenderTextFontTraceFamilyClassV1,
) -> bool {
    use fret_core::RendererTextFontTraceFamilyClass as H;
    use fret_diag_protocol::UiRenderTextFontTraceFamilyClassV1 as W;
    matches!(
        (have, want),
        (H::Requested, W::Requested)
            | (H::CommonFallback, W::CommonFallback)
            | (H::SystemFallback, W::SystemFallback)
            | (H::Unknown, W::Unknown)
    )
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

fn semantics_live_from_protocol(
    live: fret_diag_protocol::UiSemanticsLiveV1,
) -> fret_core::SemanticsLive {
    match live {
        fret_diag_protocol::UiSemanticsLiveV1::Off => fret_core::SemanticsLive::Off,
        fret_diag_protocol::UiSemanticsLiveV1::Polite => fret_core::SemanticsLive::Polite,
        fret_diag_protocol::UiSemanticsLiveV1::Assertive => fret_core::SemanticsLive::Assertive,
    }
}

fn semantics_action_value(
    actions: fret_core::SemanticsActions,
    action: fret_diag_protocol::UiSemanticsActionV1,
) -> bool {
    match action {
        fret_diag_protocol::UiSemanticsActionV1::Focus => actions.focus,
        fret_diag_protocol::UiSemanticsActionV1::Invoke => actions.invoke,
        fret_diag_protocol::UiSemanticsActionV1::SetValue => actions.set_value,
        fret_diag_protocol::UiSemanticsActionV1::Decrement => actions.decrement,
        fret_diag_protocol::UiSemanticsActionV1::Increment => actions.increment,
        fret_diag_protocol::UiSemanticsActionV1::ScrollBy => actions.scroll_by,
        fret_diag_protocol::UiSemanticsActionV1::SetTextSelection => actions.set_text_selection,
    }
}

fn semantics_relation_includes(
    source: &fret_core::SemanticsNode,
    relation: fret_diag_protocol::UiSemanticsRelationV1,
    target: fret_core::NodeId,
) -> bool {
    match relation {
        fret_diag_protocol::UiSemanticsRelationV1::ActiveDescendant => {
            source.active_descendant == Some(target)
        }
        fret_diag_protocol::UiSemanticsRelationV1::LabelledBy => {
            source.labelled_by.contains(&target)
        }
        fret_diag_protocol::UiSemanticsRelationV1::DescribedBy => {
            source.described_by.contains(&target)
        }
        fret_diag_protocol::UiSemanticsRelationV1::Controls => source.controls.contains(&target),
    }
}

fn semantics_relation_is_empty(
    source: &fret_core::SemanticsNode,
    relation: fret_diag_protocol::UiSemanticsRelationV1,
) -> bool {
    match relation {
        fret_diag_protocol::UiSemanticsRelationV1::ActiveDescendant => {
            source.active_descendant.is_none()
        }
        fret_diag_protocol::UiSemanticsRelationV1::LabelledBy => source.labelled_by.is_empty(),
        fret_diag_protocol::UiSemanticsRelationV1::DescribedBy => source.described_by.is_empty(),
        fret_diag_protocol::UiSemanticsRelationV1::Controls => source.controls.is_empty(),
    }
}

fn app_snapshot_field_equals(
    app_snapshot: Option<&serde_json::Value>,
    pointer: &str,
    want: &serde_json::Value,
) -> Option<bool> {
    let app_snapshot = app_snapshot?;
    Some(app_snapshot.pointer(pointer) == Some(want))
}

fn bounds_metric_value(bounds: Rect, metric: UiBoundsMetricV1) -> f32 {
    let left = bounds.origin.x.0;
    let top = bounds.origin.y.0;
    let width = bounds.size.width.0.max(0.0);
    let height = bounds.size.height.0.max(0.0);
    match metric {
        UiBoundsMetricV1::Left => left,
        UiBoundsMetricV1::Top => top,
        UiBoundsMetricV1::Right => left + width,
        UiBoundsMetricV1::Bottom => top + height,
        UiBoundsMetricV1::Width => width,
        UiBoundsMetricV1::Height => height,
        UiBoundsMetricV1::CenterX => left + width * 0.5,
        UiBoundsMetricV1::CenterY => top + height * 0.5,
    }
}

fn compare_px_delta(have: f32, comparison: UiComparisonV1, want: f32, eps: f32) -> bool {
    match comparison {
        UiComparisonV1::Eq => (have - want).abs() <= eps,
        UiComparisonV1::Ge => have + eps >= want,
        UiComparisonV1::Le => have <= want + eps,
    }
}

fn window_inner_size_approx_equal(
    window_bounds: Rect,
    width_px: f32,
    height_px: f32,
    eps_px: f32,
) -> bool {
    let eps = eps_px.max(0.0);
    width_px.is_finite()
        && height_px.is_finite()
        && eps.is_finite()
        && (window_bounds.size.width.0 - width_px).abs() <= eps
        && (window_bounds.size.height.0 - height_px).abs() <= eps
}

fn rect_within_rect(inner: Rect, outer: Rect, eps: f32) -> bool {
    let eps = eps.max(0.0);
    let inner_left = inner.origin.x.0;
    let inner_top = inner.origin.y.0;
    let inner_right = inner.origin.x.0 + inner.size.width.0.max(0.0);
    let inner_bottom = inner.origin.y.0 + inner.size.height.0.max(0.0);

    let outer_left = outer.origin.x.0;
    let outer_top = outer.origin.y.0;
    let outer_right = outer.origin.x.0 + outer.size.width.0.max(0.0);
    let outer_bottom = outer.origin.y.0 + outer.size.height.0.max(0.0);

    [
        inner_left,
        inner_top,
        inner_right,
        inner_bottom,
        outer_left,
        outer_top,
        outer_right,
        outer_bottom,
        eps,
    ]
    .into_iter()
    .all(f32::is_finite)
        && inner_left >= outer_left - eps
        && inner_top >= outer_top - eps
        && inner_right <= outer_right + eps
        && inner_bottom <= outer_bottom + eps
}

fn eval_predicate_without_semantics(
    window: AppWindowId,
    known_windows: &[AppWindowId],
    open_window_count: u32,
    app_snapshot: Option<&serde_json::Value>,
    platform_caps: Option<&fret_runtime::PlatformCapabilities>,
    window_style: Option<&fret_runtime::RunnerWindowStyleDiagnosticsStore>,
    platform_window_receiver: Option<&fret_runtime::RunnerPlatformWindowReceiverDiagnosticsStore>,
    docking: Option<&fret_runtime::DockingInteractionDiagnostics>,
    workspace: Option<&fret_runtime::WorkspaceInteractionDiagnostics>,
    dock_drag_runtime: Option<&DockDragRuntimeState>,
    pred: &UiPredicateV1,
) -> Option<bool> {
    match pred {
        UiPredicateV1::AppSnapshotFieldEquals { pointer, value } => {
            app_snapshot_field_equals(app_snapshot, pointer, value)
        }
        UiPredicateV1::RawSemanticsHiddenIs { .. }
        | UiPredicateV1::SemanticsRelationIncludes { .. }
        | UiPredicateV1::SemanticsRelationIsEmpty { .. } => None,
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
        UiPredicateV1::DockDragPayloadGhostVisibleIs { visible } => {
            Some(match docking.and_then(|d| d.dock_drag) {
                Some(drag) => (drag.dragging && drag.payload_ghost_visible) == *visible,
                None => !*visible,
            })
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
        UiPredicateV1::DockViewportCaptureActiveIs { active } => Some(
            docking
                .and_then(|d| d.viewport_capture)
                .is_some()
                == *active,
        ),
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
    if let Some(opacity) = want.opacity_alpha_u8
        && have.opacity.0 != opacity
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
    app_snapshot: Option<&serde_json::Value>,
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
    let select_relation_source = |target: &UiSelectorV1| {
        select_semantics_relation_endpoint_scoped(
            snapshot,
            window,
            element_runtime,
            target,
            scope_root,
        )
        .or_else(|| {
            select_semantics_relation_endpoint_scoped(snapshot, window, element_runtime, target, None)
        })
    };
    let select_relation_target = |target: &UiSelectorV1| {
        select_semantics_relation_endpoint_scoped(snapshot, window, element_runtime, target, None)
    };
    let select_raw_node = |target: &UiSelectorV1| {
        select_raw_semantics_node_scoped(snapshot, window, element_runtime, target, scope_root)
    };

    match pred {
        UiPredicateV1::AppSnapshotFieldEquals { pointer, value } => {
            app_snapshot_field_equals(app_snapshot, pointer, value).unwrap_or(false)
        }
        UiPredicateV1::Exists { target } => select_node(target).is_some(),
        UiPredicateV1::NotExists { target } => select_node(target).is_none(),
        UiPredicateV1::RawSemanticsHiddenIs { target, hidden } => {
            let Some(node) = select_raw_node(target) else {
                return false;
            };
            let index = SemanticsIndex::new(snapshot);
            let node_id = node.id.data().as_ffi();
            index.nearest_semantics_hidden_ancestor_or_self(node_id).is_some() == *hidden
        }
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
        UiPredicateV1::LevelIs { target, level } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.extra.level == Some(*level)
        }
        UiPredicateV1::CheckedIs { target, checked } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.flags.checked == Some(*checked)
        }
        UiPredicateV1::ExpandedIs { target, expanded } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.flags.expanded == *expanded
        }
        UiPredicateV1::SemanticsLiveIs { target, live } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.flags.live == live.map(semantics_live_from_protocol)
        }
        UiPredicateV1::SemanticsLiveAtomicIs {
            target,
            live_atomic,
        } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.flags.live_atomic == *live_atomic
        }
        UiPredicateV1::SelectedIs { target, selected } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.flags.selected == *selected
        }
        UiPredicateV1::DisabledIs { target, disabled } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.flags.disabled == *disabled
        }
        UiPredicateV1::ReadOnlyIs { target, read_only } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            node.flags.read_only == *read_only
        }
        UiPredicateV1::SemanticsActionIs {
            target,
            action,
            enabled,
        } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            semantics_action_value(node.actions, *action) == *enabled
        }
        UiPredicateV1::CapturedIs { target, captured } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            let is_captured = snapshot.captured == Some(node.id) || node.flags.captured;
            is_captured == *captured
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
        UiPredicateV1::SemanticsRelationIncludes {
            source,
            relation,
            target,
        } => {
            let Some(source_node) = select_relation_source(source) else {
                return false;
            };
            let Some(target_node) = select_relation_target(target) else {
                return false;
            };
            semantics_relation_includes(source_node, *relation, target_node.id)
        }
        UiPredicateV1::SemanticsRelationIsEmpty { source, relation } => {
            let Some(source_node) = select_relation_source(source) else {
                return false;
            };
            semantics_relation_is_empty(source_node, *relation)
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
        UiPredicateV1::RenderTextFontTraceEntriesMatchingGe {
            min,
            text_contains,
            font,
            wrap,
            overflow,
            missing_glyphs,
            family_contains,
            family_class,
        } => {
            let Some(trace) = render_text_font_trace else {
                return false;
            };
            trace
                .entries
                .iter()
                .filter(|entry| {
                    font_trace_entry_matches(
                        entry,
                        text_contains.as_deref(),
                        font.as_deref(),
                        wrap.as_deref(),
                        overflow.as_deref(),
                        *missing_glyphs,
                        family_contains.as_deref(),
                        *family_class,
                    )
                })
                .count() as u64
                >= *min
        }
        UiPredicateV1::TextFontStackKeyStable { stable_frames } => {
            text_font_stack_key_stable_frames >= *stable_frames
        }
        UiPredicateV1::FontCatalogPopulated => font_catalog_populated,
        UiPredicateV1::SystemFontRescanIdle => system_font_rescan_idle,
        UiPredicateV1::AssetLoadMissingBundleAssetRequestsGe { .. }
        | UiPredicateV1::AssetLoadStaleManifestRequestsGe { .. }
        | UiPredicateV1::AssetLoadUnsupportedFileRequestsGe { .. }
        | UiPredicateV1::AssetLoadUnsupportedUrlRequestsGe { .. }
        | UiPredicateV1::AssetLoadExternalReferenceUnavailableRequestsGe { .. }
        | UiPredicateV1::AssetLoadRevisionChangeRequestsGe { .. }
        | UiPredicateV1::AssetLoadRecentOutcomeSeen { .. }
        | UiPredicateV1::AssetLoadRecentRevisionTransitionSeen { .. }
        | UiPredicateV1::BundledFontBaselineSourceIs { .. }
        | UiPredicateV1::SvgTextBridgeSelectionMissesGe { .. }
        | UiPredicateV1::SvgTextBridgeMissingGlyphsGe { .. }
        | UiPredicateV1::SvgTextBridgeDiagnosticsCleanIs { .. }
        | UiPredicateV1::SvgTextBridgeFallbackSeen { .. }
        | UiPredicateV1::RendererFontEnvironmentRevisionGe { .. }
        | UiPredicateV1::RendererFontSourceLaneSeen { .. }
        | UiPredicateV1::RendererFontSourceAssetKeySeen { .. }
        | UiPredicateV1::AssetReloadEpochGe { .. }
        | UiPredicateV1::AssetReloadConfiguredBackendIs { .. }
        | UiPredicateV1::AssetReloadActiveBackendIs { .. }
        | UiPredicateV1::AssetReloadFallbackReasonIs { .. }
        | UiPredicateV1::VirtualListWindowShiftSamplesLenLe { .. }
        | UiPredicateV1::VirtualListWindowShiftSamplesMatchingGe { .. }
        | UiPredicateV1::VirtualListWindowsMatchingGe { .. }
        | UiPredicateV1::RetainedVirtualListReconcilesMatchingGe { .. }
        | UiPredicateV1::ScrollHandleChangesMatchingGe { .. } => false,
        UiPredicateV1::RunnerAccessibilityActivated => false,
        UiPredicateV1::WindowInnerSizeApproxEqual {
            width_px,
            height_px,
            eps_px,
        } => window_inner_size_approx_equal(window_bounds, *width_px, *height_px, *eps_px),
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
        UiPredicateV1::TextInputImeCursorAreaWithinBounds {
            target,
            padding_px,
            padding_insets_px,
            eps_px,
        } => {
            let Some(node) = select_node(target) else {
                return false;
            };
            let Some(cursor_area) = text_input_snapshot.and_then(|snapshot| snapshot.ime_cursor_area)
            else {
                return false;
            };

            let pad = padding_px.max(0.0);
            let pad_insets = padding_insets_px.unwrap_or_else(|| UiPaddingInsetsV1::uniform(0.0));
            let eps = eps_px.max(0.0);

            let bounds = node.bounds;
            let bounds_left = bounds.origin.x.0 + pad + pad_insets.left_px.max(0.0);
            let bounds_top = bounds.origin.y.0 + pad + pad_insets.top_px.max(0.0);
            let bounds_right =
                bounds.origin.x.0 + bounds.size.width.0 - pad - pad_insets.right_px.max(0.0);
            let bounds_bottom =
                bounds.origin.y.0 + bounds.size.height.0 - pad - pad_insets.bottom_px.max(0.0);

            let area_left = cursor_area.origin.x.0;
            let area_top = cursor_area.origin.y.0;
            let area_right = cursor_area.origin.x.0 + cursor_area.size.width.0.max(0.0);
            let area_bottom = cursor_area.origin.y.0 + cursor_area.size.height.0.max(0.0);

            area_left >= bounds_left - eps
                && area_top >= bounds_top - eps
                && area_right <= bounds_right + eps
                && area_bottom <= bounds_bottom + eps
        }
        UiPredicateV1::TextInputVisibleTextWithinViewport { eps_px } => {
            let Some(visual) = text_input_snapshot.and_then(|snapshot| snapshot.visual) else {
                return false;
            };
            let eps = eps_px.max(0.0);

            let Some(visible_text_bounds) = visual.visible_text_bounds else {
                return visual.content_width_px.max(0.0) <= eps;
            };

            rect_within_rect(visible_text_bounds, visual.viewport_bounds, eps)
        }
        UiPredicateV1::TextInputHorizontalOffsetInRange { eps_px } => {
            let Some(visual) = text_input_snapshot.and_then(|snapshot| snapshot.visual) else {
                return false;
            };
            let eps = eps_px.max(0.0);
            let offset = visual.offset_x_px;
            let max_offset = visual.max_offset_x_px;

            [
                visual.content_width_px,
                visual.viewport_width_px,
                offset,
                max_offset,
                eps,
            ]
            .into_iter()
            .all(f32::is_finite)
                && visual.content_width_px >= -eps
                && visual.viewport_width_px >= -eps
                && max_offset >= -eps
                && offset >= -eps
                && offset <= max_offset + eps
        }
        UiPredicateV1::TextInputHorizontalOverflowIs {
            overflowing,
            eps_px,
        } => {
            let Some(visual) = text_input_snapshot.and_then(|snapshot| snapshot.visual) else {
                return false;
            };
            let eps = eps_px.max(0.0);
            if !visual.content_width_px.is_finite()
                || !visual.viewport_width_px.is_finite()
                || !eps.is_finite()
            {
                return false;
            }
            (visual.content_width_px > visual.viewport_width_px + eps) == *overflowing
        }
        UiPredicateV1::TextInputViewportCoversTextHeight { eps_px } => {
            let Some(visual) = text_input_snapshot.and_then(|snapshot| snapshot.visual) else {
                return false;
            };
            let eps = eps_px.max(0.0);
            let viewport_h = visual.viewport_bounds.size.height.0.max(0.0);
            let text_h = visual.unclipped_text_bounds.size.height.0.max(0.0);

            viewport_h.is_finite()
                && text_h.is_finite()
                && eps.is_finite()
                && viewport_h + eps >= text_h
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
        UiPredicateV1::BoundsMetricDelta {
            a,
            b,
            metric,
            comparison,
            value_px,
            eps_px,
        } => {
            let Some(a) = select_node(a) else {
                return false;
            };
            let Some(b) = select_node(b) else {
                return false;
            };

            let delta =
                bounds_metric_value(a.bounds, *metric) - bounds_metric_value(b.bounds, *metric);
            compare_px_delta(delta, *comparison, *value_px, eps_px.max(0.0))
        }
        UiPredicateV1::BoundsMetricPairDelta {
            a,
            b,
            a_metric,
            b_metric,
            comparison,
            value_px,
            eps_px,
        } => {
            let Some(a) = select_node(a) else {
                return false;
            };
            let Some(b) = select_node(b) else {
                return false;
            };

            let delta = bounds_metric_value(a.bounds, *a_metric)
                - bounds_metric_value(b.bounds, *b_metric);
            compare_px_delta(delta, *comparison, *value_px, eps_px.max(0.0))
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
        UiPredicateV1::DockDragPayloadGhostVisibleIs { visible } => {
            match docking.and_then(|d| d.dock_drag) {
                Some(drag) => (drag.dragging && drag.payload_ghost_visible) == *visible,
                None => !*visible,
            }
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
        UiPredicateV1::DockViewportCaptureActiveIs { active } => {
            docking.and_then(|d| d.viewport_capture).is_some() == *active
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
        UiPredicateV1::InputPointerCaptureActiveIs { .. } => false,
    }
}

#[cfg(test)]
mod predicate_tests {
    use super::*;
    use fret_core::{
        NodeId, Point, PointerId, Px, Rect, RenderTargetId, SemanticsActions, SemanticsFlags,
        SemanticsLive, SemanticsNode, SemanticsRole, SemanticsRoot, SemanticsSnapshot, Size,
    };
    use fret_diag_protocol::UiSemanticsRelationV1;
    use slotmap::KeyData;

    fn node_id(id: u64) -> NodeId {
        NodeId::from(KeyData::from_ffi(id))
    }

    fn window_id(id: u64) -> AppWindowId {
        AppWindowId::from(KeyData::from_ffi(id))
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    fn semantics_node(id: u64, test_id: &str, captured: bool) -> SemanticsNode {
        SemanticsNode {
            id: node_id(id),
            parent: None,
            role: SemanticsRole::ScrollBar,
            bounds: rect(0.0, 0.0, 10.0, 50.0),
            flags: SemanticsFlags {
                captured,
                ..Default::default()
            },
            test_id: Some(test_id.to_string()),
            active_descendant: None,
            pos_in_set: None,
            set_size: None,
            label: None,
            value: None,
            extra: Default::default(),
            text_selection: None,
            text_composition: None,
            actions: SemanticsActions::default(),
            labelled_by: Vec::new(),
            described_by: Vec::new(),
            controls: Vec::new(),
            inline_spans: Vec::new(),
        }
    }

    #[test]
    fn app_snapshot_field_equals_matches_json_pointer_value() {
        let app_snapshot = serde_json::json!({
            "shell": {
                "settings_open": true,
                "last_action": "cmd.settings"
            }
        });

        let ok = eval_predicate_without_semantics(
            window_id(1),
            &[],
            1,
            Some(&app_snapshot),
            None,
            None,
            None,
            None,
            None,
            None,
            &UiPredicateV1::AppSnapshotFieldEquals {
                pointer: "/shell/settings_open".to_string(),
                value: serde_json::json!(true),
            },
        );

        assert_eq!(ok, Some(true));
    }

    #[test]
    fn app_snapshot_field_equals_returns_false_for_missing_field() {
        let app_snapshot = serde_json::json!({
            "shell": {
                "settings_open": false
            }
        });

        let ok = eval_predicate_without_semantics(
            window_id(1),
            &[],
            1,
            Some(&app_snapshot),
            None,
            None,
            None,
            None,
            None,
            None,
            &UiPredicateV1::AppSnapshotFieldEquals {
                pointer: "/shell/last_action".to_string(),
                value: serde_json::json!("cmd.settings"),
            },
        );

        assert_eq!(ok, Some(false));
    }

    #[test]
    fn dock_viewport_capture_active_matches_docking_snapshot() {
        let docking = fret_runtime::DockingInteractionDiagnostics {
            viewport_capture: Some(fret_runtime::ViewportCaptureDiagnostics {
                pointer_id: PointerId(1),
                target: RenderTargetId::default(),
            }),
            ..Default::default()
        };

        assert_eq!(
            eval_predicate_without_semantics(
                window_id(1),
                &[],
                1,
                None,
                None,
                None,
                None,
                Some(&docking),
                None,
                None,
                &UiPredicateV1::DockViewportCaptureActiveIs { active: true },
            ),
            Some(true)
        );
        assert_eq!(
            eval_predicate_without_semantics(
                window_id(1),
                &[],
                1,
                None,
                None,
                None,
                None,
                Some(&docking),
                None,
                None,
                &UiPredicateV1::DockViewportCaptureActiveIs { active: false },
            ),
            Some(false)
        );
    }

    #[test]
    fn captured_is_matches_semantics_capture_owner() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Generic;
        let mut scrollbar = semantics_node(2, "scrollbar", false);
        scrollbar.parent = Some(node_id(1));
        let mut viewport = semantics_node(3, "viewport", false);
        viewport.parent = Some(node_id(1));

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: Some(node_id(2)),
            nodes: vec![root, scrollbar, viewport],
        };

        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::CapturedIs {
                target: UiSelectorV1::TestId {
                    id: "scrollbar".to_string(),
                    root_z_index: None,
                },
                captured: true,
            },
        ));
        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::CapturedIs {
                target: UiSelectorV1::TestId {
                    id: "viewport".to_string(),
                    root_z_index: None,
                },
                captured: false,
            },
        ));
    }

    #[test]
    fn expanded_is_matches_semantics_expanded_flag() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut trigger = semantics_node(2, "accordion-trigger", false);
        trigger.parent = Some(root.id);
        trigger.role = SemanticsRole::Button;
        trigger.flags.expanded = true;

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root, trigger],
        };

        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::ExpandedIs {
                target: UiSelectorV1::TestId {
                    id: "accordion-trigger".to_string(),
                    root_z_index: None,
                },
                expanded: true,
            },
        ));
    }

    #[test]
    fn collection_metadata_predicates_match_semantics_position_fields() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut item = semantics_node(2, "command-item-code-editor", false);
        item.parent = Some(root.id);
        item.role = SemanticsRole::ListBoxOption;
        item.pos_in_set = Some(23);
        item.set_size = Some(23);

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root, item],
        };
        let target = UiSelectorV1::TestId {
            id: "command-item-code-editor".to_string(),
            root_z_index: None,
        };

        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::PosInSetIs {
                target: target.clone(),
                pos_in_set: 23,
            },
        ));
        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::SetSizeIs {
                target,
                set_size: 23,
            },
        ));
    }

    #[test]
    fn semantics_relation_predicates_match_semantics_edges() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut source = semantics_node(2, "relation-source", false);
        source.parent = Some(root.id);
        source.role = SemanticsRole::ComboBox;
        let mut label = semantics_node(3, "relation-label", false);
        label.parent = Some(root.id);
        let mut description = semantics_node(4, "relation-description", false);
        description.parent = Some(root.id);
        let mut controlled = semantics_node(5, "relation-controlled", false);
        controlled.parent = Some(root.id);
        let mut active = semantics_node(6, "relation-active", false);
        active.parent = Some(root.id);
        active.role = SemanticsRole::ListBoxOption;
        source.active_descendant = Some(active.id);
        source.labelled_by.push(label.id);
        source.described_by.push(description.id);
        source.controls.push(controlled.id);

        let present_snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![
                root.clone(),
                source.clone(),
                label,
                description,
                controlled,
                active,
            ],
        };
        let source_selector = UiSelectorV1::TestId {
            id: "relation-source".to_string(),
            root_z_index: None,
        };
        let eval = |snapshot: &SemanticsSnapshot, predicate: &UiPredicateV1| {
            eval_predicate(
                snapshot,
                rect(0.0, 0.0, 100.0, 100.0),
                window,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                &[],
                1,
                None,
                None,
                None,
                None,
                None,
                None,
                0,
                false,
                true,
                predicate,
            )
        };

        assert!(eval(
            &present_snapshot,
            &UiPredicateV1::SemanticsRelationIncludes {
                source: source_selector.clone(),
                relation: UiSemanticsRelationV1::ActiveDescendant,
                target: UiSelectorV1::TestId {
                    id: "relation-active".to_string(),
                    root_z_index: None,
                },
            },
        ));
        assert!(eval(
            &present_snapshot,
            &UiPredicateV1::SemanticsRelationIncludes {
                source: source_selector.clone(),
                relation: UiSemanticsRelationV1::LabelledBy,
                target: UiSelectorV1::TestId {
                    id: "relation-label".to_string(),
                    root_z_index: None,
                },
            },
        ));
        assert!(eval(
            &present_snapshot,
            &UiPredicateV1::SemanticsRelationIncludes {
                source: source_selector.clone(),
                relation: UiSemanticsRelationV1::DescribedBy,
                target: UiSelectorV1::TestId {
                    id: "relation-description".to_string(),
                    root_z_index: None,
                },
            },
        ));
        assert!(eval(
            &present_snapshot,
            &UiPredicateV1::SemanticsRelationIncludes {
                source: source_selector.clone(),
                relation: UiSemanticsRelationV1::Controls,
                target: UiSelectorV1::TestId {
                    id: "relation-controlled".to_string(),
                    root_z_index: None,
                },
            },
        ));

        source.active_descendant = None;
        source.labelled_by.clear();
        source.described_by.clear();
        source.controls.clear();
        let detached_snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root, source],
        };

        assert!(eval(
            &detached_snapshot,
            &UiPredicateV1::SemanticsRelationIsEmpty {
                source: source_selector.clone(),
                relation: UiSemanticsRelationV1::ActiveDescendant,
            },
        ));
        assert!(eval(
            &detached_snapshot,
            &UiPredicateV1::SemanticsRelationIsEmpty {
                source: source_selector.clone(),
                relation: UiSemanticsRelationV1::LabelledBy,
            },
        ));
        assert!(eval(
            &detached_snapshot,
            &UiPredicateV1::SemanticsRelationIsEmpty {
                source: source_selector.clone(),
                relation: UiSemanticsRelationV1::DescribedBy,
            },
        ));
        assert!(eval(
            &detached_snapshot,
            &UiPredicateV1::SemanticsRelationIsEmpty {
                source: source_selector,
                relation: UiSemanticsRelationV1::Controls,
            },
        ));
    }

    #[test]
    fn semantics_relation_includes_can_cross_scope_roots() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut page_root = semantics_node(2, "page-root", false);
        page_root.parent = Some(root.id);
        let mut overlay_root = semantics_node(3, "overlay-root", false);
        overlay_root.parent = Some(root.id);
        let mut source = semantics_node(4, "relation-source", false);
        source.parent = Some(page_root.id);
        source.role = SemanticsRole::ComboBox;
        let mut target = semantics_node(5, "relation-target", false);
        target.parent = Some(overlay_root.id);
        target.role = SemanticsRole::ListBox;
        source.controls.push(target.id);

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: root.id,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root, page_root.clone(), overlay_root, source, target],
        };

        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            Some(page_root.id.data().as_ffi()),
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::SemanticsRelationIncludes {
                source: UiSelectorV1::TestId {
                    id: "relation-source".to_string(),
                    root_z_index: None,
                },
                relation: UiSemanticsRelationV1::Controls,
                target: UiSelectorV1::TestId {
                    id: "relation-target".to_string(),
                    root_z_index: None,
                },
            },
        ));
    }

    #[test]
    fn semantics_relation_includes_can_cross_modal_barrier_to_underlay_source() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut underlay_root = semantics_node(2, "underlay-root", false);
        underlay_root.parent = Some(root.id);
        let mut barrier_root = semantics_node(3, "barrier-root", false);
        barrier_root.parent = Some(root.id);
        let mut trigger = semantics_node(4, "select-trigger", false);
        trigger.parent = Some(underlay_root.id);
        trigger.role = SemanticsRole::ComboBox;
        let mut listbox = semantics_node(5, "select-listbox", false);
        listbox.parent = Some(barrier_root.id);
        listbox.role = SemanticsRole::ListBox;
        trigger.controls.push(listbox.id);
        listbox.labelled_by.push(trigger.id);

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: root.id,
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: Some(barrier_root.id),
            focus_barrier_root: Some(barrier_root.id),
            focus: Some(listbox.id),
            captured: None,
            nodes: vec![root, underlay_root, barrier_root.clone(), trigger, listbox],
        };

        let trigger_selector = UiSelectorV1::TestId {
            id: "select-trigger".to_string(),
            root_z_index: None,
        };
        let listbox_selector = UiSelectorV1::TestId {
            id: "select-listbox".to_string(),
            root_z_index: None,
        };

        assert!(
            !eval_predicate(
                &snapshot,
                rect(0.0, 0.0, 100.0, 100.0),
                window,
                Some(barrier_root.id.data().as_ffi()),
                None,
                None,
                None,
                None,
                None,
                None,
                &[],
                1,
                None,
                None,
                None,
                None,
                None,
                None,
                0,
                false,
                true,
                &UiPredicateV1::Exists {
                    target: trigger_selector.clone(),
                },
            ),
            "ordinary selectors should still respect the modal barrier scope"
        );
        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            Some(barrier_root.id.data().as_ffi()),
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::SemanticsRelationIncludes {
                source: trigger_selector.clone(),
                relation: UiSemanticsRelationV1::Controls,
                target: listbox_selector.clone(),
            },
        ));
        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            Some(barrier_root.id.data().as_ffi()),
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::SemanticsRelationIncludes {
                source: listbox_selector,
                relation: UiSemanticsRelationV1::LabelledBy,
                target: trigger_selector,
            },
        ));
    }

    #[test]
    fn level_is_matches_semantics_hierarchy_level() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut item = semantics_node(2, "tree-row-folder", false);
        item.parent = Some(root.id);
        item.role = SemanticsRole::TreeItem;
        item.extra.level = Some(2);

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root, item],
        };

        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::LevelIs {
                target: UiSelectorV1::TestId {
                    id: "tree-row-folder".to_string(),
                    root_z_index: None,
                },
                level: 2,
            },
        ));
    }

    #[test]
    fn disabled_is_matches_semantics_disabled_flag() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut button = semantics_node(2, "pagination-prev", false);
        button.parent = Some(root.id);
        button.role = SemanticsRole::Button;
        button.flags.disabled = true;

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root, button],
        };

        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::DisabledIs {
                target: UiSelectorV1::TestId {
                    id: "pagination-prev".to_string(),
                    root_z_index: None,
                },
                disabled: true,
            },
        ));
    }

    #[test]
    fn read_only_is_matches_semantics_read_only_flag() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut switch = semantics_node(2, "readonly-switch", false);
        switch.parent = Some(root.id);
        switch.role = SemanticsRole::Switch;
        switch.flags.read_only = true;

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root, switch],
        };

        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::ReadOnlyIs {
                target: UiSelectorV1::TestId {
                    id: "readonly-switch".to_string(),
                    root_z_index: None,
                },
                read_only: true,
            },
        ));
    }

    #[test]
    fn semantics_action_is_matches_all_exported_action_flags() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut slider = semantics_node(2, "volume", false);
        slider.parent = Some(root.id);
        slider.role = SemanticsRole::Slider;
        slider.actions = SemanticsActions {
            focus: true,
            invoke: false,
            set_value: true,
            decrement: true,
            increment: true,
            scroll_by: false,
            set_text_selection: false,
        };

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root, slider],
        };

        for (action, enabled) in [
            (fret_diag_protocol::UiSemanticsActionV1::Focus, true),
            (fret_diag_protocol::UiSemanticsActionV1::Invoke, false),
            (fret_diag_protocol::UiSemanticsActionV1::SetValue, true),
            (fret_diag_protocol::UiSemanticsActionV1::Decrement, true),
            (fret_diag_protocol::UiSemanticsActionV1::Increment, true),
            (fret_diag_protocol::UiSemanticsActionV1::ScrollBy, false),
            (
                fret_diag_protocol::UiSemanticsActionV1::SetTextSelection,
                false,
            ),
        ] {
            assert!(
                eval_predicate(
                    &snapshot,
                    rect(0.0, 0.0, 100.0, 100.0),
                    window,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    &[],
                    1,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    0,
                    false,
                    true,
                    &UiPredicateV1::SemanticsActionIs {
                        target: UiSelectorV1::TestId {
                            id: "volume".to_string(),
                            root_z_index: None,
                        },
                        action,
                        enabled,
                    },
                ),
                "expected {action:?} to be {enabled}"
            );
        }
    }

    #[test]
    fn semantics_live_predicates_match_semantics_live_flags() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut status = semantics_node(2, "toast-viewport", false);
        status.parent = Some(root.id);
        status.role = SemanticsRole::Viewport;
        status.flags.live = Some(SemanticsLive::Polite);
        status.flags.live_atomic = false;

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root, status],
        };
        let target = UiSelectorV1::TestId {
            id: "toast-viewport".to_string(),
            root_z_index: None,
        };

        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::SemanticsLiveIs {
                target: target.clone(),
                live: Some(fret_diag_protocol::UiSemanticsLiveV1::Polite),
            },
        ));
        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::SemanticsLiveAtomicIs {
                target,
                live_atomic: false,
            },
        ));
    }

    #[test]
    fn render_text_font_trace_matching_predicate_matches_renderer_text_facts() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root],
        };
        let trace = fret_core::RendererTextFontTraceSnapshot {
            frame_id: fret_core::FrameId(10),
            entries: vec![
                fret_core::RendererTextFontTraceEntry {
                    text_preview: "Enterprise Observability Platform With Long Label".to_string(),
                    text_len_bytes: 55,
                    font: fret_core::FontId::Ui,
                    font_size: Px(14.0),
                    scale_factor: 1.0,
                    wrap: fret_core::TextWrap::None,
                    overflow: fret_core::TextOverflow::Ellipsis,
                    max_width: Some(Px(160.0)),
                    locale_bcp47: None,
                    missing_glyphs: 0,
                    families: vec![fret_core::RendererTextFontTraceFamilyUsage {
                        family: "Inter".to_string(),
                        glyphs: 42,
                        missing_glyphs: 0,
                        class: fret_core::RendererTextFontTraceFamilyClass::Requested,
                    }],
                },
                fret_core::RendererTextFontTraceEntry {
                    text_preview: "Short".to_string(),
                    text_len_bytes: 5,
                    font: fret_core::FontId::Ui,
                    font_size: Px(14.0),
                    scale_factor: 1.0,
                    wrap: fret_core::TextWrap::None,
                    overflow: fret_core::TextOverflow::Clip,
                    max_width: None,
                    locale_bcp47: None,
                    missing_glyphs: 0,
                    families: Vec::new(),
                },
            ],
        };

        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            Some(&trace),
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::RenderTextFontTraceEntriesMatchingGe {
                min: 1,
                text_contains: Some("Enterprise Observability".to_string()),
                font: Some("ui".to_string()),
                wrap: Some("none".to_string()),
                overflow: Some("ellipsis".to_string()),
                missing_glyphs: Some(0),
                family_contains: Some("Inter".to_string()),
                family_class: Some(
                    fret_diag_protocol::UiRenderTextFontTraceFamilyClassV1::Requested,
                ),
            },
        ));
        assert!(!eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            Some(&trace),
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::RenderTextFontTraceEntriesMatchingGe {
                min: 1,
                text_contains: Some("Enterprise Observability".to_string()),
                font: Some("ui".to_string()),
                wrap: Some("none".to_string()),
                overflow: Some("clip".to_string()),
                missing_glyphs: Some(0),
                family_contains: Some("Inter".to_string()),
                family_class: Some(
                    fret_diag_protocol::UiRenderTextFontTraceFamilyClassV1::Requested,
                ),
            },
        ));
    }

    #[test]
    fn default_predicates_exclude_semantics_hidden_subtrees_but_flags_remain_observable() {
        let window = window_id(1);
        let mut root = semantics_node(1, "root", false);
        root.role = SemanticsRole::Window;
        let mut hidden_parent = semantics_node(2, "hidden-parent", false);
        hidden_parent.parent = Some(root.id);
        hidden_parent.flags.hidden = true;
        let mut hidden_child = semantics_node(3, "hidden-child", false);
        hidden_child.parent = Some(hidden_parent.id);

        let snapshot = SemanticsSnapshot {
            window,
            roots: vec![SemanticsRoot {
                root: node_id(1),
                visible: true,
                blocks_underlay_input: false,
                hit_testable: true,
                z_index: 0,
            }],
            barrier_root: None,
            focus_barrier_root: None,
            focus: None,
            captured: None,
            nodes: vec![root, hidden_parent, hidden_child],
        };
        let hidden_parent_selector = UiSelectorV1::TestId {
            id: "hidden-parent".to_string(),
            root_z_index: None,
        };
        let hidden_child_selector = UiSelectorV1::TestId {
            id: "hidden-child".to_string(),
            root_z_index: None,
        };

        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::NotExists {
                target: hidden_parent_selector.clone(),
            },
        ));
        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::NotExists {
                target: hidden_child_selector.clone(),
            },
        ));
        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::RawSemanticsHiddenIs {
                target: hidden_parent_selector,
                hidden: true,
            },
        ));
        assert!(eval_predicate(
            &snapshot,
            rect(0.0, 0.0, 100.0, 100.0),
            window,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &[],
            1,
            None,
            None,
            None,
            None,
            None,
            None,
            0,
            false,
            true,
            &UiPredicateV1::RawSemanticsHiddenIs {
                target: hidden_child_selector,
                hidden: true,
            },
        ));
    }

    #[test]
    fn window_style_effective_matches_opacity_alpha() {
        let have = fret_runtime::RunnerWindowStyleEffectiveSnapshotV1 {
            opacity: fret_runtime::WindowOpacity(128),
            ..Default::default()
        };

        assert!(window_style_effective_matches(
            &have,
            &UiWindowStyleMatchV1 {
                opacity_alpha_u8: Some(128),
                ..Default::default()
            }
        ));
        assert!(!window_style_effective_matches(
            &have,
            &UiWindowStyleMatchV1 {
                opacity_alpha_u8: Some(255),
                ..Default::default()
            }
        ));
    }
}
