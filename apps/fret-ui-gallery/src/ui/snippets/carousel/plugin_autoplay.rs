#[allow(dead_code)]
pub const SOURCE: &str = DOCS_SOURCE;
pub const DOCS_SOURCE: &str = include_str!("plugin_autoplay.docs.rs.txt");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, HoverRegionProps, MainAlign};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::time::Duration;

#[derive(Default)]
struct HoverState {
    hovered: bool,
}

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

    let autoplay_api = cx.local_model_keyed("autoplay_api", || None::<shadcn::CarouselAutoplayApi>);
    let autoplay = shadcn::CarouselAutoplayConfig::new(Duration::from_millis(2000))
        .pause_on_hover(false)
        .reset_on_hover_leave(false);

    let visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide(cx, idx, visual).into_element(cx)))
        .collect::<Vec<_>>();

    let carousel = shadcn::Carousel::new(items)
        .plugins([shadcn::CarouselPlugin::Autoplay(autoplay)])
        .autoplay_api_handle_model(autoplay_api.clone())
        .refine_layout(LayoutRefinement::default().w_full())
        .test_id("ui-gallery-carousel-plugin")
        .into_element(cx);

    // Self-drawn hover routing replaces the docs' DOM mouse-enter / mouse-leave handlers.
    let hover_overlay = {
        let autoplay_api = autoplay_api.clone();
        cx.hover_region(
            HoverRegionProps {
                layout: decl_style::layout_style(
                    &Theme::global(&*cx.app).clone(),
                    LayoutRefinement::default()
                        .absolute()
                        .top(Space::N0)
                        .right(Space::N0)
                        .bottom(Space::N0)
                        .left(Space::N0),
                ),
            },
            move |cx, hovered| {
                let changed = cx.slot_state(HoverState::default, |st| {
                    let changed = st.hovered != hovered;
                    st.hovered = hovered;
                    changed
                });
                if !changed {
                    return Vec::new();
                }

                let api = cx
                    .app
                    .models_mut()
                    .read(&autoplay_api, |v| v.clone())
                    .ok()
                    .flatten();
                if let Some(api) = api {
                    if hovered {
                        api.stop_store(cx.app.models_mut());
                    } else {
                        api.reset_store(cx.app.models_mut());
                    }
                    cx.request_frame();
                }

                Vec::new()
            },
        )
    };

    ui::container(move |_cx| vec![carousel, hover_overlay])
        .w_full()
        .max_w(max_w_xs)
        .mx_auto()
        .relative()
        .into_element(cx)
}
// endregion: example
