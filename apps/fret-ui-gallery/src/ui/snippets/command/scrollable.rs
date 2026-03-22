pub const SOURCE: &str = include_str!("scrollable.rs");

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
    let entries = vec![
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Home")
                .leading_icon(icon_id("lucide.house"))
                .shortcut("⌘H")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Inbox")
                .leading_icon(icon_id("lucide.inbox"))
                .shortcut("⌘I")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Documents")
                .leading_icon(icon_id("lucide.file-text"))
                .shortcut("⌘D")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Folders")
                .leading_icon(icon_id("lucide.folder"))
                .shortcut("⌘F")
                .on_select_action(noop.clone()),
        ])
        .heading("Navigation")
        .into(),
        shadcn::CommandSeparator::new().into(),
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("New File")
                .leading_icon(icon_id("lucide.plus"))
                .shortcut("⌘N")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("New Folder")
                .leading_icon(icon_id("lucide.folder-plus"))
                .shortcut("⇧⌘N")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Copy")
                .leading_icon(icon_id("lucide.copy"))
                .shortcut("⌘C")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Cut")
                .leading_icon(icon_id("lucide.scissors"))
                .shortcut("⌘X")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Paste")
                .leading_icon(icon_id("lucide.clipboard-paste"))
                .shortcut("⌘V")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Delete")
                .leading_icon(icon_id("lucide.trash"))
                .shortcut("⌫")
                .on_select_action(noop.clone()),
        ])
        .heading("Actions")
        .into(),
        shadcn::CommandSeparator::new().into(),
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Grid View")
                .leading_icon(icon_id("lucide.layout-grid"))
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("List View")
                .leading_icon(icon_id("lucide.list"))
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Zoom In")
                .leading_icon(icon_id("lucide.zoom-in"))
                .shortcut("⌘+")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Zoom Out")
                .leading_icon(icon_id("lucide.zoom-out"))
                .shortcut("⌘-")
                .on_select_action(noop.clone()),
        ])
        .heading("View")
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
            shadcn::CommandItem::new("Notifications")
                .leading_icon(icon_id("lucide.bell"))
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Help & Support")
                .leading_icon(icon_id("lucide.circle-help"))
                .on_select_action(noop.clone()),
        ])
        .heading("Account")
        .into(),
        shadcn::CommandSeparator::new().into(),
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Calculator")
                .leading_icon(icon_id("lucide.calculator"))
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Calendar")
                .leading_icon(icon_id("lucide.calendar"))
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Image Editor")
                .leading_icon(icon_id("lucide.image"))
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("Code Editor")
                .leading_icon(icon_id("lucide.code"))
                .on_select_action(noop.clone()),
        ])
        .heading("Tools")
        .into(),
    ];

    shadcn::CommandDialog::new(open.clone(), query.clone(), Vec::new())
        .entries(entries)
        .empty_text("No results found.")
        .test_id_input("ui-gallery-command-scrollable-input")
        .list_test_id("ui-gallery-command-scrollable-listbox")
        .list_viewport_test_id("ui-gallery-command-scrollable-viewport")
        .test_id_item_prefix("ui-gallery-command-scrollable-item-")
        .into_element(cx, |cx| {
            shadcn::Button::new("Open Menu")
                .variant(shadcn::ButtonVariant::Outline)
                .on_activate(open_dialog.clone())
                .test_id("ui-gallery-command-scrollable-trigger")
                .into_element(cx)
        })
        .test_id("ui-gallery-command-scrollable")
}
// endregion: example
