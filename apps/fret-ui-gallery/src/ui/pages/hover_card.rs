use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_hover_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let profile_card = |cx: &mut ElementContext<'_, App>,
                        name: &'static str,
                        desc: &'static str,
                        test_id: &'static str| {
        shadcn::HoverCardContent::new(vec![
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![
                    shadcn::CardTitle::new(name).into_element(cx),
                    shadcn::CardDescription::new(desc).into_element(cx),
                ])
                .into_element(cx),
                shadcn::CardContent::new(vec![shadcn::typography::muted(
                    cx,
                    "Joined December 2021",
                )])
                .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_px(Px(260.0)))
            .into_element(cx),
        ])
        .into_element(cx)
        .test_id(test_id)
    };

    let demo = {
        let trigger = shadcn::Button::new("Hover Here")
            .variant(shadcn::ButtonVariant::Link)
            .test_id("ui-gallery-hover-card-demo-trigger")
            .into_element(cx);
        let content = profile_card(
            cx,
            "@nextjs",
            "The React framework created and maintained by @vercel.",
            "ui-gallery-hover-card-demo-content",
        );
        let hover = shadcn::HoverCard::new(trigger, content)
            .open_delay_frames(10)
            .close_delay_frames(100)
            .into_element(cx)
            .test_id("ui-gallery-hover-card-demo");
        hover
    };

    let trigger_delays = {
        let instant = shadcn::HoverCard::new(
            shadcn::Button::new("Instant")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-delay-instant-trigger")
                .into_element(cx),
            profile_card(
                cx,
                "Instant",
                "openDelay=0 closeDelay=0",
                "ui-gallery-hover-card-delay-instant-content",
            ),
        )
        .open_delay_frames(0)
        .close_delay_frames(0)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-delay-instant");

        let delayed = shadcn::HoverCard::new(
            shadcn::Button::new("Delayed")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-delay-delayed-trigger")
                .into_element(cx),
            profile_card(
                cx,
                "Delayed",
                "openDelay=16 closeDelay=12",
                "ui-gallery-hover-card-delay-delayed-content",
            ),
        )
        .open_delay_frames(16)
        .close_delay_frames(12)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-delay-delayed");

        let content = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N3).items_center(),
            |_cx| vec![instant, delayed],
        );

        content
    };

    let positioning = {
        let top_start = shadcn::HoverCard::new(
            shadcn::Button::new("Top / Start")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-pos-top-start-trigger")
                .into_element(cx),
            profile_card(
                cx,
                "Top Start",
                "side=top align=start",
                "ui-gallery-hover-card-pos-top-start-content",
            ),
        )
        .side(shadcn::HoverCardSide::Top)
        .align(shadcn::HoverCardAlign::Start)
        .open_delay_frames(8)
        .close_delay_frames(8)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-pos-top-start");

        let right_end = shadcn::HoverCard::new(
            shadcn::Button::new("Right / End")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-pos-right-end-trigger")
                .into_element(cx),
            profile_card(
                cx,
                "Right End",
                "side=right align=end",
                "ui-gallery-hover-card-pos-right-end-content",
            ),
        )
        .side(shadcn::HoverCardSide::Right)
        .align(shadcn::HoverCardAlign::End)
        .open_delay_frames(8)
        .close_delay_frames(8)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-pos-right-end");

        let content = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N3).items_center(),
            |_cx| vec![top_start, right_end],
        );

        content
    };

    let basic = {
        let trigger = shadcn::Button::new("Basic")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-basic-trigger")
            .into_element(cx);
        let content = profile_card(
            cx,
            "Basic",
            "Simple hover preview card.",
            "ui-gallery-hover-card-basic-content",
        );
        let hover = shadcn::HoverCard::new(trigger, content)
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx)
            .test_id("ui-gallery-hover-card-basic");
        hover
    };

    let sides = {
        let left = shadcn::HoverCard::new(
            shadcn::Button::new("left")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-side-left-trigger")
                .into_element(cx),
            profile_card(
                cx,
                "Left",
                "Appears on the left side.",
                "ui-gallery-hover-card-side-left-content",
            ),
        )
        .side(shadcn::HoverCardSide::Left)
        .open_delay_frames(10)
        .close_delay_frames(10)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-side-left");

        let top = shadcn::HoverCard::new(
            shadcn::Button::new("top")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-side-top-trigger")
                .into_element(cx),
            profile_card(
                cx,
                "Top",
                "Appears on the top side.",
                "ui-gallery-hover-card-side-top-content",
            ),
        )
        .side(shadcn::HoverCardSide::Top)
        .open_delay_frames(10)
        .close_delay_frames(10)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-side-top");

        let bottom = shadcn::HoverCard::new(
            shadcn::Button::new("bottom")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-side-bottom-trigger")
                .into_element(cx),
            profile_card(
                cx,
                "Bottom",
                "Appears on the bottom side.",
                "ui-gallery-hover-card-side-bottom-content",
            ),
        )
        .side(shadcn::HoverCardSide::Bottom)
        .open_delay_frames(10)
        .close_delay_frames(10)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-side-bottom");

        let right = shadcn::HoverCard::new(
            shadcn::Button::new("right")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-side-right-trigger")
                .into_element(cx),
            profile_card(
                cx,
                "Right",
                "Appears on the right side.",
                "ui-gallery-hover-card-side-right-content",
            ),
        )
        .side(shadcn::HoverCardSide::Right)
        .open_delay_frames(10)
        .close_delay_frames(10)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-side-right");

        let content = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![left, top, bottom, right],
        );

        content
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::HoverCard::new(
                    shadcn::Button::new("??? ??????")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-hover-card-rtl-trigger")
                        .into_element(cx),
                    profile_card(
                        cx,
                        "????? ??????",
                        "??? ???? RTL ??????? ???????.",
                        "ui-gallery-hover-card-rtl-content",
                    ),
                )
                .open_delay_frames(10)
                .close_delay_frames(10)
                .side(shadcn::HoverCardSide::Left)
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-hover-card-rtl");

        rtl_content
    };

    let notes = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Hover card interactions depend on hover-intent delays, so examples include both instant and delayed scenarios.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Sides and positioning are separated to make placement parity checks deterministic.",
                ),
                shadcn::typography::muted(
                    cx,
                    "RTL sample is included because side resolution can differ in right-to-left layouts.",
                ),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Hover Card docs order: Demo, Trigger Delays, Positioning, Basic, Sides, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Basic hover card surface with copy and delayed close.")
                .code(
                    "rust",
                    r#"let hover = shadcn::HoverCard::new(trigger, content)
    .open_delay_frames(10)
    .close_delay_frames(100)
    .into_element(cx);"#,
                ),
            DocSection::new("Trigger Delays", trigger_delays)
                .description("Compare instant vs delayed open/close behavior."),
            DocSection::new("Positioning", positioning)
                .description("Placement is controlled by `side` and `align`.")
                .code(
                    "rust",
                    r#"shadcn::HoverCard::new(trigger, content)
    .side(shadcn::HoverCardSide::Top)
    .align(shadcn::HoverCardAlign::Start);"#,
                ),
            DocSection::new("Basic", basic)
                .description("A minimal hover card with shorter delays."),
            DocSection::new("Sides", sides).description("Visual sweep of side placements."),
            DocSection::new("RTL", rtl)
                .description("Hover card should respect right-to-left direction context."),
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines."),
        ],
    );

    vec![body]
}
