pub const SOURCE: &str = include_str!("plugin_autoplay_stop_on_last_snap.rs");

// region: example
use fret::UiCx;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign, TextProps};
use fret_ui_kit::declarative::ModelWatchExt;
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

    let autoplay_api = cx.local_model_keyed("autoplay_api", || None::<shadcn::CarouselAutoplayApi>);
    let api_handle = cx.local_model_keyed("api_handle", || None::<shadcn::CarouselApi>);
    let api_cursor = cx.local_model_keyed("api_cursor", shadcn::CarouselEventCursor::default);
    let current = cx.local_model_keyed("current", || 0usize);
    let count = cx.local_model_keyed("count", || 0usize);

    if let Some(api_now) = cx.watch_model(&api_handle).cloned().flatten() {
        let mut cursor_now = cx.watch_model(&api_cursor).copied().unwrap_or_default();
        let events = api_now.events_since(&mut *cx.app, &mut cursor_now);
        let snap = api_now.snapshot(&mut *cx.app);
        let count_now = cx.watch_model(&count).copied().unwrap_or(0);

        if !events.is_empty() || (count_now == 0 && snap.snap_count > 0) {
            let next_count = snap.snap_count;
            let next_current = if next_count > 0 {
                snap.selected_index.saturating_add(1)
            } else {
                0
            };
            let _ = cx.app.models_mut().update(&api_cursor, |v| *v = cursor_now);
            let _ = cx.app.models_mut().update(&count, |v| *v = next_count);
            let _ = cx.app.models_mut().update(&current, |v| *v = next_current);
        }
    }

    let autoplay = cx
        .watch_model(&autoplay_api)
        .cloned()
        .flatten()
        .map(|api| api.snapshot(&mut *cx.app))
        .unwrap_or_default();

    let visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide_card(cx, idx, visual)))
        .collect::<Vec<_>>();

    let carousel = shadcn::Carousel::default()
        .plugins([shadcn::CarouselPlugin::Autoplay(
            shadcn::CarouselAutoplayConfig::new(Duration::from_millis(200))
                .pause_on_hover(false)
                .reset_on_hover_leave(false)
                .stop_on_last_snap(true),
        )])
        .api_handle_model(api_handle.clone())
        .autoplay_api_handle_model(autoplay_api.clone())
        .refine_layout(LayoutRefinement::default().w_full().max_w(max_w_xs))
        .test_id("ui-gallery-carousel-plugin-stop-on-last-snap")
        .into_element_parts(
            cx,
            |_cx| shadcn::CarouselContent::new(items),
            shadcn::CarouselPrevious::new(),
            shadcn::CarouselNext::new(),
        );

    let slide_counter = {
        let text = format!(
            "Slide {} of {} • playing={} • stopped_by_last_snap={}",
            cx.watch_model(&current).copied().unwrap_or(0),
            cx.watch_model(&count).copied().unwrap_or(0),
            autoplay.playing,
            autoplay.stopped_by_last_snap
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

        ui::container(move |_cx| vec![text]).py_2().into_element(cx)
    };

    let stopped_marker = if autoplay.stopped_by_last_snap {
        let badge = shadcn::Badge::new("Stopped at last snap")
            .variant(shadcn::BadgeVariant::Secondary)
            .test_id("ui-gallery-carousel-plugin-stop-on-last-snap-stopped")
            .into_element(cx);
        Some(
            ui::container(move |_cx| vec![badge])
                .py_1()
                .into_element(cx),
        )
    } else {
        None
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
        move |_cx| {
            let mut out = vec![carousel, slide_counter];
            if let Some(marker) = stopped_marker {
                out.push(marker);
            }
            out
        },
    )
}
// endregion: example
