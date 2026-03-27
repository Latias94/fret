use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Point, Size};
use fret_ui::{GlobalElementId, UiHost};

use super::{
    FloatingAreaOptions, FloatingAreaResponse, FloatingWindowOptions, FloatingWindowResizeOptions,
    FloatingWindowResponse, ImUiFacade, UiWriterImUiFacadeExt, floating_window_on_area,
};

pub(super) fn floating_window_show<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    initial_position: Point,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window_impl_show(ui, id, title.into(), None, initial_position, None, None, f)
}

pub(super) fn floating_window_open_show<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    open: &fret_runtime::Model<bool>,
    initial_position: Point,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window_impl_show(
        ui,
        id,
        title.into(),
        Some(open),
        initial_position,
        None,
        None,
        f,
    )
}

pub(super) fn floating_window_resizable_show<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    initial_position: Point,
    initial_size: Size,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window_resizable_with_options_show(
        ui,
        id,
        title,
        initial_position,
        initial_size,
        FloatingWindowResizeOptions::default(),
        FloatingWindowOptions::default(),
        f,
    )
}

pub(super) fn floating_window_resizable_with_options_show<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    initial_position: Point,
    initial_size: Size,
    resize: FloatingWindowResizeOptions,
    options: FloatingWindowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window_impl_show_with_options(
        ui,
        id,
        title.into(),
        None,
        initial_position,
        Some(initial_size),
        Some(resize),
        options,
        f,
    )
}

pub(super) fn floating_window_open_resizable_show<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    open: &fret_runtime::Model<bool>,
    initial_position: Point,
    initial_size: Size,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window_open_resizable_with_options_show(
        ui,
        id,
        title,
        open,
        initial_position,
        initial_size,
        FloatingWindowResizeOptions::default(),
        FloatingWindowOptions::default(),
        f,
    )
}

pub(super) fn floating_window_open_resizable_with_options_show<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    open: &fret_runtime::Model<bool>,
    initial_position: Point,
    initial_size: Size,
    resize: FloatingWindowResizeOptions,
    options: FloatingWindowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window_impl_show_with_options(
        ui,
        id,
        title.into(),
        Some(open),
        initial_position,
        Some(initial_size),
        Some(resize),
        options,
        f,
    )
}

pub(super) fn floating_window_impl_show<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    title: Arc<str>,
    open: Option<&fret_runtime::Model<bool>>,
    initial_position: Point,
    initial_size: Option<Size>,
    resize: Option<FloatingWindowResizeOptions>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window_impl_on_area_show_with_options(
        ui,
        id,
        title,
        open,
        initial_position,
        initial_size,
        resize,
        FloatingWindowOptions::default(),
        f,
    )
}

pub(super) fn floating_window_impl_show_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    title: Arc<str>,
    open: Option<&fret_runtime::Model<bool>>,
    initial_position: Point,
    initial_size: Option<Size>,
    resize: Option<FloatingWindowResizeOptions>,
    options: FloatingWindowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window_impl_on_area_show_with_options(
        ui,
        id,
        title,
        open,
        initial_position,
        initial_size,
        resize,
        options,
        f,
    )
}

pub(super) fn floating_window_impl_on_area_show_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    title: Arc<str>,
    open: Option<&fret_runtime::Model<bool>>,
    initial_position: Point,
    initial_size: Option<Size>,
    resize: Option<FloatingWindowResizeOptions>,
    options: FloatingWindowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    if let Some(open) = open {
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

    let open_model = open.cloned();

    let chrome = Rc::new(Cell::new(super::FloatingWindowChromeResponse::default()));
    let chrome_out = chrome.clone();

    let area = ui.floating_area_with_options(
        id,
        initial_position,
        FloatingAreaOptions {
            test_id_prefix: "imui.float_window.window:",
            test_id: None,
            hit_test_passthrough: options.pointer_passthrough,
            no_inputs: options.no_inputs,
        },
        move |ui, area| {
            let chrome = floating_window_on_area::render_floating_window_in_area(
                ui,
                area,
                id,
                title,
                open_model.clone(),
                initial_position,
                initial_size,
                resize,
                options,
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
