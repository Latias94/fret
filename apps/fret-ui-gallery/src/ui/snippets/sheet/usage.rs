pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Sheet::new_controllable(cx, None, false)
        .compose()
        .trigger(shadcn::SheetTrigger::build(
            shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline),
        ))
        .content_with(move |cx| {
            shadcn::SheetContent::new([shadcn::SheetHeader::new([
                shadcn::SheetTitle::new("Are you absolutely sure?").into_element(cx),
                shadcn::SheetDescription::new("This action cannot be undone.").into_element(cx),
            ])
            .into_element(cx)])
            .into_element(cx)
        })
        .into_element(cx)
}
// endregion: example
