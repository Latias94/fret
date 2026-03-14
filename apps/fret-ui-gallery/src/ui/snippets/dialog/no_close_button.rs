pub const SOURCE: &str = include_str!("no_close_button.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    let open_for_trigger = open.clone();

    shadcn::Dialog::new(open.clone()).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("No Close Button")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-no-close-trigger")
                .toggle_model(open_for_trigger.clone())
                .into_element(cx)
        },
        move |cx| {
            shadcn::DialogContent::new([shadcn::DialogHeader::new([
                shadcn::DialogTitle::new("No Close Button").into_element(cx),
                shadcn::DialogDescription::new(
                    "This dialog omits explicit close controls and relies on Escape or overlay dismissal.",
                )
                .into_element(cx),
            ])
            .into_element(cx)])
            .show_close_button(false)
            .into_element(cx)
            .test_id("ui-gallery-dialog-no-close-content")
        },
    )
}
// endregion: example
