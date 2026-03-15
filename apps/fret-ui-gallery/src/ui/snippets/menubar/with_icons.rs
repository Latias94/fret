pub const SOURCE: &str = include_str!("with_icons.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    let icon_id = |id: &'static str| IconId::new_static(id);

    let file = shadcn::MenubarMenu::new("File")
        .test_id("ui-gallery-menubar-with-icons-file")
        .entries([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("New File")
                    .action(CommandId::new("ui_gallery.menubar.with_icons.new_file"))
                    .leading_icon(icon_id("lucide.file"))
                    .test_id("ui-gallery-menubar-with-icons-new-file")
                    .trailing(shadcn::MenubarShortcut::new("⌘N").into_element(cx)),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Open Folder")
                    .action(CommandId::new("ui_gallery.menubar.with_icons.open_folder"))
                    .leading_icon(icon_id("lucide.folder"))
                    .test_id("ui-gallery-menubar-with-icons-open-folder"),
            ),
            shadcn::MenubarEntry::Separator,
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Save")
                    .action(CommandId::new("ui_gallery.menubar.with_icons.save"))
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
                    .action(CommandId::new("ui_gallery.menubar.with_icons.settings"))
                    .leading_icon(icon_id("lucide.settings"))
                    .test_id("ui-gallery-menubar-with-icons-settings"),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Help")
                    .action(CommandId::new("ui_gallery.menubar.with_icons.help"))
                    .leading_icon(icon_id("lucide.info"))
                    .test_id("ui-gallery-menubar-with-icons-help"),
            ),
            shadcn::MenubarEntry::Separator,
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Delete")
                    .action(CommandId::new("ui_gallery.menubar.with_icons.delete"))
                    .leading_icon(icon_id("lucide.trash"))
                    .test_id("ui-gallery-menubar-with-icons-delete")
                    .variant(fret_ui_shadcn::menubar::MenubarItemVariant::Destructive),
            ),
        ]))]);

    shadcn::Menubar::new([file, more])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
