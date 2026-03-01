// region: example
use fret_core::Px;
use fret_ui_kit::primitives::direction::{LayoutDirection, with_direction_provider};
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let file = shadcn::MenubarMenu::new("ملف").entries([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("علامة تبويب جديدة")
                    .trailing(shadcn::MenubarShortcut::new("⌘T").into_element(cx)),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("نافذة جديدة")
                    .trailing(shadcn::MenubarShortcut::new("⌘N").into_element(cx)),
            ),
            shadcn::MenubarEntry::Separator,
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("طباعة...")
                    .trailing(shadcn::MenubarShortcut::new("⌘P").into_element(cx)),
            ),
        ]);
        shadcn::Menubar::new([file])
            .refine_layout(width.clone())
            .into_element(cx)
    })
}
// endregion: example

