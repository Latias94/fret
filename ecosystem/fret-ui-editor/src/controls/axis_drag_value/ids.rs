use std::sync::Arc;

use crate::primitives::input_group::derived_test_id;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AxisDragValueTestIds {
    pub(super) scrub: Option<Arc<str>>,
    pub(super) active_typing: Option<Arc<str>>,
    pub(super) scrub_axis: Option<Arc<str>>,
    pub(super) scrub_value: Option<Arc<str>>,
    pub(super) scrub_prefix: Option<Arc<str>>,
    pub(super) scrub_suffix: Option<Arc<str>>,
    pub(super) typing_axis: Option<Arc<str>>,
    pub(super) typing_input: Option<Arc<str>>,
    pub(super) typing_prefix: Option<Arc<str>>,
    pub(super) typing_suffix: Option<Arc<str>>,
    pub(super) typing_error_icon: Option<Arc<str>>,
    pub(super) scrub_reset: Option<Arc<str>>,
    pub(super) typing_reset: Option<Arc<str>>,
}

pub(super) fn axis_drag_value_test_ids(
    scrub_test_id: Option<Arc<str>>,
    explicit_reset_test_id: Option<Arc<str>>,
    typing: bool,
) -> AxisDragValueTestIds {
    let typing_test_id = derived_test_id(scrub_test_id.as_ref(), "typing");
    let active_typing = if typing { typing_test_id } else { None };

    let scrub_reset = explicit_reset_test_id
        .clone()
        .or_else(|| derived_test_id(scrub_test_id.as_ref(), "reset"));
    let typing_reset = explicit_reset_test_id
        .as_ref()
        .and_then(|id| typing.then(|| Arc::<str>::from(format!("{}.typing", id.as_ref()))))
        .or_else(|| derived_test_id(active_typing.as_ref(), "reset"));

    AxisDragValueTestIds {
        scrub_axis: derived_test_id(scrub_test_id.as_ref(), "axis"),
        scrub_value: derived_test_id(scrub_test_id.as_ref(), "value"),
        scrub_prefix: derived_test_id(scrub_test_id.as_ref(), "prefix"),
        scrub_suffix: derived_test_id(scrub_test_id.as_ref(), "suffix"),
        typing_axis: derived_test_id(active_typing.as_ref(), "axis"),
        typing_input: derived_test_id(active_typing.as_ref(), "input"),
        typing_prefix: derived_test_id(active_typing.as_ref(), "prefix"),
        typing_suffix: derived_test_id(active_typing.as_ref(), "suffix"),
        typing_error_icon: derived_test_id(active_typing.as_ref(), "error"),
        scrub_reset,
        typing_reset,
        scrub: scrub_test_id,
        active_typing,
    }
}
