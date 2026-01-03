//! Menu trigger helpers (Radix-aligned outcomes).
//!
//! Radix wrappers like `DropdownMenu` and `Menubar` expose a dedicated trigger component that
//! opens the menu via keyboard affordances (APG “menu button” behavior).
//!
//! In Fret we keep the trigger rendering in wrapper crates, but centralize reusable wiring here
//! so keyboard behavior stays consistent.

use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, KeyDownCx, UiFocusActionHost};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

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
                if down.repeat {
                    return false;
                }
                let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                if !is_shift_f10 {
                    return false;
                }
                let _ = host.models_mut().update(&open, |v| *v = true);
                true
            },
        ),
    );
}
