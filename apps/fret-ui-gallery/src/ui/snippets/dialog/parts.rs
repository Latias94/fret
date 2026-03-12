pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model_keyed("open", || false);

    let trigger = shadcn::DialogTrigger::new(
        shadcn::Button::new("Open Dialog (Parts)")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-dialog-parts-trigger")
            .into_element(cx),
    );

    shadcn::Dialog::new(open)
        .compose()
        .trigger(trigger)
        .portal(shadcn::DialogPortal::new())
        .overlay(shadcn::DialogOverlay::new())
        .content_with(move |cx| {
            shadcn::DialogContent::new([
                shadcn::DialogHeader::new([
                    shadcn::DialogTitle::new("Parts dialog").into_element(cx),
                    shadcn::DialogDescription::new(
                        "Part surface adapter for shadcn-style authoring.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DialogClose::from_scope()
                    .into_element(cx)
                    .test_id("ui-gallery-dialog-parts-close"),
            ])
            .show_close_button(false)
            .into_element(cx)
            .test_id("ui-gallery-dialog-parts-content")
        })
        .into_element(cx)
}
// endregion: example
