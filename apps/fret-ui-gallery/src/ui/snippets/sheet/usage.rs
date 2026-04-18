pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Sheet::new_controllable(cx, None, false)
        .children([
            shadcn::SheetPart::trigger(shadcn::SheetTrigger::build(
                shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline),
            )),
            shadcn::SheetPart::content_with(|cx| {
                shadcn::SheetContent::new([]).with_children(cx, |cx| {
                    vec![shadcn::SheetHeader::new([]).with_children(cx, |cx| {
                        vec![
                            shadcn::SheetTitle::new("Are you absolutely sure?").into_element(cx),
                            shadcn::SheetDescription::new("This action cannot be undone.")
                                .into_element(cx),
                        ]
                    })]
                })
            }),
        ])
        .into_element(cx)
}
// endregion: example
