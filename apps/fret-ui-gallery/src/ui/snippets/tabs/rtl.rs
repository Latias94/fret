pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn metric_card(
    title: &'static str,
    description: &'static str,
    content: &'static str,
) -> impl IntoUiElement<fret_app::App> + use<> {
    shadcn::card(move |cx| {
        ui::children![
            cx;
            shadcn::card_header(move |cx| {
                ui::children![
                    cx;
                    shadcn::card_title(title),
                    shadcn::card_description(description),
                ]
            }),
            shadcn::card_content(move |cx| {
                ui::children![cx; shadcn::raw::typography::muted(content)]
            }),
        ]
    })
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl)
        .into_element(cx, |cx| {
            shadcn::tabs_uncontrolled(cx, Some("overview"), |cx| {
                [
                    shadcn::TabsItem::new(
                        "overview",
                        "Overview",
                        [metric_card(
                            "Overview",
                            "View your key metrics and recent project activity.",
                            "You have 12 active projects and 3 pending tasks.",
                        )
                        .into_element(cx)],
                    )
                    .trigger_test_id("ui-gallery-tabs-rtl-trigger-overview"),
                    shadcn::TabsItem::new(
                        "analytics",
                        "Analytics",
                        [metric_card(
                            "Analytics",
                            "Track performance and user engagement trends.",
                            "Page views are up 25% compared to last month.",
                        )
                        .into_element(cx)],
                    )
                    .trigger_test_id("ui-gallery-tabs-rtl-trigger-analytics"),
                    shadcn::TabsItem::new(
                        "reports",
                        "Reports",
                        [metric_card(
                            "Reports",
                            "Generate and export detailed reports for analysis.",
                            "You have 5 reports ready to export.",
                        )
                        .into_element(cx)],
                    )
                    .trigger_test_id("ui-gallery-tabs-rtl-trigger-reports"),
                    shadcn::TabsItem::new(
                        "settings",
                        "Settings",
                        [metric_card(
                            "Settings",
                            "Manage account preferences and experience options.",
                            "Configure notifications, security, and themes.",
                        )
                        .into_element(cx)],
                    )
                    .trigger_test_id("ui-gallery-tabs-rtl-trigger-settings"),
                ]
            })
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .max_w(Px(384.0))
                    .min_w_0(),
            )
            .into_element(cx)
        })
        .test_id("ui-gallery-tabs-rtl")
}

// endregion: example
