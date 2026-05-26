//! Immediate-mode menu-bar helpers.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{ImUiFacade, MenuBarOptions};
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

mod menu;
mod submenu;
mod submenu_state;
mod trigger;
mod visual;

pub(super) use menu::begin_menu_with_options;
pub(super) use submenu::begin_submenu_with_options;

#[derive(Debug, Clone)]
pub(in crate::imui) struct ImUiMenubarPolicyState {
    pub(super) open_menu: Model<Option<Arc<str>>>,
    pub(super) group_active: Model<Option<menubar_trigger_row::MenubarActiveTrigger>>,
    pub(super) registry: Model<Vec<menubar_trigger_row::MenubarTriggerRowEntry>>,
    pub(super) suppress_close_auto_focus_once: Model<bool>,
}

pub(super) fn menu_bar_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: MenuBarOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let gap = options.gap;
    let test_id = options.test_id;
    cx.named("fret-ui-kit.imui.menu-bar", move |cx| {
        let group = cx.root_id();
        let open_menu = cx.local_model_keyed("open_menu", || None::<Arc<str>>);
        let group_active = menubar_trigger_row::ensure_group_active_model(cx, group);
        let registry = menubar_trigger_row::ensure_group_registry_model(cx, group);
        let suppress_close_auto_focus_once =
            cx.local_model_keyed("suppress_close_auto_focus_once", || false);
        let policy = ImUiMenubarPolicyState {
            open_menu,
            group_active,
            registry,
            suppress_close_auto_focus_once,
        };

        let mut builder = crate::ui::h_flex_build(move |cx: &mut ElementContext<'_, H>, out| {
            let _ = cx.app.models_mut().update(
                &policy.registry,
                |entries: &mut Vec<menubar_trigger_row::MenubarTriggerRowEntry>| entries.clear(),
            );
            cx.provide(policy.clone(), move |cx| {
                super::containers::build_imui_children_with_focus(cx, out, build_focus, f);
            });
        });
        builder = builder
            .gap_metric(gap)
            .justify(crate::Justify::Start)
            .items(crate::Items::Center)
            .no_wrap()
            .role(SemanticsRole::MenuBar);
        if let Some(test_id) = test_id {
            builder = builder.test_id(test_id);
        }
        builder.into_element(cx)
    })
}

#[cfg(test)]
mod tests;
