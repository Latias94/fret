use super::ColumnId;

/// TanStack-inspired transient column sizing info for interactive resizing.
#[derive(Debug, Clone, Default)]
pub struct ColumnSizingInfoState {
    pub column_sizing_start: Vec<(ColumnId, f32)>,
    pub delta_offset: Option<f32>,
    pub delta_percentage: Option<f32>,
    pub is_resizing_column: Option<ColumnId>,
    pub start_offset: Option<f32>,
    pub start_size: Option<f32>,
}
