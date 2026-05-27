use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{KeyCode, Modifiers};
use fret_runtime::{KeyChord, Model};
use fret_ui::GlobalElementId;
use fret_ui::action::{KeyDownCx, UiActionHostExt as _};
use fret_ui::{ElementContext, UiHost};

use super::super::interaction_runtime::ImUiLifecycleSessionState;

pub(super) struct SelectableKeyboardOptions {
    pub(super) close_popup: Option<Model<bool>>,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_selectable_keyboard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    focusable: bool,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
    options: SelectableKeyboardOptions,
) {
    let nav_items = selectable_menu_nav_items(cx, id, focusable);
    let close_popup_for_key = options.close_popup;
    let lifecycle_model_for_shortcut = lifecycle_model;
    cx.key_on_key_down_for(
        id,
        Arc::new(move |host, acx, down| {
            if let Some(shortcut) = options.activate_shortcut {
                let matches_shortcut = down.key == shortcut.key && down.modifiers == shortcut.mods;
                if matches_shortcut
                    && (!down.repeat || options.shortcut_repeat)
                    && !down.ime_composing
                {
                    super::super::mark_lifecycle_instant_if_inactive(
                        host,
                        acx,
                        &lifecycle_model_for_shortcut,
                        false,
                    );
                    if let Some(open) = close_popup_for_key.as_ref() {
                        let _ = host.update_model(open, |v| *v = false);
                    }
                    host.record_transient_event(acx, super::super::KEY_CLICKED);
                    host.notify(acx);
                    return true;
                }
            }

            if record_context_menu_request(host, acx, &down) {
                return true;
            }

            let Some(nav_items) = nav_items.as_ref() else {
                return false;
            };
            move_popup_menu_focus(host, acx, &down, nav_items, id)
        }),
    );
}

fn selectable_menu_nav_items<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    focusable: bool,
) -> Option<Rc<RefCell<Vec<GlobalElementId>>>> {
    if !focusable {
        return None;
    }

    let nav_items = cx
        .inherited_state::<super::super::popup_overlay::ImUiMenuNavState>()
        .map(|st| st.items.clone());
    if let Some(nav_items) = nav_items.as_ref() {
        nav_items.borrow_mut().push(id);
    }
    nav_items
}

fn record_context_menu_request(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    acx: fret_ui::action::ActionCx,
    down: &KeyDownCx,
) -> bool {
    let is_menu_key = down.key == KeyCode::ContextMenu;
    let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
    if !(is_menu_key || is_shift_f10) {
        return false;
    }

    host.record_transient_event(acx, super::super::KEY_CONTEXT_MENU_REQUESTED);
    host.notify(acx);
    true
}

fn move_popup_menu_focus(
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
