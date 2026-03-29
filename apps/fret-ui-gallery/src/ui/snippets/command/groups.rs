pub const SOURCE: &str = include_str!("groups.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model(String::new);
    let noop: fret_ui::action::OnActivate = Arc::new(|_host, _action_cx, _reason| {});
    let open_dialog: fret_ui::action::OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |value| *value = true);
            host.request_redraw(action_cx.window);
        })
    };
    let icon_id = fret_icons::IconId::new_static;

    let entries: Vec<shadcn::CommandEntry> = vec![
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Calendar")
                .leading_icon(icon_id("lucide.calendar"))
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Search Emoji")
                .leading_icon(icon_id("lucide.smile"))
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Calculator")
                .leading_icon(icon_id("lucide.calculator"))
                .disabled(true),
        ])
        .heading("Suggestions")
        .into(),
        shadcn::CommandSeparator::new().into(),
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Profile")
                .leading_icon(icon_id("lucide.user"))
                .shortcut("⌘P")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Billing")
                .leading_icon(icon_id("lucide.credit-card"))
                .shortcut("⌘B")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Settings")
                .leading_icon(icon_id("lucide.settings"))
                .shortcut("⌘S")
                .on_select_action(noop.clone()),
        ])
        .heading("Settings")
        .into(),
    ];

    shadcn::CommandDialog::new(open.clone(), query.clone(), Vec::new())
        .entries(entries)
        .placeholder("Type a command or search...")
        .empty_text("No results found.")
        .test_id_input("ui-gallery-command-groups-input")
        .list_test_id("ui-gallery-command-groups-listbox")
        .test_id_item_prefix("ui-gallery-command-groups-item-")
        .into_element(cx, |cx| {
            shadcn::Button::new("Open Menu")
                .variant(shadcn::ButtonVariant::Outline)
                .on_activate(open_dialog.clone())
                .test_id("ui-gallery-command-groups-trigger")
                .into_element(cx)
        })
        .test_id("ui-gallery-command-groups")
}
// endregion: example
