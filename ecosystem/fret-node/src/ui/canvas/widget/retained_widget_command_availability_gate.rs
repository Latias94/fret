use super::*;

pub(super) fn should_handle_command<H: UiHost>(cx: &CommandAvailabilityCx<'_, H>) -> bool {
    cx.focus == Some(cx.node)
}

pub(super) fn can_write_clipboard<H: UiHost>(cx: &CommandAvailabilityCx<'_, H>) -> bool {
    cx.input_ctx.caps.clipboard.text.write
}

pub(super) fn can_paste<H: UiHost>(cx: &CommandAvailabilityCx<'_, H>) -> bool {
    cx.input_ctx.caps.clipboard.text.read && cx.window.is_some()
}
