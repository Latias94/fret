use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PointerDownTailRoute {
    RightClick,
    LeftClick,
    Ignore,
}

pub(in super::super) fn dispatch_tail_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) {
    match tail_pointer_down_route(button) {
        PointerDownTailRoute::RightClick => {
            let _ =
                right_click::handle_right_click_pointer_down(canvas, cx, snapshot, position, zoom);
        }
        PointerDownTailRoute::LeftClick => {
            let _ = left_click::handle_left_click_pointer_down(
                canvas, cx, snapshot, position, modifiers, zoom,
            );
        }
        PointerDownTailRoute::Ignore => {}
    }
}

fn tail_pointer_down_route(button: MouseButton) -> PointerDownTailRoute {
    match button {
        MouseButton::Right => PointerDownTailRoute::RightClick,
        MouseButton::Left => PointerDownTailRoute::LeftClick,
        _ => PointerDownTailRoute::Ignore,
    }
}

#[cfg(test)]
mod tests;
