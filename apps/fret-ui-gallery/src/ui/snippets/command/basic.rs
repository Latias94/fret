pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let last_action = super::last_action_model(cx);
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);
    let on_select = super::on_select_for_last_action(last_action.clone());

    let items = vec![
        shadcn::CommandItem::new("Calendar")
            .shortcut("Cmd+C")
            .keywords(["events", "schedule"])
            .on_select_action(on_select(Arc::from("command.basic.calendar"))),
        shadcn::CommandItem::new("Search Emoji")
            .shortcut("Cmd+E")
            .keywords(["emoji", "insert"])
            .on_select_action(on_select(Arc::from("command.basic.search-emoji"))),
        shadcn::CommandItem::new("Calculator")
            .shortcut("Cmd+K")
            .keywords(["math", "calc"])
            .on_select_action(on_select(Arc::from("command.basic.calculator"))),
    ];

    shadcn::CommandDialog::new(open.clone(), query.clone(), items)
        .a11y_label("Basic command dialog")
        .empty_text("No results found.")
        .into_element(cx, |cx| {
            shadcn::Button::new("Open Command Menu")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open.clone())
                .test_id("ui-gallery-command-basic-trigger")
                .into_element(cx)
        })
        .test_id("ui-gallery-command-basic")
}
// endregion: example
