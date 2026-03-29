pub const SOURCE: &str = include_str!("use_direction.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn direction_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    local: Option<shadcn::LayoutDirection>,
    test_id: &'static str,
) -> AnyElement {
    let resolved = shadcn::use_direction(cx, local);
    let resolved_label = match resolved {
        shadcn::LayoutDirection::Ltr => "ltr",
        shadcn::LayoutDirection::Rtl => "rtl",
    };

    ui::h_flex(|cx| {
        vec![
            shadcn::Badge::new(label)
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            cx.text(format!("resolved = {resolved_label}")),
        ]
    })
    .gap(Space::N3)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui::v_stack(|cx| {
        vec![
            direction_row(
                cx,
                "Default fallback",
                None,
                "ui-gallery-direction-use-direction-default",
            ),
            shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl).into_element(cx, |cx| {
                ui::v_stack(|cx| {
                    vec![
                        direction_row(
                            cx,
                            "Inherited scope",
                            None,
                            "ui-gallery-direction-use-direction-inherited",
                        ),
                        direction_row(
                            cx,
                            "Local override",
                            Some(shadcn::LayoutDirection::Ltr),
                            "ui-gallery-direction-use-direction-local-override",
                        ),
                    ]
                })
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx)
            }),
        ]
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
    .test_id("ui-gallery-direction-use-direction")
}
// endregion: example
