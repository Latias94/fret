use super::*;

pub(super) fn take_matching_pending_paste(
    pending_paste: &mut Option<PendingPaste>,
    token: fret_core::ClipboardToken,
) -> Option<PendingPaste> {
    let pending = pending_paste.take()?;
    if pending.token == token {
        Some(pending)
    } else {
        *pending_paste = Some(pending);
        None
    }
}

pub(super) fn clear_pending_if_matches(
    pending_paste: &mut Option<PendingPaste>,
    token: fret_core::ClipboardToken,
) -> bool {
    pending_paste
        .as_ref()
        .is_some_and(|pending| pending.token == token)
        .then(|| pending_paste.take())
        .flatten()
        .is_some()
}
