use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::Point;
use fret_ui::{GlobalElementId, UiHost};

use super::{
    FloatingAreaOptions, FloatingAreaResponse, FloatingWindowResponse, ImUiFacade,
    UiWriterImUiFacadeExt, WindowOptions, floating_window_on_area,
};

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

    if let Some(open) = open.as_ref() {
        let is_open = ui
            .with_cx_mut(|cx| cx.read_model(open, fret_ui::Invalidation::Paint, |_app, v| *v))
            .unwrap_or(false);
        if !is_open {
            return FloatingWindowResponse {
                area: FloatingAreaResponse {
                    id: GlobalElementId(0),
                    rect: None,
                    position: initial_position,
                    dragging: false,
                    drag_kind: super::float_window_drag_kind_for_element(GlobalElementId(0)),
                },
                size: initial_size,
                resizing: false,
                collapsed: false,
            };
        }
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
    FloatingWindowResponse {
        area,
        size: chrome.size,
        resizing: chrome.resizing,
        collapsed: chrome.collapsed,
    }
}
