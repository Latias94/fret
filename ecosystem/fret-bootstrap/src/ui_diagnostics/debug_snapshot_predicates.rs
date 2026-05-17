use super::*;

pub(super) fn eval_debug_snapshot_predicate_from_recent_snapshot(
    svc: &UiDiagnosticsService,
    window: AppWindowId,
    predicate: &UiPredicateV1,
    max_age_ms: u64,
) -> Option<bool> {
    let ring = svc.per_window.get(&window)?;
    let snapshot = ring.snapshots.back()?;
    let age_ms = unix_ms_now().saturating_sub(snapshot.timestamp_unix_ms);
    if age_ms > max_age_ms {
        return None;
    }
    if predicate_uses_debug_snapshot_history(predicate) {
        if let Some(ok) = eval_debug_snapshot_predicate_from_ring(ring, predicate) {
            return Some(ok);
        }
    }
    eval_debug_snapshot_predicate(&snapshot.debug, predicate)
}

fn predicate_uses_debug_snapshot_history(predicate: &UiPredicateV1) -> bool {
    matches!(
        predicate,
        UiPredicateV1::VirtualListWindowShiftSamplesLenLe { .. }
            | UiPredicateV1::VirtualListWindowShiftSamplesMatchingGe { .. }
            | UiPredicateV1::VirtualListWindowsMatchingGe { .. }
            | UiPredicateV1::RetainedVirtualListReconcilesMatchingGe { .. }
            | UiPredicateV1::ScrollHandleChangesMatchingGe { .. }
    )
}

fn eval_debug_snapshot_predicate_from_ring(
    ring: &WindowRing,
    predicate: &UiPredicateV1,
) -> Option<bool> {
    match predicate {
        UiPredicateV1::VirtualListWindowShiftSamplesLenLe { max } => {
            let samples = ring.snapshots.iter().fold(0_u64, |total, snapshot| {
                total.saturating_add(snapshot.debug.virtual_list_window_shift_samples.len() as u64)
            });
            Some(samples <= *max)
        }
        UiPredicateV1::VirtualListWindowShiftSamplesMatchingGe {
            min,
            shift_kind,
            reason,
            apply_mode,
            source,
            invalidation_detail,
        } => {
            let samples = ring.snapshots.iter().fold(0_u64, |total, snapshot| {
                total.saturating_add(count_matching_virtual_list_shift_samples(
                    &snapshot.debug,
                    shift_kind.as_deref(),
                    reason.as_deref(),
                    apply_mode.as_deref(),
                    source.as_deref(),
                    invalidation_detail.as_deref(),
                ))
            });
            Some(samples >= *min)
        }
        UiPredicateV1::VirtualListWindowsMatchingGe {
            min,
            shift_kind,
            reason,
            apply_mode,
            source,
            invalidation_detail,
        } => {
            let windows = ring.snapshots.iter().fold(0_u64, |total, snapshot| {
                total.saturating_add(count_matching_virtual_list_windows(
                    &snapshot.debug,
                    shift_kind.as_deref(),
                    reason.as_deref(),
                    apply_mode.as_deref(),
                    source.as_deref(),
                    invalidation_detail.as_deref(),
                ))
            });
            Some(windows >= *min)
        }
        UiPredicateV1::RetainedVirtualListReconcilesMatchingGe {
            min,
            reconcile_kind,
            attached_items_min,
            detached_items_min,
            reused_from_keep_alive_items_min,
            kept_alive_items_min,
        } => {
            let reconciles = ring.snapshots.iter().fold(0_u64, |total, snapshot| {
                total.saturating_add(count_matching_retained_virtual_list_reconciles(
                    &snapshot.debug,
                    reconcile_kind.as_deref(),
                    *attached_items_min,
                    *detached_items_min,
                    *reused_from_keep_alive_items_min,
                    *kept_alive_items_min,
                ))
            });
            Some(reconciles >= *min)
        }
        UiPredicateV1::ScrollHandleChangesMatchingGe {
            min,
            change_kind,
            offset_y_min,
            prev_offset_y_max,
            offset_changed,
            upgraded_to_layout_bindings_min,
        } => {
            let changes = ring.snapshots.iter().fold(0_u64, |total, snapshot| {
                total.saturating_add(count_matching_scroll_handle_changes(
                    &snapshot.debug,
                    change_kind.as_deref(),
                    *offset_y_min,
                    *prev_offset_y_max,
                    *offset_changed,
                    *upgraded_to_layout_bindings_min,
                ))
            });
            Some(changes >= *min)
        }
        _ => None,
    }
}

pub(super) fn eval_debug_snapshot_predicate(
    debug: &UiTreeDebugSnapshotV1,
    predicate: &UiPredicateV1,
) -> Option<bool> {
    if let Some(ok) = eval_input_arbitration_predicate_from_debug_snapshot(debug, predicate) {
        return Some(ok);
    }

    if let Some(ok) = eval_virtual_list_predicate_from_debug_snapshot(debug, predicate) {
        return Some(ok);
    }

    if let Some(ok) = debug
        .docking_interaction
        .as_ref()
        .and_then(|docking| eval_docking_predicate_from_debug_snapshot(docking, predicate))
    {
        return Some(ok);
    }

    debug
        .resource_loading
        .as_ref()
        .and_then(|resource_loading| {
            eval_resource_loading_predicate_from_debug_snapshot(resource_loading, predicate)
        })
}

fn eval_input_arbitration_predicate_from_debug_snapshot(
    debug: &UiTreeDebugSnapshotV1,
    predicate: &UiPredicateV1,
) -> Option<bool> {
    match predicate {
        UiPredicateV1::InputPointerCaptureActiveIs { active } => {
            Some(debug.input_arbitration.pointer_capture_active == *active)
        }
        _ => None,
    }
}

fn eval_virtual_list_predicate_from_debug_snapshot(
    debug: &UiTreeDebugSnapshotV1,
    predicate: &UiPredicateV1,
) -> Option<bool> {
    match predicate {
        UiPredicateV1::VirtualListWindowShiftSamplesLenLe { max } => {
            Some((debug.virtual_list_window_shift_samples.len() as u64) <= *max)
        }
        UiPredicateV1::VirtualListWindowShiftSamplesMatchingGe {
            min,
            shift_kind,
            reason,
            apply_mode,
            source,
            invalidation_detail,
        } => Some(
            count_matching_virtual_list_shift_samples(
                debug,
                shift_kind.as_deref(),
                reason.as_deref(),
                apply_mode.as_deref(),
                source.as_deref(),
                invalidation_detail.as_deref(),
            ) >= *min,
        ),
        UiPredicateV1::VirtualListWindowsMatchingGe {
            min,
            shift_kind,
            reason,
            apply_mode,
            source,
            invalidation_detail,
        } => Some(
            count_matching_virtual_list_windows(
                debug,
                shift_kind.as_deref(),
                reason.as_deref(),
                apply_mode.as_deref(),
                source.as_deref(),
                invalidation_detail.as_deref(),
            ) >= *min,
        ),
        UiPredicateV1::RetainedVirtualListReconcilesMatchingGe {
            min,
            reconcile_kind,
            attached_items_min,
            detached_items_min,
            reused_from_keep_alive_items_min,
            kept_alive_items_min,
        } => Some(
            count_matching_retained_virtual_list_reconciles(
                debug,
                reconcile_kind.as_deref(),
                *attached_items_min,
                *detached_items_min,
                *reused_from_keep_alive_items_min,
                *kept_alive_items_min,
            ) >= *min,
        ),
        UiPredicateV1::ScrollHandleChangesMatchingGe {
            min,
            change_kind,
            offset_y_min,
            prev_offset_y_max,
            offset_changed,
            upgraded_to_layout_bindings_min,
        } => Some(
            count_matching_scroll_handle_changes(
                debug,
                change_kind.as_deref(),
                *offset_y_min,
                *prev_offset_y_max,
                *offset_changed,
                *upgraded_to_layout_bindings_min,
            ) >= *min,
        ),
        _ => None,
    }
}

fn count_matching_virtual_list_shift_samples(
    debug: &UiTreeDebugSnapshotV1,
    shift_kind: Option<&str>,
    reason: Option<&str>,
    apply_mode: Option<&str>,
    source: Option<&str>,
    invalidation_detail: Option<&str>,
) -> u64 {
    debug
        .virtual_list_window_shift_samples
        .iter()
        .filter(|sample| {
            matches_optional(
                shift_kind,
                virtual_list_shift_kind_name(sample.window_shift_kind),
            ) && matches_optional(
                reason,
                virtual_list_shift_reason_name(sample.window_shift_reason),
            ) && matches_optional(
                apply_mode,
                virtual_list_shift_apply_mode_name(sample.window_shift_apply_mode),
            ) && matches_optional(source, virtual_list_window_source_name(sample.source))
                && optional_matches_optional(
                    invalidation_detail,
                    sample.window_shift_invalidation_detail.as_deref(),
                )
        })
        .count() as u64
}

fn count_matching_virtual_list_windows(
    debug: &UiTreeDebugSnapshotV1,
    shift_kind: Option<&str>,
    reason: Option<&str>,
    apply_mode: Option<&str>,
    source: Option<&str>,
    invalidation_detail: Option<&str>,
) -> u64 {
    debug
        .virtual_list_windows
        .iter()
        .filter(|window| {
            matches_optional(
                shift_kind,
                virtual_list_shift_kind_name(window.window_shift_kind),
            ) && optional_matches_optional(
                reason,
                window
                    .window_shift_reason
                    .map(virtual_list_shift_reason_name),
            ) && optional_matches_optional(
                apply_mode,
                window
                    .window_shift_apply_mode
                    .map(virtual_list_shift_apply_mode_name),
            ) && matches_optional(source, virtual_list_window_source_name(window.source))
                && optional_matches_optional(
                    invalidation_detail,
                    window.window_shift_invalidation_detail.as_deref(),
                )
        })
        .count() as u64
}

fn count_matching_retained_virtual_list_reconciles(
    debug: &UiTreeDebugSnapshotV1,
    reconcile_kind: Option<&str>,
    attached_items_min: Option<u64>,
    detached_items_min: Option<u64>,
    reused_from_keep_alive_items_min: Option<u64>,
    kept_alive_items_min: Option<u64>,
) -> u64 {
    debug
        .retained_virtual_list_reconciles
        .iter()
        .filter(|reconcile| {
            optional_matches_optional(
                reconcile_kind,
                reconcile
                    .reconcile_kind
                    .map(retained_virtual_list_reconcile_kind_name),
            ) && min_matches_optional(attached_items_min, reconcile.attached_items)
                && min_matches_optional(detached_items_min, reconcile.detached_items)
                && min_matches_optional(
                    reused_from_keep_alive_items_min,
                    reconcile.reused_from_keep_alive_items,
                )
                && min_matches_optional(kept_alive_items_min, reconcile.kept_alive_items)
        })
        .count() as u64
}

fn count_matching_scroll_handle_changes(
    debug: &UiTreeDebugSnapshotV1,
    change_kind: Option<&str>,
    offset_y_min: Option<f64>,
    prev_offset_y_max: Option<f64>,
    offset_changed: Option<bool>,
    upgraded_to_layout_bindings_min: Option<u64>,
) -> u64 {
    debug
        .scroll_handle_changes
        .iter()
        .filter(|change| {
            matches_optional(change_kind, scroll_handle_change_kind_name(change.kind))
                && min_matches_optional_f64(offset_y_min, change.offset_y.into())
                && max_matches_optional_f64(prev_offset_y_max, change.prev_offset_y)
                && optional_matches_optional_bool(offset_changed, Some(change.offset_changed))
                && min_matches_optional(
                    upgraded_to_layout_bindings_min,
                    change.upgraded_to_layout_bindings.into(),
                )
        })
        .count() as u64
}

fn matches_optional(expected: Option<&str>, actual: &str) -> bool {
    expected.is_none_or(|expected| expected == actual)
}

fn optional_matches_optional(expected: Option<&str>, actual: Option<&str>) -> bool {
    expected.is_none_or(|expected| actual == Some(expected))
}

fn min_matches_optional(expected_min: Option<u64>, actual: u64) -> bool {
    expected_min.is_none_or(|expected_min| actual >= expected_min)
}

fn min_matches_optional_f64(expected_min: Option<f64>, actual: f64) -> bool {
    expected_min.is_none_or(|expected_min| actual >= expected_min)
}

fn max_matches_optional_f64(expected_max: Option<f64>, actual: Option<f32>) -> bool {
    expected_max
        .is_none_or(|expected_max| actual.is_some_and(|actual| (actual as f64) <= expected_max))
}

fn optional_matches_optional_bool(expected: Option<bool>, actual: Option<bool>) -> bool {
    expected.is_none_or(|expected| actual == Some(expected))
}

fn virtual_list_shift_kind_name(kind: UiVirtualListWindowShiftKindV1) -> &'static str {
    match kind {
        UiVirtualListWindowShiftKindV1::None => "none",
        UiVirtualListWindowShiftKindV1::Prefetch => "prefetch",
        UiVirtualListWindowShiftKindV1::Escape => "escape",
    }
}

fn virtual_list_shift_reason_name(reason: UiVirtualListWindowShiftReasonV1) -> &'static str {
    match reason {
        UiVirtualListWindowShiftReasonV1::ScrollOffset => "scroll_offset",
        UiVirtualListWindowShiftReasonV1::ViewportResize => "viewport_resize",
        UiVirtualListWindowShiftReasonV1::ItemsRevision => "items_revision",
        UiVirtualListWindowShiftReasonV1::ScrollToItem => "scroll_to_item",
        UiVirtualListWindowShiftReasonV1::InputsChange => "inputs_change",
        UiVirtualListWindowShiftReasonV1::Unknown => "unknown",
    }
}

fn virtual_list_shift_apply_mode_name(
    apply_mode: UiVirtualListWindowShiftApplyModeV1,
) -> &'static str {
    match apply_mode {
        UiVirtualListWindowShiftApplyModeV1::RetainedReconcile => "retained_reconcile",
        UiVirtualListWindowShiftApplyModeV1::NonRetainedRerender => "non_retained_rerender",
    }
}

fn retained_virtual_list_reconcile_kind_name(
    kind: UiRetainedVirtualListReconcileKindV1,
) -> &'static str {
    match kind {
        UiRetainedVirtualListReconcileKindV1::Prefetch => "prefetch",
        UiRetainedVirtualListReconcileKindV1::Escape => "escape",
    }
}

fn scroll_handle_change_kind_name(kind: UiScrollHandleChangeKindV1) -> &'static str {
    match kind {
        UiScrollHandleChangeKindV1::Layout => "layout",
        UiScrollHandleChangeKindV1::HitTestOnly => "hit_test_only",
    }
}

fn virtual_list_window_source_name(source: UiVirtualListWindowSourceV1) -> &'static str {
    match source {
        UiVirtualListWindowSourceV1::Prepaint => "prepaint",
        UiVirtualListWindowSourceV1::Layout => "layout",
    }
}

fn eval_docking_predicate_from_debug_snapshot(
    docking: &UiDockingInteractionSnapshotV1,
    predicate: &UiPredicateV1,
) -> Option<bool> {
    match predicate {
        UiPredicateV1::DockDropPreviewKindIs { preview_kind } => {
            let preview = docking.dock_drop_resolve.as_ref()?.preview.as_ref()?;
            let have = match preview.kind {
                UiDockDropPreviewKindDiagnosticsV1::WrapBinary => "wrap_binary",
                UiDockDropPreviewKindDiagnosticsV1::InsertIntoSplit { .. } => "insert_into_split",
            };
            Some(have == preview_kind.as_str())
        }
        UiPredicateV1::DockDropResolveSourceIs { source } => {
            let resolve = docking.dock_drop_resolve.as_ref()?;
            let have = match resolve.source {
                UiDockDropResolveSourceV1::InvertDocking => "invert_docking",
                UiDockDropResolveSourceV1::OutsideWindow => "outside_window",
                UiDockDropResolveSourceV1::FloatZone => "float_zone",
                UiDockDropResolveSourceV1::EmptyDockSpace => "empty_dock_space",
                UiDockDropResolveSourceV1::LayoutBoundsMiss => "layout_bounds_miss",
                UiDockDropResolveSourceV1::LatchedPreviousHover => "latched_previous_hover",
                UiDockDropResolveSourceV1::TabBar => "tab_bar",
                UiDockDropResolveSourceV1::FloatingTitleBar => "floating_title_bar",
                UiDockDropResolveSourceV1::OuterHintRect => "outer_hint_rect",
                UiDockDropResolveSourceV1::InnerHintRect => "inner_hint_rect",
                UiDockDropResolveSourceV1::None => "none",
            };
            Some(have == source.as_str())
        }
        UiPredicateV1::DockDropResolvedIsSome { some } => {
            Some(docking.dock_drop_resolve.as_ref()?.resolved.is_some() == *some)
        }
        UiPredicateV1::DockDropResolvedZoneIs { zone } => {
            let resolved = docking.dock_drop_resolve.as_ref()?.resolved.as_ref()?;
            let have = match resolved.zone {
                UiDropZoneV1::Center => "center",
                UiDropZoneV1::Left => "left",
                UiDropZoneV1::Right => "right",
                UiDropZoneV1::Top => "top",
                UiDropZoneV1::Bottom => "bottom",
            };
            Some(have == zone.as_str())
        }
        UiPredicateV1::DockDropResolvedInsertIndexIs { index } => {
            let resolved = docking.dock_drop_resolve.as_ref()?.resolved.as_ref()?;
            Some(resolved.insert_index == Some(*index as u64))
        }
        UiPredicateV1::DockGraphCanonicalIs { canonical } => {
            Some(docking.dock_graph_stats.as_ref()?.canonical_ok == *canonical)
        }
        UiPredicateV1::DockGraphHasNestedSameAxisSplitsIs { has_nested } => Some(
            docking
                .dock_graph_stats
                .as_ref()?
                .has_nested_same_axis_splits
                == *has_nested,
        ),
        UiPredicateV1::DockGraphNodeCountLe { max } => {
            Some(docking.dock_graph_stats.as_ref()?.node_count <= *max)
        }
        UiPredicateV1::DockGraphMaxSplitDepthLe { max } => {
            Some(docking.dock_graph_stats.as_ref()?.max_split_depth <= *max)
        }
        UiPredicateV1::DockGraphSignatureIs { signature } => {
            Some(docking.dock_graph_signature.as_ref()?.signature == *signature)
        }
        UiPredicateV1::DockGraphSignatureContains { needle } => Some(
            docking
                .dock_graph_signature
                .as_ref()?
                .signature
                .contains(needle),
        ),
        UiPredicateV1::DockGraphSignatureFingerprint64Is { fingerprint64 } => {
            Some(docking.dock_graph_signature.as_ref()?.fingerprint64 == *fingerprint64)
        }
        _ => None,
    }
}

fn eval_resource_loading_predicate_from_debug_snapshot(
    resource_loading: &UiResourceLoadingDiagnosticsSnapshotV1,
    predicate: &UiPredicateV1,
) -> Option<bool> {
    match predicate {
        UiPredicateV1::BundledFontBaselineSourceIs { source } => Some(
            resource_loading
                .font_environment
                .as_ref()?
                .bundled_baseline_source
                == *source,
        ),
        UiPredicateV1::RendererFontEnvironmentRevisionGe { min } => Some(
            resource_loading
                .font_environment
                .as_ref()?
                .renderer_font_environment_revision
                .unwrap_or(0)
                >= *min,
        ),
        UiPredicateV1::RendererFontSourceLaneSeen { lane } => Some(
            resource_loading
                .font_environment
                .as_ref()?
                .renderer_font_sources
                .iter()
                .any(|source| source.source_lane == *lane),
        ),
        UiPredicateV1::RendererFontSourceAssetKeySeen { asset_key } => Some(
            resource_loading
                .font_environment
                .as_ref()?
                .renderer_font_sources
                .iter()
                .any(|source| source.asset_key.as_deref() == Some(asset_key.as_str())),
        ),
        UiPredicateV1::SvgTextBridgeSelectionMissesGe { min } => Some(
            resource_loading
                .svg_text_bridge
                .as_ref()?
                .selection_misses
                .len() as u64
                >= *min,
        ),
        UiPredicateV1::SvgTextBridgeMissingGlyphsGe { min } => Some(
            resource_loading
                .svg_text_bridge
                .as_ref()?
                .missing_glyphs
                .len() as u64
                >= *min,
        ),
        UiPredicateV1::SvgTextBridgeDiagnosticsCleanIs { clean } => {
            Some(resource_loading.svg_text_bridge.as_ref()?.is_clean() == *clean)
        }
        UiPredicateV1::SvgTextBridgeFallbackSeen {
            from_family,
            to_family,
        } => Some(
            resource_loading
                .svg_text_bridge
                .as_ref()?
                .fallback_records
                .iter()
                .any(|record| record.from_family == *from_family && record.to_family == *to_family),
        ),
        UiPredicateV1::AssetReloadEpochGe { min } => {
            Some(resource_loading.asset_reload.as_ref()?.epoch.unwrap_or(0) >= *min)
        }
        UiPredicateV1::AssetReloadConfiguredBackendIs { backend } => Some(
            resource_loading
                .asset_reload
                .as_ref()?
                .configured_backend
                .as_deref()
                == Some(backend.as_str()),
        ),
        UiPredicateV1::AssetReloadActiveBackendIs { backend } => Some(
            resource_loading
                .asset_reload
                .as_ref()?
                .active_backend
                .as_deref()
                == Some(backend.as_str()),
        ),
        UiPredicateV1::AssetReloadFallbackReasonIs { reason } => Some(
            resource_loading
                .asset_reload
                .as_ref()?
                .fallback_reason
                .as_deref()
                == Some(reason.as_str()),
        ),
        UiPredicateV1::AssetLoadMissingBundleAssetRequestsGe { min } => Some(
            resource_loading
                .asset_load
                .as_ref()?
                .missing_bundle_asset_requests
                >= *min,
        ),
        UiPredicateV1::AssetLoadStaleManifestRequestsGe { min } => Some(
            resource_loading
                .asset_load
                .as_ref()?
                .stale_manifest_requests
                >= *min,
        ),
        UiPredicateV1::AssetLoadUnsupportedFileRequestsGe { min } => Some(
            resource_loading
                .asset_load
                .as_ref()?
                .unsupported_file_requests
                >= *min,
        ),
        UiPredicateV1::AssetLoadUnsupportedUrlRequestsGe { min } => Some(
            resource_loading
                .asset_load
                .as_ref()?
                .unsupported_url_requests
                >= *min,
        ),
        UiPredicateV1::AssetLoadExternalReferenceUnavailableRequestsGe { min } => Some(
            resource_loading
                .asset_load
                .as_ref()?
                .external_reference_unavailable_requests
                >= *min,
        ),
        UiPredicateV1::AssetLoadRevisionChangeRequestsGe { min } => Some(
            resource_loading
                .asset_load
                .as_ref()?
                .revision_change_requests
                >= *min,
        ),
        UiPredicateV1::AssetLoadRecentOutcomeSeen { outcome_kind } => Some(
            resource_loading
                .asset_load
                .as_ref()?
                .recent
                .iter()
                .any(|event| event.outcome_kind == *outcome_kind),
        ),
        UiPredicateV1::AssetLoadRecentRevisionTransitionSeen { transition } => Some(
            resource_loading
                .asset_load
                .as_ref()?
                .recent
                .iter()
                .any(|event| event.revision_transition.as_deref() == Some(transition.as_str())),
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot_with_virtual_list_shift_samples(
        samples: usize,
        invalidation_detail: Option<&str>,
    ) -> UiDiagnosticsSnapshotV1 {
        let mut debug = UiTreeDebugSnapshotV1::default();
        debug.virtual_list_window_shift_samples = (0..samples)
            .map(|idx| UiVirtualListWindowShiftSampleV1 {
                frame_id: idx as u64,
                source: UiVirtualListWindowSourceV1::Layout,
                node: idx as u64 + 1,
                element: idx as u64 + 10,
                window_shift_kind: UiVirtualListWindowShiftKindV1::Escape,
                window_shift_reason: UiVirtualListWindowShiftReasonV1::ScrollOffset,
                window_shift_apply_mode: UiVirtualListWindowShiftApplyModeV1::RetainedReconcile,
                window_shift_invalidation_detail: invalidation_detail.map(str::to_string),
                prev_window_range: None,
                window_range: None,
                render_window_range: None,
            })
            .collect();

        snapshot_with_debug(debug)
    }

    fn snapshot_with_virtual_list_window(
        shift_kind: UiVirtualListWindowShiftKindV1,
        reason: Option<UiVirtualListWindowShiftReasonV1>,
        apply_mode: Option<UiVirtualListWindowShiftApplyModeV1>,
        source: UiVirtualListWindowSourceV1,
        invalidation_detail: Option<&str>,
    ) -> UiDiagnosticsSnapshotV1 {
        let mut debug = UiTreeDebugSnapshotV1::default();
        debug.virtual_list_windows.push(UiVirtualListWindowV1 {
            node: 1,
            element: 10,
            source,
            axis: UiAxisV1::Vertical,
            is_probe_layout: false,
            items_len: 10_000,
            items_revision: 1,
            prev_items_revision: 1,
            measure_mode: UiVirtualListMeasureModeV1::Fixed,
            overscan: 4,
            policy_key: 1,
            inputs_key: 1,
            viewport: 640.0,
            prev_viewport: 640.0,
            offset: 240.0,
            prev_offset: 0.0,
            window_range: None,
            prev_window_range: None,
            render_window_range: None,
            deferred_scroll_to_item: false,
            deferred_scroll_consumed: false,
            window_mismatch: false,
            window_shift_kind: shift_kind,
            window_shift_reason: reason,
            window_shift_apply_mode: apply_mode,
            window_shift_invalidation_detail: invalidation_detail.map(str::to_string),
        });

        snapshot_with_debug(debug)
    }

    fn snapshot_with_retained_virtual_list_reconcile(
        reconcile_kind: UiRetainedVirtualListReconcileKindV1,
        attached_items: u64,
        detached_items: u64,
        reused_from_keep_alive_items: u64,
    ) -> UiDiagnosticsSnapshotV1 {
        let mut debug = UiTreeDebugSnapshotV1::default();
        debug
            .retained_virtual_list_reconciles
            .push(UiRetainedVirtualListReconcileV1 {
                node: 1,
                element: 10,
                reconcile_kind: Some(reconcile_kind),
                prev_items: 25,
                next_items: 35,
                preserved_items: 11,
                attached_items,
                detached_items,
                reused_from_keep_alive_items,
                kept_alive_items: detached_items,
                evicted_keep_alive_items: 0,
                keep_alive_pool_len_before: 0,
                keep_alive_pool_len_after: detached_items,
            });

        snapshot_with_debug(debug)
    }

    fn snapshot_with_scroll_handle_change(
        change_kind: UiScrollHandleChangeKindV1,
        offset_y: f32,
        prev_offset_y: Option<f32>,
        upgraded_to_layout_bindings: u32,
    ) -> UiDiagnosticsSnapshotV1 {
        let mut debug = UiTreeDebugSnapshotV1::default();
        debug.scroll_handle_changes.push(UiScrollHandleChangeV1 {
            handle_key: 1,
            kind: change_kind,
            revision: 1,
            prev_revision: Some(0),
            offset_x: 0.0,
            offset_y,
            prev_offset_x: None,
            prev_offset_y,
            viewport_w: 1080.0,
            viewport_h: 700.0,
            prev_viewport_w: Some(1080.0),
            prev_viewport_h: Some(700.0),
            content_w: 1080.0,
            content_h: 10_000.0,
            prev_content_w: Some(1080.0),
            prev_content_h: Some(10_000.0),
            offset_changed: true,
            viewport_changed: false,
            content_changed: false,
            bound_elements: 1,
            bound_nodes_sample: vec![1],
            upgraded_to_layout_bindings,
        });

        snapshot_with_debug(debug)
    }

    fn snapshot_with_pointer_capture_active(active: bool) -> UiDiagnosticsSnapshotV1 {
        let mut debug = UiTreeDebugSnapshotV1::default();
        debug.input_arbitration.pointer_capture_active = active;

        snapshot_with_debug(debug)
    }

    fn snapshot_with_debug(debug: UiTreeDebugSnapshotV1) -> UiDiagnosticsSnapshotV1 {
        UiDiagnosticsSnapshotV1 {
            schema_version: 1,
            tick_id: 0,
            frame_id: 0,
            window_snapshot_seq: 0,
            window: 1,
            timestamp_unix_ms: unix_ms_now(),
            frame_clock: None,
            scale_factor: 1.0,
            window_bounds: RectV1 {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 100.0,
            },
            scene_ops: 0,
            scene_fingerprint: 0,
            semantics_fingerprint: None,
            changed_models: Vec::new(),
            changed_globals: Vec::new(),
            changed_model_sources_top: Vec::new(),
            resource_caches: None,
            app_snapshot: None,
            safe_area_insets: None,
            occlusion_insets: None,
            focus_is_text_input: None,
            is_composing: None,
            clipboard: None,
            primary_pointer_type: None,
            caps: None,
            wgpu_adapter: None,
            debug,
        }
    }

    #[test]
    fn virtual_list_window_shift_samples_predicate_counts_ring_snapshots() {
        let mut ring = WindowRing::default();
        ring.snapshots
            .push_back(snapshot_with_virtual_list_shift_samples(0, None));
        ring.snapshots
            .push_back(snapshot_with_virtual_list_shift_samples(1, None));

        assert_eq!(
            eval_debug_snapshot_predicate_from_ring(
                &ring,
                &UiPredicateV1::VirtualListWindowShiftSamplesLenLe { max: 0 },
            ),
            Some(false)
        );
        assert_eq!(
            eval_debug_snapshot_predicate_from_ring(
                &ring,
                &UiPredicateV1::VirtualListWindowShiftSamplesLenLe { max: 1 },
            ),
            Some(true)
        );
    }

    #[test]
    fn input_pointer_capture_active_predicate_reads_debug_snapshot() {
        let captured = snapshot_with_pointer_capture_active(true);
        let released = snapshot_with_pointer_capture_active(false);

        assert_eq!(
            eval_debug_snapshot_predicate(
                &captured.debug,
                &UiPredicateV1::InputPointerCaptureActiveIs { active: true },
            ),
            Some(true)
        );
        assert_eq!(
            eval_debug_snapshot_predicate(
                &released.debug,
                &UiPredicateV1::InputPointerCaptureActiveIs { active: true },
            ),
            Some(false)
        );
        assert_eq!(
            eval_debug_snapshot_predicate(
                &released.debug,
                &UiPredicateV1::InputPointerCaptureActiveIs { active: false },
            ),
            Some(true)
        );
    }

    #[test]
    fn current_state_predicates_do_not_match_stale_ring_snapshots() {
        let mut svc = UiDiagnosticsService::default();
        let window = AppWindowId::from(KeyData::from_ffi(1));
        let ring = svc.per_window.entry(window).or_default();
        ring.snapshots
            .push_back(snapshot_with_pointer_capture_active(true));
        ring.snapshots
            .push_back(snapshot_with_pointer_capture_active(false));

        assert_eq!(
            eval_debug_snapshot_predicate_from_recent_snapshot(
                &svc,
                window,
                &UiPredicateV1::InputPointerCaptureActiveIs { active: true },
                250,
            ),
            Some(false)
        );
        assert_eq!(
            eval_debug_snapshot_predicate_from_recent_snapshot(
                &svc,
                window,
                &UiPredicateV1::InputPointerCaptureActiveIs { active: false },
                250,
            ),
            Some(true)
        );
    }

    #[test]
    fn virtual_list_window_shift_samples_matching_predicate_counts_ring_snapshots() {
        let mut ring = WindowRing::default();
        ring.snapshots
            .push_back(snapshot_with_virtual_list_shift_samples(0, None));
        ring.snapshots
            .push_back(snapshot_with_virtual_list_shift_samples(
                2,
                Some("scroll_handle_inputs_change_window_update"),
            ));

        assert_eq!(
            eval_debug_snapshot_predicate_from_ring(
                &ring,
                &UiPredicateV1::VirtualListWindowShiftSamplesMatchingGe {
                    min: 2,
                    shift_kind: Some("escape".to_string()),
                    reason: Some("scroll_offset".to_string()),
                    apply_mode: Some("retained_reconcile".to_string()),
                    source: Some("layout".to_string()),
                    invalidation_detail: Some(
                        "scroll_handle_inputs_change_window_update".to_string(),
                    ),
                },
            ),
            Some(true)
        );
        assert_eq!(
            eval_debug_snapshot_predicate_from_ring(
                &ring,
                &UiPredicateV1::VirtualListWindowShiftSamplesMatchingGe {
                    min: 1,
                    shift_kind: Some("prefetch".to_string()),
                    reason: Some("scroll_offset".to_string()),
                    apply_mode: Some("retained_reconcile".to_string()),
                    source: Some("layout".to_string()),
                    invalidation_detail: Some("scroll_handle_prefetch_window_update".to_string(),),
                },
            ),
            Some(false)
        );
    }

    #[test]
    fn virtual_list_windows_matching_predicate_counts_ring_snapshots() {
        let mut ring = WindowRing::default();
        ring.snapshots.push_back(snapshot_with_virtual_list_window(
            UiVirtualListWindowShiftKindV1::Prefetch,
            Some(UiVirtualListWindowShiftReasonV1::ScrollOffset),
            Some(UiVirtualListWindowShiftApplyModeV1::RetainedReconcile),
            UiVirtualListWindowSourceV1::Prepaint,
            Some("scroll_handle_prefetch_window_update"),
        ));

        assert_eq!(
            eval_debug_snapshot_predicate_from_ring(
                &ring,
                &UiPredicateV1::VirtualListWindowsMatchingGe {
                    min: 1,
                    shift_kind: Some("prefetch".to_string()),
                    reason: Some("scroll_offset".to_string()),
                    apply_mode: Some("retained_reconcile".to_string()),
                    source: Some("prepaint".to_string()),
                    invalidation_detail: Some("scroll_handle_prefetch_window_update".to_string(),),
                },
            ),
            Some(true)
        );
        assert_eq!(
            eval_debug_snapshot_predicate_from_ring(
                &ring,
                &UiPredicateV1::VirtualListWindowsMatchingGe {
                    min: 1,
                    shift_kind: Some("prefetch".to_string()),
                    reason: Some("viewport_resize".to_string()),
                    apply_mode: Some("retained_reconcile".to_string()),
                    source: Some("prepaint".to_string()),
                    invalidation_detail: Some("scroll_handle_prefetch_window_update".to_string(),),
                },
            ),
            Some(false)
        );
    }

    #[test]
    fn retained_virtual_list_reconciles_matching_predicate_counts_ring_snapshots() {
        let mut ring = WindowRing::default();
        ring.snapshots
            .push_back(snapshot_with_retained_virtual_list_reconcile(
                UiRetainedVirtualListReconcileKindV1::Prefetch,
                2,
                1,
                0,
            ));
        ring.snapshots
            .push_back(snapshot_with_retained_virtual_list_reconcile(
                UiRetainedVirtualListReconcileKindV1::Escape,
                24,
                14,
                8,
            ));

        assert_eq!(
            eval_debug_snapshot_predicate_from_ring(
                &ring,
                &UiPredicateV1::RetainedVirtualListReconcilesMatchingGe {
                    min: 1,
                    reconcile_kind: Some("escape".to_string()),
                    attached_items_min: Some(1),
                    detached_items_min: Some(1),
                    reused_from_keep_alive_items_min: Some(1),
                    kept_alive_items_min: None,
                },
            ),
            Some(true)
        );
        assert_eq!(
            eval_debug_snapshot_predicate_from_ring(
                &ring,
                &UiPredicateV1::RetainedVirtualListReconcilesMatchingGe {
                    min: 1,
                    reconcile_kind: Some("escape".to_string()),
                    attached_items_min: Some(25),
                    detached_items_min: Some(1),
                    reused_from_keep_alive_items_min: None,
                    kept_alive_items_min: None,
                },
            ),
            Some(false)
        );
    }

    #[test]
    fn scroll_handle_changes_matching_predicate_counts_ring_snapshots() {
        let mut ring = WindowRing::default();
        ring.snapshots.push_back(snapshot_with_scroll_handle_change(
            UiScrollHandleChangeKindV1::HitTestOnly,
            720.0,
            Some(0.0),
            1,
        ));
        ring.snapshots.push_back(snapshot_with_scroll_handle_change(
            UiScrollHandleChangeKindV1::Layout,
            0.0,
            Some(0.0),
            0,
        ));

        assert_eq!(
            eval_debug_snapshot_predicate_from_ring(
                &ring,
                &UiPredicateV1::ScrollHandleChangesMatchingGe {
                    min: 1,
                    change_kind: Some("hit_test_only".to_string()),
                    offset_y_min: Some(720.0),
                    prev_offset_y_max: Some(0.0),
                    offset_changed: Some(true),
                    upgraded_to_layout_bindings_min: Some(1),
                },
            ),
            Some(true)
        );
        assert_eq!(
            eval_debug_snapshot_predicate_from_ring(
                &ring,
                &UiPredicateV1::ScrollHandleChangesMatchingGe {
                    min: 2,
                    change_kind: Some("hit_test_only".to_string()),
                    offset_y_min: Some(720.0),
                    prev_offset_y_max: Some(0.0),
                    offset_changed: Some(true),
                    upgraded_to_layout_bindings_min: Some(1),
                },
            ),
            Some(false)
        );
    }
}
