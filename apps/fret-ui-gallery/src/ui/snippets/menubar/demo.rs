pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let view_bookmarks_bar = cx.local_model_keyed("view_bookmarks_bar", || false);
    let view_full_urls = cx.local_model_keyed("view_full_urls", || true);
    let profile = cx.local_model_keyed("profile", || Some(Arc::<str>::from("benoit")));

    let file = shadcn::MenubarMenu::new("File").entries([
        shadcn::MenubarEntry::Group(shadcn::MenubarGroup::new([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("New Tab")
                    .action(CommandId::new("ui_gallery.menubar.demo.new_tab"))
                    .trailing(shadcn::MenubarShortcut::new("⌘T").into_element(cx)),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("New Window")
                    .action(CommandId::new("ui_gallery.menubar.demo.new_window")),
            ),
        ])),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Group(shadcn::MenubarGroup::new([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Share")
                    .close_on_select(false)
                    .action(CommandId::new("ui_gallery.menubar.demo.share")),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Print")
                    .close_on_select(false)
                    .action(CommandId::new("ui_gallery.menubar.demo.print")),
            ),
        ])),
    ]);

    let edit = shadcn::MenubarMenu::new("Edit").entries([
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Undo")
                .trailing(shadcn::MenubarShortcut::new("⌘Z").into_element(cx)),
        ),
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Redo")
                .trailing(shadcn::MenubarShortcut::new("⇧⌘Z").into_element(cx)),
        ),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Submenu(shadcn::MenubarItem::new("Find").submenu([
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Search the web")),
            shadcn::MenubarEntry::Separator,
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Find...")),
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Find Next")),
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Find Previous")),
        ])),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Cut")),
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Copy")),
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Paste")),
    ]);

    let view = shadcn::MenubarTrigger::new("View")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new().min_width(Px(256.0)),
            [
                shadcn::MenubarCheckboxItem::new(
                    view_bookmarks_bar.clone(),
                    "Always Show Bookmarks Bar",
                )
                .into(),
                shadcn::MenubarCheckboxItem::new(view_full_urls.clone(), "Always Show Full URLs")
                    .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Reload")
                    .inset(true)
                    .trailing(shadcn::MenubarShortcut::new("⌘R").into_element(cx))
                    .into(),
                shadcn::MenubarItem::new("Force Reload")
                    .disabled(true)
                    .inset(true)
                    .trailing(shadcn::MenubarShortcut::new("⇧⌘R").into_element(cx))
                    .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Toggle Fullscreen")
                    .inset(true)
                    .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Hide Sidebar").inset(true).into(),
            ],
        );

    let profiles = shadcn::MenubarMenu::new("Profiles").entries([
        shadcn::MenubarEntry::RadioGroup(
            shadcn::MenubarRadioGroup::new(profile)
                .item(shadcn::MenubarRadioItemSpec::new("andy", "Andy"))
                .item(shadcn::MenubarRadioItemSpec::new("benoit", "Benoit"))
                .item(shadcn::MenubarRadioItemSpec::new("luis", "Luis")),
        ),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Edit...").inset(true)),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Add Profile...").inset(true)),
    ]);

    shadcn::Menubar::new([file, edit, view, profiles]).into_element(cx)
}
// endregion: example
