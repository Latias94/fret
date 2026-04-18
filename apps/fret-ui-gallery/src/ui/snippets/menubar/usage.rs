pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_runtime::CommandId;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let file = shadcn::MenubarTrigger::new("File")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new(),
            [
                shadcn::MenubarGroup::new([
                    shadcn::MenubarItem::new("New Tab")
                        .action(CommandId::new("ui_gallery.menubar.usage.new_tab"))
                        .trailing(shadcn::MenubarShortcut::new("⌘T").into_element(cx))
                        .into(),
                    shadcn::MenubarItem::new("New Window")
                        .action(CommandId::new("ui_gallery.menubar.usage.new_window"))
                        .into(),
                ])
                .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarGroup::new([
                    shadcn::MenubarItem::new("Share")
                        .action(CommandId::new("ui_gallery.menubar.usage.share"))
                        .into(),
                    shadcn::MenubarItem::new("Print")
                        .action(CommandId::new("ui_gallery.menubar.usage.print"))
                        .into(),
                ])
                .into(),
            ],
        );

    shadcn::Menubar::new([file]).into_element(cx)
}
// endregion: example
