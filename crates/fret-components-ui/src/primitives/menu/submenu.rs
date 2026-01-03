//! Menu submenu policy helpers (Radix-aligned outcomes).
//!
//! In Radix, `DropdownMenu`, `ContextMenu`, and `Menubar` are wrappers around the lower-level
//! `Menu` primitive (`@radix-ui/react-menu`):
//! <https://github.com/radix-ui/primitives/tree/main/packages/react/menu>
//!
//! A key behavior baked into Radix Menu is submenu ergonomics:
//! - pointer grace intent while moving towards submenu content
//! - delayed close timers
//! - keyboard focus transfer into the submenu (and restore to the trigger on close)
//!
//! This module provides those outcomes as reusable policy helpers for Fret wrappers.

use std::sync::Arc;
use std::time::Duration;

use fret_core::{Point, Px, Rect};
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::action::{ActionCx, KeyDownCx, PointerMoveCx, UiActionHost, UiFocusActionHost};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::primitives::menu::pointer_grace_intent;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuSubmenuGeometry {
    pub reference: Rect,
    pub floating: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuSubmenuConfig {
    pub safe_hover_buffer: Px,
    pub close_delay: Duration,
    pub focus_delay: Duration,
}

impl MenuSubmenuConfig {
    pub fn new(safe_hover_buffer: Px, close_delay: Duration, focus_delay: Duration) -> Self {
        Self {
            safe_hover_buffer,
            close_delay,
            focus_delay,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MenuSubmenuModels {
    pub open_value: Model<Option<Arc<str>>>,
    pub trigger: Model<Option<GlobalElementId>>,
    pub last_pointer: Model<Option<Point>>,
    pub geometry: Model<Option<MenuSubmenuGeometry>>,
    pub close_timer: Model<Option<TimerToken>>,
    pub focus_target: Model<Option<GlobalElementId>>,
    pub focus_timer: Model<Option<TimerToken>>,
}

#[derive(Default)]
struct MenuSubmenuState {
    open_value: Option<Model<Option<Arc<str>>>>,
    trigger: Option<Model<Option<GlobalElementId>>>,
    last_pointer: Option<Model<Option<Point>>>,
    geometry: Option<Model<Option<MenuSubmenuGeometry>>>,
    close_timer: Option<Model<Option<TimerToken>>>,
    focus_target: Option<Model<Option<GlobalElementId>>>,
    focus_timer: Option<Model<Option<TimerToken>>>,
    was_open: bool,
}

fn cancel_timer(host: &mut dyn UiActionHost, timer: &Model<Option<TimerToken>>) {
    let token = host.models_mut().read(timer, |v| *v).ok().flatten();
    if let Some(token) = token {
        host.push_effect(Effect::CancelTimer { token });
    }
    let _ = host.models_mut().update(timer, |v| *v = None);
}

fn cancel_timer_in_element_context<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    timer: &Model<Option<TimerToken>>,
) {
    let token = cx.app.models_mut().read(timer, |v| *v).ok().flatten();
    if let Some(token) = token {
        cx.app.push_effect(Effect::CancelTimer { token });
    }
    let _ = cx.app.models_mut().update(timer, |v| *v = None);
}

fn cancel_timer_if_matches(
    host: &mut dyn UiActionHost,
    timer: &Model<Option<TimerToken>>,
    token: TimerToken,
) {
    let armed = host.models_mut().read(timer, |v| *v).ok().flatten();
    if armed != Some(token) {
        return;
    }
    let _ = host.models_mut().update(timer, |v| *v = None);
}

pub fn cancel_close_timer(host: &mut dyn UiActionHost, close_timer: &Model<Option<TimerToken>>) {
    cancel_timer(host, close_timer);
}

pub fn cancel_focus_timer(host: &mut dyn UiActionHost, focus_timer: &Model<Option<TimerToken>>) {
    cancel_timer(host, focus_timer);
}

pub fn sync_root_open<H: UiHost>(cx: &mut ElementContext<'_, H>, is_open: bool) {
    let was_open = cx.with_state(MenuSubmenuState::default, |st| st.was_open);
    if is_open && !was_open {
        if let Some(model) = cx.with_state(MenuSubmenuState::default, |st| st.open_value.clone()) {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) = cx.with_state(MenuSubmenuState::default, |st| st.trigger.clone()) {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) = cx.with_state(MenuSubmenuState::default, |st| st.last_pointer.clone())
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) = cx.with_state(MenuSubmenuState::default, |st| st.geometry.clone()) {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) = cx.with_state(MenuSubmenuState::default, |st| st.close_timer.clone()) {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) = cx.with_state(MenuSubmenuState::default, |st| st.focus_timer.clone()) {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) = cx.with_state(MenuSubmenuState::default, |st| st.focus_target.clone())
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        cx.with_state(MenuSubmenuState::default, |st| st.was_open = true);
    } else if !is_open && was_open {
        if let Some(model) = cx.with_state(MenuSubmenuState::default, |st| st.close_timer.clone()) {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) = cx.with_state(MenuSubmenuState::default, |st| st.focus_timer.clone()) {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        cx.with_state(MenuSubmenuState::default, |st| st.was_open = false);
    }
}

pub fn ensure_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> MenuSubmenuModels {
    let open_value = cx.with_state(MenuSubmenuState::default, |st| st.open_value.clone());
    let open_value = if let Some(open_value) = open_value {
        open_value
    } else {
        let open_value = cx.app.models_mut().insert(None);
        cx.with_state(MenuSubmenuState::default, |st| {
            st.open_value = Some(open_value.clone());
        });
        open_value
    };

    let trigger = cx.with_state(MenuSubmenuState::default, |st| st.trigger.clone());
    let trigger = if let Some(trigger) = trigger {
        trigger
    } else {
        let trigger = cx.app.models_mut().insert(None);
        cx.with_state(MenuSubmenuState::default, |st| {
            st.trigger = Some(trigger.clone());
        });
        trigger
    };

    let last_pointer = cx.with_state(MenuSubmenuState::default, |st| st.last_pointer.clone());
    let last_pointer = if let Some(last_pointer) = last_pointer {
        last_pointer
    } else {
        let last_pointer = cx.app.models_mut().insert(None);
        cx.with_state(MenuSubmenuState::default, |st| {
            st.last_pointer = Some(last_pointer.clone());
        });
        last_pointer
    };

    let geometry = cx.with_state(MenuSubmenuState::default, |st| st.geometry.clone());
    let geometry = if let Some(geometry) = geometry {
        geometry
    } else {
        let geometry = cx.app.models_mut().insert(None);
        cx.with_state(MenuSubmenuState::default, |st| {
            st.geometry = Some(geometry.clone());
        });
        geometry
    };

    let close_timer = cx.with_state(MenuSubmenuState::default, |st| st.close_timer.clone());
    let close_timer = if let Some(close_timer) = close_timer {
        close_timer
    } else {
        let close_timer = cx.app.models_mut().insert(None);
        cx.with_state(MenuSubmenuState::default, |st| {
            st.close_timer = Some(close_timer.clone());
        });
        close_timer
    };

    let focus_target = cx.with_state(MenuSubmenuState::default, |st| st.focus_target.clone());
    let focus_target = if let Some(focus_target) = focus_target {
        focus_target
    } else {
        let focus_target = cx.app.models_mut().insert(None);
        cx.with_state(MenuSubmenuState::default, |st| {
            st.focus_target = Some(focus_target.clone());
        });
        focus_target
    };

    let focus_timer = cx.with_state(MenuSubmenuState::default, |st| st.focus_timer.clone());
    let focus_timer = if let Some(focus_timer) = focus_timer {
        focus_timer
    } else {
        let focus_timer = cx.app.models_mut().insert(None);
        cx.with_state(MenuSubmenuState::default, |st| {
            st.focus_timer = Some(focus_timer.clone());
        });
        focus_timer
    };

    MenuSubmenuModels {
        open_value,
        trigger,
        last_pointer,
        geometry,
        close_timer,
        focus_target,
        focus_timer,
    }
}

pub fn on_timer_handler(models: MenuSubmenuModels) -> fret_ui::action::OnTimer {
    Arc::new(move |host, acx, token| {
        let close_armed = host
            .models_mut()
            .read(&models.close_timer, |v| *v)
            .ok()
            .flatten();
        let focus_armed = host
            .models_mut()
            .read(&models.focus_timer, |v| *v)
            .ok()
            .flatten();

        if close_armed == Some(token) {
            cancel_timer(host, &models.focus_timer);
            let _ = host.models_mut().update(&models.open_value, |v| *v = None);
            let _ = host.models_mut().update(&models.trigger, |v| *v = None);
            let _ = host
                .models_mut()
                .update(&models.last_pointer, |v| *v = None);
            let _ = host.models_mut().update(&models.geometry, |v| *v = None);
            cancel_timer_if_matches(host, &models.close_timer, token);
            host.request_redraw(acx.window);
            return true;
        }

        if focus_armed == Some(token) {
            let target = host
                .models_mut()
                .read(&models.focus_target, |v| *v)
                .ok()
                .flatten();
            if let Some(target) = target {
                host.request_focus(target);
            }
            cancel_timer_if_matches(host, &models.focus_timer, token);
            host.request_redraw(acx.window);
            return true;
        }

        false
    })
}

pub fn install_timer_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    element: GlobalElementId,
    models: MenuSubmenuModels,
) {
    cx.timer_on_timer_for(element, on_timer_handler(models));
}

pub fn handle_dismissible_pointer_move(
    host: &mut dyn UiActionHost,
    acx: ActionCx,
    mv: PointerMoveCx,
    models: &MenuSubmenuModels,
    cfg: MenuSubmenuConfig,
) -> bool {
    let geometry = host
        .models_mut()
        .read(&models.geometry, |v| *v)
        .ok()
        .flatten();
    let grace = geometry.map(|g| pointer_grace_intent::PointerGraceIntentGeometry {
        reference: g.reference,
        floating: g.floating,
    });
    pointer_grace_intent::drive_close_timer_on_pointer_move(
        host,
        acx,
        mv,
        grace,
        pointer_grace_intent::PointerGraceIntentConfig::new(cfg.safe_hover_buffer, cfg.close_delay),
        &models.last_pointer,
        &models.close_timer,
    )
}

pub fn set_geometry_if_changed<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    geometry: MenuSubmenuGeometry,
    geometry_model: &Model<Option<MenuSubmenuGeometry>>,
) {
    let _ = cx.app.models_mut().update(geometry_model, |v| {
        if v.as_ref() == Some(&geometry) {
            return;
        }
        *v = Some(geometry);
    });
}

pub fn set_trigger_if_none<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    trigger_model: &Model<Option<GlobalElementId>>,
) {
    let _ = cx.app.models_mut().update(trigger_model, |v| {
        if v.is_none() {
            *v = Some(trigger_id);
        }
    });
}

pub fn clear_focus_target<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    focus_target: &Model<Option<GlobalElementId>>,
) {
    let _ = cx.app.models_mut().update(focus_target, |v| *v = None);
}

pub fn set_focus_target_if_none<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    focus_target: &Model<Option<GlobalElementId>>,
    target: GlobalElementId,
) -> bool {
    let mut did_set = false;
    let _ = cx.app.models_mut().update(focus_target, |v| {
        if v.is_none() {
            *v = Some(target);
            did_set = true;
        }
    });
    did_set
}

pub fn sync_while_trigger_hovered<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
    has_submenu: bool,
    value: Arc<str>,
    item_id: GlobalElementId,
) {
    cancel_timer_in_element_context(cx, &models.close_timer);

    if has_submenu {
        let already_open = cx
            .app
            .models_mut()
            .read(&models.open_value, |v| {
                v.as_ref().is_some_and(|cur| cur.as_ref() == value.as_ref())
            })
            .ok()
            .unwrap_or(false);

        if !already_open {
            let _ = cx
                .app
                .models_mut()
                .update(&models.open_value, |v| *v = Some(value));
            let _ = cx
                .app
                .models_mut()
                .update(&models.trigger, |v| *v = Some(item_id));
        } else {
            set_trigger_if_none(cx, item_id, &models.trigger);
        }
    } else {
        let _ = cx
            .app
            .models_mut()
            .update(&models.open_value, |v| *v = None);
        let _ = cx.app.models_mut().update(&models.trigger, |v| *v = None);
        let _ = cx.app.models_mut().update(&models.geometry, |v| *v = None);
    }
}

pub fn close_if_focus_moved_without_pointer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
    focused_value: &Arc<str>,
    focused_item_id: GlobalElementId,
) {
    let no_pointer = cx
        .app
        .models_mut()
        .read(&models.last_pointer, |v| v.is_none())
        .ok()
        .unwrap_or(true);
    if !no_pointer {
        return;
    }

    let open_value = cx
        .app
        .models_mut()
        .read(&models.open_value, |v| v.clone())
        .ok()
        .flatten();
    let open_trigger = cx
        .app
        .models_mut()
        .read(&models.trigger, |v| *v)
        .ok()
        .flatten();
    let is_open_here = open_value
        .as_ref()
        .is_some_and(|cur| cur.as_ref() == focused_value.as_ref())
        && open_trigger == Some(focused_item_id);

    if is_open_here {
        return;
    }

    let _ = cx
        .app
        .models_mut()
        .update(&models.open_value, |v| *v = None);
    let _ = cx.app.models_mut().update(&models.trigger, |v| *v = None);
    let _ = cx.app.models_mut().update(&models.geometry, |v| *v = None);
    cancel_timer_in_element_context(cx, &models.close_timer);
    cancel_timer_in_element_context(cx, &models.focus_timer);
}

pub fn open_on_hover(
    host: &mut dyn UiActionHost,
    acx: ActionCx,
    models: &MenuSubmenuModels,
    has_submenu: bool,
    value: Arc<str>,
) {
    cancel_timer(host, &models.close_timer);
    let _ = host.models_mut().update(&models.geometry, |v| *v = None);
    let _ = host
        .models_mut()
        .update(&models.last_pointer, |v| *v = None);

    if has_submenu {
        let _ = host
            .models_mut()
            .update(&models.open_value, |v| *v = Some(value));
        let _ = host
            .models_mut()
            .update(&models.trigger, |v| *v = Some(acx.target));
    } else {
        let _ = host.models_mut().update(&models.open_value, |v| *v = None);
        let _ = host.models_mut().update(&models.trigger, |v| *v = None);
    }

    host.request_redraw(acx.window);
}

pub fn open_on_activate(
    host: &mut dyn UiActionHost,
    acx: ActionCx,
    models: &MenuSubmenuModels,
    value: Arc<str>,
) {
    cancel_timer(host, &models.close_timer);
    let _ = host
        .models_mut()
        .update(&models.open_value, |v| *v = Some(value));
    let _ = host
        .models_mut()
        .update(&models.trigger, |v| *v = Some(acx.target));
    host.request_redraw(acx.window);
}

pub fn open_on_arrow_right(
    host: &mut dyn UiActionHost,
    acx: ActionCx,
    models: &MenuSubmenuModels,
    value: Arc<str>,
    focus_delay: Duration,
) {
    cancel_timer(host, &models.focus_timer);
    cancel_timer(host, &models.close_timer);

    let _ = host
        .models_mut()
        .update(&models.open_value, |v| *v = Some(value));
    let _ = host
        .models_mut()
        .update(&models.trigger, |v| *v = Some(acx.target));

    let token = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window: Some(acx.window),
        token,
        after: focus_delay,
        repeat: None,
    });
    let _ = host
        .models_mut()
        .update(&models.focus_timer, |v| *v = Some(token));
    host.request_redraw(acx.window);
}

pub fn close_on_arrow_left(host: &mut dyn UiActionHost, acx: ActionCx, models: &MenuSubmenuModels) {
    let _ = host.models_mut().update(&models.open_value, |v| *v = None);
    let _ = host.models_mut().update(&models.trigger, |v| *v = None);
    let _ = host.models_mut().update(&models.geometry, |v| *v = None);
    cancel_timer(host, &models.close_timer);
    cancel_timer(host, &models.focus_timer);
    host.request_redraw(acx.window);
}

pub fn close_and_restore_trigger(
    host: &mut dyn UiFocusActionHost,
    acx: ActionCx,
    models: &MenuSubmenuModels,
) {
    let trigger = host
        .models_mut()
        .read(&models.trigger, |v| *v)
        .ok()
        .flatten();
    close_on_arrow_left(host, acx, models);
    if let Some(trigger) = trigger {
        host.request_focus(trigger);
    }
}

pub fn submenu_item_arrow_left_handler(
    models: MenuSubmenuModels,
) -> Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, KeyDownCx) -> bool + 'static> {
    Arc::new(move |host, acx, down| {
        if down.repeat || down.key != fret_core::KeyCode::ArrowLeft {
            return false;
        }
        close_and_restore_trigger(host, acx, &models);
        true
    })
}

pub fn focus_first_available_on_open<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
    item_id: GlobalElementId,
    disabled: bool,
) {
    if disabled {
        return;
    }
    let _ = set_focus_target_if_none(cx, &models.focus_target, item_id);
}
