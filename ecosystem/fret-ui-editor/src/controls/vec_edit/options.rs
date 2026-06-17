use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

#[derive(Debug, Clone)]
pub struct VecEditOptions {
    pub layout: LayoutStyle,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    /// Explicit identity source for internal element keys.
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    pub id_source: Option<Arc<str>>,
    pub variant: VecEditLayoutVariant,
    pub gap: Px,
    pub axis_gap: Px,
    pub auto_stack_below: Option<Px>,
    pub test_id: Option<Arc<str>>,
}

impl Default for VecEditOptions {
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
            prefix: None,
            suffix: None,
            id_source: None,
            variant: VecEditLayoutVariant::Row,
            gap: Px(6.0),
            axis_gap: Px(4.0),
            auto_stack_below: None,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VecEditLayoutVariant {
    Row,
    Column,
    /// Choose `Row` vs `Column` based on last frame bounds.
    ///
    /// This is a policy-only heuristic intended to avoid "tiny inputs" when a property grid is
    /// narrow (common in editor sidebars).
    #[default]
    Auto,
}
