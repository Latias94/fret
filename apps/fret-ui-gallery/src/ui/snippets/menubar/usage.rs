pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let file = shadcn::MenubarTrigger::new("File")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new(),
            [
                shadcn::MenubarItem::new("New Tab")
                    .trailing(shadcn::MenubarShortcut::new("⌘T").into_element(cx))
                    .into(),
                shadcn::MenubarItem::new("New Window").into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Share").into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Print").into(),
            ],
        );

    shadcn::Menubar::new([file]).into_element(cx)
}
// endregion: example
