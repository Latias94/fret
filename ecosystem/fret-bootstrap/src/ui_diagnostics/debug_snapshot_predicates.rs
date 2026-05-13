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
    if let Some(ok) = eval_debug_snapshot_predicate_from_ring(ring, predicate) {
        return Some(ok);
    }
    eval_debug_snapshot_predicate(&snapshot.debug, predicate)
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
        _ => None,
    }
}

pub(super) fn eval_debug_snapshot_predicate(
    debug: &UiTreeDebugSnapshotV1,
    predicate: &UiPredicateV1,
) -> Option<bool> {
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

fn eval_virtual_list_predicate_from_debug_snapshot(
    debug: &UiTreeDebugSnapshotV1,
    predicate: &UiPredicateV1,
) -> Option<bool> {
    match predicate {
        UiPredicateV1::VirtualListWindowShiftSamplesLenLe { max } => {
            Some((debug.virtual_list_window_shift_samples.len() as u64) <= *max)
        }
        _ => None,
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

    fn snapshot_with_virtual_list_shift_samples(samples: usize) -> UiDiagnosticsSnapshotV1 {
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
                window_shift_invalidation_detail: None,
                prev_window_range: None,
                window_range: None,
                render_window_range: None,
            })
            .collect();

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
            .push_back(snapshot_with_virtual_list_shift_samples(0));
        ring.snapshots
            .push_back(snapshot_with_virtual_list_shift_samples(1));

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
}
