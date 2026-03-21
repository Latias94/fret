pub const SOURCE: &str = include_str!("parts.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn panel(
    cx: &mut UiCx<'_>,
    title: &'static str,
    description: &'static str,
    action_label: &'static str,
) -> AnyElement {
    ui::v_flex(move |cx| {
        ui::children![
            cx;
            shadcn::card(|cx| {
                ui::children![
                    cx;
                    shadcn::card_header(|cx| {
                        ui::children![
                            cx;
                            shadcn::card_title(title),
                            shadcn::card_description(description),
                        ]
                    }),
                    shadcn::card_footer(|cx| {
                        ui::children![cx; shadcn::Button::new(action_label)]
                    }),
                ]
            }),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let overview_panel = panel(
        cx,
        "Overview",
        "Use `TabsRoot` when you want explicit ownership of the list and content slots.",
        "Open overview",
    );
    let activity_panel = panel(
        cx,
        "Activity",
        "Custom trigger children stay caller-owned without adding a separate root `children([...])` API.",
        "Open activity",
    );

    shadcn::TabsRoot::uncontrolled(Some("overview"))
        .list(
            shadcn::TabsList::new()
                .trigger(
                    shadcn::TabsTrigger::new("overview", "Overview")
                        .test_id("ui-gallery-tabs-parts-trigger-overview"),
                )
                .trigger(
                    shadcn::TabsTrigger::new("activity", "Activity")
                        .children([
                            cx.text("Activity"),
                            shadcn::Badge::new("2").into_element(cx),
                        ])
                        .test_id("ui-gallery-tabs-parts-trigger-activity"),
                ),
        )
        .contents([
            shadcn::TabsContent::new("overview", [overview_panel]),
            shadcn::TabsContent::new("activity", [activity_panel]),
        ])
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(460.0))
                .min_w_0(),
        )
        .test_id("ui-gallery-tabs-parts")
        .into_element(cx)
}

// endregion: example
