pub const SOURCE: &str = include_str!("line.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    shadcn::tabs_uncontrolled(cx, Some("overview"), |_cx| {
        [
            shadcn::TabsItem::new("overview", "Overview", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-line-trigger-overview"),
            shadcn::TabsItem::new("analytics", "Analytics", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-line-trigger-analytics"),
            shadcn::TabsItem::new("reports", "Reports", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-line-trigger-reports"),
        ]
    })
    .list_variant(shadcn::TabsListVariant::Line)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
    .test_id("ui-gallery-tabs-line")
    .into_element(cx)
}

// endregion: example
