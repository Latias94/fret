//! Menu trigger helpers (Radix-aligned outcomes).
//!
//! Radix wrappers like `DropdownMenu` and `Menubar` expose a dedicated trigger component that
//! opens the menu via keyboard affordances (APG “menu button” behavior).
//!
//! In Fret we keep the trigger rendering in wrapper crates, but centralize reusable wiring here
//! so keyboard behavior stays consistent.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, KeyDownCx, UiFocusActionHost};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::trigger_a11y;

fn is_context_menu_open_shortcut(down: KeyDownCx) -> bool {
    if down.repeat {
        return false;
    }

    let no_extra_modifiers = !down.modifiers.ctrl
        && !down.modifiers.alt
        && !down.modifiers.meta
        && !down.modifiers.alt_gr;

    let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift && no_extra_modifiers;
    let is_context_menu_key =
        down.key == KeyCode::ContextMenu && !down.modifiers.shift && no_extra_modifiers;

    is_shift_f10 || is_context_menu_key
}

/// Stamps Radix-like trigger relationships:
/// - `expanded` mirrors `aria-expanded`
/// - `controls_element` mirrors `aria-controls` (by element id)
pub fn apply_menu_trigger_a11y(
    trigger: AnyElement,
    expanded: bool,
    content_element: Option<GlobalElementId>,
) -> AnyElement {
    trigger_a11y::apply_trigger_controls_expanded(trigger, Some(expanded), content_element)
}

/// Wire “menu button opens on ArrowDown/ArrowUp” behavior onto an existing trigger element.
///
/// This intentionally does **not** handle Enter/Space because many triggers implement those
/// through pressable activation hooks (and double-wiring would toggle twice).
pub fn wire_open_on_arrow_keys<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    open: Model<bool>,
) {
    cx.key_on_key_down_for(
        trigger_id,
        Arc::new(
            move |host: &mut dyn UiFocusActionHost, _acx: ActionCx, down: KeyDownCx| {
                if down.repeat {
                    return false;
                }
                match down.key {
                    KeyCode::ArrowDown | KeyCode::ArrowUp => {
                        let _ = host.models_mut().update(&open, |v| *v = true);
                        true
                    }
                    _ => false,
                }
            },
        ),
    );
}

/// Wire “ArrowDown/ArrowUp opens, then focuses the first/last item when already open”.
///
/// This matches common menu button behavior: pointer-opening a menu preserves trigger focus, but
/// the first arrow key press transfers focus into the menu content.
pub fn wire_open_or_focus_on_arrow_keys<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    open: Model<bool>,
    first_item_focus_id: Rc<Cell<Option<GlobalElementId>>>,
    last_item_focus_id: Rc<Cell<Option<GlobalElementId>>>,
) {
    cx.key_on_key_down_for(
        trigger_id,
        Arc::new(
            move |host: &mut dyn UiFocusActionHost, _acx: ActionCx, down: KeyDownCx| {
                if down.repeat {
                    return false;
                }
                match down.key {
                    KeyCode::ArrowDown | KeyCode::ArrowUp => {
                        let is_open = host.models_mut().read(&open, |v| *v).ok().unwrap_or(false);
                        if !is_open {
                            let _ = host.models_mut().update(&open, |v| *v = true);
                            return true;
                        }

                        let target = match down.key {
                            KeyCode::ArrowDown => first_item_focus_id.get(),
                            KeyCode::ArrowUp => last_item_focus_id.get(),
                            _ => None,
                        };
                        let Some(target) = target else {
                            return false;
                        };
                        host.request_focus(target);
                        true
                    }
                    _ => false,
                }
            },
        ),
    );
}

/// Wire “context menu opens on Shift+F10” behavior onto an existing trigger element.
///
/// This mirrors the common desktop affordance for opening a context menu from the keyboard.
pub fn wire_open_on_shift_f10<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    open: Model<bool>,
) {
    cx.key_on_key_down_for(
        trigger_id,
        Arc::new(
            move |host: &mut dyn UiFocusActionHost, _acx: ActionCx, down: KeyDownCx| {
                if !is_context_menu_open_shortcut(down) {
                    return false;
                }
                let _ = host.models_mut().update(&open, |v| *v = true);
                true
            },
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::Modifiers;

    #[test]
    fn context_menu_shortcut_accepts_shift_f10() {
        let mut modifiers = Modifiers::default();
        modifiers.shift = true;
        assert!(is_context_menu_open_shortcut(KeyDownCx {
            key: KeyCode::F10,
            modifiers,
            repeat: false,
        }));
    }

    #[test]
    fn context_menu_shortcut_accepts_context_menu_key() {
        assert!(is_context_menu_open_shortcut(KeyDownCx {
            key: KeyCode::ContextMenu,
            modifiers: Modifiers::default(),
            repeat: false,
        }));
    }

    #[test]
    fn context_menu_shortcut_rejects_plain_f10() {
        assert!(!is_context_menu_open_shortcut(KeyDownCx {
            key: KeyCode::F10,
            modifiers: Modifiers::default(),
            repeat: false,
        }));
    }

    #[test]
    fn context_menu_shortcut_rejects_repeat_events() {
        let mut modifiers = Modifiers::default();
        modifiers.shift = true;
        assert!(!is_context_menu_open_shortcut(KeyDownCx {
            key: KeyCode::F10,
            modifiers,
            repeat: true,
        }));
    }
}
