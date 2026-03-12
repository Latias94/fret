pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model(|| false);

    let trigger = shadcn::SheetTrigger::build(
        shadcn::Button::new("Open (Parts)")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-sheet-parts-trigger"),
    );

    shadcn::Sheet::new(open)
        .compose()
        .trigger(trigger)
        .portal(shadcn::SheetPortal::new())
        .overlay(shadcn::SheetOverlay::new())
        .content_with(move |cx| {
            shadcn::SheetContent::new([
                shadcn::SheetHeader::new([
                    shadcn::SheetTitle::new("Parts sheet").into_element(cx),
                    shadcn::SheetDescription::new(
                        "Part surface adapter for shadcn-style authoring.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::SheetFooter::new([shadcn::SheetClose::from_scope()
                    .into_element(cx)
                    .test_id("ui-gallery-sheet-parts-close")])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id("ui-gallery-sheet-parts-overlay-content")
        })
        .into_element(cx)
        .test_id("ui-gallery-sheet-parts-overlay")
}
// endregion: example
