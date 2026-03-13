pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::v_flex(|cx| {
        vec![
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    shadcn::AlertTitle::new("Success! Your changes have been saved."),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-basic-title-only"),
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    shadcn::AlertTitle::new("Success! Your changes have been saved."),
                    shadcn::AlertDescription::new("This is an alert with title and description."),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-basic-title-description"),
            shadcn::alert(|cx| {
                ui::children![
                    cx;
                    shadcn::AlertDescription::new(
                        "This one has a description only. No title. No icon.",
                    ),
                ]
            })
            .variant(shadcn::AlertVariant::Default)
            .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
            .into_element(cx)
            .test_id("ui-gallery-alert-basic-description-only"),
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-alert-basic")
}
// endregion: example
