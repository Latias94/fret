use std::sync::Arc;

use fret_core::{Px, TextStyle};
use fret_ui::action::{ActionCx, OnActivate, UiActionHost};
use fret_ui::element::{FlexItemStyle, LayoutStyle, Length, SizeStyle};
use fret_ui_kit::Size;
use fret_ui_kit::typography;

use crate::controls::numeric_input::NumericInputSelectionBehavior;
use crate::primitives::{EditSessionOutcome, NumericValueConstraints};

pub(super) fn axis_drag_value_input_text_style(base: TextStyle, row_height: Px) -> TextStyle {
    typography::as_control_text(TextStyle {
        line_height: Some(row_height),
        ..base
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AxisDragValueMode {
    Scrub,
    Typing,
}

#[derive(Clone)]
pub struct AxisDragValueResetAction {
    pub icon: fret_icons::IconId,
    pub a11y_label: Arc<str>,
    pub test_id: Option<Arc<str>>,
    pub on_activate: OnActivate,
}

#[derive(Debug)]
pub(super) struct AxisDragValueState {
    pub(super) mode: AxisDragValueMode,
    pub(super) scrub_id: Option<fret_ui::GlobalElementId>,
    pub(super) scrub_revision: u64,
    pub(super) seen_input_focus: bool,
}

impl Default for AxisDragValueState {
    fn default() -> Self {
        Self {
            mode: AxisDragValueMode::Scrub,
            scrub_id: None,
            scrub_revision: 0,
            seen_input_focus: false,
        }
    }
}

#[derive(Clone)]
pub struct AxisDragValueOptions {
    pub layout: LayoutStyle,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    /// Shared numeric edit constraints applied to scrub and typed commit paths.
    pub constraints: NumericValueConstraints,
    /// Explicit identity source for internal state (scrub/typing focus restore, draft string).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub reset: Option<AxisDragValueResetAction>,
    pub enabled: bool,
    pub focusable: bool,
    pub size: Size,
    pub selection_behavior: NumericInputSelectionBehavior,
}

impl Default for AxisDragValueOptions {
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
            id_source: None,
            test_id: None,
            reset: None,
            enabled: true,
            focusable: true,
            size: Size::Small,
            selection_behavior: NumericInputSelectionBehavior::ReplaceAllOnFocus,
        }
    }
}

pub type AxisDragValueOutcome = EditSessionOutcome;
pub type OnAxisDragValueOutcome =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, AxisDragValueOutcome) + 'static>;

#[cfg(test)]
mod tests {
    use super::axis_drag_value_input_text_style;
    use fret_core::{Px, TextStyle};

    #[test]
    fn axis_drag_value_input_text_style_uses_density_row_height_for_typing_line_box() {
        let style = axis_drag_value_input_text_style(
            TextStyle {
                size: Px(12.0),
                line_height: Some(Px(16.0)),
                ..Default::default()
            },
            Px(24.0),
        );

        assert_eq!(style.line_height, Some(Px(24.0)));
    }
}
