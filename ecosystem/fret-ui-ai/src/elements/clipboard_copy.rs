use std::sync::{Arc, Mutex};

use fret_core::{ClipboardAccessError, ClipboardToken, ClipboardWriteOutcome, TimerToken};

#[derive(Debug, Default)]
struct ClipboardCopyFeedback {
    copied: bool,
    reset_token: Option<TimerToken>,
    pending_clipboard_token: Option<ClipboardToken>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ClipboardCopyFeedbackRef(Arc<Mutex<ClipboardCopyFeedback>>);

impl ClipboardCopyFeedbackRef {
    pub(crate) fn is_copied(&self) -> bool {
        self.lock().copied
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ClipboardCopyFeedback> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClipboardCopyRequest {
    pub(crate) prev_reset: Option<TimerToken>,
    pub(crate) clipboard_token: ClipboardToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClipboardCopyCompletion {
    pub(crate) prev_reset: Option<TimerToken>,
    pub(crate) next_reset: Option<TimerToken>,
    pub(crate) error: Option<ClipboardAccessError>,
}

pub(crate) fn begin_request<F>(
    feedback: &ClipboardCopyFeedbackRef,
    next_clipboard_token: F,
) -> Option<ClipboardCopyRequest>
where
    F: FnOnce() -> ClipboardToken,
{
    let mut feedback = feedback.lock();
    if feedback.copied || feedback.pending_clipboard_token.is_some() {
        return None;
    }

    let prev_reset = feedback.reset_token.take();
    let clipboard_token = next_clipboard_token();
    feedback.pending_clipboard_token = Some(clipboard_token);
    Some(ClipboardCopyRequest {
        prev_reset,
        clipboard_token,
    })
}

pub(crate) fn finish_request<F>(
    feedback: &ClipboardCopyFeedbackRef,
    token: ClipboardToken,
    outcome: &ClipboardWriteOutcome,
    next_reset_token: F,
) -> Option<ClipboardCopyCompletion>
where
    F: FnOnce() -> TimerToken,
{
    let mut feedback = feedback.lock();
    if feedback.pending_clipboard_token != Some(token) {
        return None;
    }

    feedback.pending_clipboard_token = None;
    match outcome {
        ClipboardWriteOutcome::Succeeded => {
            let prev_reset = feedback.reset_token.take();
            let reset_token = next_reset_token();
            feedback.copied = true;
            feedback.reset_token = Some(reset_token);
            Some(ClipboardCopyCompletion {
                prev_reset,
                next_reset: Some(reset_token),
                error: None,
            })
        }
        ClipboardWriteOutcome::Failed { error } => {
            let prev_reset = feedback.reset_token.take();
            feedback.copied = false;
            Some(ClipboardCopyCompletion {
                prev_reset,
                next_reset: None,
                error: Some(error.clone()),
            })
        }
    }
}

pub(crate) fn handle_reset_timer(feedback: &ClipboardCopyFeedbackRef, token: TimerToken) -> bool {
    let mut feedback = feedback.lock();
    if feedback.reset_token != Some(token) {
        return false;
    }

    feedback.reset_token = None;
    feedback.copied = false;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::ClipboardAccessErrorKind;

    #[test]
    fn begin_request_sets_pending_token_and_blocks_reentry() {
        let feedback = ClipboardCopyFeedbackRef::default();
        {
            let mut state = feedback.lock();
            state.reset_token = Some(TimerToken(7));
        }

        let request =
            begin_request(&feedback, || ClipboardToken(11)).expect("expected clipboard request");
        assert_eq!(request.prev_reset, Some(TimerToken(7)));
        assert_eq!(request.clipboard_token, ClipboardToken(11));
        assert!(begin_request(&feedback, || ClipboardToken(12)).is_none());
    }

    #[test]
    fn finish_request_success_sets_copied_and_next_reset() {
        let feedback = ClipboardCopyFeedbackRef::default();
        let _request = begin_request(&feedback, || ClipboardToken(5)).expect("request");

        let completion = finish_request(
            &feedback,
            ClipboardToken(5),
            &ClipboardWriteOutcome::Succeeded,
            || TimerToken(9),
        )
        .expect("completion");

        assert_eq!(completion.prev_reset, None);
        assert_eq!(completion.next_reset, Some(TimerToken(9)));
        assert_eq!(completion.error, None);
        assert!(feedback.is_copied());
    }

    #[test]
    fn finish_request_failure_clears_copied_and_returns_error() {
        let feedback = ClipboardCopyFeedbackRef::default();
        {
            let mut state = feedback.lock();
            state.pending_clipboard_token = Some(ClipboardToken(5));
            state.reset_token = Some(TimerToken(3));
            state.copied = true;
        }
        let error = ClipboardAccessError {
            kind: ClipboardAccessErrorKind::PermissionDenied,
            message: Some("clipboard denied".to_string()),
        };

        let completion = finish_request(
            &feedback,
            ClipboardToken(5),
            &ClipboardWriteOutcome::Failed {
                error: error.clone(),
            },
            || TimerToken(9),
        )
        .expect("completion");

        assert_eq!(completion.prev_reset, Some(TimerToken(3)));
        assert_eq!(completion.next_reset, None);
        assert_eq!(completion.error, Some(error));
        assert!(!feedback.is_copied());
    }

    #[test]
    fn finish_request_ignores_non_matching_token() {
        let feedback = ClipboardCopyFeedbackRef::default();
        let _request = begin_request(&feedback, || ClipboardToken(5)).expect("request");

        assert!(
            finish_request(
                &feedback,
                ClipboardToken(8),
                &ClipboardWriteOutcome::Succeeded,
                || TimerToken(9),
            )
            .is_none()
        );
        assert!(!feedback.is_copied());
    }

    #[test]
    fn handle_reset_timer_only_clears_matching_token() {
        let feedback = ClipboardCopyFeedbackRef::default();
        {
            let mut state = feedback.lock();
            state.copied = true;
            state.reset_token = Some(TimerToken(4));
        }

        assert!(!handle_reset_timer(&feedback, TimerToken(3)));
        assert!(feedback.is_copied());
        assert!(handle_reset_timer(&feedback, TimerToken(4)));
        assert!(!feedback.is_copied());
    }
}
