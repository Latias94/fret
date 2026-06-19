use std::sync::Arc;

use fret_core::TextStyle;
use fret_ui::action::{ActionCx, UiFocusActionHost};
use fret_ui::element::{LayoutStyle, Length, SizeStyle};
use fret_ui_kit::Size;
use fret_ui_kit::typography;

use crate::primitives::{EditorDensity, NumericInputSelectionBehavior};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct NumericInputOptions {
    pub layout: LayoutStyle,
    pub size: Size,
    pub placeholder: Option<Arc<str>>,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    /// Explicit identity source for internal state (draft/error models).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple numeric inputs from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub enabled: bool,
    pub focusable: bool,
    pub error_display: NumericInputErrorDisplay,
    pub selection_behavior: NumericInputSelectionBehavior,
}

impl Default for NumericInputOptions {
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
            size: Size::Small,
            placeholder: None,
            prefix: None,
            suffix: None,
            id_source: None,
            test_id: None,
            enabled: true,
            focusable: true,
            error_display: NumericInputErrorDisplay::TrailingIcon,
            selection_behavior: NumericInputSelectionBehavior::ReplaceAllOnFocus,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericInputErrorDisplay {
    None,
    InlineText,
    TrailingIcon,
    InlineTextAndIcon,
}

pub type NumericFormatFn<T> = Arc<dyn Fn(T) -> Arc<str> + Send + Sync + 'static>;
pub type NumericParseFn<T> = Arc<dyn Fn(&str) -> Option<T> + Send + Sync + 'static>;
pub type NumericValidateFn<T> = Arc<dyn Fn(T) -> Option<Arc<str>> + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericInputOutcome {
    Committed,
    Canceled,
}

pub type OnNumericInputOutcome =
    Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, NumericInputOutcome) + 'static>;

pub(super) fn editor_numeric_input_text_style(
    base: TextStyle,
    density: EditorDensity,
) -> TextStyle {
    typography::as_control_text(TextStyle {
        line_height: Some(density.row_height),
        ..base
    })
}
