pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();

    let file = shadcn::MenubarTrigger::new("File")
        .test_id("ui-gallery-menubar-parts-trigger-file")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new(),
            [
                shadcn::MenubarItem::new("New Tab")
                    .action(CommandId::new("ui_gallery.menubar.parts.new_tab"))
                    .test_id("ui-gallery-menubar-parts-item-new-tab")
                    .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarSub::new(
                    shadcn::MenubarSubTrigger::new("Share")
                        .refine(|item| item.test_id("ui-gallery-menubar-parts-item-share")),
                    shadcn::MenubarSubContent::new([
                        shadcn::MenubarItem::new("Email link")
                            .action(CommandId::new("ui_gallery.menubar.parts.share.email_link"))
                            .test_id("ui-gallery-menubar-parts-sub-email")
                            .into(),
                        shadcn::MenubarItem::new("Messages")
                            .action(CommandId::new("ui_gallery.menubar.parts.share.messages"))
                            .test_id("ui-gallery-menubar-parts-sub-messages")
                            .into(),
                    ]),
                )
                .into(),
            ],
        );

    shadcn::Menubar::new([file])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
