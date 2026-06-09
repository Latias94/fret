use std::sync::Arc;

use super::super::super::super::InputTextPickerOptions;
use super::super::super::candidates::resolve_text_picker_candidates;

pub(super) struct PreparedTextPickerCandidates {
    pub(super) visible_candidates: Vec<(usize, Arc<str>)>,
    pub(super) hide_for_exact_match: bool,
    pub(super) picker_candidate_visible: bool,
}

pub(super) fn prepare_text_picker_session_candidates(
    current: &str,
    candidate_sources: &[Arc<str>],
    options: &InputTextPickerOptions,
) -> PreparedTextPickerCandidates {
    let candidate_visibility = resolve_text_picker_candidates(current, candidate_sources, options);

    PreparedTextPickerCandidates {
        visible_candidates: candidate_visibility.visible_candidates,
        hide_for_exact_match: candidate_visibility.hide_for_exact_match,
        picker_candidate_visible: candidate_visibility.picker_candidate_visible,
    }
}
