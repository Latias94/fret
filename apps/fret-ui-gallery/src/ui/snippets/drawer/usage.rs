pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Drawer::new_controllable(cx, None, false)
        .children([
            shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                shadcn::Button::new("Open").variant(shadcn::ButtonVariant::Outline),
            )),
            shadcn::DrawerPart::content_with(|cx| {
                shadcn::DrawerContent::new([])
                    .children(|cx| {
                        ui::children![
                            cx;
                            shadcn::DrawerHeader::new([]).children(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::DrawerTitle::new("Are you absolutely sure?"),
                                    shadcn::DrawerDescription::new("This action cannot be undone.")
                                ]
                            }),
                            shadcn::DrawerFooter::new([]).children(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::Button::new("Submit"),
                                    shadcn::DrawerClose::from_scope().child(
                                        shadcn::Button::new("Cancel")
                                            .variant(shadcn::ButtonVariant::Outline),
                                    )
                                ]
                            })
                        ]
                    })
                    .into_element(cx)
            }),
        ])
        .into_element(cx)
}
// endregion: example
