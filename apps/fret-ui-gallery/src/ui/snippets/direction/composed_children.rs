pub const SOURCE: &str = include_str!("composed_children.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_stack(|cx| {
        let mut children = vec![cx.text(
            "`DirectionProvider::with(...)` lets the provider own multiple sibling children without forcing an extra wrapper element.",
        )];
        children.push(
            ui::h_flex(|cx| {
                shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl)
                    .dir(shadcn::LayoutDirection::Rtl)
                    .with(cx, |cx| {
                        vec![
                            shadcn::Badge::new("نشط")
                                .variant(shadcn::BadgeVariant::Secondary)
                                .into_element(cx)
                                .test_id("ui-gallery-direction-composed-children-badge"),
                            shadcn::Button::new("التالي")
                                .variant(shadcn::ButtonVariant::Outline)
                                .test_id("ui-gallery-direction-composed-children-next")
                                .into_element(cx),
                            shadcn::Button::new("تم")
                                .test_id("ui-gallery-direction-composed-children-done")
                                .into_element(cx),
                        ]
                    })
            })
            .gap(Space::N2)
            .wrap()
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx),
        );
        children
    })
    .gap(Space::N3)
    .items_start()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(520.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-direction-composed-children")
}
// endregion: example
