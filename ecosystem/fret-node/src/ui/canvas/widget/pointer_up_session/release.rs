use fret_ui::UiHost;

use super::super::low_level_adapter::CanvasPointerCaptureReleaseCx;

pub(in super::super) fn take_active_release<T, U>(
    slot: &mut Option<T>,
    pending_slot: &mut Option<U>,
) -> Option<T> {
    let value = slot.take()?;
    *pending_slot = None;
    Some(value)
}

pub(in super::super) fn finish_pending_release<H: UiHost, T>(
    slot: &mut Option<T>,
    cx: &mut impl CanvasPointerCaptureReleaseCx<H>,
) -> bool {
    if slot.take().is_none() {
        return false;
    }

    super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}

#[cfg(test)]
mod tests;
