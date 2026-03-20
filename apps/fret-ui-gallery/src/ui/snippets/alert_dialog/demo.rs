pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::AlertDialog::new(open)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Show Dialog")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-alert-dialog-demo-trigger"),
            )),
            shadcn::AlertDialogPart::content(shadcn::AlertDialogContent::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::AlertDialogHeader::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogTitle::new("Are you absolutely sure?"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogDescription::new(
                                "This action cannot be undone. This will permanently delete your account from our servers.",
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
                                .test_id("ui-gallery-alert-dialog-demo-cancel"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::AlertDialogAction::from_scope("Continue")
                                .test_id("ui-gallery-alert-dialog-demo-action"),
                        );
                    }),
                );
            })
            .test_id("ui-gallery-alert-dialog-demo-content")),
        ])
        .into_element(cx)
}
// endregion: example
