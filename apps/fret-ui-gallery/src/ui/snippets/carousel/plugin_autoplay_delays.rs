pub const SOURCE: &str = include_str!("plugin_autoplay_delays.rs");

// region: example
use fret::UiCx;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;
use std::time::Duration;

#[derive(Default)]
struct DelaysApplied {
    applied: bool,
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

    if let Some(api_now) = cx.watch_model(&autoplay_api).cloned().flatten() {
        let applied = cx.slot_state(DelaysApplied::default, |st| st.applied);
        if !applied {
            api_now.set_delays_store(
                cx.app.models_mut(),
                Arc::from([
                    Duration::from_millis(120),
                    Duration::from_millis(240),
                    Duration::from_millis(360),
                    Duration::from_millis(480),
                    Duration::from_millis(600),
                ]),
            );
            cx.slot_state(DelaysApplied::default, |st| st.applied = true);
            cx.request_frame();
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
        .map(|idx| shadcn::CarouselItem::new(slide_card(cx, idx, visual).into_element(cx)))
        .collect::<Vec<_>>();

    let carousel = shadcn::Carousel::default()
        .plugins([shadcn::CarouselPlugin::Autoplay(
            shadcn::CarouselAutoplayConfig::new(Duration::from_millis(2000))
                .pause_on_hover(false)
                .reset_on_hover_leave(false),
        )])
        .api_handle_model(api_handle.clone())
        .autoplay_api_handle_model(autoplay_api.clone())
        .refine_layout(LayoutRefinement::default().w_full().max_w(max_w_xs))
        .test_id("ui-gallery-carousel-plugin-autoplay-delays")
        .into_element_parts(
            cx,
            |_cx| shadcn::CarouselContent::new(items),
            shadcn::CarouselPrevious::new(),
            shadcn::CarouselNext::new(),
        );

    let status = {
        let rem = autoplay
            .time_until_next
            .map(|d| format!("{}ms", d.as_millis()))
            .unwrap_or_else(|| "n/a".to_string());
        let text = format!(
            "Slide {} of {} • next in {}",
            cx.watch_model(&current).copied().unwrap_or(0),
            cx.watch_model(&count).copied().unwrap_or(0),
            rem
        );

        let badge = shadcn::Badge::new(text)
            .variant(shadcn::BadgeVariant::Secondary)
            .test_id("ui-gallery-carousel-plugin-autoplay-delays-status")
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
        move |_cx| vec![carousel, status],
    )
}
// endregion: example
