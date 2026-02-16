//! Editor style façade resolved from `Theme` + `editor.*` tokens.
//!
//! Goals:
//! - Provide a single place to resolve editor density + responsive thresholds.
//! - Centralize chrome token selection so editor controls don't drift.
//! - Stay policy-only (ecosystem layer) and avoid adding runtime contracts.

use fret_core::Px;
use fret_ui::Theme;
use fret_ui_kit::recipes::input::InputTokenKeys;

use super::chrome::{ResolvedEditorFrameChrome, resolve_editor_frame_chrome};
use super::{EditorDensity, EditorTokenKeys};
use fret_ui_kit::{ChromeRefinement, Size};

#[derive(Debug, Clone, Copy)]
pub(crate) struct EditorStyle<'a> {
    theme: &'a Theme,
    pub(crate) density: EditorDensity,
    pub(crate) vec_auto_stack_below: Px,
    pub(crate) property_auto_stack_below: Px,
}

impl<'a> EditorStyle<'a> {
    pub(crate) fn resolve(theme: &'a Theme) -> Self {
        let density = EditorDensity::resolve(theme);

        let vec_auto_stack_below = theme
            .metric_by_key(EditorTokenKeys::VEC_AUTO_STACK_BELOW)
            .unwrap_or(Px(420.0));
        let property_auto_stack_below = theme
            .metric_by_key(EditorTokenKeys::PROPERTY_AUTO_STACK_BELOW)
            .unwrap_or(Px(520.0));

        Self {
            theme,
            density,
            vec_auto_stack_below,
            property_auto_stack_below,
        }
    }

    /// Standard input chrome tokens for editor controls (TextField/NumericInput/DragValue).
    pub(crate) fn text_field_input_tokens() -> InputTokenKeys {
        InputTokenKeys {
            padding_x: Some("component.text_field.padding_x"),
            padding_y: Some("component.text_field.padding_y"),
            min_height: Some("component.text_field.min_height"),
            radius: Some("component.text_field.radius"),
            border_width: Some("component.text_field.border_width"),
            bg: Some("component.text_field.bg"),
            border: Some("component.text_field.border"),
            border_focus: Some("component.text_field.border_focus"),
            fg: Some("component.text_field.fg"),
            text_px: Some("component.text_field.text_px"),
            selection: Some("component.text_field.selection"),
        }
    }

    pub(crate) fn frame_chrome_small(&self) -> ResolvedEditorFrameChrome {
        resolve_editor_frame_chrome(
            self.theme,
            Size::Small,
            &ChromeRefinement::default(),
            Self::text_field_input_tokens(),
        )
    }
}
