pub const SOURCE: &str = include_str!("no_close_button.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Sheet::new_controllable(cx, None, false)
        .children([
            shadcn::SheetPart::trigger(shadcn::SheetTrigger::build(
                shadcn::Button::new("No Close Button")
                    .variant(shadcn::ButtonVariant::Outline),
            )),
            shadcn::SheetPart::content(
                shadcn::SheetContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        shadcn::SheetHeader::build(|cx, out| {
                            out.push_ui(cx, shadcn::SheetTitle::new("Custom Close"));
                            out.push_ui(
                                cx,
                                shadcn::SheetDescription::new(
                                    "This sheet has no default close button. Use the footer buttons instead.",
                                ),
                            );
                        }),
                    );
                    out.push_ui(
                        cx,
                        shadcn::SheetFooter::build(|cx, out| {
                            let cancel = shadcn::SheetClose::from_scope().build(
                                cx,
                                shadcn::Button::new("Cancel")
                                    .variant(shadcn::ButtonVariant::Outline),
                            );
                            out.push(cancel);
                            out.push_ui(cx, shadcn::Button::new("Save"));
                        }),
                    );
                })
                .show_close_button(false),
            ),
        ])
        .into_element(cx)
        .test_id("ui-gallery-sheet-no-close-button")
}
// endregion: example
