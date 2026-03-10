pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Dialog::new_controllable(cx, None, false)
        .compose()
        .trigger(shadcn::DialogTrigger::build(
            shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline),
        ))
        .content_with(move |cx| {
            shadcn::DialogContent::new([
                shadcn::DialogHeader::new([
                    shadcn::DialogTitle::new("Are you absolutely sure?").into_element(cx),
                    shadcn::DialogDescription::new(
                        "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        })
        .into_element(cx)
}
// endregion: example
