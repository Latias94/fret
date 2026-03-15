pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        ui::v_stack(|cx| {
            vec![
                shadcn::alert(|cx| {
                    ui::children![
                        cx;
                        fret_ui_shadcn::icon::icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.info"),
                        ),
                        shadcn::AlertTitle::new("RTL alert sample"),
                        shadcn::AlertDescription::new(
                            "This alert validates right-to-left layout and text alignment.",
                        ),
                    ]
                })
                .variant(shadcn::AlertVariant::Default)
                .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
                .into_element(cx)
                .test_id("ui-gallery-alert-rtl"),
            ]
        })
        .gap(Space::N3)
        .items_start()
        .into_element(cx)
    })
}
// endregion: example
