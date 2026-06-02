use std::sync::Arc;

use fret_authoring::UiWriter as _;
use fret_core::{Point, Size};
use fret_ui::UiHost;

mod assembly;
mod state;

pub(super) fn render_floating_window_in_area<H: UiHost, Build>(
    ui: &mut super::ImUiFacade<'_, '_, H>,
    area: super::FloatingAreaContext,
    id: &str,
    title: Arc<str>,
    open_model: Option<fret_runtime::Model<bool>>,
    initial_position: Point,
    initial_size: Option<Size>,
    resize: Option<super::FloatingWindowResizeOptions>,
    options: super::FloatingWindowOptions,
    build: Build,
) -> super::FloatingWindowChromeResponse
where
    Build: for<'cx2, 'a2> FnOnce(&mut super::ImUiFacade<'cx2, 'a2, H>),
{
    let (window, chrome) = ui.with_cx_mut(|cx| {
        assembly::floating_window_in_area_element(
            cx,
            area,
            id,
            title,
            open_model,
            initial_position,
            initial_size,
            resize,
            options,
            build,
        )
    });

    ui.add(window);
    chrome
}
