pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::AlertDialog::new(open)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Default")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-alert-dialog-basic-trigger"),
            )),
            shadcn::AlertDialogPart::content(
                shadcn::AlertDialogContent::build(|cx, out| {
                    out.push(
                        shadcn::AlertDialogHeader::new([
                            shadcn::AlertDialogTitle::new("Are you absolutely sure?")
                                .into_element(cx),
                            shadcn::AlertDialogDescription::new(
                                "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                    );
                    out.push(
                        shadcn::AlertDialogFooter::new([
                            shadcn::AlertDialogCancel::from_scope("Cancel")
                                .test_id("ui-gallery-alert-dialog-basic-cancel")
                                .into_element(cx),
                            shadcn::AlertDialogAction::from_scope("Continue")
                                .test_id("ui-gallery-alert-dialog-basic-action")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    );
                })
                .test_id("ui-gallery-alert-dialog-basic-content"),
            ),
        ])
        .into_element(cx)
}
// endregion: example
