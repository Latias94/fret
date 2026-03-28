pub const SOURCE: &str = include_str!("no_close_button.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Sheet::new_controllable(cx, None, false)
        .children([
            shadcn::SheetPart::trigger(shadcn::SheetTrigger::build(
                shadcn::Button::new("Open Sheet")
                    .variant(shadcn::ButtonVariant::Outline),
            )),
            shadcn::SheetPart::content_with(|cx| {
                shadcn::SheetContent::new([])
                    .show_close_button(false)
                    .with_children(cx, |cx| {
                        vec![shadcn::SheetHeader::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::SheetTitle::new("No Close Button").into_element(cx),
                                shadcn::SheetDescription::new(
                                    "This sheet doesn't have a close button in the top-right corner. Click outside to close.",
                                )
                                .into_element(cx),
                            ]
                        })]
                    })
                    .test_id("ui-gallery-sheet-no-close-button-content")
            }),
        ])
        .into_element(cx)
        .test_id("ui-gallery-sheet-no-close-button")
}
// endregion: example
