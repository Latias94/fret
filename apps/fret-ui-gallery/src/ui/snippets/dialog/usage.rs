pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Dialog::new_controllable(cx, None, false)
        .children([
            shadcn::DialogPart::trigger(shadcn::DialogTrigger::build(
                shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dialog-usage-trigger"),
            )),
            shadcn::DialogPart::content_with(|cx| {
                shadcn::DialogContent::new([]).with_children(cx, |cx| {
                    vec![
                        shadcn::DialogHeader::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::DialogTitle::new("Are you absolutely sure?")
                                    .into_element(cx),
                                shadcn::DialogDescription::new(
                                    "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                                )
                                .into_element(cx),
                            ]
                        }),
                    ]
                })
                .test_id("ui-gallery-dialog-usage-content")
            }),
        ])
        .into_element(cx)
}
// endregion: example
