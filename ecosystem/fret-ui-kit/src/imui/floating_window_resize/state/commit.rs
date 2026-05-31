use fret_core::{Point, Size};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::FloatingWindowResizeSnapshot;
use super::initial::initial_float_window_state;
use super::output::FloatingWindowResizeStateOutput;
use mutation::{ResizeStateMutationInput, apply_resize_state_mutation};
use output_pack::{CommittedResizeState, output_from_committed_resize_state};

mod mutation;
mod output_pack;

pub(super) struct ResizeStateCommitInput<'a> {
    pub(super) window_id: GlobalElementId,
    pub(super) id: &'a str,
    pub(super) area_position: Point,
    pub(super) initial_size: Option<Size>,
    pub(super) resize: Option<super::super::super::FloatingWindowResizeOptions>,
    pub(super) resize_snapshot: Option<FloatingWindowResizeSnapshot>,
    pub(super) collapsed: bool,
    pub(super) scale_factor: f32,
    pub(super) resizing: bool,
}

pub(super) fn commit_resize_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: ResizeStateCommitInput<'_>,
) -> FloatingWindowResizeStateOutput {
    let committed = cx.state_for(
        input.window_id,
        || initial_float_window_state(input.id, input.initial_size),
        |st| {
            let mut position = apply_resize_state_mutation(
                st,
                ResizeStateMutationInput {
                    area_position: input.area_position,
                    resize: input.resize,
                    resize_snapshot: input.resize_snapshot,
                    collapsed: input.collapsed,
                },
            );
            st.size = super::super::super::snap_size_to_device_pixels(input.scale_factor, st.size);
            position =
                super::super::super::snap_point_to_device_pixels(input.scale_factor, position);

            CommittedResizeState::from_window_state(position, st)
        },
    );

    output_from_committed_resize_state(committed, input.resizing)
}
