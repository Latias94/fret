pub const SOURCE: &str = include_str!("with_icons.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    let icon_id = |id: &'static str| IconId::new_static(id);

    let file = shadcn::MenubarMenu::new("File")
        .test_id("ui-gallery-menubar-with-icons-file")
        .entries([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("New File")
                    .leading_icon(icon_id("lucide.file"))
                    .test_id("ui-gallery-menubar-with-icons-new-file")
                    .trailing(shadcn::MenubarShortcut::new("⌘N").into_element(cx)),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Open Folder")
                    .leading_icon(icon_id("lucide.folder"))
                    .test_id("ui-gallery-menubar-with-icons-open-folder"),
            ),
            shadcn::MenubarEntry::Separator,
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Save")
                    .leading_icon(icon_id("lucide.save"))
                    .test_id("ui-gallery-menubar-with-icons-save")
                    .trailing(shadcn::MenubarShortcut::new("⌘S").into_element(cx)),
            ),
        ]);

    let more = shadcn::MenubarMenu::new("More")
        .test_id("ui-gallery-menubar-with-icons-more")
        .entries([shadcn::MenubarEntry::Group(shadcn::MenubarGroup::new([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Settings")
                    .leading_icon(icon_id("lucide.settings"))
                    .test_id("ui-gallery-menubar-with-icons-settings"),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Help")
                    .leading_icon(icon_id("lucide.info"))
                    .test_id("ui-gallery-menubar-with-icons-help"),
            ),
            shadcn::MenubarEntry::Separator,
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Delete")
                    .leading_icon(icon_id("lucide.trash"))
                    .test_id("ui-gallery-menubar-with-icons-delete")
                    .variant(shadcn::menubar::MenubarItemVariant::Destructive),
            ),
        ]))]);

    shadcn::Menubar::new([file, more])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
