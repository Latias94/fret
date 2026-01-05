//! Menubar trigger-row policies (Radix-aligned outcomes).
//!
//! Radix `Menubar` differs from `DropdownMenu` primarily in its trigger-row behavior:
//! - only one top-level menu should be open at a time
//! - when any menu is open, hovering/focusing another trigger switches the open menu
//! - activating a trigger toggles its menu and updates the active trigger
//!
//! This module provides reusable helpers for coordinating those behaviors without imposing any
//! visual/layout decisions on consumers.

use std::sync::Arc;

use crate::declarative::model_watch::ModelWatchExt as _;
use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, KeyDownCx, OnActivate, OnKeyDown, UiFocusActionHost};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone)]
pub struct MenubarActiveTrigger {
    pub trigger: GlobalElementId,
    pub open: Model<bool>,
}

#[derive(Debug, Clone)]
pub struct MenubarTriggerRowEntry {
    pub key: Arc<str>,
    pub trigger: GlobalElementId,
    pub open: Model<bool>,
    pub enabled: bool,
}

#[derive(Default)]
struct MenubarTriggerRowGroupState {
    active: Option<Model<Option<MenubarActiveTrigger>>>,
    registry: Option<Model<Vec<MenubarTriggerRowEntry>>>,
}

/// Ensure a per-menubar active-trigger model exists.
///
/// Call this with a stable group ID shared by all triggers in the same menubar (typically the
/// menubar root element ID).
#[track_caller]
pub fn ensure_group_active_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    group: GlobalElementId,
) -> Model<Option<MenubarActiveTrigger>> {
    let existing = cx.with_state_for(group, MenubarTriggerRowGroupState::default, |st| {
        st.active.clone()
    });
    if let Some(existing) = existing {
        return existing;
    }

    let active = cx.app.models_mut().insert(None);
    cx.with_state_for(group, MenubarTriggerRowGroupState::default, |st| {
        st.active = Some(active.clone());
    });
    active
}

/// Ensure a per-menubar registry model exists.
///
/// The registry tracks the trigger order for ArrowLeft/ArrowRight switching while a menu is open.
#[track_caller]
pub fn ensure_group_registry_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    group: GlobalElementId,
) -> Model<Vec<MenubarTriggerRowEntry>> {
    let existing = cx.with_state_for(group, MenubarTriggerRowGroupState::default, |st| {
        st.registry.clone()
    });
    if let Some(existing) = existing {
        return existing;
    }

    let registry = cx.app.models_mut().insert(Vec::new());
    cx.with_state_for(group, MenubarTriggerRowGroupState::default, |st| {
        st.registry = Some(registry.clone());
    });
    registry
}

/// Register or update a trigger entry in the group registry.
pub fn register_trigger_in_registry<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    registry: Model<Vec<MenubarTriggerRowEntry>>,
    key: Arc<str>,
    trigger: GlobalElementId,
    open: Model<bool>,
    enabled: bool,
) {
    let _ = cx.app.models_mut().update(&registry, move |v| {
        if let Some(existing) = v.iter_mut().find(|e| e.key.as_ref() == key.as_ref()) {
            existing.trigger = trigger;
            existing.open = open;
            existing.enabled = enabled;
            return;
        }

        v.push(MenubarTriggerRowEntry {
            key,
            trigger,
            open,
            enabled,
        });
    });
}

fn find_next_enabled(
    entries: &[MenubarTriggerRowEntry],
    start: usize,
    forward: bool,
) -> Option<usize> {
    if entries.is_empty() {
        return None;
    }

    let len = entries.len();
    for step in 1..=len {
        let idx = if forward {
            (start + step) % len
        } else {
            (start + len - (step % len)) % len
        };
        if entries.get(idx).is_some_and(|e| e.enabled) {
            return Some(idx);
        }
    }
    None
}

/// Build an ArrowLeft/ArrowRight handler for switching the open menubar menu.
pub fn switch_open_menu_on_horizontal_arrows(
    group_active: Model<Option<MenubarActiveTrigger>>,
    registry: Model<Vec<MenubarTriggerRowEntry>>,
    current_key: Arc<str>,
) -> OnKeyDown {
    Arc::new(
        move |host: &mut dyn UiFocusActionHost, acx: ActionCx, down: KeyDownCx| {
            if down.repeat {
                return false;
            }

            let forward = match down.key {
                KeyCode::ArrowRight => true,
                KeyCode::ArrowLeft => false,
                _ => return false,
            };

            let Some(entries) = host.models_mut().get_cloned(&registry) else {
                return false;
            };
            let Some(current_idx) = entries
                .iter()
                .position(|e| e.key.as_ref() == current_key.as_ref())
            else {
                return false;
            };

            let Some(next_idx) = find_next_enabled(&entries, current_idx, forward) else {
                return false;
            };

            let Some(current) = entries.get(current_idx) else {
                return false;
            };
            let Some(next) = entries.get(next_idx) else {
                return false;
            };

            if current.trigger == next.trigger {
                return false;
            }

            let _ = host.models_mut().update(&current.open, |v| *v = false);
            let _ = host.models_mut().update(&next.open, |v| *v = true);
            let open_for_state = next.open.clone();
            let _ = host.models_mut().update(&group_active, |v| {
                *v = Some(MenubarActiveTrigger {
                    trigger: next.trigger,
                    open: open_for_state,
                });
            });

            host.request_focus(next.trigger);
            host.request_redraw(acx.window);
            true
        },
    )
}

/// Install ArrowLeft/ArrowRight switching onto a specific menu item element.
///
/// Use this inside the current menu's content items so the key event is observed even when the
/// focused element is a menu item pressable (key hooks do not bubble).
pub fn wire_switch_open_menu_on_horizontal_arrows<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    item_id: GlobalElementId,
    group_active: Model<Option<MenubarActiveTrigger>>,
    registry: Model<Vec<MenubarTriggerRowEntry>>,
    current_key: Arc<str>,
) {
    cx.key_add_on_key_down_for(
        item_id,
        switch_open_menu_on_horizontal_arrows(group_active, registry, current_key),
    );
}

/// Enforce Radix-aligned trigger-row invariants for a single trigger.
///
/// Intended to be called from within the trigger's render hook (e.g. a pressable closure), so
/// it can observe `hovered/pressed/focused` state and coordinate `open` with the group.
#[track_caller]
pub fn sync_trigger_row_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    group_active: Model<Option<MenubarActiveTrigger>>,
    trigger_id: GlobalElementId,
    open: Model<bool>,
    enabled: bool,
    hovered: bool,
    pressed: bool,
    focused: bool,
) {
    let active_value = cx.watch_model(&group_active).cloned().flatten();
    let is_open = cx.watch_model(&open).copied().unwrap_or(false);

    if active_value
        .as_ref()
        .is_some_and(|active_value| active_value.trigger != trigger_id)
        && is_open
    {
        let _ = cx.app.models_mut().update(&open, |v| *v = false);
    }

    if active_value
        .as_ref()
        .is_some_and(|active_value| active_value.trigger == trigger_id)
        && !is_open
    {
        let _ = cx.app.models_mut().update(&group_active, |v| *v = None);
    }

    if active_value.is_none() && is_open {
        let open_for_state = open.clone();
        let _ = cx.app.models_mut().update(&group_active, |v| {
            *v = Some(MenubarActiveTrigger {
                trigger: trigger_id,
                open: open_for_state,
            });
        });
    }

    let active_value = cx.watch_model(&group_active).cloned().flatten();
    if enabled
        && (hovered || focused)
        && !pressed
        && active_value
            .as_ref()
            .is_some_and(|active_value| active_value.trigger != trigger_id)
    {
        if let Some(prev) = active_value.as_ref() {
            let _ = cx.app.models_mut().update(&prev.open, |v| *v = false);
        }
        let _ = cx.app.models_mut().update(&open, |v| *v = true);
        let open_for_state = open.clone();
        let _ = cx.app.models_mut().update(&group_active, |v| {
            *v = Some(MenubarActiveTrigger {
                trigger: trigger_id,
                open: open_for_state,
            });
        });
    }
}

/// Build an activation handler that toggles a trigger's menu and updates group active state.
pub fn toggle_on_activate(
    group_active: Model<Option<MenubarActiveTrigger>>,
    trigger_id: GlobalElementId,
    open: Model<bool>,
) -> OnActivate {
    Arc::new(move |host, _cx, _reason| {
        let cur = host.models_mut().get_cloned(&group_active).flatten();
        match cur {
            Some(cur) if cur.trigger == trigger_id => {
                let _ = host.models_mut().update(&open, |v| *v = false);
                let _ = host.models_mut().update(&group_active, |v| *v = None);
            }
            prev => {
                if let Some(prev) = prev {
                    let _ = host.models_mut().update(&prev.open, |v| *v = false);
                }
                let _ = host.models_mut().update(&open, |v| *v = true);
                let open_for_state = open.clone();
                let _ = host.models_mut().update(&group_active, |v| {
                    *v = Some(MenubarActiveTrigger {
                        trigger: trigger_id,
                        open: open_for_state,
                    });
                });
            }
        }
    })
}
