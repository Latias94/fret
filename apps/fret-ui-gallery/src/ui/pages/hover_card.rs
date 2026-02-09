use super::super::*;

pub(super) fn preview_hover_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).clone();

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(840.0)),
            ),
            move |_cx| [body],
        )
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let profile_card =
        |cx: &mut ElementContext<'_, App>, name: &'static str, desc: &'static str| {
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
        };

    let demo = {
        let trigger = shadcn::Button::new("Hover Here")
            .variant(shadcn::ButtonVariant::Link)
            .into_element(cx);
        let content = profile_card(
            cx,
            "@nextjs",
            "The React framework created and maintained by @vercel.",
        );
        let hover = shadcn::HoverCard::new(trigger, content)
            .open_delay_frames(10)
            .close_delay_frames(100)
            .into_element(cx)
            .test_id("ui-gallery-hover-card-demo");
        section_card(cx, "Demo", hover)
    };

    let trigger_delays = {
        let instant = shadcn::HoverCard::new(
            shadcn::Button::new("Instant")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            profile_card(cx, "Instant", "openDelay=0 closeDelay=0"),
        )
        .open_delay_frames(0)
        .close_delay_frames(0)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-delay-instant");

        let delayed = shadcn::HoverCard::new(
            shadcn::Button::new("Delayed")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            profile_card(cx, "Delayed", "openDelay=16 closeDelay=12"),
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

        section_card(cx, "Trigger Delays", content)
    };

    let positioning = {
        let top_start = shadcn::HoverCard::new(
            shadcn::Button::new("Top / Start")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            profile_card(cx, "Top Start", "side=top align=start"),
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
                .into_element(cx),
            profile_card(cx, "Right End", "side=right align=end"),
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

        section_card(cx, "Positioning", content)
    };

    let basic = {
        let trigger = shadcn::Button::new("Basic")
            .variant(shadcn::ButtonVariant::Outline)
            .into_element(cx);
        let content = profile_card(cx, "Basic", "Simple hover preview card.");
        let hover = shadcn::HoverCard::new(trigger, content)
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx)
            .test_id("ui-gallery-hover-card-basic");
        section_card(cx, "Basic", hover)
    };

    let sides = {
        let left = shadcn::HoverCard::new(
            shadcn::Button::new("left")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            profile_card(cx, "Left", "Appears on the left side."),
        )
        .side(shadcn::HoverCardSide::Left)
        .open_delay_frames(10)
        .close_delay_frames(10)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-side-left");

        let top = shadcn::HoverCard::new(
            shadcn::Button::new("top")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            profile_card(cx, "Top", "Appears on the top side."),
        )
        .side(shadcn::HoverCardSide::Top)
        .open_delay_frames(10)
        .close_delay_frames(10)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-side-top");

        let bottom = shadcn::HoverCard::new(
            shadcn::Button::new("bottom")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            profile_card(cx, "Bottom", "Appears on the bottom side."),
        )
        .side(shadcn::HoverCardSide::Bottom)
        .open_delay_frames(10)
        .close_delay_frames(10)
        .into_element(cx)
        .test_id("ui-gallery-hover-card-side-bottom");

        let right = shadcn::HoverCard::new(
            shadcn::Button::new("right")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            profile_card(cx, "Right", "Appears on the right side."),
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

        section_card(cx, "Sides", content)
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::HoverCard::new(
                    shadcn::Button::new("??? ??????")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx),
                    profile_card(cx, "????? ??????", "??? ???? RTL ??????? ???????."),
                )
                .open_delay_frames(10)
                .close_delay_frames(10)
                .side(shadcn::HoverCardSide::Left)
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-hover-card-rtl");

        section_card(cx, "RTL", rtl_content)
    };

    let component_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Preview follows shadcn Hover Card docs order: Demo, Trigger Delays, Positioning, Basic, Sides, RTL.",
                ),
                demo,
                trigger_delays,
                positioning,
                basic,
                sides,
                rtl,
            ]
        },
    );
    let component_panel =
        shell(cx, component_panel_body).test_id("ui-gallery-hover-card-component");

    let code_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Basic Usage").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            "HoverCard::new(trigger, HoverCardContent::new([...]).into_element(cx)).into_element(cx);",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Delays + Position").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            "HoverCard::new(...).open_delay_frames(16).close_delay_frames(12).side(HoverCardSide::Top).align(HoverCardAlign::Start);",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        },
    );
    let code_panel = shell(cx, code_panel_body);

    let notes_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
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
    let notes_panel = shell(cx, notes_panel_body);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-hover-card",
        component_panel,
        code_panel,
        notes_panel,
    )
}
