pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::AlertDialog::new_controllable(cx, None, false)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Show Dialog").variant(shadcn::ButtonVariant::Outline),
            )),
            shadcn::AlertDialogPart::content_with(|cx| {
                shadcn::AlertDialogContent::new([]).with_children(cx, |cx| {
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
                                shadcn::AlertDialogCancel::from_scope("Cancel").into_element(cx),
                                shadcn::AlertDialogAction::from_scope("Continue").into_element(cx),
                            ]
                        }),
                    ]
                })
            }),
        ])
        .into_element(cx)
}
// endregion: example
