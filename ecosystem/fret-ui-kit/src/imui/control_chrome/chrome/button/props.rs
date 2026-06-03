use fret_core::{Corners, Edges, Px};
use fret_ui::element::ContainerProps;

use super::super::super::CONTROL_RADIUS;
use super::super::palette::ImUiControlPalette;

pub(super) fn button_container_props(palette: ImUiControlPalette) -> ContainerProps {
    let mut chrome = ContainerProps::default();
    chrome.padding = Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(4.0),
        bottom: Px(4.0),
    }
    .into();
    chrome.background = Some(palette.background);
    chrome.border = Edges::all(Px(1.0));
    chrome.border_color = Some(palette.border);
    chrome.corner_radii = Corners::all(CONTROL_RADIUS);
    chrome
}
