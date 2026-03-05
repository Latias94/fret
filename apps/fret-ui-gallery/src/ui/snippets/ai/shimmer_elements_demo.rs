pub const SOURCE: &str = include_str!("shimmer_elements_demo.rs");

// region: example
use fret_core::{FontId, FontWeight, Px, SemanticsRole, TextStyle};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let heading_style = TextStyle {
        font: FontId::ui(),
        size: Px(24.0),
        weight: FontWeight::BOLD,
        line_height: Some(Px(28.0)),
        ..Default::default()
    };
    let custom_style = TextStyle {
        font: FontId::ui(),
        size: Px(18.0),
        weight: FontWeight::SEMIBOLD,
        line_height: Some(Px(22.0)),
        ..Default::default()
    };

    let paragraph =
        ui_ai::Shimmer::new(Arc::<str>::from("This is rendered as a paragraph")).into_element(cx);
    let heading = ui_ai::Shimmer::new(Arc::<str>::from("Large Heading with Shimmer"))
        .role(SemanticsRole::Heading)
        .text_style(heading_style)
        .into_element(cx);
    let inline = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N1).items_center(),
        move |cx| {
            vec![
                cx.text("Processing your request"),
                ui_ai::Shimmer::new(Arc::<str>::from("with AI magic")).into_element(cx),
                cx.text("..."),
            ]
        },
    );
    let custom = ui_ai::Shimmer::new(Arc::<str>::from("Custom styled shimmer text"))
        .text_style(custom_style)
        .into_element(cx);

    let item = |cx: &mut ElementContext<'_, H>, label: &'static str, el: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    shadcn::Badge::new(label)
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                    el,
                ]
            },
        )
    };

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N6),
        move |cx| {
            vec![
                item(cx, "As paragraph (default)", paragraph),
                item(cx, "As heading", heading),
                item(cx, "As span (inline)", inline),
                item(cx, "As div with custom styling", custom),
            ]
        },
    )
    .test_id("ui-ai-shimmer-elements-root")
}
// endregion: example
