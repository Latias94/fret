use super::key_navigation;
use crate::ui::canvas::widget::*;

fn commit_context_menu_input_handled<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    menu: ContextMenuState,
) -> bool {
    canvas.interaction.context_menu = Some(menu);
    cx.stop_propagation();
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

pub(super) fn handle_context_menu_escape<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.context_menu.take().is_some() {
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }
    false
}
pub(super) fn handle_context_menu_key_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    key: fret_core::KeyCode,
) -> bool {
    let Some(mut menu) = canvas.interaction.context_menu.take() else {
        return false;
    };
    match key {
        fret_core::KeyCode::ArrowDown => {
            key_navigation::advance_context_menu_active_item(&mut menu, false);
            menu.typeahead.clear();
            return commit_context_menu_input_handled(canvas, cx, menu);
        }
        fret_core::KeyCode::ArrowUp => {
            key_navigation::advance_context_menu_active_item(&mut menu, true);
            menu.typeahead.clear();
            return commit_context_menu_input_handled(canvas, cx, menu);
        }
        fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter => {
            let ix = menu.active_item.min(menu.items.len().saturating_sub(1));
            canvas.activate_context_menu_selection(cx, &menu, ix);
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            return true;
        }
        fret_core::KeyCode::Backspace => {
            if key_navigation::pop_context_menu_typeahead(&mut menu) {
                return commit_context_menu_input_handled(canvas, cx, menu);
            }
        }
        _ => {}
    }
    if let Some(ch) = fret_core::keycode_to_ascii_lowercase(key) {
        key_navigation::apply_context_menu_typeahead(&mut menu, ch);
        return commit_context_menu_input_handled(canvas, cx, menu);
    }
    canvas.interaction.context_menu = Some(menu);
    false
}
