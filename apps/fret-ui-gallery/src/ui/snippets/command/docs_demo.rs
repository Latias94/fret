pub const SOURCE: &str = include_str!("docs_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let query = cx.local_model(String::new);
    let noop: fret_ui::action::OnActivate = Arc::new(|_host, _action_cx, _reason| {});

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

    shadcn::CommandPalette::new(query.clone(), Vec::new())
        .placeholder("Type a command or search...")
        .empty_text("No results found.")
        .a11y_label("Command docs demo")
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(384.0))
                .min_w_0(),
        )
        .entries(entries)
        .test_id_input("ui-gallery-command-docs-demo-input")
        .list_test_id("ui-gallery-command-docs-demo-listbox")
        .test_id_item_prefix("ui-gallery-command-docs-demo-item-")
        .into_element(cx)
        .test_id("ui-gallery-command-docs-demo")
}
// endregion: example
