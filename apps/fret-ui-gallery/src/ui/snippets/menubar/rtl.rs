pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let file = shadcn::MenubarMenu::new("ملف")
            .test_id("ui-gallery-menubar-rtl-file")
            .entries([
                shadcn::MenubarEntry::Item(
                    shadcn::MenubarItem::new("علامة تبويب جديدة")
                        .test_id("ui-gallery-menubar-rtl-new-tab")
                        .trailing(shadcn::MenubarShortcut::new("⌘T").into_element(cx)),
                ),
                shadcn::MenubarEntry::Item(
                    shadcn::MenubarItem::new("نافذة جديدة")
                        .test_id("ui-gallery-menubar-rtl-new-window")
                        .trailing(shadcn::MenubarShortcut::new("⌘N").into_element(cx)),
                ),
                shadcn::MenubarEntry::Separator,
                shadcn::MenubarEntry::Submenu(
                    shadcn::MenubarItem::new("المزيد")
                        .test_id("ui-gallery-menubar-rtl-more")
                        .submenu([
                            shadcn::MenubarEntry::Item(
                                shadcn::MenubarItem::new("Sub Alpha")
                                    .test_id("ui-gallery-menubar-rtl-sub-alpha"),
                            ),
                            shadcn::MenubarEntry::Item(
                                shadcn::MenubarItem::new("Sub Beta")
                                    .test_id("ui-gallery-menubar-rtl-sub-beta"),
                            ),
                        ]),
                ),
                shadcn::MenubarEntry::Separator,
                shadcn::MenubarEntry::Item(
                    shadcn::MenubarItem::new("طباعة...")
                        .test_id("ui-gallery-menubar-rtl-print")
                        .trailing(shadcn::MenubarShortcut::new("⌘P").into_element(cx)),
                ),
            ]);
        shadcn::Menubar::new([file])
            .refine_layout(width.clone())
            .into_element(cx)
    })
}
// endregion: example
