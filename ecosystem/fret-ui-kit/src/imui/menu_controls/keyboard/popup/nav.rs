use std::cell::RefCell;
use std::rc::Rc;

use fret_core::{KeyCode, Modifiers};
use fret_ui::action::KeyDownCx;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

pub(super) fn register_popup_menu_nav_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) -> Option<Rc<RefCell<Vec<GlobalElementId>>>> {
    let nav_items = cx
        .inherited_state::<crate::imui::popup_overlay::ImUiMenuNavState>()
        .map(|st| st.items.clone());
    if let Some(nav_items) = nav_items.as_ref() {
        nav_items.borrow_mut().push(id);
    }
    nav_items
}

pub(super) fn move_popup_menu_focus(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    acx: fret_ui::action::ActionCx,
    down: &KeyDownCx,
    nav_items: &Rc<RefCell<Vec<GlobalElementId>>>,
    item_id: GlobalElementId,
) -> bool {
    if down.repeat || down.modifiers != Modifiers::default() {
        return false;
    }

    let (dir, jump_to) = match down.key {
        KeyCode::ArrowDown => (1isize, None),
        KeyCode::ArrowUp => (-1isize, None),
        KeyCode::Home => (0isize, Some(0usize)),
        KeyCode::End => (0isize, Some(usize::MAX)),
        _ => return false,
    };

    let items = nav_items.borrow();
    if items.is_empty() {
        return false;
    }
    let len = items.len();
    let idx = items.iter().position(|id| *id == item_id);
    let next_idx = if let Some(jump) = jump_to {
        if jump == usize::MAX {
            len - 1
        } else {
            jump.min(len - 1)
        }
    } else {
        let current = idx.unwrap_or_else(|| if dir < 0 { len - 1 } else { 0 });
        ((current as isize + dir + len as isize) % len as isize) as usize
    };

    host.request_focus(items[next_idx]);
    host.notify(acx);
    true
}
