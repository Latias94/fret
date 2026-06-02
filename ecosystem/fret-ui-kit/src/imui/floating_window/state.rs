use fret_runtime::Model;
use fret_ui::{Invalidation, UiHost};

use super::super::{
    FloatingAreaResponse, FloatingWindowChromeResponse, FloatingWindowResponse,
    UiWriterImUiFacadeExt,
};

pub(super) fn floating_window_is_open<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    open: Option<&Model<bool>>,
) -> bool {
    let Some(open) = open else {
        return true;
    };

    ui.with_cx_mut(|cx| {
        cx.read_model(open, Invalidation::Paint, |_app, value| *value)
            .unwrap_or(false)
    })
}

pub(super) fn floating_window_response(
    area: FloatingAreaResponse,
    chrome: FloatingWindowChromeResponse,
) -> FloatingWindowResponse {
    FloatingWindowResponse {
        area,
        size: chrome.size,
        resizing: chrome.resizing,
        collapsed: chrome.collapsed,
    }
}
