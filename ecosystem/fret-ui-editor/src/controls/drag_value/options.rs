use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::{FlexItemStyle, LayoutStyle, Length, SizeStyle};

use crate::controls::numeric_input::NumericInputSelectionBehavior;
use crate::primitives::NumericValueConstraints;

#[derive(Debug, Clone)]
pub struct DragValueOptions {
    pub layout: LayoutStyle,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    /// Shared numeric edit constraints applied to scrub and typed commit paths.
    pub constraints: NumericValueConstraints,
    pub selection_behavior: NumericInputSelectionBehavior,
    /// Explicit identity source for internal state (scrub/typing focus restore).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple drag values from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for DragValueOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            prefix: None,
            suffix: None,
            constraints: NumericValueConstraints::default(),
            selection_behavior: NumericInputSelectionBehavior::ReplaceAllOnFocus,
            id_source: None,
            test_id: None,
        }
    }
}
