pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Debug, Clone, Copy)]
struct SlideVisual {
    text_px: Px,
    line_height_px: Px,
}

fn slide_card(
    cx: &mut AppComponentCx<'_>,
    idx: usize,
    visual: SlideVisual,
) -> impl IntoUiElement<fret_app::App> + use<> {
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

    shadcn::card(|cx| ui::children![cx; shadcn::card_content(|cx| ui::children![cx; content])])
}

fn slide(
    cx: &mut AppComponentCx<'_>,
    idx: usize,
    visual: SlideVisual,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let card = slide_card(cx, idx, visual).into_element(cx);
    ui::container(move |_cx| vec![card]).w_full().p_1()
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let max_w_xs = Px(320.0);

    let basic_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide(cx, idx, basic_visual).into_element(cx)))
        .collect::<Vec<_>>();

    shadcn::Carousel::new(items)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-basic")
        .into_element(cx)
}
// endregion: example
