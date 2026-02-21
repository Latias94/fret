use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_hover_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let muted_fg = cx.with_theme(|theme| theme.color_token("muted-foreground"));

    let demo_content = |cx: &mut ElementContext<'_, App>,
                        title: &'static str,
                        desc: &'static str,
                        joined: &'static str,
                        test_id: &'static str| {
        let title_test_id: Arc<str> = Arc::from(format!("{test_id}-title"));
        let desc_test_id: Arc<str> = Arc::from(format!("{test_id}-desc"));
        let joined_test_id: Arc<str> = Arc::from(format!("{test_id}-joined"));

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N0p5)
                .items_start(),
            move |cx| {
                vec![
                    ui::text(cx, title)
                        .font_semibold()
                        .into_element(cx)
                        .test_id(title_test_id.clone()),
                    ui::text(cx, desc)
                        .wrap(TextWrap::WordBreak)
                        .into_element(cx)
                        .test_id(desc_test_id.clone()),
                    ui::text(cx, joined)
                        .text_xs()
                        .text_color(ColorRef::Color(muted_fg))
                        .mt(Space::N1)
                        .into_element(cx)
                        .test_id(joined_test_id.clone()),
                ]
            },
        );

        shadcn::HoverCardContent::new(vec![body])
            .into_element(cx)
            .test_id(test_id)
    };

    let side_content = |cx: &mut ElementContext<'_, App>,
                        side_label: &'static str,
                        test_id: &'static str| {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1)
                .items_start(),
            move |cx| {
                vec![
                    ui::text(cx, "Hover Card").font_medium().into_element(cx),
                    ui::text(
                        cx,
                        format!("This hover card appears on the {side_label} side of the trigger."),
                    )
                    .wrap(TextWrap::WordBreak)
                    .into_element(cx),
                ]
            },
        );

        shadcn::HoverCardContent::new(vec![body])
            .into_element(cx)
            .test_id(test_id)
    };

    let demo = {
        let trigger = shadcn::Button::new("Hover Here")
            .variant(shadcn::ButtonVariant::Link)
            .test_id("ui-gallery-hover-card-demo-trigger")
            .into_element(cx);
        let content = demo_content(
            cx,
            "@nextjs",
            "The React Framework – created and maintained by @vercel.",
            "Joined December 2021",
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
            demo_content(
                cx,
                "Instant",
                "openDelay=0",
                "closeDelay=0",
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
            demo_content(
                cx,
                "Delayed",
                "openDelay=16",
                "closeDelay=12",
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
            side_content(
                cx,
                "top (align=start)",
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
            side_content(
                cx,
                "right (align=end)",
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
        let trigger = shadcn::Button::new("Hover Here")
            .variant(shadcn::ButtonVariant::Link)
            .test_id("ui-gallery-hover-card-basic-trigger")
            .into_element(cx);
        let content = demo_content(
            cx,
            "@nextjs",
            "The React Framework – created and maintained by @vercel.",
            "Joined December 2021",
            "ui-gallery-hover-card-basic-content",
        );
        let hover = shadcn::HoverCard::new(trigger, content)
            .open_delay_frames(10)
            .close_delay_frames(100)
            .into_element(cx)
            .test_id("ui-gallery-hover-card-basic");
        hover
    };

    let sides = {
        let entries = [
            (
                shadcn::HoverCardSide::Left,
                "left",
                "ui-gallery-hover-card-side-left-trigger",
                "ui-gallery-hover-card-side-left-content",
                "ui-gallery-hover-card-side-left",
            ),
            (
                shadcn::HoverCardSide::Top,
                "top",
                "ui-gallery-hover-card-side-top-trigger",
                "ui-gallery-hover-card-side-top-content",
                "ui-gallery-hover-card-side-top",
            ),
            (
                shadcn::HoverCardSide::Bottom,
                "bottom",
                "ui-gallery-hover-card-side-bottom-trigger",
                "ui-gallery-hover-card-side-bottom-content",
                "ui-gallery-hover-card-side-bottom",
            ),
            (
                shadcn::HoverCardSide::Right,
                "right",
                "ui-gallery-hover-card-side-right-trigger",
                "ui-gallery-hover-card-side-right-content",
                "ui-gallery-hover-card-side-right",
            ),
        ];

        let card = |cx: &mut ElementContext<'_, App>,
                    side,
                    label: &'static str,
                    trigger_test_id: &'static str,
                    content_test_id: &'static str,
                    root_test_id: &'static str| {
            shadcn::HoverCard::new(
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id(trigger_test_id)
                    .into_element(cx),
                side_content(cx, label, content_test_id),
            )
            .side(side)
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx)
            .test_id(root_test_id)
        };

        let row_1 = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N3)
                .justify_center()
                .items_center(),
            move |cx| {
                let (side, label, trigger_test_id, content_test_id, root_test_id) = entries[0];
                let left = card(
                    cx,
                    side,
                    label,
                    trigger_test_id,
                    content_test_id,
                    root_test_id,
                );
                let (side, label, trigger_test_id, content_test_id, root_test_id) = entries[1];
                let top = card(
                    cx,
                    side,
                    label,
                    trigger_test_id,
                    content_test_id,
                    root_test_id,
                );
                vec![left, top]
            },
        );

        let row_2 = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N3)
                .justify_center()
                .items_center(),
            move |cx| {
                let (side, label, trigger_test_id, content_test_id, root_test_id) = entries[2];
                let bottom = card(
                    cx,
                    side,
                    label,
                    trigger_test_id,
                    content_test_id,
                    root_test_id,
                );
                let (side, label, trigger_test_id, content_test_id, root_test_id) = entries[3];
                let right = card(
                    cx,
                    side,
                    label,
                    trigger_test_id,
                    content_test_id,
                    root_test_id,
                );
                vec![bottom, right]
            },
        );

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(
                    LayoutRefinement::default()
                        .w_full()
                        .min_w_0()
                        .min_h(Px(240.0)),
                )
                .gap(Space::N3)
                .justify_center()
                .items_center(),
            move |_cx| vec![row_1, row_2],
        )
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1)
                .items_start(),
            move |cx| {
                vec![
                    ui::text(cx, "نموذج RTL").font_semibold().into_element(cx),
                    ui::text(cx, "تحقق من محاذاة HoverCard تحت RTL.")
                        .wrap(TextWrap::WordBreak)
                        .text_color(ColorRef::Color(muted_fg))
                        .into_element(cx),
                ]
            },
        );

        shadcn::HoverCard::new(
            shadcn::Button::new("مرر هنا")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-hover-card-rtl-trigger")
                .into_element(cx),
            shadcn::HoverCardContent::new(vec![body])
                .into_element(cx)
                .test_id("ui-gallery-hover-card-rtl-content"),
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
                .description("Basic hover card surface with a short open delay and a longer close delay.")
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
                .description("A basic hover card surface matching the upstream example.")
                .code(
                    "rust",
                    r#"let trigger = shadcn::Button::new("Hover Here")
    .variant(shadcn::ButtonVariant::Link)
    .into_element(cx);
let content = shadcn::HoverCardContent::new(vec![/* content */]).into_element(cx);

shadcn::HoverCard::new(trigger, content)
    .open_delay_frames(10)
    .close_delay_frames(100)
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
