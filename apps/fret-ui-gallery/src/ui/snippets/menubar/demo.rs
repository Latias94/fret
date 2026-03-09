pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();

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

    shadcn::Menubar::new([file])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
