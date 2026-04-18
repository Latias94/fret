pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let flex_1_triggers = shadcn::tabs_uncontrolled(cx, Some("overview"), |_cx| {
        [
            shadcn::TabsItem::new("overview", "Overview", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-flex1-trigger-overview"),
            shadcn::TabsItem::new("analytics", "Analytics", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-flex1-trigger-analytics"),
            shadcn::TabsItem::new("reports", "Reports", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-flex1-trigger-reports"),
        ]
    })
    .list_full_width(true)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
    .into_element(cx)
    .test_id("ui-gallery-tabs-flex1");

    ui::v_flex(move |_cx| vec![flex_1_triggers])
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-tabs-extras")
}

// endregion: example
