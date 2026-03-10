pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::Drawer::new_controllable(cx, None, false)
        .compose()
        .trigger(shadcn::DrawerTrigger::build(
            shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline),
        ))
        .content_with(move |cx| {
            shadcn::DrawerContent::new([
                shadcn::DrawerHeader::new([
                    shadcn::DrawerTitle::new("Are you absolutely sure?").into_element(cx),
                    shadcn::DrawerDescription::new("This action cannot be undone.")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DrawerFooter::new([
                    shadcn::Button::new("Submit").into_element(cx),
                    shadcn::DrawerClose::from_scope().build(
                        cx,
                        shadcn::Button::new("Cancel").variant(shadcn::ButtonVariant::Outline),
                    ),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        })
        .into_element(cx)
}
// endregion: example
