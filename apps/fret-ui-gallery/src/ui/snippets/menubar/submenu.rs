pub const SOURCE: &str = include_str!("submenu.rs");

// region: example
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();

    let file = shadcn::MenubarMenu::new("File").entries([
        shadcn::MenubarEntry::Submenu(shadcn::MenubarItem::new("Share").submenu([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Email link")
                    .action(CommandId::new("ui_gallery.menubar.submenu.share.email_link")),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Messages")
                    .action(CommandId::new("ui_gallery.menubar.submenu.share.messages")),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Notes")
                    .action(CommandId::new("ui_gallery.menubar.submenu.share.notes")),
            ),
        ])),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Print...")
                .action(CommandId::new("ui_gallery.menubar.submenu.print"))
                .trailing(shadcn::MenubarShortcut::new("⌘P").into_element(cx)),
        ),
    ]);

    let edit = shadcn::MenubarMenu::new("Edit").entries([
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Undo")
                .action(CommandId::new("ui_gallery.menubar.submenu.undo"))
                .trailing(shadcn::MenubarShortcut::new("⌘Z").into_element(cx)),
        ),
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Redo")
                .action(CommandId::new("ui_gallery.menubar.submenu.redo"))
                .trailing(shadcn::MenubarShortcut::new("⇧⌘Z").into_element(cx)),
        ),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Submenu(shadcn::MenubarItem::new("Find").submenu([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Find...")
                    .action(CommandId::new("ui_gallery.menubar.submenu.find.find")),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Find Next")
                    .action(CommandId::new("ui_gallery.menubar.submenu.find.next")),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Find Previous")
                    .action(CommandId::new("ui_gallery.menubar.submenu.find.previous")),
            ),
        ])),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Cut").action(CommandId::new("ui_gallery.menubar.submenu.cut")),
        ),
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Copy").action(CommandId::new("ui_gallery.menubar.submenu.copy")),
        ),
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Paste")
                .action(CommandId::new("ui_gallery.menubar.submenu.paste")),
        ),
    ]);

    shadcn::Menubar::new([file, edit])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
