use super::widget_tail::{WidgetHandledCx, WidgetPaintInvalidationCx};
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, ViewSnapshot, menu_session};
use fret_ui::UiHost;

pub(super) fn handle_pan_activation_key_down<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: WidgetHandledCx<H>,
{
    if modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr {
        return false;
    }

    if !snapshot.interaction.space_to_pan
        || menu_session::has_active_menu_session(&canvas.interaction)
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

pub(super) fn handle_pan_activation_key_up<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    key: fret_core::KeyCode,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: WidgetPaintInvalidationCx<H>,
{
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

fn invalidate_pan_activation<H: UiHost>(cx: &mut impl WidgetPaintInvalidationCx<H>) {
    super::widget_tail::invalidate_widget_paint(cx);
}
