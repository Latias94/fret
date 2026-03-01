// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();

    let file = shadcn::MenubarMenu::new("File").entries([
        shadcn::MenubarEntry::Group(shadcn::MenubarGroup::new([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("New Tab")
                    .trailing(shadcn::MenubarShortcut::new("⌘T").into_element(cx)),
            ),
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("New Window")),
        ])),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Group(shadcn::MenubarGroup::new([
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Share").close_on_select(false)),
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Print").close_on_select(false)),
        ])),
    ]);

    shadcn::Menubar::new([file])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example

