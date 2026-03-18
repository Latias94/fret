pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model(|| false);
    let open_for_trigger = open.clone();
    let open_for_close = open.clone();

    with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
        shadcn::Drawer::new(open.clone())
            .children([
                shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                    shadcn::Button::new("Open RTL Drawer")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(open_for_trigger.clone())
                        .test_id("ui-gallery-drawer-rtl-trigger"),
                )),
                shadcn::DrawerPart::content_with(move |cx| {
                    shadcn::DrawerContent::new([
                        shadcn::DrawerHeader::new([
                            shadcn::DrawerTitle::new("RTL Drawer").into_element(cx),
                            shadcn::DrawerDescription::new(
                                "Drawer layout should follow right-to-left direction context.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::DrawerFooter::new([shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(open_for_close.clone())
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id("ui-gallery-drawer-rtl-content")
                }),
            ])
            .into_element(cx)
    })
}
// endregion: example
