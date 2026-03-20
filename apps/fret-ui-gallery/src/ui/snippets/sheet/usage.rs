pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Sheet::new_controllable(cx, None, false)
        .compose()
        .trigger(shadcn::SheetTrigger::build(
            shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline),
        ))
        .content_with(move |cx| {
            shadcn::SheetContent::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::SheetHeader::build(|cx, out| {
                        out.push_ui(cx, shadcn::SheetTitle::new("Are you absolutely sure?"));
                        out.push_ui(
                            cx,
                            shadcn::SheetDescription::new("This action cannot be undone."),
                        );
                    }),
                );
            })
            .into_element(cx)
        })
        .into_element(cx)
}
// endregion: example
