pub const SOURCE: &str = include_str!("plugin_autoplay_controlled.rs");

// region: example
use fret::UiCx;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;
use std::time::Duration;

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

    shadcn::Card::new([content]).into_element(cx)
}

fn slide(
    cx: &mut UiCx<'_>,
    idx: usize,
    visual: SlideVisual,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let card = slide_card(cx, idx, visual).into_element(cx);
    ui::container(move |_cx| vec![card]).w_full().p_1()
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let max_w_xs = Px(320.0);

    let autoplay_api = cx.local_model_keyed("autoplay_api", || None::<shadcn::CarouselAutoplayApi>);

    let visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide(cx, idx, visual).into_element(cx)))
        .collect::<Vec<_>>();

    let stop = {
        let autoplay_api = autoplay_api.clone();
        shadcn::Button::new("Stop")
            .variant(shadcn::ButtonVariant::Secondary)
            .on_activate(Arc::new(move |host, action_cx, _reason| {
                let api = host
                    .models_mut()
                    .read(&autoplay_api, |v| v.clone())
                    .ok()
                    .flatten();
                if let Some(api) = api {
                    api.stop_store(host.models_mut());
                    host.request_redraw(action_cx.window);
                }
            }))
            .test_id("ui-gallery-carousel-autoplay-controlled-stop")
            .into_element(cx)
    };

    let reset = {
        let autoplay_api = autoplay_api.clone();
        shadcn::Button::new("Reset")
            .variant(shadcn::ButtonVariant::Secondary)
            .on_activate(Arc::new(move |host, action_cx, _reason| {
                let api = host
                    .models_mut()
                    .read(&autoplay_api, |v| v.clone())
                    .ok()
                    .flatten();
                if let Some(api) = api {
                    api.reset_store(host.models_mut());
                    host.request_redraw(action_cx.window);
                }
            }))
            .test_id("ui-gallery-carousel-autoplay-controlled-reset")
            .into_element(cx)
    };

    let pause = {
        let autoplay_api = autoplay_api.clone();
        shadcn::Button::new("Pause")
            .variant(shadcn::ButtonVariant::Secondary)
            .on_activate(Arc::new(move |host, action_cx, _reason| {
                let api = host
                    .models_mut()
                    .read(&autoplay_api, |v| v.clone())
                    .ok()
                    .flatten();
                if let Some(api) = api {
                    api.pause_store(host.models_mut());
                    host.request_redraw(action_cx.window);
                }
            }))
            .test_id("ui-gallery-carousel-autoplay-controlled-pause")
            .into_element(cx)
    };

    let play = {
        let autoplay_api = autoplay_api.clone();
        shadcn::Button::new("Play")
            .variant(shadcn::ButtonVariant::Secondary)
            .on_activate(Arc::new(move |host, action_cx, _reason| {
                let api = host
                    .models_mut()
                    .read(&autoplay_api, |v| v.clone())
                    .ok()
                    .flatten();
                if let Some(api) = api {
                    api.play_store(host.models_mut());
                    host.request_redraw(action_cx.window);
                }
            }))
            .test_id("ui-gallery-carousel-autoplay-controlled-play")
            .into_element(cx)
    };

    let controls = ui::h_row(|_cx| vec![stop, reset, pause, play])
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    let carousel = shadcn::Carousel::default()
        .plugins([shadcn::CarouselPlugin::Autoplay(
            shadcn::CarouselAutoplayConfig::new(Duration::from_millis(2000))
                .pause_on_hover(false)
                .reset_on_hover_leave(false),
        )])
        .autoplay_api_handle_model(autoplay_api.clone())
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w_xs)
                .mx_auto(),
        )
        .test_id("ui-gallery-carousel-plugin-autoplay-controlled")
        .into_element_parts(
            cx,
            |_cx| shadcn::CarouselContent::new(items),
            shadcn::CarouselPrevious::new(),
            shadcn::CarouselNext::new(),
        );

    ui::v_flex(|_cx| vec![controls, carousel])
        .gap(Space::N2)
        .items_stretch()
        .layout(LayoutRefinement::default().w_full())
        .into_element(cx)
}
// endregion: example
