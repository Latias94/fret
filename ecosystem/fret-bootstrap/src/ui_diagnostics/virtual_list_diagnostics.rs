#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListMeasureModeV1 {
    Fixed,
    Measured,
    Known,
}

impl UiVirtualListMeasureModeV1 {
    fn from_mode(mode: fret_ui::element::VirtualListMeasureMode) -> Self {
        match mode {
            fret_ui::element::VirtualListMeasureMode::Fixed => Self::Fixed,
            fret_ui::element::VirtualListMeasureMode::Measured => Self::Measured,
            fret_ui::element::VirtualListMeasureMode::Known => Self::Known,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum UiVirtualListWindowShiftKindV1 {
    #[default]
    None,
    Prefetch,
    Escape,
}


impl UiVirtualListWindowShiftKindV1 {
    fn from_kind(kind: fret_ui::tree::UiDebugVirtualListWindowShiftKind) -> Self {
        match kind {
            fret_ui::tree::UiDebugVirtualListWindowShiftKind::None => Self::None,
            fret_ui::tree::UiDebugVirtualListWindowShiftKind::Prefetch => Self::Prefetch,
            fret_ui::tree::UiDebugVirtualListWindowShiftKind::Escape => Self::Escape,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListWindowShiftReasonV1 {
    ScrollOffset,
    ViewportResize,
    ItemsRevision,
    ScrollToItem,
    InputsChange,
    Unknown,
}

impl UiVirtualListWindowShiftReasonV1 {
    fn from_reason(reason: fret_ui::tree::UiDebugVirtualListWindowShiftReason) -> Self {
        match reason {
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ScrollOffset => Self::ScrollOffset,
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ViewportResize => {
                Self::ViewportResize
            }
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ItemsRevision => {
                Self::ItemsRevision
            }
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::ScrollToItem => Self::ScrollToItem,
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::InputsChange => Self::InputsChange,
            fret_ui::tree::UiDebugVirtualListWindowShiftReason::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiVirtualListWindowShiftApplyModeV1 {
    RetainedReconcile,
    NonRetainedRerender,
}

impl UiVirtualListWindowShiftApplyModeV1 {
    fn from_mode(mode: fret_ui::tree::UiDebugVirtualListWindowShiftApplyMode) -> Self {
        match mode {
            fret_ui::tree::UiDebugVirtualListWindowShiftApplyMode::RetainedReconcile => {
                Self::RetainedReconcile
            }
            fret_ui::tree::UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender => {
                Self::NonRetainedRerender
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiVirtualRangeV1 {
    pub start_index: u64,
    pub end_index: u64,
    pub overscan: u64,
    pub count: u64,
}

impl UiVirtualRangeV1 {
    fn from_range(range: fret_ui::virtual_list::VirtualRange) -> Self {
        Self {
            start_index: range.start_index as u64,
            end_index: range.end_index as u64,
            overscan: range.overscan as u64,
            count: range.count as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiVirtualListWindowV1 {
    pub node: u64,
    pub element: u64,
    #[serde(default)]
    pub source: UiVirtualListWindowSourceV1,
    pub axis: UiAxisV1,
    #[serde(default)]
    pub is_probe_layout: bool,
    pub items_len: u64,
    pub items_revision: u64,
    pub prev_items_revision: u64,
    pub measure_mode: UiVirtualListMeasureModeV1,
    pub overscan: u64,
    #[serde(default)]
    pub policy_key: u64,
    #[serde(default)]
    pub inputs_key: u64,
    pub viewport: f32,
    pub prev_viewport: f32,
    pub offset: f32,
    pub prev_offset: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_window_range: Option<UiVirtualRangeV1>,
    #[serde(default)]
    pub deferred_scroll_to_item: bool,
    #[serde(default)]
    pub deferred_scroll_consumed: bool,
    #[serde(default)]
    pub window_mismatch: bool,
    #[serde(default)]
    pub window_shift_kind: UiVirtualListWindowShiftKindV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_reason: Option<UiVirtualListWindowShiftReasonV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_apply_mode: Option<UiVirtualListWindowShiftApplyModeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_invalidation_detail: Option<String>,
}

impl UiVirtualListWindowV1 {
    fn from_window(window: &fret_ui::tree::UiDebugVirtualListWindow) -> Self {
        Self {
            node: key_to_u64(window.node),
            element: window.element.0,
            source: UiVirtualListWindowSourceV1::from_source(window.source),
            axis: UiAxisV1::from_axis(window.axis),
            is_probe_layout: window.is_probe_layout,
            items_len: window.items_len as u64,
            items_revision: window.items_revision,
            prev_items_revision: window.prev_items_revision,
            measure_mode: UiVirtualListMeasureModeV1::from_mode(window.measure_mode),
            overscan: window.overscan as u64,
            policy_key: window.policy_key,
            inputs_key: window.inputs_key,
            viewport: window.viewport.0,
            prev_viewport: window.prev_viewport.0,
            offset: window.offset.0,
            prev_offset: window.prev_offset.0,
            window_range: window.window_range.map(UiVirtualRangeV1::from_range),
            prev_window_range: window.prev_window_range.map(UiVirtualRangeV1::from_range),
            render_window_range: window.render_window_range.map(UiVirtualRangeV1::from_range),
            deferred_scroll_to_item: window.deferred_scroll_to_item,
            deferred_scroll_consumed: window.deferred_scroll_consumed,
            window_mismatch: window.window_mismatch,
            window_shift_kind: UiVirtualListWindowShiftKindV1::from_kind(window.window_shift_kind),
            window_shift_reason: window
                .window_shift_reason
                .map(UiVirtualListWindowShiftReasonV1::from_reason),
            window_shift_apply_mode: window
                .window_shift_apply_mode
                .map(UiVirtualListWindowShiftApplyModeV1::from_mode),
            window_shift_invalidation_detail: window
                .window_shift_invalidation_detail
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiVirtualListWindowShiftSampleV1 {
    pub frame_id: u64,
    pub source: UiVirtualListWindowSourceV1,
    pub node: u64,
    pub element: u64,
    pub window_shift_kind: UiVirtualListWindowShiftKindV1,
    pub window_shift_reason: UiVirtualListWindowShiftReasonV1,
    pub window_shift_apply_mode: UiVirtualListWindowShiftApplyModeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_shift_invalidation_detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_range: Option<UiVirtualRangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render_window_range: Option<UiVirtualRangeV1>,
}

impl UiVirtualListWindowShiftSampleV1 {
    fn from_sample(sample: &fret_ui::tree::UiDebugVirtualListWindowShiftSample) -> Self {
        Self {
            frame_id: sample.frame_id.0,
            source: UiVirtualListWindowSourceV1::from_source(sample.source),
            node: key_to_u64(sample.node),
            element: sample.element.0,
            window_shift_kind: UiVirtualListWindowShiftKindV1::from_kind(sample.window_shift_kind),
            window_shift_reason: UiVirtualListWindowShiftReasonV1::from_reason(
                sample.window_shift_reason,
            ),
            window_shift_apply_mode: UiVirtualListWindowShiftApplyModeV1::from_mode(
                sample.window_shift_apply_mode,
            ),
            window_shift_invalidation_detail: sample
                .window_shift_invalidation_detail
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
            prev_window_range: sample.prev_window_range.map(UiVirtualRangeV1::from_range),
            window_range: sample.window_range.map(UiVirtualRangeV1::from_range),
            render_window_range: sample.render_window_range.map(UiVirtualRangeV1::from_range),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWindowedRowsSurfaceWindowV1 {
    pub callsite_id: u64,
    pub location: UiSourceLocationV1,

    pub len: u64,
    pub row_height: f32,
    pub overscan: u64,
    pub gap: f32,
    pub scroll_margin: f32,

    pub viewport_height: f32,
    pub offset_y: f32,
    pub content_height: f32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_start: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_end: Option<u64>,
    pub visible_count: u64,
}

impl UiWindowedRowsSurfaceWindowV1 {
    fn from_telemetry(
        telemetry: &fret_ui_kit::declarative::windowed_rows_surface::WindowedRowsSurfaceWindowTelemetry,
    ) -> Self {
        Self {
            callsite_id: telemetry.callsite_id,
            location: UiSourceLocationV1 {
                file: telemetry.file.to_string(),
                line: telemetry.line,
                column: telemetry.column,
            },
            len: telemetry.len,
            row_height: telemetry.row_height.0,
            overscan: telemetry.overscan,
            gap: telemetry.gap.0,
            scroll_margin: telemetry.scroll_margin.0,
            viewport_height: telemetry.viewport_height.0,
            offset_y: telemetry.offset_y.0,
            content_height: telemetry.content_height.0,
            visible_start: telemetry.visible_start,
            visible_end: telemetry.visible_end,
            visible_count: telemetry.visible_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiRetainedVirtualListReconcileV1 {
    pub node: u64,
    pub element: u64,
    pub prev_items: u64,
    pub next_items: u64,
    pub preserved_items: u64,
    pub attached_items: u64,
    pub detached_items: u64,
    #[serde(default)]
    pub reused_from_keep_alive_items: u64,
    #[serde(default)]
    pub kept_alive_items: u64,
    #[serde(default)]
    pub evicted_keep_alive_items: u64,
    #[serde(default)]
    pub keep_alive_pool_len_before: u64,
    #[serde(default)]
    pub keep_alive_pool_len_after: u64,
}

impl UiRetainedVirtualListReconcileV1 {
    fn from_record(record: &fret_ui::tree::UiDebugRetainedVirtualListReconcile) -> Self {
        Self {
            node: key_to_u64(record.node),
            element: record.element.0,
            prev_items: record.prev_items as u64,
            next_items: record.next_items as u64,
            preserved_items: record.preserved_items as u64,
            attached_items: record.attached_items as u64,
            detached_items: record.detached_items as u64,
            reused_from_keep_alive_items: record.reused_from_keep_alive_items as u64,
            kept_alive_items: record.kept_alive_items as u64,
            evicted_keep_alive_items: record.evicted_keep_alive_items as u64,
            keep_alive_pool_len_before: record.keep_alive_pool_len_before as u64,
            keep_alive_pool_len_after: record.keep_alive_pool_len_after as u64,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum UiVirtualListWindowSourceV1 {
    Prepaint,
    #[serde(other)]
    #[default]
    Layout,
}


impl UiVirtualListWindowSourceV1 {
    fn from_source(source: fret_ui::tree::UiDebugVirtualListWindowSource) -> Self {
        match source {
            fret_ui::tree::UiDebugVirtualListWindowSource::Layout => Self::Layout,
            fret_ui::tree::UiDebugVirtualListWindowSource::Prepaint => Self::Prepaint,
        }
    }
}
