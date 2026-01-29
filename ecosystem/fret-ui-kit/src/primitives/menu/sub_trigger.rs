//! MenuSubTrigger helpers (Radix-aligned outcomes).
//!
//! Radix `MenuSubTrigger` is responsible for opening nested menus via:
//! - pointer hover intent (with grace corridor)
//! - click/activation
//! - ArrowRight / ArrowLeft keyboard affordances
//!
//! In Fret, wrappers call these helpers from within a pressable item closure.

use std::sync::Arc;

use fret_core::{KeyCode, Rect, Size};
use fret_ui::element::PressableState;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::model_watch::ModelWatchExt as _;
use crate::primitives::direction::{self as direction_prim, LayoutDirection};
use crate::primitives::menu::sub;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuSubTriggerGeometryHint {
    pub outer: Rect,
    pub desired: Size,
}

/// Wire submenu-trigger behavior onto a pressable item.
///
/// Returns `Some(expanded)` when the item has a submenu, otherwise `None`.
#[allow(clippy::too_many_arguments)]
pub fn wire<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    st: PressableState,
    item_id: GlobalElementId,
    disabled: bool,
    has_submenu: bool,
    value: Arc<str>,
    models: &sub::MenuSubmenuModels,
    cfg: sub::MenuSubmenuConfig,
    geometry_hint: Option<MenuSubTriggerGeometryHint>,
) -> Option<bool> {
    if disabled {
        return has_submenu.then_some(false);
    }

    if has_submenu {
        // Submenu open/focus/close timers are emitted from submenu-trigger interactions (hover,
        // arrow keys). Install a timer handler on the trigger element so timer routing remains
        // stable even when the overlay root is not the timer event target.
        sub::install_timer_handler(cx, item_id, models.clone(), cfg);

        let models_for_hover = models.clone();
        let value_for_hover = value.clone();
        let cfg_for_hover = cfg;
        let trigger_id_for_hover = item_id;
        cx.pressable_add_on_hover_change(Arc::new(move |host, acx, is_hovered| {
            sub::handle_sub_trigger_hover_change(
                host,
                acx,
                &models_for_hover,
                cfg_for_hover,
                trigger_id_for_hover,
                is_hovered,
                value_for_hover.clone(),
            );
        }));
    }

    if st.hovered {
        sub::sync_while_trigger_hovered(cx, models, cfg, has_submenu, value.clone(), item_id);
    }

    if st.focused {
        sub::close_if_focus_moved_without_pointer(cx, models, &value, item_id);
    }

    if has_submenu {
        let models_for_activate = models.clone();
        let value_for_activate = value.clone();
        cx.pressable_add_on_activate(Arc::new(move |host, acx, _reason| {
            sub::open_on_activate(host, acx, &models_for_activate, value_for_activate.clone());
        }));
    }

    let key_has_submenu = has_submenu;
    let models_for_key = models.clone();
    let value_for_key = value.clone();
    let cfg_for_key = cfg;
    let trigger_id_for_key = item_id;
    let dir = direction_prim::use_direction_in_scope(cx, None);
    cx.key_on_key_down_for(
        item_id,
        Arc::new(move |host, acx, down| {
            if down.repeat {
                return false;
            }
            let is_open_key = match (down.key, dir) {
                (KeyCode::ArrowRight, LayoutDirection::Ltr) => true,
                (KeyCode::ArrowLeft, LayoutDirection::Rtl) => true,
                _ => false,
            };
            if is_open_key {
                if !key_has_submenu {
                    return false;
                }
                sub::open_on_arrow_right(
                    host,
                    acx,
                    &models_for_key,
                    trigger_id_for_key,
                    value_for_key.clone(),
                    cfg_for_key.focus_delay,
                );
                return true;
            }

            let is_close_key = match (down.key, dir) {
                (KeyCode::ArrowLeft, LayoutDirection::Ltr) => true,
                (KeyCode::ArrowRight, LayoutDirection::Rtl) => true,
                _ => false,
            };
            if is_close_key {
                let is_open = host
                    .models_mut()
                    .read(&models_for_key.open_value, |v| v.is_some())
                    .ok()
                    .unwrap_or(false);
                if !is_open {
                    return false;
                }

                sub::close_on_arrow_left(host, acx, &models_for_key);
                return true;
            }

            false
        }),
    );

    let expanded = cx
        .watch_model(&models.open_value)
        .cloned()
        .unwrap_or(None)
        .as_ref()
        .is_some_and(|cur: &Arc<str>| cur.as_ref() == value.as_ref());

    if has_submenu && expanded {
        sub::set_trigger_if_none(cx, item_id, &models.trigger);

        let open_trigger = cx
            .app
            .models_mut()
            .read(&models.trigger, |v| *v)
            .ok()
            .flatten();
        if open_trigger.is_none_or(|t| t == item_id)
            && let Some(hint) = geometry_hint
        {
            sub::set_geometry_from_element_anchor_if_present(
                cx,
                item_id,
                models,
                hint.outer,
                hint.desired,
            );
        }
    }

    has_submenu.then_some(expanded)
}
