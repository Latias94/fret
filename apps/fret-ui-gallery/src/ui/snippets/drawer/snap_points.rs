pub const SOURCE: &str = include_str!("snap_points.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::Drawer::new_controllable(cx, None, false)
        .snap_points(vec![
            shadcn::DrawerSnapPoint::Fraction(0.25),
            shadcn::DrawerSnapPoint::Fraction(0.5),
            shadcn::DrawerSnapPoint::Fraction(1.0),
        ])
        .compose()
        .trigger(shadcn::DrawerTrigger::build(
            shadcn::Button::new("Snap Points")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-drawer-snap-points-trigger"),
        ))
        .content_with(move |cx| {
            shadcn::DrawerContent::new([
                shadcn::DrawerHeader::new([
                    shadcn::DrawerTitle::new("Snap Points").into_element(cx),
                    shadcn::DrawerDescription::new(
                        "Releasing a drag settles to the nearest snap point (Vaul-style).",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DrawerFooter::new([shadcn::DrawerClose::from_scope().build(
                    cx,
                    shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-drawer-snap-points-close"),
                )])
                .into_element(cx),
            ])
            .drag_handle_test_id("ui-gallery-drawer-snap-points-handle")
            .into_element(cx)
            .test_id("ui-gallery-drawer-snap-points-content")
        })
        .into_element(cx)
}
// endregion: example
