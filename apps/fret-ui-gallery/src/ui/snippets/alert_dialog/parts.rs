pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret::children::UiElementSinkExt;
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
                out.push_ui(
                    cx,
                    shadcn::AlertDialogHeader::build(|cx, out| {
                        out.push_ui(cx, shadcn::AlertDialogTitle::new("Part-based AlertDialog"));
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogDescription::new(
                                "Thin adapters for shadcn-style authoring (Trigger/Portal/Overlay).",
                            ),
                        );
                    }),
                );
                out.push_ui(
                    cx,
                    shadcn::AlertDialogFooter::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogCancel::from_scope("Cancel")
                                .test_id("ui-gallery-alert-dialog-parts-cancel"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogAction::from_scope("Continue")
                                .test_id("ui-gallery-alert-dialog-parts-action"),
                        );
                    }),
                );
            })
            .test_id("ui-gallery-alert-dialog-parts-content")
            .into_element(cx)
        })
        .into_element(cx)
}
// endregion: example
