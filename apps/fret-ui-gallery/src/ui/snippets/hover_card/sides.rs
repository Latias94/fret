pub const SOURCE: &str = include_str!("sides.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn side_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    side_label: &'static str,
    test_id: &'static str,
) -> shadcn::HoverCardContent {
    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N1)
            .items_start(),
        move |cx| {
            vec![
                ui::text(side_label)
                    .wrap(TextWrap::WordBreak)
                    .into_element(cx),
            ]
        },
    );

    shadcn::HoverCardContent::new(vec![body]).test_id(test_id)
}

fn card<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    side: shadcn::HoverCardSide,
    label: &'static str,
    trigger_test_id: &'static str,
    content_test_id: &'static str,
    root_test_id: &'static str,
) -> AnyElement {
    let content = side_content(cx, label, content_test_id).side(side);
    shadcn::HoverCard::new(
        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .test_id(trigger_test_id)
            .into_element(cx),
        content,
    )
    .open_delay_frames(10)
    .close_delay_frames(10)
    .into_element(cx)
    .test_id(root_test_id)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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
}
// endregion: example
