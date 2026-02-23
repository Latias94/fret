use super::*;

#[derive(Default)]
pub(super) struct WindowRing {
    pub(super) last_pointer_position: Option<Point>,
    pub(super) last_pointer_type: Option<fret_core::PointerType>,
    pub(super) events: VecDeque<RecordedUiEventV1>,
    pub(super) snapshots: VecDeque<UiDiagnosticsSnapshotV1>,
    pub(super) snapshot_seq: u64,
    pub(super) viewport_input_this_frame: Vec<UiViewportInputEventV1>,
    pub(super) last_changed_models: Vec<u64>,
    pub(super) last_changed_globals: Vec<String>,
}

impl WindowRing {
    pub(super) fn update_pointer_position(&mut self, event: &Event) {
        match event {
            Event::Pointer(e) => match e {
                fret_core::PointerEvent::Move {
                    position,
                    pointer_type,
                    ..
                }
                | fret_core::PointerEvent::Down {
                    position,
                    pointer_type,
                    ..
                }
                | fret_core::PointerEvent::Up {
                    position,
                    pointer_type,
                    ..
                }
                | fret_core::PointerEvent::Wheel {
                    position,
                    pointer_type,
                    ..
                }
                | fret_core::PointerEvent::PinchGesture {
                    position,
                    pointer_type,
                    ..
                } => {
                    self.last_pointer_position = Some(*position);
                    self.last_pointer_type = Some(*pointer_type);
                }
            },
            Event::PointerCancel(e) => {
                self.last_pointer_position = e.position;
                self.last_pointer_type = Some(e.pointer_type);
            }
            _ => {}
        }
    }

    pub(super) fn clear(&mut self) {
        self.last_pointer_position = None;
        self.last_pointer_type = None;
        self.events.clear();
        self.snapshots.clear();
        self.snapshot_seq = 0;
        self.viewport_input_this_frame.clear();
        self.last_changed_models.clear();
        self.last_changed_globals.clear();
    }

    pub(super) fn push_event(&mut self, cfg: &UiDiagnosticsConfig, event: RecordedUiEventV1) {
        self.events.push_back(event);
        while self.events.len() > cfg.max_events {
            self.events.pop_front();
        }
    }

    pub(super) fn push_snapshot(&mut self, cfg: &UiDiagnosticsConfig, snapshot: UiDiagnosticsSnapshotV1) {
        self.snapshots.push_back(snapshot);
        while self.snapshots.len() > cfg.max_snapshots {
            self.snapshots.pop_front();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiFrameClockSnapshotV1 {
    pub now_monotonic_ms: u64,
    pub delta_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_delta_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiDiagnosticsSnapshotV1 {
    pub schema_version: u32,
    pub tick_id: u64,
    pub frame_id: u64,
    /// Per-window monotonic snapshot sequence (contiguous within a run).
    pub window_snapshot_seq: u64,
    pub window: u64,
    pub timestamp_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_clock: Option<UiFrameClockSnapshotV1>,
    pub scale_factor: f32,
    pub window_bounds: RectV1,
    pub scene_ops: u64,
    #[serde(default)]
    pub scene_fingerprint: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantics_fingerprint: Option<u64>,

    pub changed_models: Vec<u64>,
    pub changed_globals: Vec<String>,

    /// Aggregated writers for `changed_models`, derived from `ModelStore` debug info.
    ///
    /// This is best-effort and only populated in debug builds.
    #[serde(default)]
    pub changed_model_sources_top: Vec<UiChangedModelSourceHotspotV1>,

    #[serde(default)]
    pub resource_caches: Option<UiResourceCachesV1>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_snapshot: Option<serde_json::Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_area_insets: Option<UiEdgesV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occlusion_insets: Option<UiEdgesV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_is_text_input: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_composing: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clipboard: Option<UiClipboardDiagnosticsSnapshotV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_pointer_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caps: Option<UiPlatformCapabilitiesSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wgpu_adapter: Option<serde_json::Value>,

    pub debug: UiTreeDebugSnapshotV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiClipboardDiagnosticsSnapshotV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_read_token: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_read_unavailable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_read_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_write_unavailable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_write_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPlatformCapabilitiesSummaryV1 {
    pub platform: String,
    pub ui_window_hover_detection: String,
    pub clipboard_text: bool,
    pub clipboard_text_read: bool,
    pub clipboard_text_write: bool,
    pub clipboard_primary_text: bool,
    pub ime: bool,
    pub ime_set_cursor_area: bool,
    pub fs_file_dialogs: bool,
    pub shell_share_sheet: bool,
    pub shell_incoming_open: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiChangedModelSourceHotspotV1 {
    pub type_name: String,
    pub changed_at: UiSourceLocationV1,
    pub count: u32,
}

