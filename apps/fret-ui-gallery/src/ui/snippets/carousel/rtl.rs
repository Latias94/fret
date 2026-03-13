pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Edges, LayoutDirection};
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Debug, Clone, Copy)]
struct SlideVisual {
    text_px: Px,
    line_height_px: Px,
}

fn slide_card(
    cx: &mut UiCx<'_>,
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
    cx: &mut UiCx<'_>,
    idx: usize,
    visual: SlideVisual,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let card = slide_card(cx, idx, visual).into_element(cx);
    ui::container(move |_cx| vec![card]).w_full().p_1()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let max_w_xs = Px(320.0);

    let api_handle = cx.local_model_keyed("api_handle", || None::<shadcn::CarouselApi>);

    let visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide(cx, idx, visual).into_element(cx)))
        .collect::<Vec<_>>();

    let carousel = shadcn::DirectionProvider::new(LayoutDirection::Rtl).into_element(cx, |cx| {
        shadcn::Carousel::default()
            .opts(shadcn::CarouselOptions::new().direction(LayoutDirection::Rtl))
            .api_handle_model(api_handle.clone())
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .max_w(max_w_xs)
                    .mx_auto(),
            )
            .test_id("ui-gallery-carousel-rtl")
            .into_element_parts(
                cx,
                |_cx| shadcn::CarouselContent::new(items),
                shadcn::CarouselPrevious::new().test_id("ui-gallery-carousel-rtl-previous"),
                shadcn::CarouselNext::new().test_id("ui-gallery-carousel-rtl-next"),
            )
    });

    let selected_marker = {
        let selected_index = cx
            .watch_model(&api_handle)
            .cloned()
            .flatten()
            .map(|api| api.snapshot(&mut *cx.app).selected_index)
            .unwrap_or(0);

        let badge = shadcn::Badge::new(format!("Selected index: {selected_index}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .test_id(format!(
                "ui-gallery-carousel-rtl-selected-index-{selected_index}"
            ))
            .into_element(cx);

        ui::container(move |_cx| vec![badge])
            .py_2()
            .into_element(cx)
    };

    cx.flex(
        FlexProps {
            layout: decl_style::layout_style(
                &Theme::global(&*cx.app).snapshot(),
                LayoutRefinement::default()
                    .w_full()
                    .max_w(max_w_xs)
                    .mx_auto(),
            ),
            direction: fret_core::Axis::Vertical,
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            ..Default::default()
        },
        move |_cx| vec![carousel, selected_marker],
    )
}
// endregion: example
