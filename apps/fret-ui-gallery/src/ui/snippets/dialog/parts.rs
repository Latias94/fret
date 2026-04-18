pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::UiElementTestIdExt};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
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
            shadcn::DialogContent::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::DialogHeader::build(|cx, out| {
                        out.push_ui(cx, shadcn::DialogTitle::new("Parts dialog"));
                        out.push_ui(
                            cx,
                            shadcn::DialogDescription::new(
                                "Part surface adapter for shadcn-style authoring.",
                            ),
                        );
                    }),
                );
                out.push_ui(
                    cx,
                    shadcn::DialogClose::from_scope().test_id("ui-gallery-dialog-parts-close"),
                );
            })
            .show_close_button(false)
            .into_element(cx)
            .test_id("ui-gallery-dialog-parts-content")
        })
        .into_element(cx)
}
// endregion: example
