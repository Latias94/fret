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
    eval_debug_snapshot_predicate(&snapshot.debug, predicate)
}

pub(super) fn eval_debug_snapshot_predicate(
    debug: &UiTreeDebugSnapshotV1,
    predicate: &UiPredicateV1,
) -> Option<bool> {
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
