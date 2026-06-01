use std::sync::Arc;

use fret_core::Point;
use fret_ui::UiHost;

use super::{FloatingWindowResponse, ImUiFacade, UiWriterImUiFacadeExt, WindowOptions};

mod closed;
mod entry;
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
    entry::floating_window_show_with_options(ui, id, title, initial_position, options, f)
}
