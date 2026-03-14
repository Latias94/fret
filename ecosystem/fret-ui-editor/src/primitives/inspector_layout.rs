use fret_core::Px;
use fret_ui::Theme;

use super::{EditorDensity, EditorTokenKeys};

#[derive(Debug, Clone, Copy)]
pub(crate) struct InspectorLayoutMetrics {
    pub(crate) density: EditorDensity,
    pub(crate) column_gap: Px,
    pub(crate) trailing_gap: Px,
    pub(crate) row_gap: Px,
    pub(crate) label_width: Px,
    pub(crate) value_max_width: Px,
    pub(crate) status_slot_width: Px,
    pub(crate) reset_slot_width: Px,
    pub(crate) auto_stack_below: Px,
    pub(crate) group_header_height: Px,
    pub(crate) group_content_gap: Px,
    pub(crate) panel_gap: Px,
    pub(crate) panel_header_gap: Px,
}

impl InspectorLayoutMetrics {
    pub(crate) fn resolve(theme: &Theme) -> Self {
        let density = EditorDensity::resolve(theme);

        Self {
            density,
            column_gap: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_COLUMN_GAP)
                .unwrap_or(Px(10.0)),
            trailing_gap: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_TRAILING_GAP)
                .unwrap_or(Px(6.0)),
            row_gap: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_ROW_GAP)
                .unwrap_or(Px(5.0)),
            label_width: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_LABEL_WIDTH)
                .unwrap_or(Px(124.0)),
            value_max_width: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_VALUE_MAX_WIDTH)
                .unwrap_or(Px(1024.0)),
            status_slot_width: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_STATUS_SLOT_WIDTH)
                .unwrap_or(Px(56.0)),
            reset_slot_width: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_RESET_SLOT_WIDTH)
                .unwrap_or(density.hit_thickness),
            auto_stack_below: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_AUTO_STACK_BELOW)
                .unwrap_or(Px(520.0)),
            group_header_height: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_GROUP_HEADER_HEIGHT)
                .unwrap_or(density.row_height),
            group_content_gap: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_GROUP_CONTENT_GAP)
                .unwrap_or(Px(6.0)),
            panel_gap: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_PANEL_GAP)
                .unwrap_or(Px(10.0)),
            panel_header_gap: theme
                .metric_by_key(EditorTokenKeys::PROPERTY_PANEL_HEADER_GAP)
                .unwrap_or(Px(8.0)),
        }
    }
}
