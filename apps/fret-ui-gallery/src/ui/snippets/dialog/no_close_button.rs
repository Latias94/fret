pub const SOURCE: &str = include_str!("no_close_button.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::Dialog::new(open.clone())
        .children([
            shadcn::DialogPart::trigger(shadcn::DialogTrigger::build(
                shadcn::Button::new("No Close Button")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dialog-no-close-trigger"),
            )),
            shadcn::DialogPart::content_with(|cx| {
                shadcn::DialogContent::new([])
                    .show_close_button(false)
                    .with_children(cx, |cx| {
                        vec![
                            shadcn::DialogHeader::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DialogTitle::new("No Close Button").into_element(cx),
                                    shadcn::DialogDescription::new(
                                        "This dialog omits explicit close controls and relies on Escape or overlay dismissal.",
                                    )
                                    .into_element(cx),
                                ]
                            }),
                        ]
                    })
                    .test_id("ui-gallery-dialog-no-close-content")
            }),
        ])
        .into_element(cx)
}
// endregion: example
