pub const SOURCE: &str = include_str!("options.rs");

// region: example
use fret::UiCx;
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

fn slide_card(cx: &mut UiCx<'_>, idx: usize, visual: SlideVisual) -> AnyElement {
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

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let max_w_xs = Px(320.0);

    let visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide_card(cx, idx, visual)))
        .collect::<Vec<_>>();

    shadcn::Carousel::default()
        .opts(
            shadcn::CarouselOptions::new()
                .align(shadcn::CarouselAlign::Start)
                .loop_enabled(true),
        )
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-options")
        .into_element_parts(
            cx,
            |_cx| shadcn::CarouselContent::new(items),
            shadcn::CarouselPrevious::new(),
            shadcn::CarouselNext::new(),
        )
}
// endregion: example
