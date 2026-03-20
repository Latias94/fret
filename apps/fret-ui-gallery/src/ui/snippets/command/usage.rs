pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let query = cx.local_model(String::new);
    let noop: fret_ui::action::OnActivate = Arc::new(|_host, _action_cx, _reason| {});

    let entries: Vec<shadcn::CommandEntry> = vec![
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Calendar").on_select_action(noop.clone()),
            shadcn::CommandItem::new("Search Emoji").on_select_action(noop.clone()),
            shadcn::CommandItem::new("Calculator").on_select_action(noop.clone()),
        ])
        .heading("Suggestions")
        .into(),
        shadcn::CommandSeparator::new().into(),
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Profile").on_select_action(noop.clone()),
            shadcn::CommandItem::new("Billing").on_select_action(noop.clone()),
            shadcn::CommandItem::new("Settings").on_select_action(noop.clone()),
        ])
        .heading("Settings")
        .into(),
    ];

    shadcn::CommandPalette::new(query.clone(), Vec::new())
        .placeholder("Type a command or search...")
        .empty_text("No results found.")
        .a11y_label("Command usage")
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(384.0))
                .min_w_0(),
        )
        .entries(entries)
        .test_id_input("ui-gallery-command-usage-input")
        .list_test_id("ui-gallery-command-usage-listbox")
        .test_id_item_prefix("ui-gallery-command-usage-item-")
        .into_element(cx)
        .test_id("ui-gallery-command-usage")
}
// endregion: example
