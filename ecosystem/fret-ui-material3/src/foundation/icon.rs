use fret_icons::IconId;
use fret_ui::SvgSource;
use fret_ui::{ElementContext, UiHost};

use fret_ui_kit::declarative::icon as icon_runtime;

pub(crate) fn svg_source_for_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: &IconId,
) -> SvgSource {
    icon_runtime::resolve_svg_source_from_globals(cx.app, icon, "fret_ui_material3.foundation")
}
