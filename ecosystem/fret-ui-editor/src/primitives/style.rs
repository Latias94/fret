//! Editor style façade resolved from `Theme` + `editor.*` tokens.
//!
//! Goals:
//! - Provide a single place to resolve editor density + responsive thresholds.
//! - Centralize chrome token selection so editor controls don't drift.
//! - Stay policy-only (ecosystem layer) and avoid adding runtime contracts.

use fret_core::Px;
use fret_ui::Theme;

use super::chrome::{ResolvedEditorFrameChrome, resolve_editor_text_field_frame_chrome};
use super::{EditorDensity, EditorTokenKeys};
use fret_ui_kit::{ChromeRefinement, Size};

#[derive(Debug, Clone, Copy)]
pub(crate) struct EditorStyle<'a> {
    theme: &'a Theme,
    pub(crate) density: EditorDensity,
    pub(crate) vec_auto_stack_below: Px,
    pub(crate) vec_axis_min_width: Px,
}

impl<'a> EditorStyle<'a> {
    pub(crate) fn resolve(theme: &'a Theme) -> Self {
        let density = EditorDensity::resolve(theme);

        let vec_auto_stack_below = theme
            .metric_by_key(EditorTokenKeys::VEC_AUTO_STACK_BELOW)
            .unwrap_or(Px(420.0));
        let vec_axis_min_width = theme
            .metric_by_key(EditorTokenKeys::VEC_AXIS_MIN_WIDTH)
            .unwrap_or(Px(140.0));
        Self {
            theme,
            density,
            vec_auto_stack_below,
            vec_axis_min_width,
        }
    }

    pub(crate) fn frame_chrome_small(&self) -> ResolvedEditorFrameChrome {
        self.frame_chrome(Size::Small)
    }

    pub(crate) fn frame_chrome(&self, size: Size) -> ResolvedEditorFrameChrome {
        resolve_editor_text_field_frame_chrome(self.theme, size, &ChromeRefinement::default())
    }
}
