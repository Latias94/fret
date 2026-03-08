use super::*;

pub(super) fn handle_pan_activation_key_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool {
    if modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr {
        return false;
    }

    if !snapshot.interaction.space_to_pan
        || canvas.interaction.searcher.is_some()
        || canvas.interaction.context_menu.is_some()
    {
        return false;
    }

    let Some(crate::io::NodeGraphKeyCode(key_code)) = snapshot.interaction.pan_activation_key_code
    else {
        return false;
    };

    if key != key_code || canvas.interaction.pan_activation_key_held {
        return false;
    }

    canvas.interaction.pan_activation_key_held = true;
    invalidate_pan_activation(cx);
    cx.stop_propagation();
    true
}

pub(super) fn handle_pan_activation_key_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
) -> bool {
    let Some(crate::io::NodeGraphKeyCode(key_code)) = snapshot.interaction.pan_activation_key_code
    else {
        return false;
    };

    if key != key_code || !canvas.interaction.pan_activation_key_held {
        return false;
    }

    canvas.interaction.pan_activation_key_held = false;
    invalidate_pan_activation(cx);
    true
}

fn invalidate_pan_activation<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}
