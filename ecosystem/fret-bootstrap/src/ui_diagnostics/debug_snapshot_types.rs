#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub hit_test: Option<UiHitTestSnapshotV1>,
    pub element_runtime: Option<ElementDiagnosticsSnapshotV1>,
    pub semantics: Option<UiSemanticsSnapshotV1>,
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
