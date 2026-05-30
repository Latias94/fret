use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ColumnProps, Length, SpacingLength};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

mod list;
mod panel;
mod selection;

use crate::imui::{TabBarOptions, TabBarResponse};

pub(super) struct BuiltTabItem {
    pub(super) id: Arc<str>,
    pub(super) label: Arc<str>,
    pub(super) enabled: bool,
    pub(super) default_selected: bool,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) panel_test_id: Option<Arc<str>>,
    pub(super) activate_shortcut: Option<fret_runtime::KeyChord>,
    pub(super) shortcut_repeat: bool,
    pub(super) panel_children: Vec<AnyElement>,
}

pub(super) fn render_tab_bar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: Model<Option<Arc<str>>>,
    items: Vec<BuiltTabItem>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: TabBarOptions,
) -> (AnyElement, TabBarResponse) {
    let selected = selection::normalize_selected_tab(cx, &selected_model, &items);
    let selected_changed =
        super::super::model_value_changed_for(cx, cx.root_id(), selected.clone());
    let list = list::render_tab_list(cx, &selected_model, selected.as_deref(), &items, &options);

    if let Some(state) = build_focus.as_ref()
        && state.get().is_none()
    {
        state.set(list.selected_trigger_id.or(list.first_focusable));
    }

    let panel =
        panel::render_selected_tab_panel(cx, selected.clone(), list.selected_trigger_id, items);

    let mut children = vec![list.element];
    if let Some(panel) = panel {
        children.push(panel);
    }

    let mut column = ColumnProps::default();
    column.layout.size.width = Length::Fill;
    column.layout.size.height = Length::Auto;
    column.gap = SpacingLength::Px(Px(0.0));
    (
        cx.column(column, move |_cx| children),
        TabBarResponse {
            selected,
            selected_changed,
            triggers: list.trigger_responses,
        },
    )
}
