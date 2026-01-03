//! MenuSubTrigger helpers (Radix-aligned outcomes).
//!
//! Radix `MenuSubTrigger` is responsible for opening nested menus via:
//! - pointer hover intent (with grace corridor)
//! - click/activation
//! - ArrowRight / ArrowLeft keyboard affordances
//!
//! In Fret, wrappers call these helpers from within a pressable item closure.

use std::sync::Arc;
use std::time::Duration;

use fret_core::KeyCode;
use fret_ui::element::PressableState;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::model_watch::ModelWatchExt as _;
use crate::primitives::menu::sub;

/// Wire submenu-trigger behavior onto a pressable item.
///
/// Returns `Some(expanded)` when the item has a submenu, otherwise `None`.
pub fn wire<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    st: PressableState,
    item_id: GlobalElementId,
    disabled: bool,
    has_submenu: bool,
    value: Arc<str>,
    models: &sub::MenuSubmenuModels,
    focus_delay: Duration,
) -> Option<bool> {
    if disabled {
        return has_submenu.then_some(false);
    }

    if has_submenu {
        let models_for_hover = models.clone();
        let value_for_hover = value.clone();
        cx.pressable_add_on_hover_change(Arc::new(move |host, acx, is_hovered| {
            if !is_hovered {
                return;
            }
            sub::open_on_hover(host, acx, &models_for_hover, true, value_for_hover.clone());
        }));
    }

    if st.hovered {
        sub::sync_while_trigger_hovered(cx, models, has_submenu, value.clone(), item_id);
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
    cx.key_on_key_down_for(
        item_id,
        Arc::new(move |host, acx, down| {
            if down.repeat {
                return false;
            }
            match down.key {
                KeyCode::ArrowRight => {
                    if !key_has_submenu {
                        return false;
                    }
                    sub::open_on_arrow_right(
                        host,
                        acx,
                        &models_for_key,
                        value_for_key.clone(),
                        focus_delay,
                    );
                    true
                }
                KeyCode::ArrowLeft => {
                    sub::close_on_arrow_left(host, acx, &models_for_key);
                    true
                }
                _ => false,
            }
        }),
    );

    let expanded = cx
        .watch_model(&models.open_value)
        .cloned()
        .unwrap_or(None)
        .as_ref()
        .is_some_and(|cur: &Arc<str>| cur.as_ref() == value.as_ref());

    has_submenu.then_some(expanded)
}
