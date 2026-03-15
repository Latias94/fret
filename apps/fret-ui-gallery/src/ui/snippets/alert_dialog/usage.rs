pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::AlertDialog::new_controllable(cx, None, false)
        .compose()
        .trigger(shadcn::AlertDialogTrigger::build(
            shadcn::Button::new("Show Dialog").variant(shadcn::ButtonVariant::Outline),
        ))
        .content_with(move |cx| {
            shadcn::AlertDialogContent::new([
                shadcn::AlertDialogHeader::new([
                    shadcn::AlertDialogTitle::new("Are you absolutely sure?").into_element(cx),
                    shadcn::AlertDialogDescription::new(
                        "This action cannot be undone. This will permanently delete your account from our servers.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::AlertDialogFooter::new([
                    shadcn::AlertDialogCancel::from_scope("Cancel").into_element(cx),
                    shadcn::AlertDialogAction::from_scope("Continue").into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        })
        .into_element(cx)
}
// endregion: example
