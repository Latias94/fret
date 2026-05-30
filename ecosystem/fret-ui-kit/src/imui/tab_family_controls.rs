//! Immediate-mode tab-bar helpers.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{TabBarOptions, TabBarResponse};

mod item_methods;
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

#[cfg(test)]
mod tests;
