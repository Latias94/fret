pub const SOURCE: &str = include_str!("snap_points.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
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
                shadcn::DrawerContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        shadcn::DrawerHeader::build(|cx, out| {
                            out.push_ui(cx, shadcn::DrawerTitle::new("Snap Points"));
                            out.push_ui(
                                cx,
                                shadcn::DrawerDescription::new(
                                    "Releasing a drag settles to the nearest snap point (Vaul-style).",
                                ),
                            );
                        }),
                    );
                    out.push_ui(
                        cx,
                        shadcn::DrawerFooter::build(|cx, out| {
                            let close = shadcn::DrawerClose::from_scope().build(
                                cx,
                                shadcn::Button::new("Close")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .test_id("ui-gallery-drawer-snap-points-close"),
                            );
                            out.push(close);
                        }),
                    );
                })
                .drag_handle_test_id("ui-gallery-drawer-snap-points-handle")
                .test_id("ui-gallery-drawer-snap-points-content")
                .into_element(cx)
            }),
        ])
        .into_element(cx)
}
// endregion: example
