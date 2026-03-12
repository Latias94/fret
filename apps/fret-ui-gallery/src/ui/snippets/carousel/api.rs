pub const SOURCE: &str = include_str!("api.rs");

// region: example
use fret::UiCx;
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

    shadcn::Card::new([content]).into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let max_w_xs = Px(320.0);

    let api_handle = cx.local_model_keyed("api_handle", || None::<shadcn::CarouselApi>);
    let api_cursor = cx.local_model_keyed("api_cursor", shadcn::CarouselEventCursor::default);
    let api_effect_current = cx.local_model_keyed("api_effect_current", || 0usize);
    let api_effect_count = cx.local_model_keyed("api_effect_count", || 0usize);

    // Upstream uses `setApi` + `api.on("select"|"reInit", ...)` to update counters.
    // Here we poll a cursor for the same outcomes.
    if let Some(api_now) = cx.watch_model(&api_handle).cloned().flatten() {
        let mut cursor_now = cx.watch_model(&api_cursor).copied().unwrap_or_default();
        let events = api_now.events_since(&mut *cx.app, &mut cursor_now);
        let snapshot = api_now.snapshot(&mut *cx.app);
        let count_now = cx.watch_model(&api_effect_count).copied().unwrap_or(0);

        if !events.is_empty() || (count_now == 0 && snapshot.snap_count > 0) {
            let next_count = snapshot.snap_count;
            let next_current = if next_count > 0 {
                snapshot.selected_index.saturating_add(1)
            } else {
                0
            };
            let _ = cx.app.models_mut().update(&api_cursor, |v| *v = cursor_now);
            let _ = cx
                .app
                .models_mut()
                .update(&api_effect_count, |v| *v = next_count);
            let _ = cx
                .app
                .models_mut()
                .update(&api_effect_current, |v| *v = next_current);
        }
    }

    let api_visual = SlideVisual {
        text_px: Px(36.0),
        line_height_px: Px(40.0),
    };
    let items = (1..=5)
        .map(|idx| shadcn::CarouselItem::new(slide_card(cx, idx, api_visual).into_element(cx)))
        .collect::<Vec<_>>();
    let api_carousel = shadcn::Carousel::default()
        .api_handle_model(api_handle.clone())
        .refine_layout(LayoutRefinement::default().w_full().max_w(max_w_xs))
        .test_id("ui-gallery-carousel-api")
        .into_element_parts(
            cx,
            |_cx| shadcn::CarouselContent::new(items),
            shadcn::CarouselPrevious::new(),
            shadcn::CarouselNext::new(),
        );

    let current = cx.watch_model(&api_effect_current).copied().unwrap_or(0);
    let count = cx.watch_model(&api_effect_count).copied().unwrap_or(0);
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
        move |_cx| vec![api_carousel, api_counter],
    )
}
// endregion: example
