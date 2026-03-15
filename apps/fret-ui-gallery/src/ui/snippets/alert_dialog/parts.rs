pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::AlertDialog::new(open)
        .compose()
        .trigger(
            shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Show Dialog (Parts)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-alert-dialog-parts-trigger"),
            ),
        )
        .portal(shadcn::AlertDialogPortal::new())
        .overlay(shadcn::AlertDialogOverlay::new())
        .content_with(move |cx| {
            shadcn::AlertDialogContent::build(|cx, out| {
                out.push(
                    shadcn::AlertDialogHeader::build(|cx, out| {
                        out.push(
                            shadcn::AlertDialogTitle::new("Part-based AlertDialog").into_element(cx),
                        );
                        out.push(
                            shadcn::AlertDialogDescription::new(
                                "Thin adapters for shadcn-style authoring (Trigger/Portal/Overlay).",
                            )
                            .into_element(cx),
                        );
                    })
                    .into_element(cx),
                );
                out.push(
                    shadcn::AlertDialogFooter::build(|cx, out| {
                        out.push(
                            shadcn::AlertDialogCancel::from_scope("Cancel")
                                .test_id("ui-gallery-alert-dialog-parts-cancel")
                                .into_element(cx),
                        );
                        out.push(
                            shadcn::AlertDialogAction::from_scope("Continue")
                                .test_id("ui-gallery-alert-dialog-parts-action")
                                .into_element(cx),
                        );
                    })
                    .into_element(cx),
                );
            })
            .into_element(cx)
            .test_id("ui-gallery-alert-dialog-parts-content")
        })
        .into_element(cx)
}
// endregion: example
