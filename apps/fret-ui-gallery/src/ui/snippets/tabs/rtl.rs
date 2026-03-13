pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl)
        .into_element(cx, |cx| {
            shadcn::tabs_uncontrolled(cx, Some("preview"), |cx| {
                [
                    shadcn::TabsItem::new(
                        "preview",
                        "Preview",
                        vec![
                            shadcn::raw::typography::muted("Preview panel (RTL keynav gate).")
                                .into_element(cx)
                                .test_id("ui-gallery-tabs-rtl-panel-preview"),
                        ],
                    )
                    .trigger_test_id("ui-gallery-tabs-rtl-trigger-preview"),
                    shadcn::TabsItem::new(
                        "code",
                        "Code",
                        vec![
                            shadcn::raw::typography::muted("Code panel (RTL keynav gate).")
                                .into_element(cx)
                                .test_id("ui-gallery-tabs-rtl-panel-code"),
                        ],
                    )
                    .trigger_test_id("ui-gallery-tabs-rtl-trigger-code"),
                ]
            })
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .into_element(cx)
        })
        .test_id("ui-gallery-tabs-rtl")
}

// endregion: example
