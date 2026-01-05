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

use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone)]
pub struct MenubarActiveTrigger {
    pub trigger: GlobalElementId,
    pub open: Model<bool>,
}

#[derive(Default)]
struct MenubarTriggerRowGroupState {
    active: Option<Model<Option<MenubarActiveTrigger>>>,
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
