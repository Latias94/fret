pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let flex_1_triggers = shadcn::Tabs::uncontrolled(Some(Arc::<str>::from("overview")))
        .list_full_width(true)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("overview", "Overview", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-flex1-trigger-overview"),
            shadcn::TabsItem::new("analytics", "Analytics", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-flex1-trigger-analytics"),
            shadcn::TabsItem::new("reports", "Reports", Vec::<AnyElement>::new())
                .trigger_test_id("ui-gallery-tabs-flex1-trigger-reports"),
        ])
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
