pub const SOURCE: &str = include_str!("suggestions_demo.rs");

// region: example
use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::SemanticsProps;
use fret_ui::{Invalidation, Theme};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    clicked: Option<Model<bool>>,
    prompt: Option<Model<String>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let clicked = cx.with_state(DemoModels::default, |st| st.clicked.clone());
    let clicked = match clicked {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DemoModels::default, |st| st.clicked = Some(model.clone()));
            model
        }
    };

    let prompt = cx.with_state(DemoModels::default, |st| st.prompt.clone());
    let prompt = match prompt {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(DemoModels::default, |st| st.prompt = Some(model.clone()));
            model
        }
    };

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
            vec![ui::v_flex(move |cx| {
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
            .into_element(cx)]
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
