pub const SOURCE: &str = include_str!("no_close_button.rs");

// region: example
use fret::children::UiElementSinkExt;
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
                shadcn::SheetContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        shadcn::SheetHeader::build(|cx, out| {
                            out.push_ui(cx, shadcn::SheetTitle::new("No Close Button"));
                            out.push_ui(
                                cx,
                                shadcn::SheetDescription::new(
                                    "This example intentionally omits footer actions. Use outside press or Escape to close.",
                                ),
                            );
                        }),
                    );
                })
                .show_close_button(false)
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-sheet-no-close-button")
}
// endregion: example
