use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_suggestions_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui::element::SemanticsProps;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Default)]
    struct DemoModels {
        clicked: Option<Model<bool>>,
    }

    let clicked = cx.with_state(DemoModels::default, |st| st.clicked.clone());
    let clicked = match clicked {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DemoModels::default, |st| st.clicked = Some(model.clone()));
            model
        }
    };

    let clicked_now = cx
        .get_model_copied(&clicked, Invalidation::Paint)
        .unwrap_or(false);
    let marker = clicked_now.then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-ai-suggestions-clicked-marker")),
                ..Default::default()
            },
            |cx| vec![cx.text("")],
        )
    });

    let on_click: ui_ai::OnSuggestionClick = Arc::new({
        let clicked = clicked.clone();
        move |host, _action_cx, _suggestion| {
            let _ = host.models_mut().update(&clicked, |v| *v = true);
        }
    });

    let suggestions = ui_ai::Suggestions::new([
        ui_ai::Suggestion::new("Explain the architecture")
            .label("Architecture")
            .test_id("ui-ai-suggestion-architecture")
            .on_click(on_click.clone())
            .into_element(cx),
        ui_ai::Suggestion::new("Show a minimal example")
            .label("Minimal example")
            .on_click(on_click.clone())
            .into_element(cx),
        ui_ai::Suggestion::new("What are the tradeoffs?")
            .label("Tradeoffs")
            .on_click(on_click)
            .into_element(cx),
    ])
    .test_id_root("ui-ai-suggestions-root")
    .into_element(cx);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            let mut out = vec![
                cx.text("Suggestions (AI Elements)"),
                cx.text("Suggestion pills emit intents; apps own prompt insertion."),
                suggestions,
            ];
            if let Some(m) = marker {
                out.push(m);
            }
            out
        },
    )]
}
