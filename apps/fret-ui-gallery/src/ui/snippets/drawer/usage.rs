pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Drawer::new_controllable(cx, None, false)
        .children([
            shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline),
            )),
            shadcn::DrawerPart::content_with(|cx| {
                shadcn::DrawerContent::new([]).with_children(cx, |cx| {
                    vec![
                        shadcn::DrawerHeader::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::DrawerTitle::new("Are you absolutely sure?")
                                    .into_element(cx),
                                shadcn::DrawerDescription::new("This action cannot be undone.")
                                    .into_element(cx),
                            ]
                        }),
                        shadcn::DrawerFooter::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::Button::new("Submit").into_element(cx),
                                shadcn::DrawerClose::from_scope().build(
                                    cx,
                                    shadcn::Button::new("Cancel")
                                        .variant(shadcn::ButtonVariant::Outline),
                                ),
                            ]
                        }),
                    ]
                })
            }),
        ])
        .into_element(cx)
}
// endregion: example
