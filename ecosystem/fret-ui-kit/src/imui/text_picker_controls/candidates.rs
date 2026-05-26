use std::sync::Arc;

use super::super::InputTextPickerOptions;

pub(super) struct InputTextPickerCandidateVisibility {
    pub(super) visible_candidates: Vec<(usize, Arc<str>)>,
    pub(super) hide_for_exact_match: bool,
    pub(super) picker_candidate_visible: bool,
}

pub(super) fn resolve_text_picker_candidates(
    current: &str,
    candidates: &[Arc<str>],
    options: &InputTextPickerOptions,
) -> InputTextPickerCandidateVisibility {
    let visible_candidates = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| options.filter.matches(current, candidate.as_ref()))
        .take(options.max_items)
        .map(|(index, candidate)| (index, candidate.clone()))
        .collect::<Vec<_>>();
    let hide_for_exact_match = options.hide_when_exact_match
        && candidates
            .iter()
            .any(|candidate| candidate.as_ref() == current);
    let picker_candidate_visible =
        !visible_candidates.is_empty() && (options.open_when_empty || !current.is_empty());

    InputTextPickerCandidateVisibility {
        visible_candidates,
        hide_for_exact_match,
        picker_candidate_visible,
    }
}
