pub const SOURCE: &str = include_str!("plugin_autoplay_stop_on_focus.rs");

// region: example
use fret_app::App;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign, SpacingLength, TextProps};
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;
use std::time::Duration;

#[derive(Default, Clone)]
struct Models {
    autoplay_api: Option<Model<Option<shadcn::CarouselAutoplayApi>>>,
}

#[derive(Debug, Clone, Copy)]
struct SlideVisual {
    text_px: Px,
    line_height_px: Px,
}

fn slide(cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let number = ui::text(cx, format!("{idx}"))
        .text_size_px(visual.text_px)
        .line_height_px(visual.line_height_px)
        .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
        .font_semibold()
        .into_element(cx);

    let button = shadcn::Button::new("Focusable")
        .variant(shadcn::ButtonVariant::Secondary)
        .test_id(format!(
            "ui-gallery-carousel-plugin-stop-on-interaction-focus-slide-button-{idx}"
        ))
        .into_element(cx);

    let content = cx.flex(
        FlexProps {
            layout: decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().aspect_ratio(1.0),
            ),
            direction: fret_core::Axis::Vertical,
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            padding: Edges::all(Px(24.0)).into(),
            gap: SpacingLength::Px(Px(12.0)),
            ..Default::default()
        },
        move |_cx| vec![number, button],
    );

    let card = shadcn::Card::new([content]).into_element(cx);
    ui::container(cx, move |_cx| vec![card])
        .w_full()
        .p_1()
        .into_element(cx)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_xs = Px(320.0);

    let state = cx.with_state(Models::default, |st| st.clone());
    let autoplay_api = match state.autoplay_api {
        Some(model) => model,
        None => {
            let model: Model<Option<shadcn::CarouselAutoplayApi>> =
                cx.app.models_mut().insert(None);
            cx.with_state(Models::default, |st| st.autoplay_api = Some(model.clone()));
            model
        }
    };

    let autoplay = cx
        .watch_model(&autoplay_api)
        .cloned()
        .flatten()
        .map(|api| api.snapshot(&mut *cx.app))
        .unwrap_or_default();

    let focus_start = shadcn::Button::new("Click here, then press Tab")
        .variant(shadcn::ButtonVariant::Secondary)
        .test_id("ui-gallery-carousel-plugin-stop-on-interaction-focus-start")
        .into_element(cx);

    let visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide(cx, idx, visual)))
        .collect::<Vec<_>>();

    let carousel = shadcn::Carousel::default()
        .controls(false)
        .plugins([shadcn::CarouselPlugin::Autoplay(
            shadcn::CarouselAutoplayConfig::new(Duration::from_millis(2000))
                .pause_on_hover(false)
                .reset_on_hover_leave(false)
                .stop_on_interaction(true),
        )])
        .autoplay_api_handle_model(autoplay_api.clone())
        .items(items)
        .refine_layout(LayoutRefinement::default().w_full())
        .test_id("ui-gallery-carousel-plugin-stop-on-interaction-focus")
        .into_element(cx);

    let status_text = {
        let text = format!(
            "playing={} • stopped_by_interaction={}",
            autoplay.playing, autoplay.stopped_by_interaction
        );
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
            text: Arc::from(text),
            style: Some(style),
            color: Some(color),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Center,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        });

        ui::container(cx, move |_cx| vec![text])
            .py_1()
            .into_element(cx)
    };

    let stopped_marker = if autoplay.stopped_by_interaction {
        let badge = shadcn::Badge::new("Stopped by interaction")
            .variant(shadcn::BadgeVariant::Secondary)
            .test_id("ui-gallery-carousel-plugin-stop-on-interaction-focus-stopped")
            .into_element(cx);
        Some(
            ui::container(cx, move |_cx| vec![badge])
                .py_1()
                .into_element(cx),
        )
    } else {
        None
    };

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_stretch()
            .layout(
                LayoutRefinement::default()
                    .w_full()
                    .max_w(max_w_xs)
                    .mx_auto(),
            ),
        |_cx| {
            let mut out = vec![focus_start, carousel, status_text];
            if let Some(marker) = stopped_marker {
                out.push(marker);
            }
            out
        },
    )
}
// endregion: example
