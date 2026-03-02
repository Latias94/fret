pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let muted = shadcn::typography::muted(
        cx,
        "Extras are Fret-specific regression gates (not part of upstream shadcn TabsDemo).",
    );

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

    let rtl = shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl)
        .into_element(cx, |cx| {
            shadcn::Tabs::uncontrolled(Some(Arc::<str>::from("preview")))
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
                .items([
                    shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new()),
                    shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new()),
                ])
                .into_element(cx)
        })
        .test_id("ui-gallery-tabs-rtl");

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N4)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |_cx| vec![muted, flex_1_triggers, rtl],
    )
    .test_id("ui-gallery-tabs-extras")
}

// endregion: example
