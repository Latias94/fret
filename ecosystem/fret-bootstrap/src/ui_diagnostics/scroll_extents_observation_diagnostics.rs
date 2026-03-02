#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiScrollAxisV1 {
    X,
    Y,
    Both,
}

impl UiScrollAxisV1 {
    fn from_axis(axis: fret_ui::tree::UiDebugScrollAxis) -> Self {
        match axis {
            fret_ui::tree::UiDebugScrollAxis::X => Self::X,
            fret_ui::tree::UiDebugScrollAxis::Y => Self::Y,
            fret_ui::tree::UiDebugScrollAxis::Both => Self::Both,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiScrollOverflowObservationTelemetryV1 {
    pub extent_may_be_stale: bool,
    pub barrier_roots: u8,
    pub wrapper_peel_budget: u8,
    pub wrapper_peeled_max: u8,
    pub wrapper_peel_budget_hit: bool,
    pub immediate_children_visited: u16,
    pub immediate_children_skipped_absolute: u16,
    pub deep_scan_enabled: bool,
    pub deep_scan_budget_nodes: u16,
    pub deep_scan_visited: u16,
    pub deep_scan_budget_hit: bool,
    pub deep_scan_skipped_absolute: u16,
}

impl UiScrollOverflowObservationTelemetryV1 {
    fn from_telemetry(t: fret_ui::tree::UiDebugScrollOverflowObservationTelemetry) -> Self {
        Self {
            extent_may_be_stale: t.extent_may_be_stale,
            barrier_roots: t.barrier_roots,
            wrapper_peel_budget: t.wrapper_peel_budget,
            wrapper_peeled_max: t.wrapper_peeled_max,
            wrapper_peel_budget_hit: t.wrapper_peel_budget_hit,
            immediate_children_visited: t.immediate_children_visited,
            immediate_children_skipped_absolute: t.immediate_children_skipped_absolute,
            deep_scan_enabled: t.deep_scan_enabled,
            deep_scan_budget_nodes: t.deep_scan_budget_nodes,
            deep_scan_visited: t.deep_scan_visited,
            deep_scan_budget_hit: t.deep_scan_budget_hit,
            deep_scan_skipped_absolute: t.deep_scan_skipped_absolute,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiScrollNodeTelemetryV1 {
    pub node: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<u64>,
    pub axis: UiScrollAxisV1,
    pub offset_x: f32,
    pub offset_y: f32,
    pub viewport_w: f32,
    pub viewport_h: f32,
    pub content_w: f32,
    pub content_h: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_w: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_h: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow_observation: Option<UiScrollOverflowObservationTelemetryV1>,
}

impl UiScrollNodeTelemetryV1 {
    fn from_record(record: &fret_ui::tree::UiDebugScrollNodeTelemetry) -> Self {
        Self {
            node: key_to_u64(record.node),
            element: record.element.map(|e| e.0),
            axis: UiScrollAxisV1::from_axis(record.axis),
            offset_x: record.offset.x.0,
            offset_y: record.offset.y.0,
            viewport_w: record.viewport.width.0,
            viewport_h: record.viewport.height.0,
            content_w: record.content.width.0,
            content_h: record.content.height.0,
            observed_w: record.observed_extent.map(|s| s.width.0),
            observed_h: record.observed_extent.map(|s| s.height.0),
            overflow_observation: record
                .overflow_observation
                .map(UiScrollOverflowObservationTelemetryV1::from_telemetry),
        }
    }
}

