pub const SOURCE: &str = include_str!("spacing.rs");

// region: example
use fret_app::App;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Debug, Clone, Copy)]
struct SlideVisual {
    text_px: Px,
    line_height_px: Px,
}

fn slide_card(cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let number = ui::text(format!("{idx}"))
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
            padding: Edges::all(Px(24.0)).into(),
            ..Default::default()
        },
        move |_cx| vec![number],
    );

    shadcn::Card::new([content]).into_element(cx)
}

fn slide(cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual) -> AnyElement {
    let card = slide_card(cx, idx, visual);
    ui::container(move |_cx| vec![card])
        .w_full()
        .p_1()
        .into_element(cx)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_sm = Px(384.0);

    let visual = SlideVisual {
        text_px: Px(24.0),
        line_height_px: Px(32.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide(cx, idx, visual)))
        .map(|item| item.padding_start(Space::N1))
        .collect::<Vec<_>>();

    shadcn::Carousel::default()
        .item_basis_main_px(Px(129.328))
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_sm)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-spacing")
        .into_element_parts(
            cx,
            |_cx| shadcn::CarouselContent::new(items).track_start_neg_margin(Space::N1),
            shadcn::CarouselPrevious::new(),
            shadcn::CarouselNext::new(),
        )
}
// endregion: example
