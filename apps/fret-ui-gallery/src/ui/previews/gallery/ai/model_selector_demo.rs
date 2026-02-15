use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_model_selector_demo(
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
        open: Option<Model<bool>>,
        query: Option<Model<String>>,
        selected: Option<Model<Option<Arc<str>>>>,
    }

    let needs_init = cx.with_state(DemoModels::default, |st| {
        st.open.is_none() || st.query.is_none() || st.selected.is_none()
    });
    if needs_init {
        let open = cx.app.models_mut().insert(false);
        let query = cx.app.models_mut().insert(String::new());
        let selected = cx.app.models_mut().insert(None::<Arc<str>>);
        cx.with_state(DemoModels::default, move |st| {
            st.open = Some(open.clone());
            st.query = Some(query.clone());
            st.selected = Some(selected.clone());
        });
    }

    let (open, query, selected) = cx.with_state(DemoModels::default, |st| {
        (
            st.open.clone().expect("open"),
            st.query.clone().expect("query"),
            st.selected.clone().expect("selected"),
        )
    });

    let selected_now = cx
        .get_model_cloned(&selected, Invalidation::Paint)
        .unwrap_or(None);

    let selected_marker = selected_now.is_some().then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-ai-model-selector-selected-some")),
                ..Default::default()
            },
            |cx| vec![cx.text("")],
        )
    });

    let model_selector = ui_ai::ModelSelector::new()
        .open_model(open.clone())
        .into_element(
            cx,
            move |cx, open| {
                ui_ai::ModelSelectorTrigger::new(
                    open,
                    shadcn::Button::new("Select model")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx),
                )
                .test_id("ui-ai-model-selector-trigger")
                .into_element(cx)
            },
            move |cx, open| {
                let on_select = |value: &'static str| -> fret_ui::action::OnActivate {
                    let selected = selected.clone();
                    let open = open.clone();
                    Arc::new(move |host, action_cx, _reason| {
                        let _ = host
                            .models_mut()
                            .update(&selected, |v| *v = Some(Arc::<str>::from(value)));
                        let _ = host.models_mut().update(&open, |v| *v = false);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                    })
                };

                ui_ai::ModelSelectorContent::new([
                    ui_ai::ModelSelectorInput::new(query.clone())
                        .placeholder("Search models...")
                        .into_element(cx)
                        .test_id("ui-ai-model-selector-input"),
                    ui_ai::ModelSelectorList::new([
                        ui_ai::ModelSelectorItem::new("GPT-4.1 mini")
                            .test_id("ui-ai-model-selector-item-openai-gpt-4-1-mini")
                            .on_select_action(on_select("openai-gpt-4-1-mini"))
                            .into(),
                        ui_ai::ModelSelectorItem::new("Claude 3.5 Sonnet")
                            .test_id("ui-ai-model-selector-item-anthropic-claude-3-5-sonnet")
                            .on_select_action(on_select("anthropic-claude-3-5-sonnet"))
                            .into(),
                        ui_ai::ModelSelectorItem::new("Gemini 2.0 Flash")
                            .test_id("ui-ai-model-selector-item-google-gemini-2-0-flash")
                            .on_select_action(on_select("google-gemini-2-0-flash"))
                            .into(),
                    ])
                    .highlight_query_model(query.clone())
                    .into_element(cx),
                ])
                .test_id_root("ui-ai-model-selector-content")
                .into_element(cx)
            },
        );

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            let mut out = vec![
                cx.text("ModelSelector (AI Elements)"),
                cx.text("Dialog + Command surfaces; selection is app-owned."),
                model_selector,
            ];
            if let Some(marker) = selected_marker {
                out.push(marker);
            }
            out
        },
    )]
}
