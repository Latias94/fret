use fret_core::Px;
use fret_icons::IconId;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::EditorDensity;

#[track_caller]
pub(crate) fn editor_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    icon: IconId,
    size: Option<Px>,
) -> AnyElement {
    let size = size.unwrap_or(density.icon_size);
    fret_ui_kit::declarative::icon::icon_with(cx, icon, Some(size), None)
}
