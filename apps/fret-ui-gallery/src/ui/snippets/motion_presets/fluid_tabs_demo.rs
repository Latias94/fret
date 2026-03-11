pub const SOURCE: &str = include_str!("fluid_tabs_demo.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn panel(
    cx: &mut ElementContext<'_, App>,
    title: &'static str,
    description: &'static str,
) -> AnyElement {
    shadcn::Alert::new([
        fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.sparkles")),
        shadcn::AlertTitle::new(title).into_element(cx),
        shadcn::AlertDescription::new(description).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let tabs = shadcn::Tabs::uncontrolled(Some("accounts"))
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .shared_indicator_motion(true)
        .content_presence_motion(true)
        .test_id("ui-gallery-motion-presets-fluid-tabs")
        .items([
            shadcn::TabsItem::new(
                "accounts",
                "Accounts",
                [panel(
                    cx,
                    "Accounts",
                    "Crossfade content on selection change (semantic presence tokens).",
                )],
            )
            .trigger_test_id("ui-gallery-motion-presets-fluid-tabs-trigger-accounts"),
            shadcn::TabsItem::new(
                "deposits",
                "Deposits",
                [panel(
                    cx,
                    "Deposits",
                    "Uses a Duration-based driver so it stays stable across refresh rates.",
                )],
            )
            .trigger_test_id("ui-gallery-motion-presets-fluid-tabs-trigger-deposits"),
            shadcn::TabsItem::new(
                "funds",
                "Funds",
                [panel(
                    cx,
                    "Funds",
                    "This is intentionally not DOM/Framer Motion: same semantics, different runtime.",
                )],
            )
            .trigger_test_id("ui-gallery-motion-presets-fluid-tabs-trigger-funds"),
        ])
        .into_element(cx);

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Fluid tabs demo").into_element(cx),
            shadcn::CardDescription::new(
                "Shared indicator + content presence (crossfade) using semantic motion tokens.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([tabs]).into_element(cx),
    ])
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(760.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-motion-presets-fluid-tabs-demo")
}
// endregion: example
