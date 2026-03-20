pub const SOURCE: &str = include_str!("small.rs");

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
                    .test_id("ui-gallery-alert-dialog-small-trigger"),
            )),
            shadcn::AlertDialogPart::content(
                shadcn::AlertDialogContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        shadcn::AlertDialogHeader::build(|cx, out| {
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogTitle::new("Allow accessory to connect?"),
                            );
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogDescription::new(
                                    "Do you want to allow the USB accessory to connect to this device?",
                                ),
                            );
                        }),
                    );
                    out.push_ui(
                        cx,
                        shadcn::AlertDialogFooter::build(|cx, out| {
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogCancel::from_scope("Don't allow")
                                    .test_id("ui-gallery-alert-dialog-small-cancel"),
                            );
                            out.push_ui(
                                cx,
                                shadcn::AlertDialogAction::from_scope("Allow")
                                    .test_id("ui-gallery-alert-dialog-small-action"),
                            );
                        }),
                    );
                })
                .size(shadcn::AlertDialogContentSize::Sm)
                .test_id("ui-gallery-alert-dialog-small-content"),
            ),
        ])
        .into_element(cx)
}
// endregion: example
