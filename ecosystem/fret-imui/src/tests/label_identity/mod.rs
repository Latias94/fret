use super::*;

use fret_ui_kit::imui::{
    BeginMenuOptions, ButtonOptions, CheckboxOptions, CollapsingHeaderOptions, ComboOptions,
    MenuBarOptions, MenuItemOptions, RadioOptions, SelectableOptions, SeparatorTextOptions,
    SliderOptions, SwitchOptions, TabBarOptions, TabItemOptions, TableColumn, TableOptions,
    TableSortDirection, TreeNodeOptions,
};

fn current_focus_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
) -> Option<String> {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    let focus = ui.focus()?;
    let snap = ui.semantics_snapshot()?;
    snap.nodes
        .iter()
        .find(|node| node.id == focus)
        .and_then(|node| node.test_id.as_deref().map(str::to_owned))
}

mod explicit_ids;
mod model_controls;
mod table_headers;
mod visible_suffixes;
