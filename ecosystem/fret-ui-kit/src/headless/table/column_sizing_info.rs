use super::ColumnId;

/// TanStack-inspired transient column sizing info for interactive resizing.
#[derive(Debug, Clone, Default)]
pub struct ColumnSizingInfoState {
    pub is_resizing_column: Option<ColumnId>,
    pub start_pointer_x: f32,
    pub start_size: f32,
}
