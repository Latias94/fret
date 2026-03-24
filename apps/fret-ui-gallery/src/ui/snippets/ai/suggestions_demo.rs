pub const SOURCE: &str = include_str!("suggestions_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_icons_lucide::generated_ids::lucide;
use fret_ui::element::SemanticsProps;
use fret_ui::{Invalidation, Theme};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::{icon as decl_icon, style as decl_style};
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let clicked = cx.local_model_keyed("clicked", || false);
    let prompt = cx.local_model_keyed("prompt", String::new);

    let clicked_now = cx
        .get_model_copied(&clicked, Invalidation::Paint)
        .unwrap_or(false);
    let marker = clicked_now.then(|| {
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-ai-suggestions-clicked-marker")),
                ..Default::default()
            },
            |cx| vec![cx.text("")],
        )
    });

    let on_click: ui_ai::OnSuggestionClick = Arc::new({
        let clicked = clicked.clone();
        let prompt = prompt.clone();
        move |host, _action_cx, suggestion| {
            let _ = host.models_mut().update(&clicked, |v| *v = true);
            let _ = host
                .models_mut()
                .update(&prompt, |v| *v = suggestion.as_ref().to_string());
        }
    });

    let suggestions = ui_ai::Suggestions::new([
        ui_ai::Suggestion::new("Can you explain how to play tennis?")
            .test_id("ui-ai-suggestion-tennis")
            .on_click(on_click.clone())
            .into_element(cx),
        ui_ai::Suggestion::new("What is the weather in Tokyo?")
            .test_id("ui-ai-suggestion-weather-tokyo")
            .on_click(on_click.clone())
            .into_element(cx),
        ui_ai::Suggestion::new("How do I make a really good fish taco?")
            .test_id("ui-ai-suggestion-fish-taco")
            .on_click(on_click)
            .into_element(cx),
    ])
    .viewport_test_id("ui-ai-suggestions-viewport")
    .test_id_root("ui-ai-suggestions-root")
    .into_element(cx);

    let prompt_input = ui_ai::PromptInput::new(prompt)
        .test_id_root("ui-ai-suggestions-prompt-input")
        .test_id_textarea("ui-ai-suggestions-prompt-input-textarea")
        .test_id_send("ui-ai-suggestions-prompt-input-send")
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(672.0)))
        .into_element(cx);

    let shell = {
        let theme = Theme::global(&*cx.app).clone();

        let chrome = ChromeRefinement::default()
            .radius(theme.metric_token("metric.radius.lg"))
            .border_1()
            .p(Space::N6)
            .bg(ColorRef::Color(theme.color_token("background")))
            .border_color(ColorRef::Color(theme.color_token("border")));

        let layout = LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_w(Px(896.0))
            .h_px(Px(600.0));

        let mut props = decl_style::container_props(&theme.snapshot(), chrome, layout);
        props.padding = Edges::all(Px(24.0)).into();
        props.corner_radii = Corners::all(theme.metric_token("metric.radius.lg"));

        cx.container(props, move |cx| {
            vec![
                ui::v_flex(move |cx| {
                    vec![
                        suggestions,
                        ui::h_flex(move |_cx| vec![prompt_input])
                            .layout(LayoutRefinement::default().w_full())
                            .justify_center()
                            .into_element(cx),
                    ]
                })
                .layout(
                    LayoutRefinement::default()
                        .w_full()
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                )
                .gap(Space::N4)
                .into_element(cx),
            ]
        })
    };

    let mut out = vec![
        cx.text("Suggestions (AI Elements)"),
        cx.text("Suggestion pills emit intents; apps own prompt insertion."),
        shell,
    ];
    if let Some(m) = marker {
        out.push(m);
    }

    ui::v_flex(move |_cx| out)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .into_element(cx)
}
// endregion: example

// region: custom_children_example
pub fn render_custom_children(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let sparkles = decl_icon::icon_with(cx, lucide::SPARKLES, Some(Px(16.0)), None);
    let globe = decl_icon::icon_with(cx, lucide::GLOBE, Some(Px(16.0)), None);

    let suggestions = ui_ai::Suggestions::new([
        ui_ai::Suggestion::new("Summarize the release notes")
            .test_id("ui-ai-suggestion-custom-summary")
            .children([sparkles, cx.text("Summarize the release notes")])
            .into_element(cx),
        ui_ai::Suggestion::new("Draft a Tokyo travel brief")
            .test_id("ui-ai-suggestion-custom-brief")
            .children([globe, cx.text("Draft a Tokyo travel brief")])
            .into_element(cx),
    ])
    .viewport_test_id("ui-ai-suggestions-custom-viewport")
    .test_id_root("ui-ai-suggestions-custom-root")
    .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text(
                "Composable children let callers add icons or extra inline structure while the suggestion payload stays app-owned.",
            ),
            suggestions,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: custom_children_example
