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

    pub(super) fn push_snapshot(
        &mut self,
        cfg: &UiDiagnosticsConfig,
        snapshot: UiDiagnosticsSnapshotV1,
    ) {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiResourceCachesV1 {
    #[serde(default)]
    pub icon_svg_cache: Option<UiRetainedSvgCacheStatsV1>,
    #[serde(default)]
    pub canvas: Vec<UiCanvasCacheEntryV1>,
    #[serde(default)]
    pub render_text: Option<UiRendererTextPerfSnapshotV1>,
    #[serde(default)]
    pub render_text_font_trace: Option<UiRendererTextFontTraceSnapshotV1>,
    #[serde(default)]
    pub render_text_fallback_policy: Option<UiRendererTextFallbackPolicySnapshotV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRetainedSvgCacheStatsV1 {
    pub entries: usize,
    pub bytes_ready: u64,
    pub stats: UiCacheStatsV1,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiCacheStatsV1 {
    pub get_calls: u64,
    pub get_hits: u64,
    pub get_misses: u64,
    pub prepare_calls: u64,
    pub prepare_hits: u64,
    pub prepare_misses: u64,
    pub prune_calls: u64,
    pub clear_calls: u64,
    pub evict_calls: u64,
    pub release_replaced: u64,
    pub release_prune_age: u64,
    pub release_prune_budget: u64,
    pub release_clear: u64,
    pub release_evict: u64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiRendererTextPerfSnapshotV1 {
    pub frame_id: u64,

    pub font_stack_key: u64,
    pub font_db_revision: u64,
    #[serde(default)]
    pub fallback_policy_key: u64,

    #[serde(default)]
    pub frame_missing_glyphs: u64,
    #[serde(default)]
    pub frame_texts_with_missing_glyphs: u64,

    pub blobs_live: u64,
    pub blob_cache_entries: u64,
    pub shape_cache_entries: u64,
    pub measure_cache_buckets: u64,

    #[serde(default)]
    pub unwrapped_layout_cache_entries: u64,
    #[serde(default)]
    pub frame_unwrapped_layout_cache_hits: u64,
    #[serde(default)]
    pub frame_unwrapped_layout_cache_misses: u64,
    #[serde(default)]
    pub frame_unwrapped_layouts_created: u64,

    pub frame_cache_resets: u64,
    pub frame_blob_cache_hits: u64,
    pub frame_blob_cache_misses: u64,
    pub frame_blobs_created: u64,
    pub frame_shape_cache_hits: u64,
    pub frame_shape_cache_misses: u64,
    pub frame_shapes_created: u64,

    pub mask_atlas: UiRendererGlyphAtlasPerfSnapshotV1,
    pub color_atlas: UiRendererGlyphAtlasPerfSnapshotV1,
    pub subpixel_atlas: UiRendererGlyphAtlasPerfSnapshotV1,
}

impl UiRendererTextPerfSnapshotV1 {
    pub(super) fn from_core(snapshot: fret_core::RendererTextPerfSnapshot) -> Self {
        Self {
            frame_id: snapshot.frame_id.0,
            font_stack_key: snapshot.font_stack_key,
            font_db_revision: snapshot.font_db_revision,
            fallback_policy_key: snapshot.fallback_policy_key,
            frame_missing_glyphs: snapshot.frame_missing_glyphs,
            frame_texts_with_missing_glyphs: snapshot.frame_texts_with_missing_glyphs,
            blobs_live: snapshot.blobs_live,
            blob_cache_entries: snapshot.blob_cache_entries,
            shape_cache_entries: snapshot.shape_cache_entries,
            measure_cache_buckets: snapshot.measure_cache_buckets,
            unwrapped_layout_cache_entries: snapshot.unwrapped_layout_cache_entries,
            frame_unwrapped_layout_cache_hits: snapshot.frame_unwrapped_layout_cache_hits,
            frame_unwrapped_layout_cache_misses: snapshot.frame_unwrapped_layout_cache_misses,
            frame_unwrapped_layouts_created: snapshot.frame_unwrapped_layouts_created,
            frame_cache_resets: snapshot.frame_cache_resets,
            frame_blob_cache_hits: snapshot.frame_blob_cache_hits,
            frame_blob_cache_misses: snapshot.frame_blob_cache_misses,
            frame_blobs_created: snapshot.frame_blobs_created,
            frame_shape_cache_hits: snapshot.frame_shape_cache_hits,
            frame_shape_cache_misses: snapshot.frame_shape_cache_misses,
            frame_shapes_created: snapshot.frame_shapes_created,
            mask_atlas: UiRendererGlyphAtlasPerfSnapshotV1::from_core(snapshot.mask_atlas),
            color_atlas: UiRendererGlyphAtlasPerfSnapshotV1::from_core(snapshot.color_atlas),
            subpixel_atlas: UiRendererGlyphAtlasPerfSnapshotV1::from_core(snapshot.subpixel_atlas),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextCommonFallbackInjectionV1 {
    PlatformDefault,
    None,
    CommonFallback,
}

impl Default for UiTextCommonFallbackInjectionV1 {
    fn default() -> Self {
        Self::PlatformDefault
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRendererTextFallbackPolicySnapshotV1 {
    pub frame_id: u64,
    pub font_stack_key: u64,
    pub font_db_revision: u64,
    pub fallback_policy_key: u64,

    #[serde(default)]
    pub system_fonts_enabled: bool,
    #[serde(default)]
    pub prefer_common_fallback: bool,

    #[serde(default)]
    pub common_fallback_injection: UiTextCommonFallbackInjectionV1,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale_bcp47: Option<String>,

    #[serde(default)]
    pub common_fallback_stack_suffix: String,
    #[serde(default)]
    pub common_fallback_candidates: Vec<String>,
}

impl UiRendererTextFallbackPolicySnapshotV1 {
    pub(super) fn from_core(
        snapshot: fret_core::RendererTextFallbackPolicySnapshot,
        max_debug_string_bytes: usize,
    ) -> Self {
        fn injection_from_core(
            injection: fret_core::TextCommonFallbackInjection,
        ) -> UiTextCommonFallbackInjectionV1 {
            match injection {
                fret_core::TextCommonFallbackInjection::PlatformDefault => {
                    UiTextCommonFallbackInjectionV1::PlatformDefault
                }
                fret_core::TextCommonFallbackInjection::None => {
                    UiTextCommonFallbackInjectionV1::None
                }
                fret_core::TextCommonFallbackInjection::CommonFallback => {
                    UiTextCommonFallbackInjectionV1::CommonFallback
                }
            }
        }

        let mut locale_bcp47 = snapshot.locale_bcp47;
        if let Some(locale) = locale_bcp47.as_mut() {
            truncate_string_bytes(locale, max_debug_string_bytes);
        }

        let mut common_fallback_stack_suffix = snapshot.common_fallback_stack_suffix;
        truncate_string_bytes(&mut common_fallback_stack_suffix, max_debug_string_bytes);

        let mut common_fallback_candidates = snapshot.common_fallback_candidates;
        for s in &mut common_fallback_candidates {
            truncate_string_bytes(s, max_debug_string_bytes);
        }

        Self {
            frame_id: snapshot.frame_id.0,
            font_stack_key: snapshot.font_stack_key,
            font_db_revision: snapshot.font_db_revision,
            fallback_policy_key: snapshot.fallback_policy_key,
            system_fonts_enabled: snapshot.system_fonts_enabled,
            prefer_common_fallback: snapshot.prefer_common_fallback,
            common_fallback_injection: injection_from_core(snapshot.common_fallback_injection),
            locale_bcp47,
            common_fallback_stack_suffix,
            common_fallback_candidates,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRendererTextFontTraceSnapshotV1 {
    pub frame_id: u64,
    #[serde(default)]
    pub entries: Vec<UiRendererTextFontTraceEntryV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRendererTextFontTraceEntryV1 {
    pub text_preview: String,
    pub text_len_bytes: u32,

    pub font: String,
    pub font_size_px: f32,
    pub scale_factor: f32,

    pub wrap: String,
    pub overflow: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width_px: Option<f32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale_bcp47: Option<String>,

    pub missing_glyphs: u32,

    #[serde(default)]
    pub families: Vec<UiRendererTextFontTraceFamilyUsageV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiRendererTextFontTraceFamilyUsageV1 {
    pub family: String,
    pub glyphs: u32,
    pub missing_glyphs: u32,
    pub class: UiRendererTextFontTraceFamilyClassV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRendererTextFontTraceFamilyClassV1 {
    Requested,
    CommonFallback,
    SystemFallback,
    Unknown,
}

impl Default for UiRendererTextFontTraceFamilyClassV1 {
    fn default() -> Self {
        Self::Unknown
    }
}

impl UiRendererTextFontTraceSnapshotV1 {
    pub(super) fn from_core(
        snapshot: fret_core::RendererTextFontTraceSnapshot,
        redact_text: bool,
        max_debug_string_bytes: usize,
    ) -> Self {
        fn wrap_to_string(wrap: fret_core::TextWrap) -> &'static str {
            match wrap {
                fret_core::TextWrap::None => "none",
                fret_core::TextWrap::Word => "word",
                fret_core::TextWrap::Balance => "balance",
                fret_core::TextWrap::WordBreak => "word_break",
                fret_core::TextWrap::Grapheme => "grapheme",
            }
        }

        fn overflow_to_string(overflow: fret_core::TextOverflow) -> &'static str {
            match overflow {
                fret_core::TextOverflow::Clip => "clip",
                fret_core::TextOverflow::Ellipsis => "ellipsis",
            }
        }

        fn font_id_to_string(font: &fret_core::FontId) -> String {
            match font {
                fret_core::FontId::Ui => "ui".to_string(),
                fret_core::FontId::Serif => "serif".to_string(),
                fret_core::FontId::Monospace => "monospace".to_string(),
                fret_core::FontId::Family(name) => format!("family:{name}"),
            }
        }

        fn class_from_core(
            class: fret_core::RendererTextFontTraceFamilyClass,
        ) -> UiRendererTextFontTraceFamilyClassV1 {
            match class {
                fret_core::RendererTextFontTraceFamilyClass::Requested => {
                    UiRendererTextFontTraceFamilyClassV1::Requested
                }
                fret_core::RendererTextFontTraceFamilyClass::CommonFallback => {
                    UiRendererTextFontTraceFamilyClassV1::CommonFallback
                }
                fret_core::RendererTextFontTraceFamilyClass::SystemFallback => {
                    UiRendererTextFontTraceFamilyClassV1::SystemFallback
                }
                fret_core::RendererTextFontTraceFamilyClass::Unknown => {
                    UiRendererTextFontTraceFamilyClassV1::Unknown
                }
            }
        }

        let mut entries = snapshot
            .entries
            .into_iter()
            .map(|mut e| {
                if redact_text {
                    e.text_preview = "<redacted>".to_string();
                }
                truncate_string_bytes(&mut e.text_preview, max_debug_string_bytes);
                if let Some(locale) = e.locale_bcp47.as_mut() {
                    truncate_string_bytes(locale, max_debug_string_bytes);
                }

                let mut families: Vec<UiRendererTextFontTraceFamilyUsageV1> = e
                    .families
                    .into_iter()
                    .map(|mut f| {
                        truncate_string_bytes(&mut f.family, max_debug_string_bytes);
                        UiRendererTextFontTraceFamilyUsageV1 {
                            family: f.family,
                            glyphs: f.glyphs,
                            missing_glyphs: f.missing_glyphs,
                            class: class_from_core(f.class),
                        }
                    })
                    .collect();
                families.sort_by(|a, b| {
                    b.glyphs
                        .cmp(&a.glyphs)
                        .then_with(|| a.family.cmp(&b.family))
                });

                UiRendererTextFontTraceEntryV1 {
                    text_preview: e.text_preview,
                    text_len_bytes: e.text_len_bytes,
                    font: font_id_to_string(&e.font),
                    font_size_px: e.font_size.0,
                    scale_factor: e.scale_factor,
                    wrap: wrap_to_string(e.wrap).to_string(),
                    overflow: overflow_to_string(e.overflow).to_string(),
                    max_width_px: e.max_width.map(|px| px.0),
                    locale_bcp47: e.locale_bcp47,
                    missing_glyphs: e.missing_glyphs,
                    families,
                }
            })
            .collect::<Vec<_>>();
        entries.truncate(4096);

        Self {
            frame_id: snapshot.frame_id.0,
            entries,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiRendererGlyphAtlasPerfSnapshotV1 {
    pub width: u32,
    pub height: u32,
    pub pages: u32,
    pub entries: u64,

    pub used_px: u64,
    pub capacity_px: u64,

    pub frame_hits: u64,
    pub frame_misses: u64,
    pub frame_inserts: u64,
    pub frame_evict_glyphs: u64,
    pub frame_evict_pages: u64,
    pub frame_out_of_space: u64,
    pub frame_too_large: u64,

    pub frame_pending_uploads: u64,
    pub frame_pending_upload_bytes: u64,
    pub frame_upload_bytes: u64,
}

impl UiRendererGlyphAtlasPerfSnapshotV1 {
    pub(super) fn from_core(snapshot: fret_core::RendererGlyphAtlasPerfSnapshot) -> Self {
        Self {
            width: snapshot.width,
            height: snapshot.height,
            pages: snapshot.pages,
            entries: snapshot.entries,
            used_px: snapshot.used_px,
            capacity_px: snapshot.capacity_px,
            frame_hits: snapshot.frame_hits,
            frame_misses: snapshot.frame_misses,
            frame_inserts: snapshot.frame_inserts,
            frame_evict_glyphs: snapshot.frame_evict_glyphs,
            frame_evict_pages: snapshot.frame_evict_pages,
            frame_out_of_space: snapshot.frame_out_of_space,
            frame_too_large: snapshot.frame_too_large,
            frame_pending_uploads: snapshot.frame_pending_uploads,
            frame_pending_upload_bytes: snapshot.frame_pending_upload_bytes,
            frame_upload_bytes: snapshot.frame_upload_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiSceneOpTileCacheStatsV1 {
    pub calls: u64,
    pub hits: u64,
    pub misses: u64,
    pub stored_tiles: u64,
    pub recorded_ops: u64,
    pub replayed_ops: u64,
    pub clear_calls: u64,
    pub prune_calls: u64,
    pub evict_calls: u64,
    pub evict_prune_age: u64,
    pub evict_prune_budget: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiSceneOpTileCacheSnapshotV1 {
    pub entries: usize,
    #[serde(default)]
    pub requested_tiles: usize,
    #[serde(default)]
    pub budget_limit: u32,
    #[serde(default)]
    pub budget_used: u32,
    #[serde(default)]
    pub skipped_tiles: u32,
    pub stats: UiSceneOpTileCacheStatsV1,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiWorkBudgetSnapshotV1 {
    pub requested_units: u32,
    pub limit: u32,
    pub used: u32,
    pub skipped_units: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiCanvasCacheEntryV1 {
    pub node: u64,
    pub name: String,
    #[serde(default)]
    pub path: Option<UiCacheKindSnapshotV1>,
    #[serde(default)]
    pub svg: Option<UiCacheKindSnapshotV1>,
    #[serde(default)]
    pub text: Option<UiCacheKindSnapshotV1>,
    #[serde(default)]
    pub scene_op_tiles: Option<UiSceneOpTileCacheSnapshotV1>,
    #[serde(default)]
    pub work_budget: Option<UiWorkBudgetSnapshotV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiCacheKindSnapshotV1 {
    pub entries: usize,
    pub bytes_ready: u64,
    pub stats: UiCacheStatsV1,
}
