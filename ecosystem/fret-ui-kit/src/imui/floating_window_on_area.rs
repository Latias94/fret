use std::sync::Arc;

use fret_authoring::UiWriter as _;
use fret_core::{Point, Size};
use fret_ui::UiHost;

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
        let window_id = area.id;
        let prepared = state::prepare_floating_window_in_area_state(
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

        let title_bar_row = super::floating_window_title_bar::floating_window_title_bar_row(
            cx,
            area,
            title_for_window,
            open_for_window,
            prepared.resize_state.title_bar_test_id.clone(),
            prepared.resize_state.close_button_test_id.clone(),
            prepared.resizable_layout,
            options,
        );

        let content = super::floating_window_content::floating_window_content_element(
            cx,
            window_id,
            prepared.resizable_layout,
            options,
            build,
        );

        let window = super::floating_window_shell::floating_window_shell_element(
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
    });

    ui.add(window);
    chrome
}
