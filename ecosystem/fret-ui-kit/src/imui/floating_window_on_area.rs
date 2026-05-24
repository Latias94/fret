use std::sync::Arc;

use fret_authoring::UiWriter as _;
use fret_core::{Point, Size};
use fret_ui::UiHost;

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
        let resizable_layout = initial_size.is_some();
        let resize_enabled = options.inputs_enabled && options.resizable && resizable_layout;

        let resize_snapshot =
            super::floating_window_resize::current_resize_snapshot(cx, window_id, resize_enabled);
        let collapsed_model = super::float_window_collapsed_model_for(cx, window_id);
        if options.inputs_enabled
            && options.collapsible
            && cx.take_transient_for(window_id, super::KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED)
        {
            let _ = cx.app.models_mut().update(&collapsed_model, |v| {
                *v = !*v;
            });
        }
        let collapsed = cx
            .read_model(&collapsed_model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);

        let scale_factor = cx
            .app
            .global::<fret_core::window::WindowMetricsService>()
            .and_then(|svc| svc.scale_factor(cx.window))
            .unwrap_or(1.0);

        let resize_state = super::floating_window_resize::prepare_resize_state(
            cx,
            window_id,
            id,
            area.position,
            initial_size,
            resize,
            resize_snapshot,
            collapsed,
            scale_factor,
        );

        if resize_state.position_after_resize != area.position {
            cx.state_for(
                window_id,
                || super::FloatingAreaState {
                    position: initial_position,
                    last_drag_position: None,
                    test_id: Arc::from(format!("imui.float_window.window:{id}")),
                },
                |st| {
                    st.position = resize_state.position_after_resize;
                },
            );
        }

        let chrome = super::FloatingWindowChromeResponse {
            size: resizable_layout.then_some(resize_state.size),
            resizing: resize_state.resizing,
            collapsed,
        };
        let title_for_window = title.clone();
        let open_for_window = open_model.clone();

        let title_bar_row = super::floating_window_title_bar::floating_window_title_bar_row(
            cx,
            area,
            title_for_window,
            open_for_window,
            resize_state.title_bar_test_id.clone(),
            resize_state.close_button_test_id.clone(),
            resizable_layout,
            options,
        );

        let content = super::floating_window_content::floating_window_content_element(
            cx,
            window_id,
            resizable_layout,
            options,
            build,
        );

        let window = super::floating_window_shell::floating_window_shell_element(
            cx,
            window_id,
            title_bar_row,
            content,
            resize_state.size,
            resizable_layout,
            collapsed,
            resize_enabled,
            options,
            resize_state.handle_test_ids.clone(),
        );
        (window, chrome)
    });

    ui.add(window);
    chrome
}
