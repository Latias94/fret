use fret_core::{Point, Size};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::FloatingWindowResizeSnapshot;
use commit::{ResizeStateCommitInput, commit_resize_state};
pub(in crate::imui) use output::FloatingWindowResizeStateOutput;

mod commit;
mod drag_apply;
mod initial;
mod output;

pub(in crate::imui) fn prepare_resize_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    id: &str,
    area_position: Point,
    initial_size: Option<Size>,
    resize: Option<super::super::FloatingWindowResizeOptions>,
    resize_snapshot: Option<FloatingWindowResizeSnapshot>,
    collapsed: bool,
    scale_factor: f32,
) -> FloatingWindowResizeStateOutput {
    let resizing = resize_snapshot
        .map(|snapshot| snapshot.dragging)
        .unwrap_or(false)
        && !collapsed;

    commit_resize_state(
        cx,
        ResizeStateCommitInput {
            window_id,
            id,
            area_position,
            initial_size,
            resize,
            resize_snapshot,
            collapsed,
            scale_factor,
            resizing,
        },
    )
}
