pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::UiCx;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn slide_card(cx: &mut UiCx<'_>, idx: usize) -> impl IntoUiElement<fret_app::App> + use<> {
    let theme = Theme::global(&*cx.app).clone();

    let number = ui::text(format!("{idx}"))
        .text_size_px(Px(36.0))
        .line_height_px(Px(40.0))
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

    shadcn::Card::new([content])
}

fn slide(cx: &mut UiCx<'_>, idx: usize) -> impl IntoUiElement<fret_app::App> + use<> {
    let card = slide_card(cx, idx).into_element(cx);
    ui::container(move |_cx| vec![card]).w_full().p_1()
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let items = (1..=3)
        .map(|idx| shadcn::CarouselItem::new(slide(cx, idx).into_element(cx)))
        .collect::<Vec<_>>();

    shadcn::Carousel::default()
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(320.0))
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-usage")
        .into_element_parts(
            cx,
            |_cx| shadcn::CarouselContent::new(items),
            shadcn::CarouselPrevious::new(),
            shadcn::CarouselNext::new(),
        )
}
// endregion: example
