pub const SOURCE: &str = include_str!("fluid_tabs_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn panel(
    title: &'static str,
    description: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    shadcn::alert(move |cx| {
        ui::children![
            cx;
            fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.sparkles")),
            shadcn::AlertTitle::new(title),
            shadcn::AlertDescription::new(description),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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
                    "Accounts",
                    "Crossfade content on selection change (semantic presence tokens).",
                )
                .into_element(cx)],
            )
            .trigger_test_id("ui-gallery-motion-presets-fluid-tabs-trigger-accounts"),
            shadcn::TabsItem::new(
                "deposits",
                "Deposits",
                [panel(
                    "Deposits",
                    "Uses a Duration-based driver so it stays stable across refresh rates.",
                )
                .into_element(cx)],
            )
            .trigger_test_id("ui-gallery-motion-presets-fluid-tabs-trigger-deposits"),
            shadcn::TabsItem::new(
                "funds",
                "Funds",
                [panel(
                    "Funds",
                    "This is intentionally not DOM/Framer Motion: same semantics, different runtime.",
                )
                .into_element(cx)],
            )
            .trigger_test_id("ui-gallery-motion-presets-fluid-tabs-trigger-funds"),
        ])
        .into_element(cx);

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Fluid tabs demo"),
                    shadcn::card_description(
                        "Shared indicator + content presence (crossfade) using semantic motion tokens.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; tabs]),
        ]
    })
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
