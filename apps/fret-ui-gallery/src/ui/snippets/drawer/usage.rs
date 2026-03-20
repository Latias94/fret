pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Drawer::new_controllable(cx, None, false)
        .children([
            shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline),
            )),
            shadcn::DrawerPart::content_with(move |cx| {
                shadcn::DrawerContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        shadcn::DrawerHeader::build(|cx, out| {
                            out.push_ui(cx, shadcn::DrawerTitle::new("Are you absolutely sure?"));
                            out.push_ui(
                                cx,
                                shadcn::DrawerDescription::new("This action cannot be undone."),
                            );
                        }),
                    );
                    out.push_ui(
                        cx,
                        shadcn::DrawerFooter::build(|cx, out| {
                            out.push_ui(cx, shadcn::Button::new("Submit"));
                            let cancel = shadcn::DrawerClose::from_scope().build(
                                cx,
                                shadcn::Button::new("Cancel")
                                    .variant(shadcn::ButtonVariant::Outline),
                            );
                            out.push(cancel);
                        }),
                    );
                })
                .into_element(cx)
            }),
        ])
        .into_element(cx)
}
// endregion: example
