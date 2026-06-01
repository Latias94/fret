use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::Point;
use fret_ui::UiHost;

use super::{
    FloatingAreaOptions, FloatingWindowResponse, ImUiFacade, UiWriterImUiFacadeExt, WindowOptions,
    floating_window_on_area,
};

mod closed;
mod state;

pub(super) fn floating_window_show<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    initial_position: Point,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window_show_with_options(ui, id, title, initial_position, WindowOptions::default(), f)
}

pub(super) fn floating_window_show_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    initial_position: Point,
    options: WindowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    let title = title.into();
    let open = options.open.clone();
    let initial_size = options.size;
    let resize = options.resize;
    let behavior = options.behavior;

    if !state::floating_window_is_open(ui, open.as_ref()) {
        return closed::closed_floating_window_response(initial_position, initial_size);
    }

    let chrome = Rc::new(Cell::new(super::FloatingWindowChromeResponse::default()));
    let chrome_out = chrome.clone();

    let area = ui.floating_area_with_options(
        id,
        initial_position,
        FloatingAreaOptions {
            test_id_prefix: "imui.float_window.window:",
            test_id: None,
            hit_test_passthrough: behavior.pointer_passthrough,
            no_inputs: behavior.no_inputs,
        },
        move |ui, area| {
            let chrome = floating_window_on_area::render_floating_window_in_area(
                ui,
                area,
                id,
                title,
                open.clone(),
                initial_position,
                initial_size,
                resize,
                behavior,
                f,
            );
            chrome_out.set(chrome);
        },
    );

    let chrome = chrome.get();
    state::floating_window_response(area, chrome)
}
