pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_runtime::CommandId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let file = shadcn::MenubarTrigger::new("File")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new(),
            [
                shadcn::MenubarItem::new("New Tab")
                    .action(CommandId::new("ui_gallery.menubar.usage.new_tab"))
                    .trailing(shadcn::MenubarShortcut::new("⌘T").into_element(cx))
                    .into(),
                shadcn::MenubarItem::new("New Window")
                    .action(CommandId::new("ui_gallery.menubar.usage.new_window"))
                    .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Share")
                    .action(CommandId::new("ui_gallery.menubar.usage.share"))
                    .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Print")
                    .action(CommandId::new("ui_gallery.menubar.usage.print"))
                    .into(),
            ],
        );

    shadcn::Menubar::new([file]).into_element(cx)
}
// endregion: example
