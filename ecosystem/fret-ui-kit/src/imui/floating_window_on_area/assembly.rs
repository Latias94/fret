use std::sync::Arc;

use fret_core::{Point, Size};
use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::element::AnyElement;

use super::super::{
    FloatingAreaContext, FloatingWindowChromeResponse, FloatingWindowOptions,
    FloatingWindowResizeOptions, ImUiFacade,
};

pub(in crate::imui::floating_window_on_area) fn floating_window_in_area_element<H: UiHost, Build>(
    cx: &mut ElementContext<'_, H>,
    area: FloatingAreaContext,
    id: &str,
    title: Arc<str>,
    open_model: Option<fret_runtime::Model<bool>>,
    initial_position: Point,
    initial_size: Option<Size>,
    resize: Option<FloatingWindowResizeOptions>,
    options: FloatingWindowOptions,
    build: Build,
) -> (AnyElement, FloatingWindowChromeResponse)
where
    Build: for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
{
    let window_id = area.id;
    let prepared = super::state::prepare_floating_window_in_area_state(
        cx,
        window_id,
        id,
        area.position,
        initial_position,
        initial_size,
        resize,
        options,
    );
    let title_for_window = title.clone();
    let open_for_window = open_model.clone();

    let title_bar_row = super::super::floating_window_title_bar::floating_window_title_bar_row(
        cx,
        area,
        title_for_window,
        open_for_window,
        prepared.resize_state.title_bar_test_id.clone(),
        prepared.resize_state.close_button_test_id.clone(),
        prepared.resizable_layout,
        options,
    );

    let content = super::super::floating_window_content::floating_window_content_element(
        cx,
        window_id,
        prepared.resizable_layout,
        options,
        build,
    );

    let window = super::super::floating_window_shell::floating_window_shell_element(
        cx,
        window_id,
        title_bar_row,
        content,
        prepared.resize_state.size,
        prepared.resizable_layout,
        prepared.collapsed,
        prepared.resize_enabled,
        options,
        prepared.resize_state.handle_test_ids.clone(),
    );
    (window, prepared.chrome)
}
