pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let tags: Vec<Arc<str>> = (1..=50)
        .map(|idx| Arc::<str>::from(format!("v1.2.0-beta.{}", 51 - idx)))
        .collect();

    let content = ui::container(move |cx| {
        let heading = ui::text("Tags")
            .text_size_px(Px(14.0))
            .line_height_px(Px(14.0))
            .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
            .font_medium()
            .into_element(cx);

        let tags_list = ui::v_flex(move |cx| {
            let mut out: Vec<AnyElement> = Vec::with_capacity(tags.len() * 2);
            for tag in tags {
                out.push(
                    ui::text(tag)
                        .text_size_px(Px(14.0))
                        .line_height_px(Px(20.0))
                        .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
                        .into_element(cx),
                );
                out.push(
                    shadcn::Separator::new()
                        .refine_layout(LayoutRefinement::default().w_full().my(Space::N2))
                        .into_element(cx),
                );
            }
            out
        })
        .gap(Space::N0)
        .layout(LayoutRefinement::default().w_full())
        .into_element(cx);

        vec![
            ui::v_flex(|_cx| vec![heading, tags_list])
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full())
                .into_element(cx),
        ]
    })
    .p_4()
    .into_element(cx);

    let area = shadcn::ScrollArea::build(|cx, out| {
        out.push_ui(cx, content);
    })
    .axis(fret_ui::element::ScrollAxis::Y)
    .viewport_test_id("ui-gallery-scroll-area-demo-viewport")
    .refine_layout(LayoutRefinement::default().w_full().h_full())
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-scroll-area-demo"),
    );

    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().border_1().rounded(Radius::Md),
        LayoutRefinement::default()
            .w_px(Px(192.0))
            .h_px(Px(288.0))
            .overflow_hidden(),
    );

    cx.container(props, move |_cx| [area])
}
// endregion: example
