use fret_ui::UiHost;

use crate::ui::canvas::widget::*;

use super::super::ui;
use super::{active_item, typeahead};

pub(super) fn handle_context_menu_key_down_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    key: fret_core::KeyCode,
) -> bool {
    let Some(mut menu) = super::super::take_context_menu(&mut canvas.interaction) else {
        return false;
    };

    match key {
        fret_core::KeyCode::ArrowDown => {
            active_item::advance_context_menu_active_item(&mut menu, false);
            menu.typeahead.clear();
            return ui::restore_context_menu_event(canvas, cx, menu);
        }
        fret_core::KeyCode::ArrowUp => {
            active_item::advance_context_menu_active_item(&mut menu, true);
            menu.typeahead.clear();
            return ui::restore_context_menu_event(canvas, cx, menu);
        }
        fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter => {
            let _ = canvas.activate_context_menu_active_selection(cx, &menu);
            return ui::finish_context_menu_event(cx);
        }
        fret_core::KeyCode::Backspace => {
            if typeahead::pop_context_menu_typeahead(&mut menu) {
                return ui::restore_context_menu_event(canvas, cx, menu);
            }
        }
        _ => {}
    }

    if let Some(ch) = fret_core::keycode_to_ascii_lowercase(key) {
        typeahead::apply_context_menu_typeahead(&mut menu, ch);
        return ui::restore_context_menu_event(canvas, cx, menu);
    }

    super::super::restore_context_menu(&mut canvas.interaction, menu);
    false
}
