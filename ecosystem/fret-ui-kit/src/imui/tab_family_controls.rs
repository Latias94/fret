//! Immediate-mode tab-bar helpers.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::containers::build_imui_children_with_focus;
use super::label_identity::parse_label_identity;
use super::{ImUiFacade, TabBarOptions, TabBarResponse, TabItemOptions};

mod items;
mod trigger;
mod visual;

use items::BuiltTabItem;

pub struct ImUiTabBar<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    items: &'cx mut Vec<BuiltTabItem>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
}

pub(super) fn tab_bar_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: TabBarOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
) -> (AnyElement, TabBarResponse) {
    let root_name = format!("fret-ui-kit.imui.tab_bar.{id}");
    cx.with_root_name(root_name.as_str(), |cx| {
        let selected = options
            .selected
            .clone()
            .unwrap_or_else(|| cx.local_model_keyed("selected", || None::<Arc<str>>));
        let mut items = Vec::new();

        {
            let mut tab_bar = ImUiTabBar {
                cx,
                items: &mut items,
                build_focus: build_focus.clone(),
            };
            f(&mut tab_bar);
        }

        items::render_tab_bar(cx, selected, items, build_focus, options)
    })
}

impl<'cx, 'a, H: UiHost> ImUiTabBar<'cx, 'a, H> {
    pub fn tab_item(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.tab_item_with_options(id, label, TabItemOptions::default(), f);
    }

    pub fn tab_item_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: TabItemOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let id = Arc::<str>::from(id);
        let raw_label = label.into();
        let parts = parse_label_identity(raw_label.as_ref());
        let label = Arc::<str>::from(parts.visible);
        let test_id = options.test_id.clone();
        let panel_test_id = options.panel_test_id.or_else(|| {
            test_id
                .as_ref()
                .map(|test_id| Arc::from(format!("{test_id}.panel")))
        });
        let build_focus = self.build_focus.clone();
        let panel_children = self.cx.keyed(id.clone(), |cx| {
            let mut out = Vec::new();
            build_imui_children_with_focus(cx, &mut out, build_focus, f);
            out
        });
        self.items.push(BuiltTabItem {
            id,
            label,
            enabled: options.enabled,
            default_selected: options.default_selected,
            test_id,
            panel_test_id,
            activate_shortcut: options.activate_shortcut,
            shortcut_repeat: options.shortcut_repeat,
            panel_children,
        });
    }

    pub fn begin_tab_item(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.begin_tab_item_with_options(id, label, TabItemOptions::default(), f);
    }

    pub fn begin_tab_item_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: TabItemOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.tab_item_with_options(id, label, options, f);
    }
}

#[cfg(test)]
mod tests;
