pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);
    let noop: fret_ui::action::OnActivate = Arc::new(|_host, _action_cx, _reason| {});
    let open_dialog: fret_ui::action::OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |value| *value = true);
            host.request_redraw(action_cx.window);
        })
    };

    let items = vec![
        shadcn::CommandItem::new("Calendar").on_select_action(noop.clone()),
        shadcn::CommandItem::new("Search Emoji").on_select_action(noop.clone()),
        shadcn::CommandItem::new("Calculator").on_select_action(noop.clone()),
    ];
    let entries = vec![
        shadcn::CommandGroup::new(items)
            .heading("Suggestions")
            .into(),
    ];

    shadcn::CommandDialog::new(open.clone(), query.clone(), Vec::new())
        .entries(entries)
        .placeholder("Type a command or search...")
        .empty_text("No results found.")
        .test_id_input("ui-gallery-command-basic-input")
        .list_test_id("ui-gallery-command-basic-listbox")
        .test_id_item_prefix("ui-gallery-command-basic-item-")
        .into_element(cx, |cx| {
            shadcn::Button::new("Open Menu")
                .variant(shadcn::ButtonVariant::Outline)
                .on_activate(open_dialog.clone())
                .test_id("ui-gallery-command-basic-trigger")
                .into_element(cx)
        })
        .test_id("ui-gallery-command-basic")
}
// endregion: example
