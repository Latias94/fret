pub const SOURCE: &str = include_str!("snap_points.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Drawer::new_controllable(cx, None, false)
        .snap_points(vec![
            shadcn::DrawerSnapPoint::Fraction(0.25),
            shadcn::DrawerSnapPoint::Fraction(0.5),
            shadcn::DrawerSnapPoint::Fraction(1.0),
        ])
        .children([
            shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                shadcn::Button::new("Snap Points")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-drawer-snap-points-trigger"),
            )),
            shadcn::DrawerPart::content_with(move |cx| {
                shadcn::DrawerContent::new([])
                    .drag_handle_test_id("ui-gallery-drawer-snap-points-handle")
                    .children(|cx| {
                        ui::children![
                            cx;
                            shadcn::DrawerHeader::new([]).children(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::DrawerTitle::new("Snap Points"),
                                    shadcn::DrawerDescription::new(
                                        "Releasing a drag settles to the nearest snap point (Vaul-style).",
                                    )
                                ]
                            }),
                            shadcn::DrawerFooter::new([]).children(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::DrawerClose::from_scope().child(
                                        shadcn::Button::new("Close")
                                            .variant(shadcn::ButtonVariant::Outline)
                                            .test_id("ui-gallery-drawer-snap-points-close"),
                                    )
                                ]
                            })
                        ]
                    })
                    .test_id("ui-gallery-drawer-snap-points-content")
                    .into_element(cx)
            }),
        ])
        .into_element(cx)
}
// endregion: example
