pub const SOURCE: &str = include_str!("no_close_button.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model(|| false);

    let trigger_open = open.clone();

    shadcn::Sheet::new(open.clone())
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open Sheet")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(trigger_open.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::SheetContent::new([shadcn::SheetHeader::new([
                    shadcn::SheetTitle::new("No Close Button").into_element(cx),
                    shadcn::SheetDescription::new(
                        "This example intentionally omits footer actions. Use outside press or Escape to close.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx)])
                .show_close_button(false)
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-sheet-no-close-button")
}
// endregion: example
