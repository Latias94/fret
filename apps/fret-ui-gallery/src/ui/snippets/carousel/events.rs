pub const SOURCE: &str = DOCS_SOURCE;
pub const DOCS_SOURCE: &str = include_str!("events.docs.rs.txt");

// region: example
use fret::{UiChild, UiCx};
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let max_w_xs = Px(320.0);
    const EVENT_BASELINE_STABLE_FRAMES: usize = 2;

    let api_handle = cx.local_model_keyed("api_handle", || None::<shadcn::CarouselApi>);
    let api_cursor = cx.local_model_keyed("api_cursor", shadcn::CarouselEventCursor::default);
    let events_armed = cx.local_model_keyed("events_armed", || false);
    let baseline_reinit_generation =
        cx.local_model_keyed("baseline_reinit_generation", || None::<u64>);
    let baseline_stable_frames = cx.local_model_keyed("baseline_stable_frames", || 0usize);
    let select_seen = cx.local_model_keyed("select_seen", || false);
    let reinit_seen = cx.local_model_keyed("reinit_seen", || false);
    let selected_index = cx.local_model_keyed("selected_index", || 0usize);

    // Upstream docs: `setApi` + `api.on("select"|"reInit", ...)`.
    // Fret: poll a cursor for the same outcomes.
    if let Some(api_now) = cx.watch_model(&api_handle).cloned().flatten() {
        let snapshot = api_now.snapshot(&mut *cx.app);
        let is_armed = cx.watch_model(&events_armed).copied().unwrap_or(false);

        if !is_armed {
            let baseline_generation_now = cx
                .watch_model(&baseline_reinit_generation)
                .copied()
                .flatten();
            let stable_frames_now = cx
                .watch_model(&baseline_stable_frames)
                .copied()
                .unwrap_or(0);
            let baseline_matches = baseline_generation_now == Some(snapshot.reinit_generation);
            let next_stable_frames = if snapshot.snap_count > 0 && baseline_matches {
                stable_frames_now + 1
            } else {
                1
            };

            let _ = cx.app.models_mut().update(&api_cursor, |v| {
                *v = shadcn::CarouselEventCursor {
                    select_generation: snapshot.select_generation,
                    reinit_generation: snapshot.reinit_generation,
                };
            });
            let _ = cx
                .app
                .models_mut()
                .update(&baseline_reinit_generation, |v| {
                    *v = Some(snapshot.reinit_generation)
                });
            let _ = cx
                .app
                .models_mut()
                .update(&baseline_stable_frames, |v| *v = next_stable_frames);

            if snapshot.snap_count > 0 && next_stable_frames >= EVENT_BASELINE_STABLE_FRAMES {
                let _ = cx.app.models_mut().update(&events_armed, |v| *v = true);
            }
        } else {
            let mut cursor_now = cx.watch_model(&api_cursor).copied().unwrap_or_default();
            let events = api_now.events_since(&mut *cx.app, &mut cursor_now);

            if !events.is_empty() {
                let mut select_seen_now = cx.watch_model(&select_seen).copied().unwrap_or(false);
                let mut reinit_seen_now = cx.watch_model(&reinit_seen).copied().unwrap_or(false);
                let mut selected_index_now = cx.watch_model(&selected_index).copied().unwrap_or(0);

                for ev in events {
                    match ev {
                        shadcn::CarouselEvent::ReInit => {
                            reinit_seen_now = true;
                        }
                        shadcn::CarouselEvent::Select { selected_index } => {
                            select_seen_now = true;
                            selected_index_now = selected_index;
                        }
                    }
                }

                let _ = cx.app.models_mut().update(&api_cursor, |v| *v = cursor_now);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&select_seen, |v| *v = select_seen_now);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&reinit_seen, |v| *v = reinit_seen_now);
                let _ = cx
                    .app
                    .models_mut()
                    .update(&selected_index, |v| *v = selected_index_now);
            }
        }
    }

    let visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide_card(cx, idx, visual).into_element(cx)))
        .collect::<Vec<_>>();

    let carousel = shadcn::Carousel::default()
        .api_handle_model(api_handle.clone())
        .refine_layout(LayoutRefinement::default().w_full().max_w(max_w_xs))
        .test_id("ui-gallery-carousel-events")
        .into_element_parts_content(
            cx,
            shadcn::CarouselContent::new(items),
            shadcn::CarouselPrevious::new().test_id("ui-gallery-carousel-events-prev"),
            shadcn::CarouselNext::new().test_id("ui-gallery-carousel-events-next"),
        );

    let status = {
        let text = format!(
            "select_seen={} • reinit_seen={} • selected_index={}",
            cx.watch_model(&select_seen).copied().unwrap_or(false),
            cx.watch_model(&reinit_seen).copied().unwrap_or(false),
            cx.watch_model(&selected_index).copied().unwrap_or(0)
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

    let seen_marker = if cx.watch_model(&select_seen).copied().unwrap_or(false) {
        Some(
            shadcn::Badge::new("Select event seen")
                .variant(shadcn::BadgeVariant::Secondary)
                .test_id("ui-gallery-carousel-events-select-seen")
                .into_element(cx),
        )
    } else {
        None
    };

    let reinit_marker = if cx.watch_model(&reinit_seen).copied().unwrap_or(false) {
        Some(
            shadcn::Badge::new("ReInit event seen")
                .variant(shadcn::BadgeVariant::Secondary)
                .test_id("ui-gallery-carousel-events-reinit-seen")
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
            let mut out = vec![carousel, status];
            if let Some(marker) = seen_marker {
                out.push(marker);
            }
            if let Some(marker) = reinit_marker {
                out.push(marker);
            }
            out
        },
    )
}
// endregion: example
