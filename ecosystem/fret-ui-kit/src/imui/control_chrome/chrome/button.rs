use fret_ui::element::{ContainerProps, PressableState};
use fret_ui::{ElementContext, UiHost};

use super::palette::ImUiControlPalette;

mod palette;
mod props;

pub(in crate::imui) fn button_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    state: PressableState,
) -> (ImUiControlPalette, ContainerProps) {
    let theme = fret_ui::Theme::global(&*cx.app);
    let palette = palette::resolve_button_palette(&theme, enabled, state);
    let chrome = props::button_container_props(palette);

    (palette, chrome)
}
