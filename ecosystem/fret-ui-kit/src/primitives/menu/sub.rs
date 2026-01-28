//! Menu Sub (submenu) policy helpers (Radix-aligned outcomes).
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

use fret_core::{Point, Px, Rect, Size};
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::action::{ActionCx, PointerMoveCx, UiActionHost, UiFocusActionHost};
use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::overlay;
use crate::primitives::direction::LayoutDirection;
use crate::primitives::menu::pointer_grace_intent;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuSubmenuGeometry {
    pub reference: Rect,
    pub floating: Rect,
}

/// Radix Menu clears `pointerGraceIntentRef` after 300ms.
pub const DEFAULT_POINTER_GRACE_TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuSubmenuConfig {
    pub safe_hover_buffer: Px,
    /// Delay before opening a submenu on pointer hover.
    ///
    /// Radix Menu uses a small delay (~100ms) to avoid accidental opens while sweeping across items.
    pub open_delay: Duration,
    pub close_delay: Duration,
    pub focus_delay: Duration,
    /// How long a submenu "pointer grace" corridor stays armed after the pointer exits the trigger.
    pub pointer_grace_timeout: Duration,
}

impl MenuSubmenuConfig {
    pub fn new(
        safe_hover_buffer: Px,
        open_delay: Duration,
        close_delay: Duration,
        focus_delay: Duration,
    ) -> Self {
        Self {
            safe_hover_buffer,
            open_delay,
            close_delay,
            focus_delay,
            pointer_grace_timeout: DEFAULT_POINTER_GRACE_TIMEOUT,
        }
    }

    pub fn pointer_grace_timeout(mut self, timeout: Duration) -> Self {
        self.pointer_grace_timeout = timeout;
        self
    }
}

impl Default for MenuSubmenuConfig {
    fn default() -> Self {
        Self {
            safe_hover_buffer: Px(6.0),
            open_delay: Duration::from_millis(100),
            close_delay: Duration::from_millis(120),
            focus_delay: Duration::from_millis(0),
            pointer_grace_timeout: DEFAULT_POINTER_GRACE_TIMEOUT,
        }
    }
}

/// Return the default submenu floating bounds (Radix Menu-like: side=Right, align=Start, offset=2px).
pub fn default_submenu_bounds(outer: Rect, trigger_anchor: Rect, desired: Size) -> Rect {
    // Radix `MenuSubContent` hard-codes `align="start"` and relies on Popper's collision logic to
    // flip the alignment when the submenu would overflow the viewport on the cross axis.
    //
    // We approximate that behavior here by flipping to `Align::End` when the desired height does
    // not fit below the trigger (LTR: side=Right).
    let desired_h = desired.height.0.max(0.0);
    let outer_bottom = outer.origin.y.0 + outer.size.height.0.max(0.0);
    let trigger_top = trigger_anchor.origin.y.0;
    let align = if trigger_top + desired_h > outer_bottom {
        Align::End
    } else {
        Align::Start
    };
    anchored_panel_bounds_sized(outer, trigger_anchor, desired, Px(2.0), Side::Right, align)
}

/// Estimate a scrollable menu panel viewport height for `row_count` rows.
///
/// This is primarily used by submenu wrappers to approximate the `desired` size passed into the
/// placement solver: Radix Menu uses the content's measured height but clamps it by the available
/// space (and any theme cap) so flip decisions remain stable while the internal list scrolls.
pub fn estimated_panel_height_for_row_count(
    row_height: Px,
    row_count: usize,
    max_height: Px,
) -> Px {
    let rows = row_count.max(1) as f32;
    let min_h = row_height.0.max(0.0);
    let max_h = max_height.0.max(min_h);
    Px((row_height.0 * rows).clamp(min_h, max_h))
}

/// Return an estimated desired size for a scrollable menu/submenu list.
pub fn estimated_desired_size_for_row_count(
    desired_width: Px,
    row_height: Px,
    row_count: usize,
    max_height: Px,
) -> Size {
    Size::new(
        Px(desired_width.0.max(0.0)),
        estimated_panel_height_for_row_count(row_height, row_count, max_height),
    )
}

pub fn clear_focus_target_in_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
) {
    clear_focus_target(cx, &models.focus_target);
}

/// Synchronize submenu geometry from the currently-registered trigger element anchor.
///
/// When a submenu is already open, its trigger can continue to move due to layout/scroll. Updating
/// geometry ahead of rendering keeps pointer-grace intent and safe-corridor heuristics stable.
pub fn sync_open_geometry_from_trigger_if_present<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
    outer: Rect,
    desired: Size,
) {
    let trigger = cx
        .app
        .models_mut()
        .read(&models.trigger, |v| *v)
        .ok()
        .flatten();
    if let Some(trigger) = trigger {
        set_geometry_from_element_anchor_if_present(cx, trigger, models, outer, desired);
    }
}

pub fn with_open_submenu<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
    outer: Rect,
    desired: Size,
    f: impl FnOnce(&mut ElementContext<'_, H>, Arc<str>, MenuSubmenuGeometry) -> R,
) -> Option<R> {
    let open_value = cx
        .app
        .models_mut()
        .read(&models.open_value, |v| v.clone())
        .ok()
        .flatten()?;

    clear_focus_target_in_models(cx, models);

    let geometry = resolve_open_geometry(cx, models, outer, desired)?;
    Some(f(cx, open_value, geometry))
}

/// Like [`with_open_submenu`], but eagerly syncs submenu geometry from the current trigger anchor
/// before resolving the geometry model.
pub fn with_open_submenu_synced<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
    outer: Rect,
    desired: Size,
    f: impl FnOnce(&mut ElementContext<'_, H>, Arc<str>, MenuSubmenuGeometry) -> R,
) -> Option<R> {
    let open_value = cx
        .app
        .models_mut()
        .read(&models.open_value, |v| v.clone())
        .ok()
        .flatten()?;

    clear_focus_target_in_models(cx, models);
    sync_open_geometry_from_trigger_if_present(cx, models, outer, desired);

    let geometry = resolve_open_geometry(cx, models, outer, desired)?;
    Some(f(cx, open_value, geometry))
}

pub fn resolve_open_geometry<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    models: &MenuSubmenuModels,
    outer: Rect,
    desired: Size,
) -> Option<MenuSubmenuGeometry> {
    let geometry = cx
        .app
        .models_mut()
        .read(&models.geometry, |v| *v)
        .ok()
        .flatten();
    if let Some(geometry) = geometry {
        return Some(geometry);
    }

    let trigger = cx
        .app
        .models_mut()
        .read(&models.trigger, |v| *v)
        .ok()
        .flatten()?;
    let trigger_anchor = overlay::anchor_bounds_for_element(cx, trigger)?;
    let placed = default_submenu_bounds(outer, trigger_anchor, desired);
    let geometry = MenuSubmenuGeometry {
        reference: trigger_anchor,
        floating: placed,
    };
    set_geometry_if_changed(cx, geometry, &models.geometry);
    Some(geometry)
}

/// Update submenu geometry from a specific trigger element's current anchor bounds.
///
/// This is typically called from within a `MenuSubTrigger` pressable closure when the submenu is
/// already open, so pointer-grace intent has up-to-date geometry even before the submenu panel is
/// rendered/measured.
pub fn set_geometry_from_element_anchor_if_present<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    element: GlobalElementId,
    models: &MenuSubmenuModels,
    outer: Rect,
    desired: Size,
) {
    let Some(anchor) = overlay::anchor_bounds_for_element(cx, element) else {
        return;
    };

    let floating = default_submenu_bounds(outer, anchor, desired);
    let geometry = MenuSubmenuGeometry {
        reference: anchor,
        floating,
    };
    set_geometry_if_changed(cx, geometry, &models.geometry);
}

#[derive(Debug, Clone)]
pub struct MenuSubmenuModels {
    pub open_value: Model<Option<Arc<str>>>,
    pub trigger: Model<Option<GlobalElementId>>,
    pub last_pointer: Model<Option<Point>>,
    pub geometry: Model<Option<MenuSubmenuGeometry>>,
    pub close_timer: Model<Option<TimerToken>>,
    pub pointer_dir: Model<Option<pointer_grace_intent::GraceSide>>,
    pub pointer_grace_intent: Model<Option<pointer_grace_intent::GraceIntent>>,
    pub pointer_grace_timer: Model<Option<TimerToken>>,
    pub focus_target: Model<Option<GlobalElementId>>,
    pub focus_timer: Model<Option<TimerToken>>,
    pub pending_open_value: Model<Option<Arc<str>>>,
    pub pending_open_trigger: Model<Option<GlobalElementId>>,
    pub open_timer: Model<Option<TimerToken>>,
}

#[derive(Default)]
struct MenuSubmenuState {
    open_value: Option<Model<Option<Arc<str>>>>,
    trigger: Option<Model<Option<GlobalElementId>>>,
    last_pointer: Option<Model<Option<Point>>>,
    geometry: Option<Model<Option<MenuSubmenuGeometry>>>,
    close_timer: Option<Model<Option<TimerToken>>>,
    pointer_dir: Option<Model<Option<pointer_grace_intent::GraceSide>>>,
    pointer_grace_intent: Option<Model<Option<pointer_grace_intent::GraceIntent>>>,
    pointer_grace_timer: Option<Model<Option<TimerToken>>>,
    focus_target: Option<Model<Option<GlobalElementId>>>,
    focus_timer: Option<Model<Option<TimerToken>>>,
    pending_open_value: Option<Model<Option<Arc<str>>>>,
    pending_open_trigger: Option<Model<Option<GlobalElementId>>>,
    open_timer: Option<Model<Option<TimerToken>>>,
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

pub fn sync_root_open_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    timer_handler_element: GlobalElementId,
    is_open: bool,
) {
    let was_open = cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
        st.was_open
    });
    if is_open && !was_open {
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.open_value.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.trigger.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.last_pointer.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.geometry.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.close_timer.clone()
            })
        {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.focus_timer.clone()
            })
        {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.open_timer.clone()
            })
        {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.pointer_dir.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.pointer_grace_intent.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.pointer_grace_timer.clone()
            })
        {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.pending_open_value.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.pending_open_trigger.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.focus_target.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.was_open = true
        });
    } else if !is_open && was_open {
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.close_timer.clone()
            })
        {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.focus_timer.clone()
            })
        {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.open_timer.clone()
            })
        {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.pointer_dir.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.pointer_grace_intent.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.pointer_grace_timer.clone()
            })
        {
            let token = cx.app.models_mut().read(&model, |v| *v).ok().flatten();
            if let Some(token) = token {
                cx.app.push_effect(Effect::CancelTimer { token });
            }
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.pending_open_value.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        if let Some(model) =
            cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
                st.pending_open_trigger.clone()
            })
        {
            let _ = cx.app.models_mut().update(&model, |v| *v = None);
        }
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.was_open = false
        });
    }
}

pub fn sync_root_open<H: UiHost>(cx: &mut ElementContext<'_, H>, is_open: bool) {
    sync_root_open_for(cx, cx.root_id(), is_open);
}

pub fn ensure_models_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    timer_handler_element: GlobalElementId,
) -> MenuSubmenuModels {
    let open_value = cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
        st.open_value.clone()
    });
    let open_value = if let Some(open_value) = open_value {
        open_value
    } else {
        let open_value = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.open_value = Some(open_value.clone());
        });
        open_value
    };

    let trigger = cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
        st.trigger.clone()
    });
    let trigger = if let Some(trigger) = trigger {
        trigger
    } else {
        let trigger = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.trigger = Some(trigger.clone());
        });
        trigger
    };

    let last_pointer = cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
        st.last_pointer.clone()
    });
    let last_pointer = if let Some(last_pointer) = last_pointer {
        last_pointer
    } else {
        let last_pointer = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.last_pointer = Some(last_pointer.clone());
        });
        last_pointer
    };

    let geometry = cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
        st.geometry.clone()
    });
    let geometry = if let Some(geometry) = geometry {
        geometry
    } else {
        let geometry = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.geometry = Some(geometry.clone());
        });
        geometry
    };

    let close_timer = cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
        st.close_timer.clone()
    });
    let close_timer = if let Some(close_timer) = close_timer {
        close_timer
    } else {
        let close_timer = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.close_timer = Some(close_timer.clone());
        });
        close_timer
    };

    let pointer_dir = cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
        st.pointer_dir.clone()
    });
    let pointer_dir = if let Some(pointer_dir) = pointer_dir {
        pointer_dir
    } else {
        let pointer_dir = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.pointer_dir = Some(pointer_dir.clone());
        });
        pointer_dir
    };

    let pointer_grace_intent =
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.pointer_grace_intent.clone()
        });
    let pointer_grace_intent = if let Some(pointer_grace_intent) = pointer_grace_intent {
        pointer_grace_intent
    } else {
        let pointer_grace_intent = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.pointer_grace_intent = Some(pointer_grace_intent.clone());
        });
        pointer_grace_intent
    };

    let pointer_grace_timer =
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.pointer_grace_timer.clone()
        });
    let pointer_grace_timer = if let Some(pointer_grace_timer) = pointer_grace_timer {
        pointer_grace_timer
    } else {
        let pointer_grace_timer = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.pointer_grace_timer = Some(pointer_grace_timer.clone());
        });
        pointer_grace_timer
    };

    let focus_target = cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
        st.focus_target.clone()
    });
    let focus_target = if let Some(focus_target) = focus_target {
        focus_target
    } else {
        let focus_target = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.focus_target = Some(focus_target.clone());
        });
        focus_target
    };

    let focus_timer = cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
        st.focus_timer.clone()
    });
    let focus_timer = if let Some(focus_timer) = focus_timer {
        focus_timer
    } else {
        let focus_timer = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.focus_timer = Some(focus_timer.clone());
        });
        focus_timer
    };

    let pending_open_value =
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.pending_open_value.clone()
        });
    let pending_open_value = if let Some(pending_open_value) = pending_open_value {
        pending_open_value
    } else {
        let pending_open_value = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.pending_open_value = Some(pending_open_value.clone());
        });
        pending_open_value
    };

    let pending_open_trigger =
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.pending_open_trigger.clone()
        });
    let pending_open_trigger = if let Some(pending_open_trigger) = pending_open_trigger {
        pending_open_trigger
    } else {
        let pending_open_trigger = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.pending_open_trigger = Some(pending_open_trigger.clone());
        });
        pending_open_trigger
    };

    let open_timer = cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
        st.open_timer.clone()
    });
    let open_timer = if let Some(open_timer) = open_timer {
        open_timer
    } else {
        let open_timer = cx.app.models_mut().insert(None);
        cx.with_state_for(timer_handler_element, MenuSubmenuState::default, |st| {
            st.open_timer = Some(open_timer.clone());
        });
        open_timer
    };

    MenuSubmenuModels {
        open_value,
        trigger,
        last_pointer,
        geometry,
        close_timer,
        pointer_dir,
        pointer_grace_intent,
        pointer_grace_timer,
        focus_target,
        focus_timer,
        pending_open_value,
        pending_open_trigger,
        open_timer,
    }
}

pub fn ensure_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> MenuSubmenuModels {
    ensure_models_for(cx, cx.root_id())
}

pub fn on_timer_handler(
    models: MenuSubmenuModels,
    cfg: MenuSubmenuConfig,
) -> fret_ui::action::OnTimer {
    #[allow(clippy::arc_with_non_send_sync)]
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
        let open_armed = host
            .models_mut()
            .read(&models.open_timer, |v| *v)
            .ok()
            .flatten();
        let pointer_grace_armed = host
            .models_mut()
            .read(&models.pointer_grace_timer, |v| *v)
            .ok()
            .flatten();

        if close_armed == Some(token) {
            cancel_timer(host, &models.open_timer);
            cancel_timer(host, &models.focus_timer);
            cancel_timer(host, &models.pointer_grace_timer);
            let _ = host.models_mut().update(&models.open_value, |v| *v = None);
            let _ = host.models_mut().update(&models.trigger, |v| *v = None);
            let _ = host
                .models_mut()
                .update(&models.last_pointer, |v| *v = None);
            let _ = host.models_mut().update(&models.pointer_dir, |v| *v = None);
            let _ = host
                .models_mut()
                .update(&models.pointer_grace_intent, |v| *v = None);
            let _ = host.models_mut().update(&models.geometry, |v| *v = None);
            let _ = host
                .models_mut()
                .update(&models.pending_open_value, |v| *v = None);
            let _ = host
                .models_mut()
                .update(&models.pending_open_trigger, |v| *v = None);
            cancel_timer_if_matches(host, &models.close_timer, token);
            host.request_redraw(acx.window);
            return true;
        }

        if pointer_grace_armed == Some(token) {
            cancel_timer_if_matches(host, &models.pointer_grace_timer, token);
            let _ = host
                .models_mut()
                .update(&models.pointer_grace_intent, |v| *v = None);
            host.request_redraw(acx.window);
            return true;
        }

        if open_armed == Some(token) {
            let pending_value = host
                .models_mut()
                .read(&models.pending_open_value, |v| v.clone())
                .ok()
                .flatten();
            let pending_trigger = host
                .models_mut()
                .read(&models.pending_open_trigger, |v| *v)
                .ok()
                .flatten();

            cancel_timer_if_matches(host, &models.open_timer, token);

            let Some(pending_value) = pending_value else {
                return false;
            };

            let open_value = host
                .models_mut()
                .read(&models.open_value, |v| v.clone())
                .ok()
                .flatten();
            let pointer = host
                .models_mut()
                .read(&models.last_pointer, |v| *v)
                .ok()
                .flatten();
            let pointer_dir = host
                .models_mut()
                .read(&models.pointer_dir, |v| *v)
                .ok()
                .flatten();
            let grace_intent = host
                .models_mut()
                .read(&models.pointer_grace_intent, |v| *v)
                .ok()
                .flatten();

            let switching_away = open_value
                .as_ref()
                .is_some_and(|cur| cur.as_ref() != pending_value.as_ref());

            let moving_towards = grace_intent
                .as_ref()
                .is_some_and(|intent| pointer_dir == Some(intent.side));
            let in_grace_area = match (pointer, grace_intent) {
                (Some(pointer), Some(intent)) => {
                    pointer_grace_intent::is_pointer_in_grace_area(pointer, intent)
                }
                _ => false,
            };
            if switching_away && moving_towards && in_grace_area {
                let token = host.next_timer_token();
                host.push_effect(Effect::SetTimer {
                    window: Some(acx.window),
                    token,
                    after: cfg.open_delay,
                    repeat: None,
                });
                let _ = host
                    .models_mut()
                    .update(&models.open_timer, |v| *v = Some(token));
                host.request_redraw(acx.window);
                return true;
            }

            let _ = host
                .models_mut()
                .update(&models.pending_open_value, |v| *v = None);
            let _ = host
                .models_mut()
                .update(&models.pending_open_trigger, |v| *v = None);
            cancel_timer(host, &models.pointer_grace_timer);
            let _ = host
                .models_mut()
                .update(&models.pointer_grace_intent, |v| *v = None);

            let _ = host
                .models_mut()
                .update(&models.open_value, |v| *v = Some(pending_value));
            let _ = host
                .models_mut()
                .update(&models.trigger, |v| *v = pending_trigger);
            let _ = host.models_mut().update(&models.geometry, |v| *v = None);
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
    cfg: MenuSubmenuConfig,
) {
    cx.timer_on_timer_for(element, on_timer_handler(models, cfg));
}

pub fn handle_dismissible_pointer_move(
    host: &mut dyn UiActionHost,
    acx: ActionCx,
    mv: PointerMoveCx,
    models: &MenuSubmenuModels,
    cfg: MenuSubmenuConfig,
) -> bool {
    let prev_pointer = host
        .models_mut()
        .read(&models.last_pointer, |v| *v)
        .ok()
        .flatten();
    let prev_dir = host
        .models_mut()
        .read(&models.pointer_dir, |v| *v)
        .ok()
        .flatten();

    let geometry = host
        .models_mut()
        .read(&models.geometry, |v| *v)
        .ok()
        .flatten();
    let grace = geometry.map(|g| pointer_grace_intent::PointerGraceIntentGeometry {
        reference: g.reference,
        floating: g.floating,
    });

    // If a submenu is open but we have no geometry yet, we still want to begin closing it when the
    // pointer wanders away. Without geometry we can't compute a safe-hover corridor, so fall back
    // to arming the close-delay timer once.
    let submenu_open = host
        .models_mut()
        .read(&models.open_value, |v| v.is_some())
        .ok()
        .unwrap_or(false);
    if !submenu_open {
        let next_dir = match prev_pointer {
            None => prev_dir,
            Some(prev) => match pointer_grace_intent::pointer_dir(prev, mv.position) {
                Some(dir) => Some(dir),
                None => prev_dir,
            },
        };
        let _ = host
            .models_mut()
            .update(&models.pointer_dir, |v| *v = next_dir);
        let _ = host
            .models_mut()
            .update(&models.last_pointer, |v| *v = Some(mv.position));
        return false;
    }

    if grace.is_none() {
        let next_dir = match prev_pointer {
            None => prev_dir,
            Some(prev) => match pointer_grace_intent::pointer_dir(prev, mv.position) {
                Some(dir) => Some(dir),
                None => prev_dir,
            },
        };
        let _ = host
            .models_mut()
            .update(&models.pointer_dir, |v| *v = next_dir);
        let _ = host
            .models_mut()
            .update(&models.last_pointer, |v| *v = Some(mv.position));

        let pending = host
            .models_mut()
            .read(&models.close_timer, |v| *v)
            .ok()
            .flatten();
        if pending.is_some() {
            return false;
        }

        let token = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window: Some(acx.window),
            token,
            after: cfg.close_delay,
            repeat: None,
        });
        let _ = host
            .models_mut()
            .update(&models.close_timer, |v| *v = Some(token));
        host.request_redraw(acx.window);
        return true;
    }

    let changed = pointer_grace_intent::drive_close_timer_on_pointer_move(
        host,
        acx,
        mv,
        grace,
        pointer_grace_intent::PointerGraceIntentConfig::new(cfg.safe_hover_buffer, cfg.close_delay),
        &models.last_pointer,
        &models.close_timer,
    );

    let next_dir = match prev_pointer {
        None => prev_dir,
        Some(prev) => match pointer_grace_intent::pointer_dir(prev, mv.position) {
            Some(dir) => Some(dir),
            None => prev_dir,
        },
    };
    let _ = host
        .models_mut()
        .update(&models.pointer_dir, |v| *v = next_dir);

    let mut did_update_grace_intent = false;
    if let Some(grace) = grace {
        // Hit-testing and layout rounding can produce 1px overlaps between adjacent menu items,
        // especially when borders are present. Radix's DOM-driven hover logic effectively treats
        // the "exit" boundary as half-open; bias our exit detection by trimming the bottom/right
        // edge by 1px so moving onto the next item reliably arms the pointer-grace corridor.
        let exit_reference = Rect {
            origin: grace.reference.origin,
            size: Size::new(
                Px((grace.reference.size.width.0 - 1.0).max(0.0)),
                Px((grace.reference.size.height.0 - 1.0).max(0.0)),
            ),
        };
        if grace.floating.contains(mv.position) {
            cancel_timer(host, &models.pointer_grace_timer);
            let _ = host
                .models_mut()
                .update(&models.pointer_grace_intent, |v| *v = None);
            did_update_grace_intent = true;
        } else if prev_pointer.is_some_and(|prev| exit_reference.contains(prev))
            && !exit_reference.contains(mv.position)
        {
            let submenu_open = host
                .models_mut()
                .read(&models.open_value, |v| v.is_some())
                .ok()
                .unwrap_or(false);
            if submenu_open {
                if let Some(intent) =
                    pointer_grace_intent::grace_intent_from_exit_point(mv.position, grace, Px(5.0))
                {
                    let _ = host
                        .models_mut()
                        .update(&models.pointer_grace_intent, |v| *v = Some(intent));
                    cancel_timer(host, &models.pointer_grace_timer);
                    let token = host.next_timer_token();
                    host.push_effect(Effect::SetTimer {
                        window: Some(acx.window),
                        token,
                        after: cfg.pointer_grace_timeout,
                        repeat: None,
                    });
                    let _ = host
                        .models_mut()
                        .update(&models.pointer_grace_timer, |v| *v = Some(token));
                    did_update_grace_intent = true;
                }
            }
        }
    }

    if did_update_grace_intent {
        host.request_redraw(acx.window);
    }

    changed || did_update_grace_intent
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
    _cfg: MenuSubmenuConfig,
    has_submenu: bool,
    value: Arc<str>,
    item_id: GlobalElementId,
) {
    cancel_timer_in_element_context(cx, &models.close_timer);

    if has_submenu {
        let open_value = cx
            .app
            .models_mut()
            .read(&models.open_value, |v| v.clone())
            .ok()
            .flatten();
        let already_open = open_value
            .as_ref()
            .is_some_and(|cur| cur.as_ref() == value.as_ref());

        if already_open {
            set_trigger_if_none(cx, item_id, &models.trigger);
            return;
        }
    } else {
        let _ = cx
            .app
            .models_mut()
            .update(&models.open_value, |v| *v = None);
        let _ = cx.app.models_mut().update(&models.trigger, |v| *v = None);
        let _ = cx.app.models_mut().update(&models.geometry, |v| *v = None);
        let _ = cx
            .app
            .models_mut()
            .update(&models.pending_open_value, |v| *v = None);
        let _ = cx
            .app
            .models_mut()
            .update(&models.pending_open_trigger, |v| *v = None);
        cancel_timer_in_element_context(cx, &models.open_timer);
        cancel_timer_in_element_context(cx, &models.focus_timer);
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
    let _ = cx
        .app
        .models_mut()
        .update(&models.pending_open_value, |v| *v = None);
    let _ = cx
        .app
        .models_mut()
        .update(&models.pending_open_trigger, |v| *v = None);
    cancel_timer_in_element_context(cx, &models.open_timer);
    cancel_timer_in_element_context(cx, &models.close_timer);
    cancel_timer_in_element_context(cx, &models.focus_timer);
}

/// Handle submenu trigger hover changes, applying a small open delay to avoid accidental opens.
pub fn handle_sub_trigger_hover_change(
    host: &mut dyn UiActionHost,
    acx: ActionCx,
    models: &MenuSubmenuModels,
    cfg: MenuSubmenuConfig,
    trigger_id: GlobalElementId,
    is_hovered: bool,
    value: Arc<str>,
) {
    if !is_hovered {
        cancel_timer(host, &models.open_timer);
        let _ = host
            .models_mut()
            .update(&models.pending_open_value, |v| *v = None);
        let _ = host
            .models_mut()
            .update(&models.pending_open_trigger, |v| *v = None);
        return;
    }

    cancel_timer(host, &models.close_timer);
    cancel_timer(host, &models.focus_timer);

    let current_open = host
        .models_mut()
        .read(&models.open_value, |v| v.clone())
        .ok()
        .flatten();
    let already_open = current_open
        .as_ref()
        .is_some_and(|cur| cur.as_ref() == value.as_ref());
    if already_open {
        cancel_timer(host, &models.open_timer);
        let _ = host
            .models_mut()
            .update(&models.pending_open_value, |v| *v = None);
        let _ = host
            .models_mut()
            .update(&models.pending_open_trigger, |v| *v = None);
        return;
    }

    if let Some(current_open) = current_open {
        // Radix prevents switching submenus while the pointer is moving toward the already-open
        // submenu panel (pointer grace intent). We mirror that by ignoring hover-enter on other
        // submenu triggers while the pointer remains inside the grace polygon.
        //
        // This keeps us closer to Radix's `onItemEnter(event).preventDefault()` semantics and avoids
        // repeatedly arming "switch submenu" open-delay timers while the pointer is in transit.
        let pointer = host
            .models_mut()
            .read(&models.last_pointer, |v| *v)
            .ok()
            .flatten();
        let pointer_dir = host
            .models_mut()
            .read(&models.pointer_dir, |v| *v)
            .ok()
            .flatten();
        let grace_intent = host
            .models_mut()
            .read(&models.pointer_grace_intent, |v| *v)
            .ok()
            .flatten();

        let switching_away = current_open.as_ref() != value.as_ref();
        let moving_towards = grace_intent
            .as_ref()
            .is_some_and(|intent| pointer_dir == Some(intent.side));
        let in_grace_area = match (pointer, grace_intent) {
            (Some(pointer), Some(intent)) => {
                pointer_grace_intent::is_pointer_in_grace_area(pointer, intent)
            }
            _ => false,
        };

        if switching_away && moving_towards && in_grace_area {
            cancel_timer(host, &models.open_timer);
            let _ = host
                .models_mut()
                .update(&models.pending_open_value, |v| *v = None);
            let _ = host
                .models_mut()
                .update(&models.pending_open_trigger, |v| *v = None);
            host.request_redraw(acx.window);
            return;
        }

        // While another submenu is open, avoid closing it immediately when hovering a different
        // submenu trigger. We only switch once the hover open-delay timer fires.
    }

    let _ = host
        .models_mut()
        .update(&models.pending_open_value, |v| *v = Some(value));
    let _ = host
        .models_mut()
        .update(&models.pending_open_trigger, |v| *v = Some(trigger_id));

    if cfg.open_delay == Duration::from_millis(0) {
        let pending_value = host
            .models_mut()
            .read(&models.pending_open_value, |v| v.clone())
            .ok()
            .flatten();
        let pending_trigger = host
            .models_mut()
            .read(&models.pending_open_trigger, |v| *v)
            .ok()
            .flatten();
        let _ = host
            .models_mut()
            .update(&models.pending_open_value, |v| *v = None);
        let _ = host
            .models_mut()
            .update(&models.pending_open_trigger, |v| *v = None);
        let _ = host.models_mut().update(&models.open_timer, |v| *v = None);

        let _ = host
            .models_mut()
            .update(&models.open_value, |v| *v = pending_value);
        let _ = host
            .models_mut()
            .update(&models.trigger, |v| *v = pending_trigger);
        host.request_redraw(acx.window);
        return;
    }

    cancel_timer(host, &models.open_timer);
    let token = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window: Some(acx.window),
        token,
        after: cfg.open_delay,
        repeat: None,
    });
    let _ = host
        .models_mut()
        .update(&models.open_timer, |v| *v = Some(token));
    host.request_redraw(acx.window);
}

pub fn open_on_activate(
    host: &mut dyn UiActionHost,
    acx: ActionCx,
    models: &MenuSubmenuModels,
    value: Arc<str>,
) {
    cancel_timer(host, &models.close_timer);
    cancel_timer(host, &models.open_timer);
    cancel_timer(host, &models.pointer_grace_timer);
    let _ = host
        .models_mut()
        .update(&models.pointer_grace_intent, |v| *v = None);
    let _ = host
        .models_mut()
        .update(&models.pending_open_value, |v| *v = None);
    let _ = host
        .models_mut()
        .update(&models.pending_open_trigger, |v| *v = None);
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
    cancel_timer(host, &models.open_timer);
    cancel_timer(host, &models.pointer_grace_timer);
    let _ = host
        .models_mut()
        .update(&models.pointer_grace_intent, |v| *v = None);
    let _ = host
        .models_mut()
        .update(&models.pending_open_value, |v| *v = None);
    let _ = host
        .models_mut()
        .update(&models.pending_open_trigger, |v| *v = None);

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
    let _ = host
        .models_mut()
        .update(&models.pointer_grace_intent, |v| *v = None);
    let _ = host
        .models_mut()
        .update(&models.pending_open_value, |v| *v = None);
    let _ = host
        .models_mut()
        .update(&models.pending_open_trigger, |v| *v = None);
    cancel_timer(host, &models.open_timer);
    cancel_timer(host, &models.close_timer);
    cancel_timer(host, &models.pointer_grace_timer);
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

pub fn submenu_item_close_key_handler(
    models: MenuSubmenuModels,
    dir: LayoutDirection,
) -> fret_ui::action::OnKeyDown {
    #[allow(clippy::arc_with_non_send_sync)]
    Arc::new(move |host, acx, down| {
        if down.repeat {
            return false;
        }
        let is_close_key = match (down.key, dir) {
            (fret_core::KeyCode::ArrowLeft, LayoutDirection::Ltr) => true,
            (fret_core::KeyCode::ArrowRight, LayoutDirection::Rtl) => true,
            _ => false,
        };
        if !is_close_key {
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_runtime::Effect;
    use fret_ui::GlobalElementId;
    use fret_ui::action::{ActionCx, UiActionHost, UiFocusActionHost};

    #[test]
    fn default_pointer_grace_timeout_matches_radix() {
        assert_eq!(
            MenuSubmenuConfig::default().pointer_grace_timeout,
            DEFAULT_POINTER_GRACE_TIMEOUT
        );
    }

    #[test]
    fn new_uses_default_pointer_grace_timeout() {
        let cfg = MenuSubmenuConfig::new(
            Px(1.0),
            Duration::from_millis(1),
            Duration::from_millis(2),
            Duration::from_millis(3),
        );
        assert_eq!(cfg.pointer_grace_timeout, DEFAULT_POINTER_GRACE_TIMEOUT);
    }

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

    fn new_models(app: &mut App) -> MenuSubmenuModels {
        MenuSubmenuModels {
            open_value: app.models_mut().insert(None),
            trigger: app.models_mut().insert(None),
            last_pointer: app.models_mut().insert(None),
            geometry: app.models_mut().insert(None),
            close_timer: app.models_mut().insert(None),
            pointer_dir: app.models_mut().insert(None),
            pointer_grace_intent: app.models_mut().insert(None),
            pointer_grace_timer: app.models_mut().insert(None),
            focus_target: app.models_mut().insert(None),
            focus_timer: app.models_mut().insert(None),
            pending_open_value: app.models_mut().insert(None),
            pending_open_trigger: app.models_mut().insert(None),
            open_timer: app.models_mut().insert(None),
        }
    }

    fn right_side_grace_intent() -> pointer_grace_intent::GraceIntent {
        let reference = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let floating = Rect::new(Point::new(Px(20.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        pointer_grace_intent::grace_intent_from_exit_point(
            Point::new(Px(12.0), Px(5.0)),
            pointer_grace_intent::PointerGraceIntentGeometry {
                reference,
                floating,
            },
            Px(5.0),
        )
        .expect("expected grace intent")
    }

    #[test]
    fn submenu_trigger_hover_does_not_switch_while_pointer_in_grace_polygon() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut host = Host { app: &mut app };

        let models = new_models(&mut host.app);
        let cfg = MenuSubmenuConfig::default();

        let _ = host
            .models_mut()
            .update(&models.open_value, |v| *v = Some(Arc::from("a")));
        let _ = host.models_mut().update(&models.last_pointer, |v| {
            *v = Some(Point::new(Px(12.0), Px(5.0)))
        });
        let _ = host.models_mut().update(&models.pointer_dir, |v| {
            *v = Some(pointer_grace_intent::GraceSide::Right)
        });
        let _ = host.models_mut().update(&models.pointer_grace_intent, |v| {
            *v = Some(right_side_grace_intent())
        });

        handle_sub_trigger_hover_change(
            &mut host,
            ActionCx {
                window,
                target: GlobalElementId(1),
            },
            &models,
            cfg,
            GlobalElementId(2),
            true,
            Arc::from("b"),
        );

        let open_value = host
            .models_mut()
            .read(&models.open_value, |v| v.clone())
            .ok()
            .flatten();
        let pending_open = host
            .models_mut()
            .read(&models.pending_open_value, |v| v.clone())
            .ok()
            .flatten();
        let open_timer = host
            .models_mut()
            .read(&models.open_timer, |v| *v)
            .ok()
            .flatten();

        assert_eq!(open_value.as_deref(), Some("a"));
        assert!(pending_open.is_none());
        assert!(open_timer.is_none());
    }

    #[test]
    fn submenu_open_timer_defers_switch_while_pointer_in_grace_polygon() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut host = Host { app: &mut app };

        let models = new_models(&mut host.app);
        let cfg = MenuSubmenuConfig::default();

        let _ = host
            .models_mut()
            .update(&models.open_value, |v| *v = Some(Arc::from("a")));
        let _ = host
            .models_mut()
            .update(&models.pending_open_value, |v| *v = Some(Arc::from("b")));
        let _ = host.models_mut().update(&models.last_pointer, |v| {
            *v = Some(Point::new(Px(12.0), Px(5.0)))
        });
        let _ = host.models_mut().update(&models.pointer_dir, |v| {
            *v = Some(pointer_grace_intent::GraceSide::Right)
        });
        let _ = host.models_mut().update(&models.pointer_grace_intent, |v| {
            *v = Some(right_side_grace_intent())
        });

        let token = host.next_timer_token();
        let _ = host
            .models_mut()
            .update(&models.open_timer, |v| *v = Some(token));

        let on_timer = on_timer_handler(models.clone(), cfg);
        assert!(on_timer(
            &mut host,
            ActionCx {
                window,
                target: GlobalElementId(1),
            },
            token
        ));

        let open_value = host
            .models_mut()
            .read(&models.open_value, |v| v.clone())
            .ok()
            .flatten();
        let open_timer = host
            .models_mut()
            .read(&models.open_timer, |v| *v)
            .ok()
            .flatten();

        assert_eq!(open_value.as_deref(), Some("a"));
        assert!(open_timer.is_some_and(|t| t != token));
    }

    #[test]
    fn pointer_grace_timer_clears_grace_intent() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut host = Host { app: &mut app };

        let models = new_models(&mut host.app);
        let cfg = MenuSubmenuConfig::default();

        let token = host.next_timer_token();
        let _ = host
            .models_mut()
            .update(&models.pointer_grace_timer, |v| *v = Some(token));
        let _ = host.models_mut().update(&models.pointer_grace_intent, |v| {
            *v = Some(right_side_grace_intent())
        });

        let on_timer = on_timer_handler(models.clone(), cfg);
        assert!(on_timer(
            &mut host,
            ActionCx {
                window,
                target: GlobalElementId(1),
            },
            token
        ));

        let intent = host
            .models_mut()
            .read(&models.pointer_grace_intent, |v| *v)
            .ok()
            .flatten();
        let armed = host
            .models_mut()
            .read(&models.pointer_grace_timer, |v| *v)
            .ok()
            .flatten();

        assert!(intent.is_none());
        assert!(armed.is_none());
    }
}
