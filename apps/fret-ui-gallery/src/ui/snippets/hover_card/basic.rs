pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let body = ui::v_flex(|cx| {
        vec![
            decl_text::text_paragraph_break_words(
                cx,
                "HoverCard content: multiline description with WordBreak wrapping.",
            )
            .test_id("ui-gallery-hover-card-basic-content-desc"),
            decl_text::text_control_readout(cx, "Joined December 2021")
                .test_id("ui-gallery-hover-card-basic-content-joined"),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N1)
    .items_stretch()
    .into_element(cx);

    let content = shadcn::HoverCardContent::build(cx, |_cx| [body])
        .test_id("ui-gallery-hover-card-basic-content");

    shadcn::HoverCard::new(
        cx,
        shadcn::Button::new("Hover")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-hover-card-basic-trigger"),
        content,
    )
    .into_element(cx)
    .test_id("ui-gallery-hover-card-basic")
}
// endregion: example
