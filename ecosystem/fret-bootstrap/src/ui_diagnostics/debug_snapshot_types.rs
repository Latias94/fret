#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiTreeDebugSnapshotV1 {
    pub stats: UiFrameStatsV1,
    #[serde(default)]
    pub invalidation_walks: Vec<UiInvalidationWalkV1>,
    #[serde(default)]
    pub hover_declarative_invalidation_hotspots: Vec<UiHoverDeclarativeInvalidationHotspotV1>,
    #[serde(default)]
    pub dirty_views: Vec<UiDirtyViewV1>,
    #[serde(default)]
    pub notify_requests: Vec<UiNotifyRequestV1>,
    #[serde(default)]
    pub virtual_list_windows: Vec<UiVirtualListWindowV1>,
    #[serde(default)]
    pub virtual_list_window_shift_samples: Vec<UiVirtualListWindowShiftSampleV1>,
    #[serde(default)]
    pub windowed_rows_surfaces: Vec<UiWindowedRowsSurfaceWindowV1>,
    #[serde(default)]
    pub retained_virtual_list_reconciles: Vec<UiRetainedVirtualListReconcileV1>,
    #[serde(default)]
    pub scroll_handle_changes: Vec<UiScrollHandleChangeV1>,
    #[serde(default)]
    pub scroll_nodes: Vec<UiScrollNodeTelemetryV1>,
    #[serde(default)]
    pub prepaint_actions: Vec<UiPrepaintActionV1>,
    #[serde(default)]
    pub model_change_hotspots: Vec<UiModelChangeHotspotV1>,
    #[serde(default)]
    pub model_change_unobserved: Vec<UiModelChangeUnobservedV1>,
    #[serde(default)]
    pub global_change_hotspots: Vec<UiGlobalChangeHotspotV1>,
    #[serde(default)]
    pub global_change_unobserved: Vec<UiGlobalChangeUnobservedV1>,
    #[serde(default)]
    pub cache_roots: Vec<UiCacheRootStatsV1>,
    #[serde(default)]
    pub overlay_synthesis: Vec<UiOverlaySynthesisEventV1>,
    /// Viewport input forwarding events observed during the current frame.
    ///
    /// This records `Effect::ViewportInput` deliveries (ADR 0132) so scripted diagnostics can
    /// gate on “viewport tooling input was actually exercised” without scraping logs.
    #[serde(default)]
    pub viewport_input: Vec<UiViewportInputEventV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_ime_bridge: Option<UiWebImeBridgeDebugSnapshotV1>,
    /// Docking interaction ownership snapshot (best-effort).
    ///
    /// This is sourced from a frame-local diagnostics store populated by policy-heavy ecosystem
    /// crates (e.g. docking), and is intended for debugging arbitration regressions without logs.
    #[serde(default)]
    pub docking_interaction: Option<UiDockingInteractionSnapshotV1>,
    /// Workspace interaction snapshot (best-effort).
    ///
    /// This is sourced from `WindowInteractionDiagnosticsStore` and is intended for gating
    /// editor-grade invariants (e.g. “active tab stays visible”) without relying on pixels.
    #[serde(default)]
    pub workspace_interaction: Option<UiWorkspaceInteractionSnapshotV1>,
    #[serde(default)]
    pub removed_subtrees: Vec<UiRemovedSubtreeV1>,
    #[serde(default)]
    pub layout_engine_solves: Vec<UiLayoutEngineSolveV1>,
    #[serde(default)]
    pub layout_hotspots: Vec<UiLayoutHotspotV1>,
    #[serde(default)]
    pub widget_measure_hotspots: Vec<UiWidgetMeasureHotspotV1>,
    #[serde(default)]
    pub paint_widget_hotspots: Vec<UiPaintWidgetHotspotV1>,
    #[serde(default)]
    pub paint_text_prepare_hotspots: Vec<UiPaintTextPrepareHotspotV1>,
    #[serde(default)]
    pub input_arbitration: UiInputArbitrationSnapshotV1,
    /// Best-effort command gating decisions for a small set of "interesting" commands.
    ///
    /// This is intended for debugging cross-surface inconsistencies (menus vs palette vs buttons)
    /// without relying on ad-hoc logs.
    #[serde(default)]
    pub command_gating_trace: Vec<UiCommandGatingTraceEntryV1>,
    /// Best-effort command dispatch trace entries observed during the current frame.
    ///
    /// This is intended to answer “what command was dispatched, from where, and was it handled?”
    /// without relying on ad-hoc logs.
    #[serde(default)]
    pub command_dispatch_trace: Vec<UiCommandDispatchTraceEntryV1>,
    pub layers_in_paint_order: Vec<UiLayerInfoV1>,
    #[serde(default)]
    pub all_layer_roots: Vec<u64>,
    #[serde(default)]
    pub layer_visible_writes: Vec<UiLayerVisibleWriteV1>,
    #[serde(default)]
    pub overlay_policy_decisions: Vec<UiOverlayPolicyDecisionV1>,
    /// A committed per-window environment snapshot (ADR 0232), exported under `debug.environment`
    /// for easy diagnostics consumption.
    ///
    /// This duplicates the committed fields also present under `debug.element_runtime.environment`
    /// (when the element runtime snapshot is enabled), but keeps a stable schema path for tools
    /// that do not want to parse the entire element runtime snapshot payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<ElementEnvironmentSnapshotV1>,
    /// Best-effort ecosystem extensions map (bounded and additive).
    ///
    /// This is the primary in-snapshot extension seam for UI diagnostics. Payloads must remain
    /// small and debug-oriented; larger dumps should use bundle-scoped sidecars instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<std::collections::BTreeMap<String, serde_json::Value>>,
    /// Best-effort window insets snapshot (safe-area + occlusion) from `WindowMetricsService`.
    ///
    /// Unlike `debug.environment`, this does not require the element runtime snapshot to be
    /// enabled. It is intended as a quick "what does the runner think the insets are" anchor
    /// during mobile bring-up.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_insets: Option<UiWindowInsetsSnapshotV1>,
    /// Best-effort platform text-input snapshot for the current window.
    ///
    /// This records `focus_is_text_input` and the last committed IME cursor area, which are
    /// frequently needed when diagnosing virtual keyboard avoidance and IME candidate placement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_input: Option<UiWindowTextInputSnapshotV1>,
    /// Runner surface lifecycle state, sourced from `RunnerSurfaceLifecycleDiagnosticsStore`.
    ///
    /// This is intended for Android/iOS bring-up to verify that background/foreground transitions
    /// are dropping and recreating surfaces as expected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_surface_lifecycle: Option<UiRunnerSurfaceLifecycleSnapshotV1>,
    /// Runner accessibility activation evidence, sourced from `RunnerAccessibilityDiagnosticsStore`.
    ///
    /// This records when the OS accessibility stack activates the AccessKit adapter for a window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_accessibility: Option<UiRunnerAccessibilitySnapshotV1>,
    /// Best-effort resource-loading diagnostics snapshot.
    ///
    /// This surfaces recent asset resolution outcomes plus the current startup font baseline /
    /// catalog state so resource-loading drift can be diagnosed from the bundle without scraping
    /// logs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_loading: Option<UiResourceLoadingDiagnosticsSnapshotV1>,
    pub hit_test: Option<UiHitTestSnapshotV1>,
    pub element_runtime: Option<ElementDiagnosticsSnapshotV1>,
    pub semantics: Option<UiSemanticsSnapshotV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCommandDispatchTraceEntryV1 {
    pub command: String,
    pub handled: bool,
    /// Best-effort handler scope classification (ADR 0307).
    ///
    /// Expected values: `"widget"`, `"window"`, `"app"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled_by_scope: Option<String>,
    /// Whether the command was handled by a runner/driver integration layer (not by a UI element).
    #[serde(default)]
    pub handled_by_driver: bool,
    #[serde(default)]
    pub stopped: bool,
    #[serde(default)]
    pub source_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handled_by_element: Option<u64>,
    #[serde(default)]
    pub started_from_focus: bool,
    #[serde(default)]
    pub used_default_root_fallback: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiWindowInsetsSnapshotV1 {
    pub safe_area_known: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_area_insets_px: Option<UiPaddingInsetsV1>,
    pub occlusion_known: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occlusion_insets_px: Option<UiPaddingInsetsV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWindowTextInputSnapshotV1 {
    pub focus_is_text_input: bool,
    pub is_composing: bool,
    /// Total length (UTF-16 code units) of the composed view.
    pub text_len_utf16: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_utf16: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marked_utf16: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ime_cursor_area: Option<RectV1>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiRunnerSurfaceLifecycleSnapshotV1 {
    pub can_create_surfaces_calls: u64,
    pub destroy_surfaces_calls: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_can_create_surfaces_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_destroy_surfaces_unix_ms: Option<u64>,
    pub surfaces_available: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiRunnerAccessibilitySnapshotV1 {
    pub activation_requests: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_activation_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_activation_frame_id: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiResourceLoadingDiagnosticsSnapshotV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_load: Option<UiAssetLoadDiagnosticsSnapshotV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_reload: Option<UiAssetReloadDiagnosticsSnapshotV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_environment: Option<UiFontEnvironmentDiagnosticsSnapshotV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub svg_text_bridge: Option<UiSvgTextBridgeDiagnosticsSnapshotV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiAssetLoadDiagnosticsSnapshotV1 {
    pub total_requests: u64,
    pub bytes_requests: u64,
    pub reference_requests: u64,
    pub missing_bundle_asset_requests: u64,
    #[serde(default)]
    pub stale_manifest_requests: u64,
    #[serde(default)]
    pub io_requests: u64,
    pub unsupported_file_requests: u64,
    pub unsupported_url_requests: u64,
    pub external_reference_unavailable_requests: u64,
    pub revision_change_requests: u64,
    #[serde(default)]
    pub recent: Vec<UiAssetLoadDiagnosticEventV1>,
}

impl UiAssetLoadDiagnosticsSnapshotV1 {
    pub(super) fn from_runtime(snapshot: fret_runtime::AssetLoadDiagnosticsSnapshot) -> Self {
        Self {
            total_requests: snapshot.total_requests,
            bytes_requests: snapshot.bytes_requests,
            reference_requests: snapshot.reference_requests,
            missing_bundle_asset_requests: snapshot.missing_bundle_asset_requests,
            stale_manifest_requests: snapshot.stale_manifest_requests,
            io_requests: snapshot.io_requests,
            unsupported_file_requests: snapshot.unsupported_file_requests,
            unsupported_url_requests: snapshot.unsupported_url_requests,
            external_reference_unavailable_requests: snapshot
                .external_reference_unavailable_requests,
            revision_change_requests: snapshot.revision_change_requests,
            recent: snapshot
                .recent
                .into_iter()
                .map(UiAssetLoadDiagnosticEventV1::from_runtime)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiAssetLoadDiagnosticEventV1 {
    pub access_kind: String,
    pub locator_kind: String,
    pub locator_debug: String,
    pub outcome_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_revision: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_transition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl UiAssetLoadDiagnosticEventV1 {
    fn from_runtime(event: fret_runtime::AssetLoadDiagnosticEvent) -> Self {
        Self {
            access_kind: match event.access_kind {
                fret_runtime::AssetLoadAccessKind::Bytes => "bytes",
                fret_runtime::AssetLoadAccessKind::ExternalReference => "external_reference",
            }
            .to_string(),
            locator_kind: match event.locator_kind {
                fret_assets::AssetLocatorKind::Memory => "memory",
                fret_assets::AssetLocatorKind::Embedded => "embedded",
                fret_assets::AssetLocatorKind::BundleAsset => "bundle_asset",
                fret_assets::AssetLocatorKind::File => "file",
                fret_assets::AssetLocatorKind::Url => "url",
            }
            .to_string(),
            locator_debug: event.locator_debug,
            outcome_kind: match event.outcome_kind {
                fret_runtime::AssetLoadOutcomeKind::Resolved => "resolved",
                fret_runtime::AssetLoadOutcomeKind::Missing => "missing",
                fret_runtime::AssetLoadOutcomeKind::StaleManifest => "stale_manifest",
                fret_runtime::AssetLoadOutcomeKind::UnsupportedLocatorKind => {
                    "unsupported_locator_kind"
                }
                fret_runtime::AssetLoadOutcomeKind::ExternalReferenceUnavailable => {
                    "external_reference_unavailable"
                }
                fret_runtime::AssetLoadOutcomeKind::ReferenceOnlyLocator => {
                    "reference_only_locator"
                }
                fret_runtime::AssetLoadOutcomeKind::ResolverUnavailable => "resolver_unavailable",
                fret_runtime::AssetLoadOutcomeKind::AccessDenied => "access_denied",
                fret_runtime::AssetLoadOutcomeKind::Io => "io",
            }
            .to_string(),
            revision: event.revision.map(|revision| revision.0),
            previous_revision: event.previous_revision.map(|revision| revision.0),
            revision_transition: event.revision_transition.map(|transition| {
                match transition {
                    fret_runtime::AssetRevisionTransitionKind::Initial => "initial",
                    fret_runtime::AssetRevisionTransitionKind::Stable => "stable",
                    fret_runtime::AssetRevisionTransitionKind::Changed => "changed",
                }
                .to_string()
            }),
            message: event.message,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiAssetReloadDiagnosticsSnapshotV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epoch: Option<u64>,
    #[serde(default)]
    pub file_watch: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configured_backend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_backend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_message: Option<String>,
}

impl UiAssetReloadDiagnosticsSnapshotV1 {
    pub(super) fn from_runtime(
        epoch: Option<fret_runtime::AssetReloadEpoch>,
        support: Option<fret_runtime::AssetReloadSupport>,
        status: Option<fret_runtime::AssetReloadStatus>,
    ) -> Option<Self> {
        if epoch.is_none() && support.is_none() && status.is_none() {
            return None;
        }

        Some(Self {
            epoch: epoch.map(|epoch| epoch.0),
            file_watch: support.is_some_and(|support| support.file_watch),
            configured_backend: status.as_ref().map(|status| {
                asset_reload_backend_kind_name(status.configured_backend).to_string()
            }),
            active_backend: status
                .as_ref()
                .map(|status| asset_reload_backend_kind_name(status.active_backend).to_string()),
            fallback_reason: status
                .as_ref()
                .and_then(|status| status.fallback_reason)
                .map(|reason| asset_reload_fallback_reason_name(reason).to_string()),
            fallback_message: status.and_then(|status| status.fallback_message),
        })
    }
}

fn asset_reload_backend_kind_name(kind: fret_runtime::AssetReloadBackendKind) -> &'static str {
    match kind {
        fret_runtime::AssetReloadBackendKind::PollMetadata => "poll_metadata",
        fret_runtime::AssetReloadBackendKind::NativeWatcher => "native_watcher",
    }
}

fn asset_reload_fallback_reason_name(
    reason: fret_runtime::AssetReloadFallbackReason,
) -> &'static str {
    match reason {
        fret_runtime::AssetReloadFallbackReason::WatcherInstallFailed => "watcher_install_failed",
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiSvgTextBridgeDiagnosticsSnapshotV1 {
    pub revision: u64,
    #[serde(default)]
    pub selection_misses: Vec<UiSvgTextFontSelectionMissRecordV1>,
    #[serde(default)]
    pub fallback_records: Vec<UiSvgTextFontFallbackRecordV1>,
    #[serde(default)]
    pub missing_glyphs: Vec<UiSvgTextMissingGlyphRecordV1>,
}

impl UiSvgTextBridgeDiagnosticsSnapshotV1 {
    pub(super) fn from_runtime(
        snapshot: Option<&fret_runtime::RendererSvgTextBridgeDiagnosticsSnapshot>,
    ) -> Option<Self> {
        let snapshot = snapshot?;
        let revision = snapshot.revision?;
        Some(Self {
            revision,
            selection_misses: snapshot
                .selection_misses
                .iter()
                .map(UiSvgTextFontSelectionMissRecordV1::from_runtime)
                .collect(),
            fallback_records: snapshot
                .fallback_records
                .iter()
                .map(UiSvgTextFontFallbackRecordV1::from_runtime)
                .collect(),
            missing_glyphs: snapshot
                .missing_glyphs
                .iter()
                .map(UiSvgTextMissingGlyphRecordV1::from_runtime)
                .collect(),
        })
    }

    pub fn is_clean(&self) -> bool {
        self.selection_misses.is_empty() && self.missing_glyphs.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSvgTextFontSelectionMissRecordV1 {
    pub requested_families: Vec<String>,
    pub weight: u16,
    pub style: String,
    pub stretch: String,
}

impl UiSvgTextFontSelectionMissRecordV1 {
    fn from_runtime(record: &fret_runtime::RendererSvgTextFontSelectionMissRecord) -> Self {
        Self {
            requested_families: record.requested_families.clone(),
            weight: record.weight,
            style: record.style.clone(),
            stretch: record.stretch.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSvgTextFontFallbackRecordV1 {
    pub text: String,
    pub from_family: String,
    pub to_family: String,
}

impl UiSvgTextFontFallbackRecordV1 {
    fn from_runtime(record: &fret_runtime::RendererSvgTextFontFallbackRecord) -> Self {
        Self {
            text: record.text.clone(),
            from_family: record.from_family.clone(),
            to_family: record.to_family.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSvgTextMissingGlyphRecordV1 {
    pub text: String,
    pub resolved_family: String,
}

impl UiSvgTextMissingGlyphRecordV1 {
    fn from_runtime(record: &fret_runtime::RendererSvgTextMissingGlyphRecord) -> Self {
        Self {
            text: record.text.clone(),
            resolved_family: record.resolved_family.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiFontEnvironmentDiagnosticsSnapshotV1 {
    pub bundled_baseline_source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundled_profile_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundled_asset_bundle: Option<String>,
    #[serde(default)]
    pub bundled_asset_keys: Vec<String>,
    #[serde(default)]
    pub bundled_roles: Vec<String>,
    #[serde(default)]
    pub bundled_guaranteed_generic_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_catalog_revision: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_catalog_family_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renderer_font_environment_revision: Option<u64>,
    #[serde(default)]
    pub renderer_font_sources: Vec<UiRendererFontSourceSnapshotV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_font_stack_key: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_font_rescan_in_flight: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_font_rescan_pending: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRendererFontSourceSnapshotV1 {
    pub source_lane: String,
    pub byte_hash_hex: String,
    pub byte_len: u64,
    pub added_face_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_locator_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_bundle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_kind_hint: Option<String>,
}

impl UiRendererFontSourceSnapshotV1 {
    fn from_runtime(record: &fret_runtime::RendererFontSourceRecord) -> Self {
        let (asset_locator_kind, asset_bundle, asset_key) = record
            .asset_request
            .as_ref()
            .map(|request| match &request.locator {
                fret_assets::AssetLocator::Memory(_) => (Some("memory"), None, None),
                fret_assets::AssetLocator::Embedded(locator) => (
                    Some("embedded"),
                    Some(locator.owner.as_str().to_string()),
                    Some(locator.key.as_str().to_string()),
                ),
                fret_assets::AssetLocator::BundleAsset(locator) => (
                    Some("bundle_asset"),
                    Some(locator.bundle.as_str().to_string()),
                    Some(locator.key.as_str().to_string()),
                ),
                fret_assets::AssetLocator::File(_) => (Some("file"), None, None),
                fret_assets::AssetLocator::Url(_) => (Some("url"), None, None),
            })
            .unwrap_or((None, None, None));

        Self {
            source_lane: match record.source_lane {
                fret_runtime::RendererFontSourceLane::BundledStartup => "bundled_startup",
                fret_runtime::RendererFontSourceLane::AssetRequest => "asset_request",
            }
            .to_string(),
            byte_hash_hex: format!("{:016x}", record.byte_hash),
            byte_len: record.byte_len,
            added_face_count: record.added_face_count,
            asset_locator_kind: asset_locator_kind.map(str::to_string),
            asset_bundle,
            asset_key,
            asset_kind_hint: record
                .asset_request
                .as_ref()
                .and_then(|request| request.kind_hint)
                .map(|kind_hint| match kind_hint {
                    fret_assets::AssetKindHint::Binary => "binary",
                    fret_assets::AssetKindHint::Image => "image",
                    fret_assets::AssetKindHint::Svg => "svg",
                    fret_assets::AssetKindHint::Font => "font",
                })
                .map(str::to_string),
        }
    }
}

impl UiFontEnvironmentDiagnosticsSnapshotV1 {
    pub(super) fn from_runtime(
        baseline: Option<&fret_runtime::BundledFontBaselineSnapshot>,
        font_catalog: Option<&fret_runtime::FontCatalog>,
        renderer_font_environment: Option<&fret_runtime::RendererFontEnvironmentSnapshot>,
        text_font_stack_key: Option<u64>,
        system_font_rescan: Option<fret_runtime::SystemFontRescanState>,
    ) -> Option<Self> {
        if baseline.is_none()
            && font_catalog.is_none()
            && renderer_font_environment.is_none()
            && text_font_stack_key.is_none()
            && system_font_rescan.is_none()
        {
            return None;
        }

        let baseline = baseline.cloned().unwrap_or_default();
        Some(Self {
            bundled_baseline_source: match baseline.source {
                fret_runtime::BundledFontBaselineSource::None => "none",
                fret_runtime::BundledFontBaselineSource::BundledProfile => "bundled_profile",
            }
            .to_string(),
            bundled_profile_name: baseline.profile_name,
            bundled_asset_bundle: baseline.asset_bundle,
            bundled_asset_keys: baseline.asset_keys,
            bundled_roles: baseline.provided_roles,
            bundled_guaranteed_generic_families: baseline.guaranteed_generic_families,
            font_catalog_revision: font_catalog.map(|catalog| catalog.revision),
            font_catalog_family_count: font_catalog.map(|catalog| catalog.families.len() as u64),
            renderer_font_environment_revision: renderer_font_environment
                .map(|snapshot| snapshot.revision),
            renderer_font_sources: renderer_font_environment
                .map(|snapshot| {
                    snapshot
                        .sources
                        .iter()
                        .map(UiRendererFontSourceSnapshotV1::from_runtime)
                        .collect()
                })
                .unwrap_or_default(),
            text_font_stack_key: text_font_stack_key.or_else(|| {
                renderer_font_environment.and_then(|snapshot| snapshot.text_font_stack_key)
            }),
            system_font_rescan_in_flight: system_font_rescan.map(|state| state.in_flight),
            system_font_rescan_pending: system_font_rescan.map(|state| state.pending),
        })
    }
}

#[cfg(test)]
mod debug_snapshot_types_tests {
    use super::{
        UiAssetReloadDiagnosticsSnapshotV1, UiFontEnvironmentDiagnosticsSnapshotV1,
        UiSvgTextBridgeDiagnosticsSnapshotV1,
    };

    #[test]
    fn asset_reload_snapshot_from_runtime_keeps_backend_and_fallback_details() {
        let snapshot = UiAssetReloadDiagnosticsSnapshotV1::from_runtime(
            Some(fret_runtime::AssetReloadEpoch(7)),
            Some(fret_runtime::AssetReloadSupport { file_watch: true }),
            Some(fret_runtime::AssetReloadStatus {
                configured_backend: fret_runtime::AssetReloadBackendKind::NativeWatcher,
                active_backend: fret_runtime::AssetReloadBackendKind::PollMetadata,
                fallback_reason: Some(
                    fret_runtime::AssetReloadFallbackReason::WatcherInstallFailed,
                ),
                fallback_message: Some("backend unavailable".to_string()),
            }),
        )
        .expect("asset reload snapshot should be emitted when runtime state exists");

        assert_eq!(snapshot.epoch, Some(7));
        assert!(snapshot.file_watch);
        assert_eq!(
            snapshot.configured_backend.as_deref(),
            Some("native_watcher")
        );
        assert_eq!(snapshot.active_backend.as_deref(), Some("poll_metadata"));
        assert_eq!(
            snapshot.fallback_reason.as_deref(),
            Some("watcher_install_failed")
        );
        assert_eq!(
            snapshot.fallback_message.as_deref(),
            Some("backend unavailable")
        );
    }

    #[test]
    fn font_environment_snapshot_from_runtime_keeps_renderer_font_sources() {
        let snapshot = UiFontEnvironmentDiagnosticsSnapshotV1::from_runtime(
            Some(&fret_runtime::BundledFontBaselineSnapshot::bundled_profile(
                "default-subset",
                "pkg:fret-fonts",
                vec!["fonts/Inter-roman-subset.ttf".to_string()],
                vec!["UiSans".to_string()],
                vec!["Sans".to_string()],
            )),
            Some(&fret_runtime::FontCatalog {
                families: vec!["Inter".to_string()],
                revision: 3,
            }),
            Some(&fret_runtime::RendererFontEnvironmentSnapshot {
                revision: 2,
                text_font_stack_key: Some(99),
                sources: vec![fret_runtime::RendererFontSourceRecord::bundled_startup(
                    fret_assets::AssetRequest::new(fret_assets::AssetLocator::bundle(
                        fret_assets::AssetBundleId::package("fret-fonts"),
                        "fonts/Inter-roman-subset.ttf",
                    ))
                    .with_kind_hint(fret_assets::AssetKindHint::Font),
                    0x1234,
                    4096,
                    1,
                )],
            }),
            Some(99),
            Some(fret_runtime::SystemFontRescanState {
                in_flight: false,
                pending: true,
            }),
        )
        .expect("font environment snapshot");

        assert_eq!(snapshot.renderer_font_environment_revision, Some(2));
        assert_eq!(snapshot.renderer_font_sources.len(), 1);
        assert_eq!(
            snapshot.renderer_font_sources[0].source_lane,
            "bundled_startup"
        );
        assert_eq!(
            snapshot.renderer_font_sources[0].asset_bundle.as_deref(),
            Some("pkg:fret-fonts")
        );
        assert_eq!(
            snapshot.renderer_font_sources[0].asset_key.as_deref(),
            Some("fonts/Inter-roman-subset.ttf")
        );
        assert_eq!(
            snapshot.renderer_font_sources[0].asset_kind_hint.as_deref(),
            Some("font")
        );
        assert_eq!(
            snapshot.renderer_font_sources[0].byte_hash_hex,
            "0000000000001234"
        );
        assert_eq!(snapshot.text_font_stack_key, Some(99));
        assert_eq!(snapshot.system_font_rescan_pending, Some(true));
    }

    #[test]
    fn svg_text_bridge_snapshot_from_runtime_keeps_structured_records() {
        let snapshot = UiSvgTextBridgeDiagnosticsSnapshotV1::from_runtime(Some(
            &fret_runtime::RendererSvgTextBridgeDiagnosticsSnapshot {
                revision: Some(7),
                selection_misses: vec![fret_runtime::RendererSvgTextFontSelectionMissRecord {
                    requested_families: vec!["Inter Missing".to_string()],
                    weight: 400,
                    style: "normal".to_string(),
                    stretch: "normal".to_string(),
                }],
                fallback_records: vec![fret_runtime::RendererSvgTextFontFallbackRecord {
                    text: "中".to_string(),
                    from_family: "Inter".to_string(),
                    to_family: "Noto Sans CJK SC".to_string(),
                }],
                missing_glyphs: vec![fret_runtime::RendererSvgTextMissingGlyphRecord {
                    text: "\u{0378}".to_string(),
                    resolved_family: "Inter".to_string(),
                }],
            },
        ))
        .expect("svg text bridge snapshot");

        assert_eq!(snapshot.revision, 7);
        assert_eq!(snapshot.selection_misses.len(), 1);
        assert_eq!(snapshot.fallback_records.len(), 1);
        assert_eq!(snapshot.missing_glyphs.len(), 1);
        assert!(!snapshot.is_clean());
        assert_eq!(
            snapshot.selection_misses[0].requested_families,
            vec!["Inter Missing".to_string()]
        );
        assert_eq!(snapshot.fallback_records[0].to_family, "Noto Sans CJK SC");
        assert_eq!(snapshot.missing_glyphs[0].resolved_family, "Inter");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWebImeBridgeDebugSnapshotV1 {
    pub enabled: bool,
    pub composing: bool,
    pub suppress_next_input: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_has_focus: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_element_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mount_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_pixel_ratio: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_value_chars: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_selection_start_utf16: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_selection_end_utf16: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_client_width_px: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_client_height_px: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_scroll_width_px: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub textarea_scroll_height_px: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_input_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_beforeinput_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_input_data: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_key_code: Option<KeyCode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_cursor_area: Option<RectV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_cursor_anchor_px: Option<(f32, f32)>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_preedit_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_preedit_cursor_utf16: Option<(u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit_text: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_events: Vec<String>,

    pub beforeinput_seen: u64,
    pub input_seen: u64,
    pub suppressed_input_seen: u64,
    pub composition_start_seen: u64,
    pub composition_update_seen: u64,
    pub composition_end_seen: u64,
    pub cursor_area_set_seen: u64,
}

impl UiWebImeBridgeDebugSnapshotV1 {
    fn from_snapshot(
        snapshot: &fret_core::input::WebImeBridgeDebugSnapshot,
        redact_text: bool,
        max_debug_string_bytes: usize,
    ) -> Self {
        let mut recent_events = if redact_text {
            Vec::new()
        } else {
            snapshot.recent_events.clone()
        };
        for ev in &mut recent_events {
            truncate_string_bytes(ev, max_debug_string_bytes);
        }

        Self {
            enabled: snapshot.enabled,
            composing: snapshot.composing,
            suppress_next_input: snapshot.suppress_next_input,
            textarea_has_focus: snapshot.textarea_has_focus,
            active_element_tag: snapshot.active_element_tag.clone(),
            position_mode: snapshot.position_mode.clone(),
            mount_kind: snapshot.mount_kind.clone(),
            device_pixel_ratio: snapshot.device_pixel_ratio,
            textarea_value_chars: snapshot.textarea_value_chars,
            textarea_selection_start_utf16: snapshot.textarea_selection_start_utf16,
            textarea_selection_end_utf16: snapshot.textarea_selection_end_utf16,
            textarea_client_width_px: snapshot.textarea_client_width_px,
            textarea_client_height_px: snapshot.textarea_client_height_px,
            textarea_scroll_width_px: snapshot.textarea_scroll_width_px,
            textarea_scroll_height_px: snapshot.textarea_scroll_height_px,
            last_input_type: snapshot.last_input_type.clone(),
            last_beforeinput_data: (!redact_text)
                .then(|| snapshot.last_beforeinput_data.clone())
                .flatten(),
            last_input_data: (!redact_text)
                .then(|| snapshot.last_input_data.clone())
                .flatten(),
            last_key_code: snapshot.last_key_code,
            last_cursor_area: snapshot.last_cursor_area.map(RectV1::from),
            last_cursor_anchor_px: snapshot.last_cursor_anchor_px,
            last_preedit_text: (!redact_text)
                .then(|| snapshot.last_preedit_text.clone())
                .flatten(),
            last_preedit_cursor_utf16: snapshot.last_preedit_cursor_utf16,
            last_commit_text: (!redact_text)
                .then(|| snapshot.last_commit_text.clone())
                .flatten(),
            recent_events,
            beforeinput_seen: snapshot.beforeinput_seen,
            input_seen: snapshot.input_seen,
            suppressed_input_seen: snapshot.suppressed_input_seen,
            composition_start_seen: snapshot.composition_start_seen,
            composition_update_seen: snapshot.composition_update_seen,
            composition_end_seen: snapshot.composition_end_seen,
            cursor_area_set_seen: snapshot.cursor_area_set_seen,
        }
    }
}
