pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::AlertDialog::new_controllable(cx, None, false)
        .children([
            shadcn::AlertDialogPart::trigger(shadcn::AlertDialogTrigger::build(
                shadcn::Button::new("Show Dialog").variant(shadcn::ButtonVariant::Outline),
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
                                "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                            ),
                        );
                    }),
                );
                out.push_ui(
                    cx,
                    shadcn::AlertDialogFooter::build(|cx, out| {
                        out.push_ui(cx, shadcn::AlertDialogCancel::from_scope("Cancel"));
                        out.push_ui(cx, shadcn::AlertDialogAction::from_scope("Continue"));
                    }),
                );
            })),
        ])
        .into_element(cx)
}
// endregion: example
