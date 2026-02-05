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
use std::time::Duration;

use crate::declarative::model_watch::ModelWatchExt as _;
use fret_core::KeyCode;
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::action::{
    ActionCx, KeyDownCx, OnActivate, OnHoverChange, OnKeyDown, OnTimer, UiActionHost,
    UiFocusActionHost,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone)]
pub struct MenubarActiveTrigger {
    pub trigger: GlobalElementId,
    pub open: Model<bool>,
}

#[derive(Debug, Clone)]
pub struct MenubarTriggerRowEntry {
    pub trigger: GlobalElementId,
    pub open: Model<bool>,
    pub enabled: bool,
}

#[derive(Default)]
struct MenubarTriggerRowGroupState {
    active: Option<Model<Option<MenubarActiveTrigger>>>,
    registry: Option<Model<Vec<MenubarTriggerRowEntry>>>,
}

#[derive(Default)]
struct MenubarTriggerHoverSwitchState {
    installed: bool,
    hovered: Option<Model<bool>>,
    timer: Option<Model<Option<TimerToken>>>,
}

const DEFAULT_HOVER_SWITCH_DELAY: Duration = Duration::from_millis(90);

fn cancel_hover_switch_timer(host: &mut dyn UiActionHost, timer: &Model<Option<TimerToken>>) {
    let pending = host.models_mut().read(timer, |v| *v).ok().flatten();
    let Some(token) = pending else {
        return;
    };
    host.push_effect(Effect::CancelTimer { token });
    let _ = host.models_mut().update(timer, |v| *v = None);
}

fn arm_hover_switch_timer(
    host: &mut dyn UiActionHost,
    window: fret_core::AppWindowId,
    delay: Duration,
    timer: &Model<Option<TimerToken>>,
) -> TimerToken {
    cancel_hover_switch_timer(host, timer);
    let token = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window: Some(window),
        token,
        after: delay,
        repeat: None,
    });
    let _ = host.models_mut().update(timer, |v| *v = Some(token));
    token
}

fn hover_switch_on_timer_handler(
    group_active: Model<Option<MenubarActiveTrigger>>,
    trigger_id: GlobalElementId,
    open: Model<bool>,
    hovered: Model<bool>,
    timer: Model<Option<TimerToken>>,
) -> OnTimer {
    Arc::new(move |host, acx, token| {
        let armed = host.models_mut().read(&timer, |v| *v).ok().flatten();
        if armed != Some(token) {
            return false;
        }

        let _ = host.models_mut().update(&timer, |v| *v = None);

        let still_hovered = host
            .models_mut()
            .read(&hovered, |v| *v)
            .ok()
            .unwrap_or(false);
        if !still_hovered {
            host.request_redraw(acx.window);
            return true;
        }

        let active = host.models_mut().get_cloned(&group_active).flatten();
        let Some(active) = active else {
            host.request_redraw(acx.window);
            return true;
        };
        if active.trigger == trigger_id {
            host.request_redraw(acx.window);
            return true;
        }

        let active_open = host
            .models_mut()
            .read(&active.open, |v| *v)
            .ok()
            .unwrap_or(false);
        if !active_open {
            host.request_redraw(acx.window);
            return true;
        }

        let _ = host.models_mut().update(&active.open, |v| *v = false);
        let _ = host.models_mut().update(&open, |v| *v = true);
        let open_for_state = open.clone();
        let _ = host.models_mut().update(&group_active, |v| {
            *v = Some(MenubarActiveTrigger {
                trigger: trigger_id,
                open: open_for_state,
            });
        });

        host.request_redraw(acx.window);
        true
    })
}

fn hover_switch_on_hover_change_handler(
    group_active: Model<Option<MenubarActiveTrigger>>,
    trigger_id: GlobalElementId,
    enabled: bool,
    hovered: Model<bool>,
    timer: Model<Option<TimerToken>>,
) -> OnHoverChange {
    Arc::new(move |host, acx, is_hovered| {
        let _ = host.models_mut().update(&hovered, |v| *v = is_hovered);
        if !is_hovered {
            cancel_hover_switch_timer(host, &timer);
            return;
        }

        if !enabled {
            return;
        }

        let active = host.models_mut().get_cloned(&group_active).flatten();
        let Some(active) = active else {
            return;
        };
        if active.trigger == trigger_id {
            return;
        }

        let active_open = host
            .models_mut()
            .read(&active.open, |v| *v)
            .ok()
            .unwrap_or(false);
        if !active_open {
            return;
        }

        arm_hover_switch_timer(host, acx.window, DEFAULT_HOVER_SWITCH_DELAY, &timer);
        host.request_redraw(acx.window);
    })
}

fn ensure_hover_switch_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
) -> (Model<bool>, Model<Option<TimerToken>>) {
    let hovered = cx.with_state_for(trigger_id, MenubarTriggerHoverSwitchState::default, |st| {
        st.hovered.clone()
    });
    let hovered = if let Some(hovered) = hovered {
        hovered
    } else {
        let hovered = cx.app.models_mut().insert(false);
        cx.with_state_for(trigger_id, MenubarTriggerHoverSwitchState::default, |st| {
            st.hovered = Some(hovered.clone());
        });
        hovered
    };

    let timer = cx.with_state_for(trigger_id, MenubarTriggerHoverSwitchState::default, |st| {
        st.timer.clone()
    });
    let timer = if let Some(timer) = timer {
        timer
    } else {
        let timer = cx.app.models_mut().insert(None);
        cx.with_state_for(trigger_id, MenubarTriggerHoverSwitchState::default, |st| {
            st.timer = Some(timer.clone());
        });
        timer
    };

    (hovered, timer)
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
    trigger: GlobalElementId,
    open: Model<bool>,
    enabled: bool,
) {
    let needs_update = cx
        .app
        .models()
        .read(&registry, |v| {
            match v.iter().find(|e| e.trigger == trigger) {
                Some(existing) => existing.open != open || existing.enabled != enabled,
                None => true,
            }
        })
        .unwrap_or(true);
    if !needs_update {
        return;
    }

    let _ = cx.app.models_mut().update(&registry, move |v| {
        if let Some(existing) = v.iter_mut().find(|e| e.trigger == trigger) {
            existing.open = open;
            existing.enabled = enabled;
            return;
        }

        v.push(MenubarTriggerRowEntry {
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
            let Some(active) = host.models_mut().get_cloned(&group_active).flatten() else {
                return false;
            };
            let Some(current_idx) = entries.iter().position(|e| e.trigger == active.trigger) else {
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

            let _ = host.models_mut().update(&active.open, |v| *v = false);
            let _ = host.models_mut().update(&next.open, |v| *v = true);
            let open_for_state = next.open.clone();
            let _ = host.models_mut().update(&group_active, |v| {
                *v = Some(MenubarActiveTrigger {
                    trigger: next.trigger,
                    open: open_for_state,
                });
            });

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
) {
    cx.key_add_on_key_down_for(
        item_id,
        switch_open_menu_on_horizontal_arrows(group_active, registry),
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
    _hovered: bool,
    pressed: bool,
    focused: bool,
) {
    let active_value = cx.watch_model(&group_active).cloned().flatten();
    let is_open = cx.watch_model(&open).copied().unwrap_or(false);

    let (hovered_model, hover_timer) = ensure_hover_switch_models(cx, trigger_id);

    let installed = cx.with_state_for(trigger_id, MenubarTriggerHoverSwitchState::default, |st| {
        st.installed
    });
    if !installed {
        cx.timer_add_on_timer_for(
            trigger_id,
            hover_switch_on_timer_handler(
                group_active.clone(),
                trigger_id,
                open.clone(),
                hovered_model.clone(),
                hover_timer.clone(),
            ),
        );

        cx.with_state_for(trigger_id, MenubarTriggerHoverSwitchState::default, |st| {
            st.installed = true;
        });
    }

    cx.pressable_add_on_hover_change(hover_switch_on_hover_change_handler(
        group_active.clone(),
        trigger_id,
        enabled,
        hovered_model.clone(),
        hover_timer.clone(),
    ));

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
        && focused
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::AppWindowId;
    use fret_ui::action::{ActionCx, UiActionHost, UiFocusActionHost};

    struct Host<'a> {
        app: &'a mut App,
    }

    impl UiActionHost for Host<'_> {
        fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
            self.app.models_mut()
        }

        fn push_effect(&mut self, effect: Effect) {
            self.app.push_effect(effect);
        }

        fn request_redraw(&mut self, window: AppWindowId) {
            self.app.request_redraw(window);
        }

        fn next_timer_token(&mut self) -> TimerToken {
            self.app.next_timer_token()
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            self.app.next_clipboard_token()
        }
    }

    impl UiFocusActionHost for Host<'_> {
        fn request_focus(&mut self, _target: GlobalElementId) {}
    }

    #[test]
    fn hover_switch_arms_timer_and_switches_on_fire() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut host = Host { app: &mut app };

        let trigger_a = GlobalElementId(1);
        let trigger_b = GlobalElementId(2);

        let open_a = host.models_mut().insert(true);
        let open_b = host.models_mut().insert(false);

        let group_active: Model<Option<MenubarActiveTrigger>> =
            host.models_mut().insert(Some(MenubarActiveTrigger {
                trigger: trigger_a,
                open: open_a.clone(),
            }));

        let hovered_b = host.models_mut().insert(false);
        let timer_b: Model<Option<TimerToken>> = host.models_mut().insert(None);

        let on_hover = hover_switch_on_hover_change_handler(
            group_active.clone(),
            trigger_b,
            true,
            hovered_b.clone(),
            timer_b.clone(),
        );
        on_hover(
            &mut host,
            ActionCx {
                window,
                target: trigger_b,
            },
            true,
        );

        let armed = host.models_mut().read(&timer_b, |v| *v).ok().flatten();
        assert!(armed.is_some());

        let on_timer = hover_switch_on_timer_handler(
            group_active.clone(),
            trigger_b,
            open_b.clone(),
            hovered_b.clone(),
            timer_b.clone(),
        );
        on_timer(
            &mut host,
            ActionCx {
                window,
                target: trigger_b,
            },
            armed.unwrap(),
        );

        let a_open = host
            .models_mut()
            .read(&open_a, |v| *v)
            .ok()
            .unwrap_or(false);
        let b_open = host
            .models_mut()
            .read(&open_b, |v| *v)
            .ok()
            .unwrap_or(false);
        assert!(!a_open);
        assert!(b_open);

        let active = host.models_mut().get_cloned(&group_active).flatten();
        assert!(active.is_some_and(|v| v.trigger == trigger_b));
    }

    #[test]
    fn hover_switch_does_not_switch_when_hover_clears_before_timer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut host = Host { app: &mut app };

        let trigger_a = GlobalElementId(1);
        let trigger_b = GlobalElementId(2);

        let open_a = host.models_mut().insert(true);
        let open_b = host.models_mut().insert(false);

        let group_active: Model<Option<MenubarActiveTrigger>> =
            host.models_mut().insert(Some(MenubarActiveTrigger {
                trigger: trigger_a,
                open: open_a.clone(),
            }));

        let hovered_b = host.models_mut().insert(false);
        let timer_b: Model<Option<TimerToken>> = host.models_mut().insert(None);

        let on_hover = hover_switch_on_hover_change_handler(
            group_active.clone(),
            trigger_b,
            true,
            hovered_b.clone(),
            timer_b.clone(),
        );
        on_hover(
            &mut host,
            ActionCx {
                window,
                target: trigger_b,
            },
            true,
        );
        let armed = host.models_mut().read(&timer_b, |v| *v).ok().flatten();
        assert!(armed.is_some());

        on_hover(
            &mut host,
            ActionCx {
                window,
                target: trigger_b,
            },
            false,
        );
        let still_hovered = host
            .models_mut()
            .read(&hovered_b, |v| *v)
            .ok()
            .unwrap_or(true);
        assert!(!still_hovered);

        let on_timer = hover_switch_on_timer_handler(
            group_active.clone(),
            trigger_b,
            open_b.clone(),
            hovered_b.clone(),
            timer_b.clone(),
        );
        on_timer(
            &mut host,
            ActionCx {
                window,
                target: trigger_b,
            },
            armed.unwrap(),
        );

        let a_open = host
            .models_mut()
            .read(&open_a, |v| *v)
            .ok()
            .unwrap_or(false);
        let b_open = host
            .models_mut()
            .read(&open_b, |v| *v)
            .ok()
            .unwrap_or(false);
        assert!(a_open);
        assert!(!b_open);
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
