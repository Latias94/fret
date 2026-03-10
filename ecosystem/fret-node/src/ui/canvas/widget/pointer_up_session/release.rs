use fret_ui::UiHost;

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
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if slot.take().is_none() {
        return false;
    }

    super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_active_release_clears_pending_companion() {
        let mut active = Some(1_u32);
        let mut pending = Some(2_u32);

        let taken = take_active_release(&mut active, &mut pending);

        assert_eq!(taken, Some(1));
        assert_eq!(active, None);
        assert_eq!(pending, None);
    }
}
