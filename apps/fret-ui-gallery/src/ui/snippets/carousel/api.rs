// region: example
use fret_app::App;
use fret_core::Edges;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, FlexProps, MainAlign, TextProps};
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    api_handle: Option<Model<Option<shadcn::CarouselApi>>>,
    api_cursor: Option<Model<shadcn::CarouselEventCursor>>,
    api_effect_current: Option<Model<usize>>,
    api_effect_count: Option<Model<usize>>,
}

#[derive(Debug, Clone, Copy)]
struct SlideVisual {
    text_px: Px,
    line_height_px: Px,
}

fn slide_card(cx: &mut ElementContext<'_, App>, idx: usize, visual: SlideVisual) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let number = ui::text(cx, format!("{idx}"))
        .text_size_px(visual.text_px)
        .line_height_px(visual.line_height_px)
        .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
        .font_semibold()
        .into_element(cx);

    let content = cx.flex(
        FlexProps {
            layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full().aspect_ratio(1.0)),
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

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let max_w_xs = Px(320.0);

    let state = cx.with_state(Models::default, |st| st.clone());
    let api_handle = match state.api_handle {
        Some(model) => model,
        None => {
            let model: Model<Option<shadcn::CarouselApi>> = cx.app.models_mut().insert(None);
            cx.with_state(Models::default, |st| st.api_handle = Some(model.clone()));
            model
        }
    };
    let api_cursor = match state.api_cursor {
        Some(model) => model,
        None => {
            let model: Model<shadcn::CarouselEventCursor> =
                cx.app.models_mut().insert(shadcn::CarouselEventCursor::default());
            cx.with_state(Models::default, |st| st.api_cursor = Some(model.clone()));
            model
        }
    };
    let api_effect_current = match state.api_effect_current {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(0usize);
            cx.with_state(Models::default, |st| st.api_effect_current = Some(model.clone()));
            model
        }
    };
    let api_effect_count = match state.api_effect_count {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(0usize);
            cx.with_state(Models::default, |st| st.api_effect_count = Some(model.clone()));
            model
        }
    };

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
            let _ = cx.app.models_mut().update(&api_effect_count, |v| *v = next_count);
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
    let api_items = (1..=5)
        .map(|idx| slide_card(cx, idx, api_visual))
        .collect::<Vec<_>>();
    let api_carousel = shadcn::Carousel::new(api_items)
        .api_handle_model(api_handle.clone())
        .refine_track_layout(LayoutRefinement::default().w_px(Px(336.0)))
        .refine_layout(LayoutRefinement::default().w_full().max_w(max_w_xs))
        .test_id("ui-gallery-carousel-api")
        .into_element(cx);

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

        ui::container(cx, move |_cx| vec![text]).py_2().into_element(cx)
    };

    cx.flex(
        FlexProps {
            layout: decl_style::layout_style(
                &Theme::global(&*cx.app).snapshot(),
                LayoutRefinement::default().w_full().max_w(max_w_xs).mx_auto(),
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
