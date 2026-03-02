// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();

    let file = shadcn::MenubarMenu::new("File").entries([
        shadcn::MenubarEntry::Submenu(shadcn::MenubarItem::new("Share").submenu([
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Email link")),
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Messages")),
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Notes")),
        ])),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Print...")
                .trailing(shadcn::MenubarShortcut::new("⌘P").into_element(cx)),
        ),
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
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Find...")),
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Find Next")),
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Find Previous")),
        ])),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Cut")),
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Copy")),
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Paste")),
    ]);

    shadcn::Menubar::new([file, edit])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
