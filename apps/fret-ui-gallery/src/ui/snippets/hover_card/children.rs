pub const SOURCE: &str = include_str!("children.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let title = decl_text::text_section_chrome_label(cx, "Release Notes")
        .test_id("ui-gallery-hover-card-children-demo-title");
    let summary = decl_text::text_paragraph_break_words(
        cx,
        "Already-built panel nodes can be passed directly into HoverCardContent::new([...]) when the content body is caller-owned.",
    )
    .test_id("ui-gallery-hover-card-children-demo-summary");
    let meta = ui::h_flex(|cx| {
        vec![
            shadcn::Badge::new("Caller-owned")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            decl_text::text_control_readout(cx, "Updated 2m ago"),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-hover-card-children-demo-meta");

    let content = shadcn::HoverCardContent::new([title, summary, meta])
        .test_id("ui-gallery-hover-card-children-demo-content")
        .refine_layout(LayoutRefinement::default().max_w(Px(288.0)));

    shadcn::HoverCard::new(
        cx,
        shadcn::Button::new("Composable content")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-children-demo-trigger"),
        content,
    )
    .open_delay_frames(8)
    .close_delay_frames(8)
    .into_element(cx)
    .test_id("ui-gallery-hover-card-children-demo")
}
// endregion: example
