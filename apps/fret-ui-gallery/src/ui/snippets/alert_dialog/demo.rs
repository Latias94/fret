pub const SOURCE: &str = include_str!("demo.rs");

// region: example
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
            shadcn::AlertDialogPart::content_with(|cx| {
                shadcn::AlertDialogContent::new([])
                    .test_id("ui-gallery-alert-dialog-demo-content")
                    .with_children(cx, |cx| {
                        vec![
                            shadcn::AlertDialogHeader::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::AlertDialogTitle::new("Are you absolutely sure?")
                                        .into_element(cx),
                                    shadcn::AlertDialogDescription::new(
                                        "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                                    )
                                    .into_element(cx),
                                ]
                            }),
                            shadcn::AlertDialogFooter::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::AlertDialogCancel::from_scope("Cancel")
                                        .test_id("ui-gallery-alert-dialog-demo-cancel")
                                        .into_element(cx),
                                    shadcn::AlertDialogAction::from_scope("Continue")
                                        .test_id("ui-gallery-alert-dialog-demo-action")
                                        .into_element(cx),
                                ]
                            }),
                        ]
                    })
            }),
        ])
        .into_element(cx)
}
// endregion: example
