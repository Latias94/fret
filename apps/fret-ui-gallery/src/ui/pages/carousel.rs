use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

use fret_ui::element::{CrossAlign, FlexProps, MainAlign};

pub(super) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Debug, Clone, Copy)]
    struct SlideVisual {
        text_px: Px,
        line_height_px: Px,
    }

    let slide = |cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual| {
        let theme = Theme::global(&*cx.app).clone();

        let number = ui::text(cx, format!("{idx}"))
            .text_size_px(visual.text_px)
            .line_height_px(visual.line_height_px)
            .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
            .font_semibold()
            .into_element(cx);

        let content = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default().w_full().aspect_ratio(1.0),
                ),
                direction: fret_core::Axis::Horizontal,
                justify: MainAlign::Center,
                align: CrossAlign::Center,
                padding: Edges::all(Px(24.0)),
                ..Default::default()
            },
            move |_cx| vec![number],
        );

        let card = shadcn::Card::new([content]).into_element(cx);
        ui::container(cx, move |_cx| vec![card])
            .p_1()
            .into_element(cx)
    };

    let max_w_sm = Px(384.0);

    let basic_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(44.0),
    };
    let basic_items = (1..=5)
        .map(|idx| slide(cx, idx, basic_visual))
        .collect::<Vec<_>>();
    let basic = shadcn::Carousel::new(basic_items)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-basic")
        .into_element(cx);

    let align_start_visual = SlideVisual {
        text_px: Px(30.0),
        line_height_px: Px(36.0),
    };
    let align_start_items = (1..=5)
        .map(|idx| slide(cx, idx, align_start_visual))
        .collect::<Vec<_>>();
    let align_start = shadcn::Carousel::new(align_start_items)
        .item_basis_main_px(Px(192.0))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-align-start")
        .into_element(cx);

    let spacing_visual = SlideVisual {
        text_px: Px(24.0),
        line_height_px: Px(32.0),
    };
    let spacing_items = (1..=5)
        .map(|idx| slide(cx, idx, spacing_visual))
        .collect::<Vec<_>>();
    let spacing = shadcn::Carousel::new(spacing_items)
        .item_basis_main_px(Px(192.0))
        .track_start_neg_margin(Space::N1)
        .item_padding_start(Space::N1)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-spacing")
        .into_element(cx);

    let notes_stack = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Carousel demo: Basic, Align Start, and Spacing.",
            "The upstream demo uses responsive item widths (`md:basis-1/2` / `lg:basis-1/3`). Fret uses a fixed `item_basis_main_px` to keep geometry deterministic in native builds.",
            "Spacing parity depends on pairing `track_start_neg_margin` with `item_padding_start`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Carousel demo cards (Fret builder API; not Embla)."),
        vec![
            DocSection::new("Basic", basic)
                .description("Default slide width (basis-full).")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-basic")
                .code(
                    "rust",
                    r#"let items = (1..=5).map(|idx| slide(cx, idx)).collect::<Vec<_>>();

shadcn::Carousel::new(items)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("Align Start", align_start)
                .description("Fixed basis (basis-1/2) to mirror the docs layout deterministically.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-align-start")
                .code(
                    "rust",
                    r#"// Upstream: responsive widths (`md:basis-1/2` / `lg:basis-1/3`).
// Here: fixed basis for deterministic native layout.
shadcn::Carousel::new(items)
    .item_basis_main_px(Px(192.0))
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("Spacing", spacing)
                .description(
                    "Tighter track negative margin + item start padding (shadcn `-ml-1` / `pl-1`).",
                )
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-carousel-spacing")
                .code(
                    "rust",
                    r#"shadcn::Carousel::new(items)
    .item_basis_main_px(Px(192.0))
    .track_start_neg_margin(Space::N1)
    .item_padding_start(Space::N1)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)).mx_auto())
    .into_element(cx);"#,
                ),
            DocSection::new("Notes", notes_stack).max_w(Px(760.0)),
        ],
    );

    vec![body.test_id("ui-gallery-carousel-component")]
}
