//! NumericInput prefix/suffix affix segment owner.

use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::input_group::editor_text_segment;

pub(super) fn numeric_input_affix_segment<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    frame_chrome: ResolvedEditorFrameChrome,
    text: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
) -> Option<AnyElement> {
    let text = text?;
    let affix_color = {
        let theme = Theme::global(&*cx.app);
        editor_muted_foreground(theme)
    };
    let mut segment = editor_text_segment(
        cx,
        density,
        frame_chrome.text_px,
        text.clone(),
        affix_color,
        frame_chrome.padding,
    );
    if let Some(test_id) = test_id.as_ref() {
        segment = segment.test_id(test_id.clone()).a11y_label(text);
    }
    Some(segment)
}
