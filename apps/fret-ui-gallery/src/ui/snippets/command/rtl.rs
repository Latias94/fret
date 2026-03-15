pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let last_action = super::last_action_model(cx);
    let query = cx.local_model(String::new);
    let on_select = super::on_select_for_last_action(last_action.clone());

    let entries = vec![
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Search")
                .shortcut("Cmd+F")
                .on_select_action(on_select(Arc::from("command.rtl.search"))),
            shadcn::CommandItem::new("Dashboard")
                .shortcut("Cmd+D")
                .on_select_action(on_select(Arc::from("command.rtl.dashboard"))),
            shadcn::CommandItem::new("Settings")
                .shortcut("Cmd+,")
                .on_select_action(on_select(Arc::from("command.rtl.settings"))),
        ])
        .heading("RTL")
        .into(),
    ];

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::CommandPalette::new(query.clone(), Vec::new())
            .placeholder("Type a command or search...")
            .a11y_label("RTL command list")
            .entries(entries)
            .test_id_input("ui-gallery-command-rtl-input")
            .list_test_id("ui-gallery-command-rtl-listbox")
            .test_id_item_prefix("ui-gallery-command-rtl-item-")
            .into_element(cx)
            .test_id("ui-gallery-command-rtl")
    })
}
// endregion: example
