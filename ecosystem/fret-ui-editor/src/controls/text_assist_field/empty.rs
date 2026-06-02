//! Text-assist inline empty-label rendering owner.

use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::readout::editor_popup_empty_text_props;
use crate::primitives::style::EditorStyle;

pub(super) fn render_text_assist_inline_empty_label<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    show: bool,
    label: Arc<str>,
    test_id: Option<Arc<str>>,
) -> Option<AnyElement> {
    if !show {
        return None;
    }

    let theme = Theme::global(&*cx.app);
    let empty = cx.text_props(editor_popup_empty_text_props(
        label,
        editor_muted_foreground(theme),
        EditorStyle::resolve(theme).density.row_height,
    ));
    Some(if let Some(test_id) = test_id {
        empty.test_id(test_id)
    } else {
        empty
    })
}
