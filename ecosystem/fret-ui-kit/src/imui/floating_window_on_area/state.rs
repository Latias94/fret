use fret_core::window::WindowMetricsService;
use fret_core::{Point, Size};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod collapsed;
mod position;

use super::super::floating_window_resize::FloatingWindowResizeStateOutput;
use super::super::{
    FloatingWindowChromeResponse, FloatingWindowOptions, FloatingWindowResizeOptions,
};

pub(super) struct PreparedFloatingWindowInAreaState {
    pub(super) resizable_layout: bool,
    pub(super) resize_enabled: bool,
    pub(super) collapsed: bool,
    pub(super) resize_state: FloatingWindowResizeStateOutput,
    pub(super) chrome: FloatingWindowChromeResponse,
}

pub(super) fn prepare_floating_window_in_area_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    id: &str,
    area_position: Point,
    initial_position: Point,
    initial_size: Option<Size>,
    resize: Option<FloatingWindowResizeOptions>,
    options: FloatingWindowOptions,
) -> PreparedFloatingWindowInAreaState {
    let resizable_layout = initial_size.is_some();
    let resize_enabled = options.inputs_enabled && options.resizable && resizable_layout;

    let resize_snapshot = super::super::floating_window_resize::current_resize_snapshot(
        cx,
        window_id,
        resize_enabled,
    );
    let collapsed = collapsed::read_floating_window_collapsed(cx, window_id, options);

    let scale_factor = cx
        .app
        .global::<WindowMetricsService>()
        .and_then(|svc| svc.scale_factor(cx.window))
        .unwrap_or(1.0);

    let resize_state = super::super::floating_window_resize::prepare_resize_state(
        cx,
        window_id,
        id,
        area_position,
        initial_size,
        resize,
        resize_snapshot,
        collapsed,
        scale_factor,
    );

    position::sync_floating_area_position_after_resize(
        cx,
        window_id,
        id,
        initial_position,
        area_position,
        resize_state.position_after_resize,
    );

    let chrome = FloatingWindowChromeResponse {
        size: resizable_layout.then_some(resize_state.size),
        resizing: resize_state.resizing,
        collapsed,
    };

    PreparedFloatingWindowInAreaState {
        resizable_layout,
        resize_enabled,
        collapsed,
        resize_state,
        chrome,
    }
}
