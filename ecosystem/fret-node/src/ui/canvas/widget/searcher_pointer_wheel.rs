mod delta;

use fret_core::Modifiers;

use super::*;

pub(super) use delta::apply_searcher_wheel_delta;

pub(super) fn scroll_searcher_from_wheel<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    delta: Point,
    modifiers: Modifiers,
) -> bool {
    if modifiers.ctrl || modifiers.meta {
        return false;
    }

    let Some(searcher) = canvas.interaction.searcher.as_mut() else {
        return false;
    };
    apply_searcher_wheel_delta::<M>(searcher, delta.y.0)
}
