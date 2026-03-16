pub const SOURCE: &str = include_str!("small.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::AlertDialog::new(open)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Small")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-alert-dialog-small-trigger"),
            )),
            shadcn::AlertDialogPart::content(
                shadcn::AlertDialogContent::build(|cx, out| {
                    out.push(
                        shadcn::AlertDialogHeader::new([
                            shadcn::AlertDialogTitle::new("Allow accessory to connect?")
                                .into_element(cx),
                            shadcn::AlertDialogDescription::new(
                                "Do you want to allow the USB accessory to connect to this device?",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    );
                    out.push(
                        shadcn::AlertDialogFooter::new([
                            shadcn::AlertDialogCancel::from_scope("Don't allow")
                                .test_id("ui-gallery-alert-dialog-small-cancel")
                                .into_element(cx),
                            shadcn::AlertDialogAction::from_scope("Allow")
                                .test_id("ui-gallery-alert-dialog-small-action")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    );
                })
                .size(shadcn::AlertDialogContentSize::Sm)
                .test_id("ui-gallery-alert-dialog-small-content"),
            ),
        ])
        .into_element(cx)
}
// endregion: example
