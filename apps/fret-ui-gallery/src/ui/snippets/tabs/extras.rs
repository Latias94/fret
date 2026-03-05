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
                    shadcn::TabsItem::new(
                        "preview",
                        "Preview",
                        vec![
                            shadcn::typography::muted(cx, "Preview panel (RTL keynav gate).")
                                .test_id("ui-gallery-tabs-rtl-panel-preview"),
                        ],
                    )
                    .trigger_test_id("ui-gallery-tabs-rtl-trigger-preview"),
                    shadcn::TabsItem::new(
                        "code",
                        "Code",
                        vec![
                            shadcn::typography::muted(cx, "Code panel (RTL keynav gate).")
                                .test_id("ui-gallery-tabs-rtl-panel-code"),
                        ],
                    )
                    .trigger_test_id("ui-gallery-tabs-rtl-trigger-code"),
                ])
                .into_element(cx)
        })
        .test_id("ui-gallery-tabs-rtl");

    ui::v_flex(move |_cx| vec![muted, flex_1_triggers, rtl])
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-tabs-extras")
}

// endregion: example
