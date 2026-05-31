use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

use super::layout::PropertyRowLayoutVariant;

#[derive(Debug, Clone)]
pub struct PropertyRowOptions {
    pub layout: LayoutStyle,
    pub label_width: Option<Px>,
    pub gap: Option<Px>,
    pub trailing_gap: Option<Px>,
    pub value_max_width: Option<Px>,
    pub status_slot_width: Option<Px>,
    pub reset_slot_width: Option<Px>,
    pub variant: PropertyRowLayoutVariant,
    pub auto_stack_below: Option<Px>,
    /// Explicit identity source for internal policy state (auto layout heuristics).
    ///
    /// This is the editor-composite equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when building rows in a loop where the callsite is not unique per row.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for PropertyRowOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            label_width: None,
            gap: None,
            trailing_gap: None,
            value_max_width: None,
            status_slot_width: None,
            reset_slot_width: None,
            variant: PropertyRowLayoutVariant::Row,
            auto_stack_below: None,
            id_source: None,
            test_id: None,
        }
    }
}
