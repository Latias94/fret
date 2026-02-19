use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_hover_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let profile_card = |cx: &mut ElementContext<'_, App>,
                        name: &'static str,
                        desc: &'static str,
                        test_id: &'static str| {
        let muted_fg = cx.with_theme(|theme| theme.color_token("muted-foreground"));

        let avatar = shadcn::Avatar::new([shadcn::AvatarFallback::new("VC").into_element(cx)])
            .refine_layout(
                LayoutRefinement::default()
                    .w_px(Px(40.0))
                    .h_px(Px(40.0))
                    .flex_shrink_0(),
            )
            .into_element(cx);

        let text = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().flex_1().min_w_0())
                .gap(Space::N1)
                .items_start(),
            |cx| {
                let description = ui::text(cx, desc)
                    .w_full()
                    .text_size_px(Px(14.0))
                    .line_height_px(Px(20.0))
                    .wrap(TextWrap::Word)
                    .into_element(cx);
                let joined = ui::text(cx, "Joined December 2021")
                    .w_full()
                    .text_size_px(Px(12.0))
                    .line_height_px(Px(16.0))
                    .text_color(ColorRef::Color(muted_fg))
                    .wrap(TextWrap::Word)
                    .into_element(cx);
                vec![
                    shadcn::CardTitle::new(name).into_element(cx),
                    description,
                    joined,
                ]
            },
        );

        let content = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N4)
                .items_start(),
            |_cx| vec![avatar, text],
        );

        shadcn::HoverCardContent::new(vec![content])
            .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
            .into_element(cx)
            .test_id(test_id)
    };

    let demo = {
        let trigger = shadcn::Button::new("@nextjs")
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

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::HoverCard::new(
            shadcn::Button::new("مرر هنا")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-rtl-trigger")
                .into_element(cx),
            profile_card(
                cx,
                "نموذج RTL",
                "تحقق من محاذاة HoverCard تحت RTL.",
                "ui-gallery-hover-card-rtl-content",
            ),
        )
        .open_delay_frames(10)
        .close_delay_frames(10)
        .side(shadcn::HoverCardSide::Left)
        .into_element(cx)
    })
    .test_id("ui-gallery-hover-card-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "Hover card interactions depend on hover-intent delays, so examples include both instant and delayed scenarios.",
            "Sides and positioning are separated to make placement parity checks deterministic.",
            "RTL sample is included because side resolution can differ in right-to-left layouts.",
        ],
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
                .description("Compare instant vs delayed open/close behavior.")
                .code(
                    "rust",
                    r#"let instant = shadcn::HoverCard::new(trigger, content)
    .open_delay_frames(0)
    .close_delay_frames(0)
    .into_element(cx);

let delayed = shadcn::HoverCard::new(trigger, content)
    .open_delay_frames(16)
    .close_delay_frames(12)
    .into_element(cx);"#,
                ),
            DocSection::new("Positioning", positioning)
                .description("Placement is controlled by `side` and `align`.")
                .code(
                    "rust",
                    r#"shadcn::HoverCard::new(trigger, content)
    .side(shadcn::HoverCardSide::Top)
    .align(shadcn::HoverCardAlign::Start);"#,
                ),
            DocSection::new("Basic", basic)
                .description("A minimal hover card with shorter delays.")
                .code(
                    "rust",
                    r#"let trigger = shadcn::Button::new("Basic")
    .variant(shadcn::ButtonVariant::Outline)
    .into_element(cx);
 let content = shadcn::HoverCardContent::new(vec![/* content */]).into_element(cx);

 shadcn::HoverCard::new(trigger, content)
     .open_delay_frames(10)
     .close_delay_frames(10)
    .into_element(cx);"#,
                ),
            DocSection::new("Sides", sides)
                .description("Visual sweep of side placements.")
                .code(
                    "rust",
                    r#"shadcn::HoverCard::new(trigger, content)
    .side(shadcn::HoverCardSide::Left)
    .into_element(cx);

shadcn::HoverCard::new(trigger, content)
    .side(shadcn::HoverCardSide::Top)
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Hover card should respect right-to-left direction context.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::HoverCard::new(trigger, content).side(shadcn::HoverCardSide::Left).into_element(cx),
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines."),
        ],
    );

    vec![body]
}
