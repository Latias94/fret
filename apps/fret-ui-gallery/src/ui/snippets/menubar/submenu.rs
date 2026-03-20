pub const SOURCE: &str = include_str!("submenu.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();

    let file = shadcn::MenubarTrigger::new("File")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new(),
            [
                shadcn::MenubarSub::new(
                    shadcn::MenubarSubTrigger::new("Share"),
                    shadcn::MenubarSubContent::new([
                        shadcn::MenubarItem::new("Email link")
                            .action(CommandId::new(
                                "ui_gallery.menubar.submenu.share.email_link",
                            ))
                            .into(),
                        shadcn::MenubarItem::new("Messages")
                            .action(CommandId::new("ui_gallery.menubar.submenu.share.messages"))
                            .into(),
                        shadcn::MenubarItem::new("Notes")
                            .action(CommandId::new("ui_gallery.menubar.submenu.share.notes"))
                            .into(),
                    ]),
                )
                .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Print...")
                    .action(CommandId::new("ui_gallery.menubar.submenu.print"))
                    .trailing(shadcn::MenubarShortcut::new("⌘P").into_element(cx))
                    .into(),
            ],
        );

    let edit = shadcn::MenubarTrigger::new("Edit")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new(),
            [
                shadcn::MenubarItem::new("Undo")
                    .action(CommandId::new("ui_gallery.menubar.submenu.undo"))
                    .trailing(shadcn::MenubarShortcut::new("⌘Z").into_element(cx))
                    .into(),
                shadcn::MenubarItem::new("Redo")
                    .action(CommandId::new("ui_gallery.menubar.submenu.redo"))
                    .trailing(shadcn::MenubarShortcut::new("⇧⌘Z").into_element(cx))
                    .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarSub::new(
                    shadcn::MenubarSubTrigger::new("Find"),
                    shadcn::MenubarSubContent::new([
                        shadcn::MenubarItem::new("Find...")
                            .action(CommandId::new("ui_gallery.menubar.submenu.find.find"))
                            .into(),
                        shadcn::MenubarItem::new("Find Next")
                            .action(CommandId::new("ui_gallery.menubar.submenu.find.next"))
                            .into(),
                        shadcn::MenubarItem::new("Find Previous")
                            .action(CommandId::new("ui_gallery.menubar.submenu.find.previous"))
                            .into(),
                    ]),
                )
                .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Cut")
                    .action(CommandId::new("ui_gallery.menubar.submenu.cut"))
                    .into(),
                shadcn::MenubarItem::new("Copy")
                    .action(CommandId::new("ui_gallery.menubar.submenu.copy"))
                    .into(),
                shadcn::MenubarItem::new("Paste")
                    .action(CommandId::new("ui_gallery.menubar.submenu.paste"))
                    .into(),
            ],
        );

    shadcn::Menubar::new([file, edit])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
