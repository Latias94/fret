pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let rtl_area = with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let content = ui::container(cx, |cx| {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N0)
                    .layout(LayoutRefinement::default().w_full()),
                |cx| {
                    let mut rows: Vec<AnyElement> = vec![
                        ui::text(cx, "العلامات")
                            .text_sm()
                            .line_height_px(Px(14.0))
                            .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
                            .font_medium()
                            .w_full()
                            .into_element(cx),
                    ];

                    for idx in 1..=40 {
                        rows.push(
                            ui::text(cx, format!("v1.2.0-beta.{:02}", 41 - idx))
                                .text_sm()
                                .line_height_px(Px(20.0))
                                .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
                                .w_full()
                                .into_element(cx),
                        );
                        rows.push(
                            shadcn::Separator::new()
                                .refine_layout(LayoutRefinement::default().w_full().my(Space::N2))
                                .into_element(cx),
                        );
                    }
                    rows
                },
            )]
        })
        .p_4()
        .w_full()
        .into_element(cx);

        shadcn::ScrollArea::new([content])
            .axis(fret_ui::element::ScrollAxis::Y)
            .viewport_test_id("ui-gallery-scroll-area-rtl-viewport")
            .refine_layout(LayoutRefinement::default().w_full().h_full())
            .into_element(cx)
    })
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-scroll-area-rtl"),
    );

    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().border_1().rounded(Radius::Md),
        LayoutRefinement::default()
            .w_px(Px(192.0))
            .h_px(Px(288.0))
            .overflow_hidden(),
    );

    cx.container(props, move |_cx| [rtl_area])
}
// endregion: example
