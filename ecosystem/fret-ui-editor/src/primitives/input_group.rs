//! Editor input-group primitives (joined frame + segments).
//!
//! This is a policy-only helper for composing "joined" controls (axis markers, value fields,
//! small action icons) into a single input-like frame without style drift.

mod frame;
pub(crate) use frame::{
    EditorInputGroupFrameOverrides, editor_input_group_frame,
    editor_input_group_frame_with_overrides,
};
mod joined;
#[allow(unused_imports)]
pub(crate) use joined::{
    EditorJoinedInputContents, editor_joined_input_frame,
    editor_joined_input_frame_segments_with_overrides, editor_joined_input_frame_with_overrides,
};

mod segments;
pub(crate) use segments::{
    derived_test_id, editor_axis_segment, editor_clear_button_segment,
    editor_clear_button_segment_multiline, editor_icon_button_segment, editor_icon_segment,
    editor_input_group_divider, editor_input_group_inset, editor_input_group_row,
    editor_input_group_segment, editor_input_value_text, editor_text_segment,
};

#[cfg(test)]
mod tests;
