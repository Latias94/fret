#[allow(dead_code)]
pub const SOURCE: &str = DOCS_SOURCE;
pub const DOCS_SOURCE: &str = include_str!("api.docs.rs.txt");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign, TextProps};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

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

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let max_w_xs = Px(320.0);
    let controls_shell_w = Px(max_w_xs.0 + 96.0);

    let api_snapshot = cx.local_model_keyed("api_snapshot", shadcn::CarouselApiSnapshot::default);

    let api_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide_card(cx, idx, api_visual).into_element(cx)))
        .collect::<Vec<_>>();
    let api_carousel = shadcn::Carousel::new(items)
        .api_snapshot_model(api_snapshot.clone())
        .refine_layout(LayoutRefinement::default().w_full().max_w(max_w_xs))
        .test_id("ui-gallery-carousel-api")
        .into_element(cx);
    let api_carousel = ui::container(move |_cx| vec![api_carousel])
        .w_full()
        .max_w(controls_shell_w)
        .h_px(Px(304.0))
        .mx_auto()
        .px(Space::N12)
        .into_element(cx);

    // The common "Slide X of Y" docs outcome only needs a snapshot model.
    let snapshot = cx.watch_model(&api_snapshot).copied().unwrap_or_default();
    let current = if snapshot.snap_count > 0 {
        snapshot.selected_index.saturating_add(1)
    } else {
        0
    };
    let count = snapshot.snap_count;
    let api_counter_text = format!("Slide {} of {}", current, count);
    let api_counter = {
        let theme = Theme::global(&*cx.app);
        let style = fret_ui_kit::typography::control_text_style(
            theme,
            fret_ui_kit::typography::UiTextSize::Sm,
        );
        let color = theme
            .color_by_key("muted-foreground")
            .or_else(|| theme.color_by_key("muted_foreground"))
            .unwrap_or_else(|| theme.color_token("foreground"));

        let text = cx.text_props(TextProps {
            layout: {
                let mut layout = fret_ui::element::LayoutStyle::default();
                layout.size.width = fret_ui::element::Length::Fill;
                layout
            },
            text: Arc::from(api_counter_text),
            style: Some(style),
            color: Some(color),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Center,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        });

        ui::container(move |_cx| vec![text]).py_2().into_element(cx)
    };

    ui::v_flex(move |_cx| vec![api_carousel, api_counter])
        .items_stretch()
        .layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(controls_shell_w)
                .mx_auto(),
        )
        .into_element(cx)
}
// endregion: example
