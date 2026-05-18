pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::element::SemanticsDecoration;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let rtl_area = with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let content = ui::container(|cx| {
            vec![
                ui::v_flex(|cx| {
                    let mut rows: Vec<AnyElement> =
                        vec![decl_text::text_section_chrome_label(cx, "العلامات")];

                    for idx in 1..=40 {
                        rows.push(decl_text::text_list_row_label(cx, (41 - idx).to_string()));
                        rows.push(
                            shadcn::Separator::new()
                                .refine_layout(LayoutRefinement::default().w_full().my(Space::N2))
                                .into_element(cx),
                        );
                    }
                    rows
                })
                .gap(Space::N0)
                .layout(LayoutRefinement::default().w_full())
                .into_element(cx),
            ]
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

    cx.container(props, move |_cx| vec![rtl_area])
}
// endregion: example
