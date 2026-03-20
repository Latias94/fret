pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::children::UiElementSinkExt;
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
            shadcn::DialogPart::content(shadcn::DialogContent::build(|cx, out| {
                out.push_ui(
                    cx,
                    shadcn::DialogHeader::build(|cx, out| {
                        out.push_ui(
                            cx,
                            shadcn::DialogTitle::new("Are you absolutely sure?"),
                        );
                        out.push_ui(
                            cx,
                            shadcn::DialogDescription::new(
                                "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                            ),
                        );
                    }),
                );
            })
            .test_id("ui-gallery-dialog-usage-content")),
        ])
        .into_element(cx)
}
// endregion: example
